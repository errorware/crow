use crate::host::{host_for, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;
use super::models::{ProcessUnit, ServiceUnit, SocketUnit};

/// Formats raw RSS in kilobytes into human-friendly string (e.g. 196M, 1.4G, 840K).
pub fn format_rss_kb(rss_kb: u64) -> String {
    if rss_kb >= 1024 * 1024 {
        format!("{:.1}G", rss_kb as f64 / (1024.0 * 1024.0))
    } else if rss_kb >= 1024 {
        format!("{:.0}M", rss_kb as f64 / 1024.0)
    } else {
        format!("{}K", rss_kb)
    }
}

/// Parses the output of `systemctl list-units --type=service --all --no-legend --no-pager`
pub fn parse_systemctl_services(stdout: &str) -> Vec<ServiceUnit> {
    let mut units = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Split by whitespace: UNIT LOAD ACTIVE SUB [DESCRIPTION...]
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let unit_name = parts[0].to_string();
        let _load_state = parts[1];
        let active_state = parts[2];
        let sub_state = parts[3];
        let description = if parts.len() > 4 {
            parts[4..].join(" ")
        } else {
            String::new()
        };

        let (status, color_hex) = if active_state == "active" && sub_state == "running" {
            ("ACTIVE", 0x4ade80)
        } else if active_state == "active" && sub_state == "exited" {
            ("INACTIVE", 0x71717a)
        } else if active_state == "active" {
            ("ACTIVE", 0x4ade80)
        } else if sub_state == "failed" {
            ("FAILED", 0xf87171)
        } else if sub_state == "degraded" {
            ("DEGRADED", 0xfacc15)
        } else {
            ("INACTIVE", 0x71717a)
        };

        units.push(ServiceUnit {
            name: unit_name,
            status: status.to_string(),
            status_color_hex: color_hex,
            pid: "—".to_string(),
            cpu: "0.0".to_string(),
            mem: "0.0".to_string(),
            rss: "—".to_string(),
            uptime: "—".to_string(),
            description,
            is_focused: false,
            show_confirm: false,
        });
    }

    units
}

/// Parses output of `ps -eo pid,user,%cpu,%mem,rss,stat,time,comm --sort=-%cpu`
pub fn parse_ps_processes(stdout: &str) -> Vec<ProcessUnit> {
    let mut procs = Vec::new();

    for line in stdout.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 8 {
            continue;
        }

        let pid: u32 = match parts[0].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let user = parts[1].to_string();
        let cpu: f32 = parts[2].parse().unwrap_or(0.0);
        let mem: f32 = parts[3].parse().unwrap_or(0.0);
        let rss_kb: u64 = parts[4].parse().unwrap_or(0);
        let stat = parts[5].to_string();
        let time = parts[6].to_string();
        let command = parts[7..].join(" ");

        procs.push(ProcessUnit {
            pid,
            user,
            cpu,
            mem,
            rss: format_rss_kb(rss_kb),
            stat,
            time,
            command,
            is_focused: false,
            show_confirm: false,
        });
    }

    procs
}

/// Parses output of `ss -tulpn` or `ss -tulnp`
pub fn parse_ss_sockets(stdout: &str) -> Vec<SocketUnit> {
    let mut sockets = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Netid") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let proto_raw = parts[0].to_uppercase();
        let state = parts[1].to_string();
        let local_endpoint = parts[4];
        let peer_endpoint = if parts.len() >= 6 { parts[5] } else { "*:*" };

        // Split local address and port
        let (local_addr, local_port) = split_endpoint(local_endpoint);
        let (peer_addr, peer_port) = split_endpoint(peer_endpoint);

        // Parse process and pid from remainder, e.g. users:(("sshd",pid=1,fd=3))
        let remaining = if parts.len() >= 7 {
            parts[6..].join(" ")
        } else {
            String::new()
        };

        let (process_name, pid) = parse_users_field(&remaining);

        sockets.push(SocketUnit {
            protocol: proto_raw,
            state,
            local_addr,
            local_port,
            peer_addr,
            peer_port,
            process: process_name,
            pid,
            is_focused: false,
        });
    }

    sockets
}

fn split_endpoint(endpoint: &str) -> (String, String) {
    if let Some(idx) = endpoint.rfind(':') {
        let addr = &endpoint[..idx];
        let port = &endpoint[idx + 1..];
        (addr.to_string(), port.to_string())
    } else {
        (endpoint.to_string(), "—".to_string())
    }
}

fn parse_users_field(field: &str) -> (String, Option<u32>) {
    if field.is_empty() {
        return ("—".to_string(), None);
    }

    let mut name = "—".to_string();
    let mut pid = None;

    // Search for ("proc_name",pid=1234
    if let Some(start_quote) = field.find('"') {
        if let Some(end_quote) = field[start_quote + 1..].find('"') {
            name = field[start_quote + 1..start_quote + 1 + end_quote].to_string();
        }
    }

    if let Some(pid_idx) = field.find("pid=") {
        let after_pid = &field[pid_idx + 4..];
        let num_str: String = after_pid.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(p) = num_str.parse::<u32>() {
            pid = Some(p);
        }
    }

    (name, pid)
}

const LIST_SERVICES: &[&str] = &["systemctl", "list-units", "--type=service", "--all", "--no-legend", "--no-pager"];
const LIST_PROCESSES: &[&str] = &["ps", "-eo", "pid,user,%cpu,%mem,rss,stat,time,comm", "--sort=-%cpu"];
const LIST_SOCKETS: &[&str] = &["ss", "-tulpn"];

/// Runs a read-only listing on the server and parses it. A failed command
/// (or an unreachable server) yields an empty table, never stand-in rows.
fn collect<T>(server: &ServerRecord, argv: &[&str], parse: fn(&str) -> Vec<T>) -> Vec<T> {
    host_for(server).exec(argv, DEFAULT_TIMEOUT).map(|out| parse(&out.stdout)).unwrap_or_default()
}

/// Live collection of systemd services for the active server
pub fn collect_services_for_server(server: &ServerRecord) -> Vec<ServiceUnit> {
    collect(server, LIST_SERVICES, parse_systemctl_services)
}

/// Live collection of processes for the active server
pub fn collect_processes_for_server(server: &ServerRecord) -> Vec<ProcessUnit> {
    collect(server, LIST_PROCESSES, parse_ps_processes)
}

/// Live collection of network sockets for the active server
pub fn collect_sockets_for_server(server: &ServerRecord) -> Vec<SocketUnit> {
    collect(server, LIST_SOCKETS, parse_ss_sockets)
}

/// Runs a systemctl lifecycle action (start/stop/restart/reload) against a unit
pub fn systemctl_service_action(server: &ServerRecord, unit: &str, action: &str) -> Result<String, String> {
    let host = host_for(server);
    host.exec_privileged(&["systemctl", action, unit], &[], DEFAULT_TIMEOUT).map_err(|e| e.to_string())?;
    Ok(format!("{} unit {} on {}", action, unit, host.label()))
}

/// Kills or signals a process
pub fn terminate_process(server: &ServerRecord, pid: u32, signal: i32) -> Result<String, String> {
    let host = host_for(server);
    host.exec_privileged(&["kill", &format!("-{}", signal), &pid.to_string()], &[], DEFAULT_TIMEOUT).map_err(|e| e.to_string())?;
    Ok(format!("Sent signal {} to PID {} on {}", signal, pid, host.label()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_systemctl_services() {
        let sample = "\
  accounts-daemon.service  loaded active running Accounts Service
  alsa-restore.service     loaded active exited  Save/Restore Sound Card State
  clamav.service           loaded failed failed  Clam AntiVirus Daemon";

        let units = parse_systemctl_services(sample);
        assert_eq!(units.len(), 3);
        assert_eq!(units[0].name, "accounts-daemon.service");
        assert_eq!(units[0].status, "ACTIVE");
        assert_eq!(units[1].name, "alsa-restore.service");
        assert_eq!(units[1].status, "INACTIVE");
        assert_eq!(units[2].name, "clamav.service");
        assert_eq!(units[2].status, "FAILED");
    }

    #[test]
    fn test_parse_ps_processes() {
        let sample = "\
    PID USER     %CPU %MEM   RSS STAT     TIME COMMAND
1714869 harakiri 28.0  0.7 221360 Ssl+ 00:00:56 crow
   8309 harakiri  3.1  1.8 517520 Sl   07:59:59 brave";

        let procs = parse_ps_processes(sample);
        assert_eq!(procs.len(), 2);
        assert_eq!(procs[0].pid, 1714869);
        assert_eq!(procs[0].user, "harakiri");
        assert_eq!(procs[0].cpu, 28.0);
        assert_eq!(procs[0].mem, 0.7);
        assert_eq!(procs[0].command, "crow");
        assert_eq!(procs[1].pid, 8309);
        assert_eq!(procs[1].command, "brave");
    }

    #[test]
    fn test_parse_ss_sockets() {
        let sample = "\
Netid State  Recv-Q Send-Q Local Address:Port Peer Address:Port Process
tcp   LISTEN 0      128          0.0.0.0:22        0.0.0.0:*    users:((\"sshd\",pid=764,fd=3))
udp   UNCONN 0      0       224.0.0.251:5353       0.0.0.0:*    users:((\"brave\",pid=8309,fd=139))";

        let sockets = parse_ss_sockets(sample);
        assert_eq!(sockets.len(), 2);
        assert_eq!(sockets[0].protocol, "TCP");
        assert_eq!(sockets[0].state, "LISTEN");
        assert_eq!(sockets[0].local_port, "22");
        assert_eq!(sockets[0].process, "sshd");
        assert_eq!(sockets[0].pid, Some(764));

        assert_eq!(sockets[1].protocol, "UDP");
        assert_eq!(sockets[1].local_port, "5353");
        assert_eq!(sockets[1].process, "brave");
        assert_eq!(sockets[1].pid, Some(8309));
    }
}
