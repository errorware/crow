//! Rebooting a server and waiting for it to come back (ERR-80).

use std::time::{Duration, Instant};

use super::{Host, DEFAULT_TIMEOUT};
use crate::views::fleet::run::{StepOutcome, StepResult};

/// What identifies one boot: the kernel's boot id (changes on a real
/// reboot) and PID 1's start time (changes when a container restarts,
/// which shares the host's kernel and boot id).
const BOOT_MARKER: &str = "cat /proc/sys/kernel/random/boot_id; stat -c %Z /proc/1";

pub fn boot_marker(host: &dyn Host) -> Result<String, String> {
    host.exec(&["sh", "-c", BOOT_MARKER], DEFAULT_TIMEOUT)
        .map(|o| o.stdout.trim().to_string())
        .map_err(|e| e.to_string())
        .and_then(|m| if m.is_empty() { Err("the host returned no boot marker".into()) } else { Ok(m) })
}

/// Reads the boot marker, runs `trigger` (the reboot request), then polls
/// every `poll` until the host answers with a different marker, for up to
/// `timeout`. A trigger error is expected when the reboot cuts the
/// connection it was sent over, so only the wait decides the outcome.
pub fn reboot_and_wait(
    host: &dyn Host,
    trigger: impl FnOnce() -> Result<(), String>,
    timeout: Duration,
    poll: Duration,
) -> StepResult {
    let before = boot_marker(host).map_err(|e| format!("not rebooted: couldn't read the boot marker first: {e}"))?;
    let started = Instant::now();
    let trigger_error = trigger().err();
    loop {
        std::thread::sleep(poll);
        if let Ok(now) = boot_marker(host) {
            if now != before {
                return Ok(StepOutcome::Done(format!("back after {}s", started.elapsed().as_secs())));
            }
        }
        if started.elapsed() >= timeout {
            return Err(match trigger_error {
                Some(e) => format!("the reboot request failed ({e}) and the server didn't restart"),
                None => format!("not back after {}s", timeout.as_secs()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{ExecOutput, HostError};
    use std::sync::Mutex;

    /// Answers the boot marker from a script of replies.
    struct Scripted(Mutex<Vec<Result<&'static str, ()>>>);

    impl Host for Scripted {
        fn label(&self) -> String {
            "scripted".into()
        }
        fn exec_stdin(&self, _argv: &[&str], _stdin: &[u8], _timeout: Duration) -> Result<ExecOutput, HostError> {
            let mut replies = self.0.lock().unwrap();
            let next = if replies.len() > 1 { replies.remove(0) } else { replies[0] };
            next.map(|s| ExecOutput { stdout: s.into(), stderr: String::new() }).map_err(|()| HostError::Unreachable("down".into()))
        }
    }

    const FAST: Duration = Duration::from_millis(1);

    #[test]
    fn waits_through_the_outage_for_a_new_boot() {
        let host = Scripted(Mutex::new(vec![Ok("boot-a\n1"), Err(()), Err(()), Ok("boot-a\n1"), Ok("boot-b\n9")]));
        let r = reboot_and_wait(&host, || Err("connection closed by remote host".into()), Duration::from_secs(5), FAST);
        assert!(matches!(r, Ok(StepOutcome::Done(_))), "{r:?}");
    }

    #[test]
    fn a_container_restart_counts_by_pid1() {
        let host = Scripted(Mutex::new(vec![Ok("boot-a\n100"), Ok("boot-a\n250")]));
        assert!(reboot_and_wait(&host, || Ok(()), Duration::from_secs(5), FAST).is_ok());
    }

    #[test]
    fn never_coming_back_fails_the_step() {
        let host = Scripted(Mutex::new(vec![Ok("boot-a\n1"), Err(())]));
        let r = reboot_and_wait(&host, || Ok(()), Duration::from_millis(20), FAST);
        assert!(r.unwrap_err().starts_with("not back after"));
    }

    #[test]
    fn a_refused_reboot_says_so() {
        let host = Scripted(Mutex::new(vec![Ok("boot-a\n1")]));
        let r = reboot_and_wait(&host, || Err("sudo needs a password".into()), Duration::from_millis(20), FAST);
        assert!(r.unwrap_err().contains("sudo needs a password"));
    }

    #[test]
    fn unreadable_marker_means_nothing_is_done() {
        let host = Scripted(Mutex::new(vec![Err(())]));
        let mut triggered = false;
        let r = reboot_and_wait(&host, || { triggered = true; Ok(()) }, Duration::from_millis(20), FAST);
        assert!(r.is_err() && !triggered, "never reboot a host we can't watch come back");
    }
}
