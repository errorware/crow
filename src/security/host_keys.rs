//! Rotating a server's SSH host keys, and re-pinning them (ERR-82).
//!
//! Every client's known_hosts breaks when host keys change, Crow's included,
//! so each step is checked and anything that fails is undone:
//!
//! 1. new keys are generated next to the old ones (`KEY.crow-new`), one per
//!    key sshd serves, of the same type;
//! 2. their public halves are read over the connection Crow already holds,
//!    which the old key authenticated: that's what makes them trustworthy;
//! 3. the old keys are copied to a root-only backup, the new ones moved into
//!    place, `sshd -t` checks them and sshd reloads (open sessions survive);
//! 4. known_hosts swaps the old entries for the new keys;
//! 5. a brand-new login with strict host key checking proves the new pin.
//!
//! If 3–5 fail, the old keys go back, sshd reloads and known_hosts is
//! restored. On success the backup (old private keys) is deleted.

use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

/// Generates `KEY.crow-new` for every host key given, same type, and prints
/// `@@key PATH` then the new public key.
const PREPARE: &str = r#"set -e
for k; do
  [ -f "$k" ] && [ -f "$k.pub" ] || continue
  case "$(cut -d' ' -f1 "$k.pub")" in
    ssh-ed25519) a="-t ed25519" ;;
    ecdsa-sha2-nistp256) a="-t ecdsa -b 256" ;;
    ecdsa-sha2-nistp384) a="-t ecdsa -b 384" ;;
    ecdsa-sha2-nistp521) a="-t ecdsa -b 521" ;;
    ssh-rsa) a="-t rsa -b 4096" ;;
    *) echo "unsupported host key type in $k.pub" >&2; exit 3 ;;
  esac
  rm -f "$k.crow-new" "$k.crow-new.pub"
  ssh-keygen -q $a -N '' -C '' -f "$k.crow-new"
  echo "@@key $k"
  cat "$k.crow-new.pub"
done"#;

/// Reloads sshd under whatever service manager the host has.
const RELOAD_SSHD: &str = r#"reload_sshd() { systemctl reload ssh 2>/dev/null || systemctl reload sshd 2>/dev/null || rc-service sshd reload 2>/dev/null || service ssh reload 2>/dev/null || service sshd reload 2>/dev/null; }"#;

/// `$1` backup dir, then the keys: back up, swap in, check, reload. Once
/// the backup is complete, any failure (sshd -t rejecting the keys, sshd
/// not reloading, a partial move) puts the old keys back before exiting
/// (ERR-107): the disk never ends up with keys sshd isn't serving.
const SWAP: &str = r#"set -e
b=$1; shift
mkdir -p -m 700 "$b"
for k; do cp -p "$k" "$k.pub" "$b/"; done
ok=0
trap '[ "$ok" = 1 ] || for k; do n=$(basename "$k"); cp -p "$b/$n" "$k"; cp -p "$b/$n.pub" "$k.pub"; done' EXIT
for k; do mv -f "$k.crow-new" "$k"; mv -f "$k.crow-new.pub" "$k.pub"; done
command -v restorecon >/dev/null 2>&1 && restorecon "$@" 2>/dev/null || true
if ! sshd -t; then echo "sshd -t rejected the new keys; the old ones are back" >&2; exit 4; fi
if ! reload_sshd; then echo "sshd couldn't be reloaded (no systemctl, rc-service or service that knows it); the old keys are back" >&2; exit 5; fi
ok=1"#;

/// `$1` backup dir, then the keys: put the old keys back and reload.
const RESTORE: &str = r#"b=$1; shift
for k; do n=$(basename "$k"); cp -p "$b/$n" "$k" && cp -p "$b/$n.pub" "$k.pub" || exit 1; done
command -v restorecon >/dev/null 2>&1 && restorecon "$@" 2>/dev/null
reload_sshd"#;

/// The local side: known_hosts entries for one `host`/`[host]:port`.
pub trait KnownHosts {
    /// The full known_hosts lines for the pattern (hashed ones included).
    fn lines(&self, pattern: &str) -> Vec<String>;
    /// Removes every entry for the pattern and adds `lines`.
    fn replace(&self, pattern: &str, lines: &[String]) -> Result<(), String>;
}

/// `~/.ssh/known_hosts`, through ssh-keygen (which keeps known_hosts.old).
pub struct UserKnownHosts;

impl KnownHosts for UserKnownHosts {
    fn lines(&self, pattern: &str) -> Vec<String> {
        crate::host::LocalHost
            .exec(&["ssh-keygen", "-F", pattern], DEFAULT_TIMEOUT)
            .map(|o| o.stdout.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(str::to_string).collect())
            .unwrap_or_default()
    }
    fn replace(&self, pattern: &str, lines: &[String]) -> Result<(), String> {
        if !self.lines(pattern).is_empty() {
            crate::host::LocalHost.exec(&["ssh-keygen", "-R", pattern], DEFAULT_TIMEOUT).map_err(|e| e.to_string())?;
        }
        crate::views::onboard::probe::trust_host_keys(lines).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rotated {
    /// The new pin, as onboarding writes it: "SHA256:… (ED25519)".
    pub fingerprint: String,
    /// One line per key: "ssh-ed25519 SHA256:…".
    pub keys: Vec<String>,
}

/// How a server appears in known_hosts.
pub fn pattern(host: &str, port: u16) -> String {
    if port == 22 || port == 0 { host.to_string() } else { format!("[{host}]:{port}") }
}

fn root(host: &dyn Host, argv: &[&str]) -> Result<String, String> {
    host.exec_privileged(argv, &[], DEFAULT_TIMEOUT).map(|o| o.stdout).map_err(|e| match e {
        HostError::Failed { stderr, .. } => stderr.trim().to_string(),
        other => other.to_string(),
    })
}

/// The host keys sshd serves (`sshd -T`), refusing certificate setups.
pub fn served_keys(sshd_t: &str) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();
    for line in sshd_t.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("hostcertificate") => return Err("this server uses host certificates; rotate them with the CA that signs them".into()),
            Some("hostkey") => keys.extend(parts.next().map(str::to_string)),
            _ => {}
        }
    }
    if keys.is_empty() {
        return Err("sshd -T lists no host keys".into());
    }
    Ok(keys)
}

/// `@@key PATH` / public key pairs from PREPARE.
fn parse_prepared(out: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut path = None;
    for line in out.lines().map(str::trim).filter(|l| !l.is_empty()) {
        match line.strip_prefix("@@key ") {
            Some(p) => path = Some(p.to_string()),
            None => pairs.extend(path.take().map(|p| (p, line.to_string()))),
        }
    }
    pairs
}

/// Rotates one server's host keys. `fresh_login` makes a brand-new
/// connection with strict host key checking.
pub fn rotate(host: &dyn Host, address: &str, port: u16, known: &dyn KnownHosts, fresh_login: &dyn Fn() -> Result<(), String>) -> Result<Rotated, String> {
    let keys = served_keys(&root(host, &["sshd", "-T"]).map_err(|e| format!("couldn't read sshd's settings: {e}"))?)?;

    let mut argv = vec!["sh", "-c", PREPARE, "crow-hostkeys"];
    argv.extend(keys.iter().map(String::as_str));
    let prepared = parse_prepared(&root(host, &argv).map_err(|e| format!("couldn't generate new keys: {e}"))?);
    if prepared.is_empty() {
        return Err("none of sshd's host keys exist on disk".into());
    }
    let paths: Vec<&str> = prepared.iter().map(|(p, _)| p.as_str()).collect();

    // The new keys as known_hosts lines, with their fingerprints.
    let pat = pattern(address, port);
    let lines: Vec<String> = prepared.iter().map(|(_, pubkey)| {
        let mut parts = pubkey.split_whitespace();
        format!("{pat} {} {}", parts.next().unwrap_or_default(), parts.next().unwrap_or_default())
    }).collect();
    let fps: Vec<(String, String)> = lines.iter().filter_map(|l| crate::views::onboard::probe::key_fingerprint(l)).collect();
    if fps.len() != lines.len() {
        return Err("the server returned a public key Crow couldn't read; nothing was changed".into());
    }
    let (alg, fp) = fps.iter().find(|(a, _)| a == "ssh-ed25519").or(fps.first()).cloned().expect("checked non-empty");
    let rotated = Rotated {
        fingerprint: format!("{fp} ({})", alg.trim_start_matches("ssh-").to_uppercase()),
        keys: fps.iter().map(|(a, f)| format!("{a} {f}")).collect(),
    };

    let backup = format!("/etc/ssh/crow-hostkeys-{}", chrono::Utc::now().format("%Y%m%dT%H%M%S"));
    let with_keys = |script: &'static str| {
        let script = format!("{RELOAD_SSHD}\n{script}");
        let mut argv = vec!["sh", "-c", script.as_str(), "crow-hostkeys", backup.as_str()];
        argv.extend(paths.iter().copied());
        root(host, &argv)
    };
    let cleanup_new = || {
        let mut argv = vec!["rm", "-f", "--"];
        let names: Vec<String> = paths.iter().flat_map(|p| [format!("{p}.crow-new"), format!("{p}.crow-new.pub")]).collect();
        argv.extend(names.iter().map(String::as_str));
        let _ = root(host, &argv);
    };
    if let Err(e) = with_keys(SWAP) {
        cleanup_new();
        return Err(format!("the new keys weren't put in place: {e}"));
    }

    let old_lines = known.lines(&pat);
    let undo = |why: String| -> String {
        let remote = with_keys(RESTORE);
        if remote.is_ok() {
            // The old keys are back in place; the copies aren't needed.
            let _ = root(host, &["rm", "-rf", "--", &backup]);
        }
        let local = known.replace(&pat, &old_lines);
        match (remote, local) {
            (Ok(_), Ok(())) => format!("{why}. Rolled back: the old host keys are back and known_hosts is as it was."),
            (r, l) => format!(
                "{why}. Rollback didn't finish ({}{}); the old keys are kept in {backup} on the server.",
                r.err().map(|e| format!("server: {e}")).unwrap_or_default(),
                l.err().map(|e| format!(" known_hosts: {e}")).unwrap_or_default()
            ),
        }
    };
    if let Err(e) = known.replace(&pat, &lines) {
        return Err(undo(format!("known_hosts couldn't be updated ({e})")));
    }
    if let Err(e) = fresh_login() {
        return Err(undo(format!("a fresh login with the new key failed ({e})")));
    }
    // Proven: the old private keys aren't needed any more.
    let _ = root(host, &["rm", "-rf", "--", &backup]);
    Ok(rotated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::ExecOutput;
    use std::cell::RefCell;
    use std::sync::Mutex;
    use std::time::Duration;

    const ED: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIjJhjDtHEe9g+lty++J79r59GEvFC8Y6DN7jA0g7xU0";
    const ED_OLD: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIIhAhf9wRcdnIlCsZgJc6sNSFZijw9WMKP+/oZuYE8M";

    /// A server whose scripts are simulated: records what ran.
    struct FakeSsh {
        ran: Mutex<Vec<String>>,
        swap_fails: bool,
    }

    impl Host for FakeSsh {
        fn label(&self) -> String {
            "fake".into()
        }
        fn exec_stdin(&self, argv: &[&str], _stdin: &[u8], _t: Duration) -> Result<ExecOutput, HostError> {
            let cmd = argv.join(" ");
            self.ran.lock().unwrap().push(cmd.clone());
            let out = |s: String| Ok(ExecOutput { stdout: s, stderr: String::new() });
            if cmd.ends_with("sshd -T") {
                return out("port 22\nhostkey /etc/ssh/ssh_host_ed25519_key\nhostkey /etc/ssh/ssh_host_rsa_key\n".into());
            }
            if cmd.contains("ssh-keygen -q") {
                // Only the ed25519 key exists on disk.
                return out(format!("@@key /etc/ssh/ssh_host_ed25519_key\n{ED}\n"));
            }
            if cmd.contains("mv -f \"$k.crow-new\"") && self.swap_fails {
                return Err(HostError::Failed { status: 4, stderr: "sshd -t rejected the new keys; the old ones are back".into() });
            }
            out(String::new())
        }
        fn write_file_atomic(&self, _path: &str, _content: &str) -> Result<(), HostError> {
            unreachable!()
        }
    }

    impl FakeSsh {
        fn new(swap_fails: bool) -> Self {
            Self { ran: Mutex::new(Vec::new()), swap_fails }
        }
        fn ran(&self, needle: &str) -> bool {
            self.ran.lock().unwrap().iter().any(|c| c.contains(needle))
        }
    }

    struct FakeKnown(RefCell<Vec<String>>);

    impl KnownHosts for FakeKnown {
        fn lines(&self, pattern: &str) -> Vec<String> {
            self.0.borrow().iter().filter(|l| l.starts_with(&format!("{pattern} "))).cloned().collect()
        }
        fn replace(&self, pattern: &str, lines: &[String]) -> Result<(), String> {
            self.0.borrow_mut().retain(|l| !l.starts_with(&format!("{pattern} ")));
            self.0.borrow_mut().extend(lines.iter().cloned());
            Ok(())
        }
    }

    fn known() -> FakeKnown {
        FakeKnown(RefCell::new(vec![format!("[10.0.0.5]:2222 {ED_OLD}"), format!("other.host {ED_OLD}")]))
    }

    #[test]
    fn rotates_repins_and_deletes_the_old_keys() {
        let (host, kh) = (FakeSsh::new(false), known());
        let r = rotate(&host, "10.0.0.5", 2222, &kh, &|| Ok(())).unwrap();
        assert!(r.fingerprint.starts_with("SHA256:") && r.fingerprint.ends_with("(ED25519)"));
        assert_eq!(*kh.0.borrow(), vec![format!("other.host {ED_OLD}"), format!("[10.0.0.5]:2222 {ED}")]);
        assert!(host.ran("rm -rf -- /etc/ssh/crow-hostkeys-"), "old private keys removed once proven");
        assert!(!host.ran("cp -p \"$b/$n\" \"$k\" &&"), "nothing restored");
    }

    #[test]
    fn a_failed_fresh_login_puts_everything_back() {
        let (host, kh) = (FakeSsh::new(false), known());
        let err = rotate(&host, "10.0.0.5", 2222, &kh, &|| Err("Host key verification failed.".into())).unwrap_err();
        assert!(err.contains("Rolled back"), "{err}");
        assert_eq!(*kh.0.borrow(), vec![format!("other.host {ED_OLD}"), format!("[10.0.0.5]:2222 {ED_OLD}")]);
        assert!(host.ran("cp -p \"$b/$n\" \"$k\" &&"), "old keys restored on the server");
        assert!(host.ran("rm -rf --"), "the restored keys' copies are removed too");
    }

    #[test]
    fn rejected_keys_change_nothing_locally() {
        let (host, kh) = (FakeSsh::new(true), known());
        let err = rotate(&host, "10.0.0.5", 2222, &kh, &|| Ok(())).unwrap_err();
        assert!(err.contains("weren't put in place"));
        assert_eq!(kh.0.borrow()[0], format!("[10.0.0.5]:2222 {ED_OLD}"));
        assert!(host.ran("rm -f -- /etc/ssh/ssh_host_ed25519_key.crow-new"));
    }

    #[test]
    fn certificates_are_left_to_their_ca() {
        assert!(served_keys("hostkey /etc/ssh/k\nhostcertificate /etc/ssh/k-cert.pub\n").is_err());
        assert_eq!(served_keys("hostkey /a\nhostkey /b\n").unwrap(), vec!["/a", "/b"]);
        assert_eq!(pattern("web", 22), "web");
    }

    /// Live run against a throwaway sshd container. Opt-in:
    ///   docker run -d --name crow-hk -p 127.0.0.1:22299:22 alpine (with
    ///   openssh, a key in /root/.ssh/authorized_keys and a `systemctl` that
    ///   HUPs sshd), then
    ///   CROW_LIVE_HOSTKEYS=/path/to/private_key cargo test live_host_key_rotation -- --ignored
    #[test]
    #[ignore]
    fn live_host_key_rotation() {
        let key = std::env::var("CROW_LIVE_HOSTKEYS").expect("set CROW_LIVE_HOSTKEYS=/path/to/key");
        let dir = std::env::temp_dir().join(format!("crow-live-hk-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let kh_path = dir.join("known_hosts");
        let scan = std::process::Command::new("ssh-keyscan").args(["-p", "22299", "127.0.0.1"]).output().unwrap();
        std::fs::write(&kh_path, &scan.stdout).unwrap();

        struct FileKnown(std::path::PathBuf);
        impl KnownHosts for FileKnown {
            fn lines(&self, pattern: &str) -> Vec<String> {
                std::fs::read_to_string(&self.0).unwrap_or_default().lines().filter(|l| l.starts_with(&format!("{pattern} "))).map(str::to_string).collect()
            }
            fn replace(&self, pattern: &str, lines: &[String]) -> Result<(), String> {
                let mut kept: Vec<String> = std::fs::read_to_string(&self.0).unwrap_or_default().lines().filter(|l| !l.starts_with(&format!("{pattern} "))).map(str::to_string).collect();
                kept.extend(lines.iter().cloned());
                std::fs::write(&self.0, kept.join("\n") + "\n").map_err(|e| e.to_string())
            }
        }
        let login = |kh: &std::path::Path| -> Result<(), String> {
            let out = std::process::Command::new("ssh")
                .args(["-F", "/dev/null", "-i", &key, "-p", "22299", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes", "-o", "IdentityAgent=none"])
                .arg("-o").arg(format!("UserKnownHostsFile={}", kh.display()))
                .args(["root@127.0.0.1", "true"])
                .output()
                .unwrap();
            if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
        };
        login(&kh_path).expect("logs in with the scanned keys first");
        let before = std::fs::read_to_string(&kh_path).unwrap();

        let srv = crate::vault::ServerRecord { name: "crow-hk".into(), host: "127.0.0.1".into(), tags: vec!["test-node".into(), "docker".into()], ..Default::default() };
        let host = crate::host::host_for(&srv);
        let known = FileKnown(kh_path.clone());

        // A login that fails is undone: the old keys work again.
        let err = rotate(host.as_ref(), "127.0.0.1", 22299, &known, &|| Err("simulated".into())).unwrap_err();
        assert!(err.contains("Rolled back"), "{err}");
        let keys = |text: &str| {
            let mut k: Vec<String> = text.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).map(str::to_string).collect();
            k.sort();
            k
        };
        assert_eq!(keys(&std::fs::read_to_string(&kh_path).unwrap()), keys(&before));
        std::thread::sleep(std::time::Duration::from_millis(500));
        login(&kh_path).expect("old keys are back");

        // A real rotation: new keys, pinned, a strict fresh login works.
        let r = rotate(host.as_ref(), "127.0.0.1", 22299, &known, &|| login(&kh_path)).unwrap();
        assert!(keys(&std::fs::read_to_string(&kh_path).unwrap()).iter().all(|k| !keys(&before).contains(k)), "every key is new");
        assert_eq!(r.keys.len(), 3, "{:?}", r.keys);
        let left = host.exec(&["sh", "-c", "ls /etc/ssh | grep -c crow || true"], DEFAULT_TIMEOUT).unwrap().stdout;
        assert_eq!(left.trim(), "0", "no .crow-new files or backups left");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
