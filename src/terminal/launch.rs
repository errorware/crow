//! What a server's terminal runs (ERR-93): its own transport, so the
//! terminal gets exactly the access Crow's other screens have.

use super::session::Launch;
use crate::host::{transport_kind, SshHost, TransportKind};
use crate::vault::ServerRecord;

/// A login shell that prefers bash where it exists.
const SHELL: &str = "command -v bash >/dev/null 2>&1 && exec bash -l || exec sh -l";

pub fn launch_for(server: &ServerRecord) -> Result<Launch, String> {
    match transport_kind(server) {
        TransportKind::Ssh => {
            let (program, args) = SshHost::for_server(server).interactive_command()?;
            Ok(Launch { program, args })
        }
        TransportKind::Container => {
            let engine = if server.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
            Ok(Launch {
                program: engine.into(),
                args: ["exec", "-it", "-e", "TERM=xterm-256color", "--", &server.name, "sh", "-c", SHELL].iter().map(|s| s.to_string()).collect(),
            })
        }
        TransportKind::Local => {
            let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/bin/sh".into());
            Ok(Launch { program: shell, args: vec!["-l".into()] })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_terminals_use_crows_connection_options() {
        let srv = ServerRecord { id: "s".into(), name: "web".into(), host: "10.0.0.5".into(), port: 2200, login_user: "ops".into(), auth_method: "agent".into(), ..Default::default() };
        let l = launch_for(&srv).unwrap();
        assert_eq!(l.program, "ssh");
        assert_eq!(l.args[0], "-tt");
        assert!(l.args.windows(2).any(|w| w == ["-o", "StrictHostKeyChecking=yes"]));
        assert!(l.args.windows(2).any(|w| w == ["-o", "ForwardAgent=no"]));
        assert_eq!(&l.args[l.args.len() - 2..], ["--", "10.0.0.5"], "the address can't be read as an option");
    }

    #[test]
    fn containers_exec_a_login_shell() {
        let srv = ServerRecord { name: "lab-1".into(), host: "127.0.0.1".into(), tags: vec!["test-node".into(), "docker".into()], ..Default::default() };
        let l = launch_for(&srv).unwrap();
        assert_eq!((l.program.as_str(), &l.args[..3]), ("docker", &["exec".to_string(), "-it".into(), "-e".into()][..]));
    }

    #[test]
    fn password_servers_have_no_terminal() {
        let srv = ServerRecord { host: "10.0.0.5".into(), auth_method: "password".into(), ..Default::default() };
        assert!(launch_for(&srv).is_err());
    }
}
