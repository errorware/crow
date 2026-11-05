//! What a server's terminal runs (ERR-93): its own transport, so the
//! terminal gets exactly the access Crow's other screens have.

use super::session::Launch;
use crate::host::{transport_kind, SshHost, TransportKind};
use crate::vault::ServerRecord;

/// A login shell that prefers bash where it exists.
const SHELL: &str = "command -v bash >/dev/null 2>&1 && exec bash -l || exec sh -l";

/// The user's own login shell, from the password database.
const LOGIN_SHELL: &str = "s=$(getent passwd \"$(id -un)\" | cut -d: -f7); exec \"${s:-/bin/sh}\" -l";

pub fn launch_for(server: &ServerRecord) -> Result<Launch, String> {
    match transport_kind(server) {
        TransportKind::Ssh => {
            let ssh_host = SshHost::for_server(server);
            let via_hop = ssh_host.via_hop();
            let (program, args) = ssh_host.interactive_command()?;
            let span = crate::wire::begin_checked(crate::wire::WireStart {
                category: crate::wire::WireCategory::Terminal,
                protocol: "SSH-PTY",
                caller: crate::wire::current_caller("terminal.open"),
                target_label: format!("pty:{}@{}", server.login_user, server.name),
                remote_host: server.host.clone(),
                remote_port: if server.port == 0 { 22 } else { server.port },
                via_hop,
                summary: format!("interactive SSH PTY -> {} ({})", server.name, server.host),
                request_payload: format!("{program} {}", args.join(" ")),
            })?;
            span.finish_ok(0, 0, "interactive PTY session spawned");
            Ok(Launch { program, args })
        }
        TransportKind::Container => {
            let engine = if server.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
            let (program, args) = crate::sandbox::host_terminal(engine.into(), ["exec", "-it", "-e", "TERM=xterm-256color", "--", &server.name, "sh", "-c", SHELL].iter().map(|s| s.to_string()).collect());
            Ok(Launch { program, args })
        }
        TransportKind::Local if crate::sandbox::flatpak() => {
            // The sandbox's SHELL isn't the user's: ask the computer.
            let (program, args) = crate::sandbox::host_terminal("sh".into(), vec!["-c".into(), LOGIN_SHELL.into()]);
            Ok(Launch { program, args })
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
