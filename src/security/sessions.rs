//! Ending every other SSH login on a server (ERR-83).
//!
//! What "revoke all sessions" means in Crow: every SSH connection to the
//! server's sshd ends now, except the one Crow is using. Console logins
//! aren't SSH and are left alone, and no key is touched: anyone whose key
//! still works can log back in, so pair this with key rotation or removal
//! when that's the point.
//!
//! Crow's own connection is found from the inside: the script walks up from
//! its own process to the sshd process that serves this connection (a
//! child of the listening sshd). Every other child of that listener is a
//! connection, and gets TERM, then KILL. If Crow can't find its own
//! connection it ends nothing.

use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

const REVOKE: &str = r#"ppid() { awk '/^PPid:/{print $2}' "/proc/$1/status" 2>/dev/null; }
is_sshd() { case "$(cat "/proc/$1/comm" 2>/dev/null)" in sshd|sshd-session) return 0 ;; esac; return 1; }
p=$$; mine=
while [ -n "$p" ] && [ "$p" != 0 ]; do
  q=$(ppid "$p")
  if is_sshd "$p" && is_sshd "$q" && ! is_sshd "$(ppid "$q")"; then mine=$p; break; fi
  [ "$p" = 1 ] && break
  p=$q
done
if [ -z "$mine" ]; then echo "@@nomine"; exit 3; fi
listener=$(ppid "$mine")
ended=
for d in /proc/[0-9]*; do
  pid=${d#/proc/}
  [ "$pid" = "$mine" ] && continue
  [ "$(ppid "$pid")" = "$listener" ] || continue
  is_sshd "$pid" || continue
  echo "@@end $pid $(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null)"
  kill -TERM "$pid" 2>/dev/null
  ended="$ended $pid"
done
[ -n "$ended" ] && sleep 1
for pid in $ended; do kill -KILL "$pid" 2>/dev/null; done
true"#;

/// One ended connection: who it belonged to, as sshd names it
/// ("sshd-session: bob [priv]" → "bob").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ended {
    pub pid: u32,
    pub user: String,
}

pub fn parse(out: &str) -> Result<Vec<Ended>, String> {
    if out.lines().any(|l| l.trim() == "@@nomine") {
        return Err("Crow couldn't find its own connection among sshd's, so it ended nothing".into());
    }
    Ok(out
        .lines()
        .filter_map(|l| l.trim().strip_prefix("@@end "))
        .filter_map(|rest| {
            let (pid, cmd) = rest.split_once(' ').unwrap_or((rest, ""));
            // "sshd: bob [priv]", "sshd-session: bob@pts/1", "sshd: unknown [net]"
            let user = cmd.split_once(':').map(|(_, who)| who.trim()).unwrap_or("").split([' ', '@']).next().unwrap_or("").to_string();
            Some(Ended { pid: pid.parse().ok()?, user: if user.is_empty() { "unknown".into() } else { user } })
        })
        .collect())
}

/// "ended 3 other SSH logins: alice, bob ×2", or that there were none.
pub fn summary(ended: &[Ended]) -> String {
    if ended.is_empty() {
        return "no other SSH logins".into();
    }
    let mut users: Vec<(String, usize)> = Vec::new();
    for e in ended {
        match users.iter_mut().find(|(u, _)| *u == e.user) {
            Some((_, n)) => *n += 1,
            None => users.push((e.user.clone(), 1)),
        }
    }
    let who = users.iter().map(|(u, n)| if *n > 1 { format!("{u} ×{n}") } else { u.clone() }).collect::<Vec<_>>().join(", ");
    format!("ended {} other SSH login{}: {who}", ended.len(), if ended.len() == 1 { "" } else { "s" })
}

/// Ends every SSH connection to the server but Crow's own.
pub fn end_other_sessions(host: &dyn Host) -> Result<Vec<Ended>, String> {
    let out = match host.exec_privileged(&["sh", "-c", REVOKE, "crow-revoke"], &[], DEFAULT_TIMEOUT) {
        Ok(o) => o.stdout,
        Err(HostError::Failed { status: 3, .. }) => "@@nomine".into(),
        Err(HostError::Failed { stderr, .. }) => return Err(stderr.trim().to_string()),
        Err(e) => return Err(e.to_string()),
    };
    parse(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_who_was_logged_out() {
        let out = "@@end 812 sshd-session: bob [priv] \n@@end 900 sshd: alice@pts/2 \n@@end 901 sshd: bob [priv] \n@@end 950 sshd: unknown [net] \n";
        let ended = parse(out).unwrap();
        assert_eq!(ended.iter().map(|e| (e.pid, e.user.as_str())).collect::<Vec<_>>(), vec![(812, "bob"), (900, "alice"), (901, "bob"), (950, "unknown")]);
        assert_eq!(summary(&ended), "ended 4 other SSH logins: bob ×2, alice, unknown");
        assert_eq!(summary(&[]), "no other SSH logins");
    }

    #[test]
    fn without_its_own_connection_nothing_is_ended() {
        assert!(parse("@@nomine\n").unwrap_err().contains("ended nothing"));
    }

    /// Live, against a throwaway sshd container (see host_keys'
    /// live test for the setup). Opt-in:
    ///   CROW_LIVE_SESSIONS=/path/to/private_key cargo test live_end_other_sessions -- --ignored
    #[test]
    #[ignore]
    fn live_end_other_sessions() {
        use std::process::{Command, Stdio};
        let key = std::env::var("CROW_LIVE_SESSIONS").expect("set CROW_LIVE_SESSIONS=/path/to/key");
        let ssh = |cmd: &str| {
            let mut c = Command::new("ssh");
            c.args(["-F", "/dev/null", "-i", &key, "-p", "22299", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null", "-o", "IdentityAgent=none", "root@127.0.0.1", cmd]);
            c
        };
        // Someone else, logged in.
        let mut other = ssh("sleep 60").stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        // Crow, running the script through its own connection.
        let out = ssh(&format!("sh -c '{}' crow-revoke", REVOKE.replace('\'', r"'\''"))).output().unwrap();
        let ended = parse(&String::from_utf8_lossy(&out.stdout)).unwrap();
        assert_eq!(ended.len(), 1, "{}", String::from_utf8_lossy(&out.stdout));
        assert_eq!(ended[0].user, "root");
        assert!(out.status.success(), "Crow's own connection survived to report");
        let status = other.wait().unwrap();
        assert!(!status.success(), "the other login was cut");
    }
}
