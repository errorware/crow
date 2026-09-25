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

/// A `systemctl list-units` line without its leading status marker.
pub fn strip_unit_marker(line: &str) -> &str {
    let t = line.trim();
    t.strip_prefix('●').or_else(|| t.strip_prefix("* ")).map(str::trim_start).unwrap_or(t)
}

/// Parses the output of `systemctl list-units --type=service --all --no-legend --no-pager`
pub fn parse_systemctl_services(stdout: &str) -> Vec<ServiceUnit> {
    let mut units = Vec::new();

    for line in stdout.lines() {
        // systemctl marks failed and not-found units with a leading "●" ("*"
        // without a UTF-8 locale); it isn't part of the unit name.
        let trimmed = strip_unit_marker(line);
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
            // Not read yet: systemctl list-units carries no per-unit usage.
            pid: "—".to_string(),
            cpu: "—".to_string(),
            mem: "—".to_string(),
            rss: "—".to_string(),
            uptime: "—".to_string(),
            description,
            is_focused: false,
            show_confirm: false,
        });
    }

    units
}

/// Parses output of `ps -eo pid,ppid,user,%cpu,%mem,rss,stat,time,comm --sort=-%cpu`.
/// Kernel threads are kthreadd (PID 2) and its children.
pub fn parse_ps_processes(stdout: &str) -> Vec<ProcessUnit> {
    let mut procs = Vec::new();

    for line in stdout.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let pid: u32 = match parts[0].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let ppid: u32 = parts[1].parse().unwrap_or(0);

        let user = parts[2].to_string();
        let cpu: f32 = parts[3].parse().unwrap_or(0.0);
        let mem: f32 = parts[4].parse().unwrap_or(0.0);
        let rss_kb: u64 = parts[5].parse().unwrap_or(0);
        let stat = parts[6].to_string();
        let time = parts[7].to_string();
        let command = parts[8..].join(" ");

        procs.push(ProcessUnit {
            pid,
            user,
            cpu,
            mem,
            rss: format_rss_kb(rss_kb),
            stat,
            time,
            command,
            is_kernel: pid == 2 || ppid == 2,
            is_focused: false,
            show_confirm: false,
        });
    }

    procs
}

/// Parses output of `ss -tulpn` or `ss -tinp`
pub fn parse_ss_sockets(stdout: &str) -> Vec<SocketUnit> {
    let mut sockets = Vec::new();

    let mut current_sock: Option<SocketUnit> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Netid") {
            continue;
        }

        // A line starting with whitespace or a tab is the TCP info extension line (from ss -i)
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(ref mut sock) = current_sock {
                if let Some(bs) = parse_bytes_metric(trimmed, "bytes_sent:") {
                    sock.bytes_sent = Some(bs);
                }
                if let Some(br) = parse_bytes_metric(trimmed, "bytes_received:") {
                    sock.bytes_recv = Some(br);
                }
            }
            continue;
        }

        // If we had a previous socket pending, push it
        if let Some(sock) = current_sock.take() {
            sockets.push(sock);
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

        current_sock = Some(SocketUnit {
            protocol: proto_raw,
            state,
            local_addr,
            local_port,
            peer_addr,
            peer_port,
            process: process_name,
            pid,
            is_focused: false,
            bytes_sent: None,
            bytes_recv: None,
        });
    }

    if let Some(sock) = current_sock {
        sockets.push(sock);
    }

    sockets
}

fn parse_bytes_metric(line: &str, prefix: &str) -> Option<u64> {
    let idx = line.find(prefix)?;
    let rem = &line[idx + prefix.len()..];
    let num_str: String = rem.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse::<u64>().ok()
}

pub fn categorize_peer(addr: &str) -> super::models::PeerCategory {
    let clean = addr.trim_start_matches('[').trim_end_matches(']');
    let ip_str = clean.split('%').next().unwrap_or(clean);

    if ip_str == "localhost"
        || ip_str.starts_with("127.")
        || ip_str == "::1"
        || ip_str == "0.0.0.0"
        || ip_str == "::"
        || ip_str == "*"
    {
        return super::models::PeerCategory::Loopback;
    }

    if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => {
                if v4.is_loopback() {
                    super::models::PeerCategory::Loopback
                } else if v4.is_private() || v4.is_link_local() {
                    super::models::PeerCategory::Private
                } else {
                    super::models::PeerCategory::Public
                }
            }
            std::net::IpAddr::V6(v6) => {
                if v6.is_loopback() {
                    super::models::PeerCategory::Loopback
                } else if (v6.segments()[0] & 0xfe00) == 0xfc00 // Unique Local IPv6 fc00::/7
                    || (v6.segments()[0] & 0xffc0) == 0xfe80 // Link-Local IPv6 fe80::/10
                {
                    super::models::PeerCategory::Private
                } else {
                    super::models::PeerCategory::Public
                }
            }
        }
    } else {
        super::models::PeerCategory::Public
    }
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
const LIST_PROCESSES: &[&str] = &["ps", "-eo", "pid,ppid,user,%cpu,%mem,rss,stat,time,comm", "--sort=-%cpu"];
const LIST_SOCKETS: &[&str] = &["ss", "-tulpn"];
const LIST_ESTABLISHED_SOCKETS: &[&str] = &["ss", "-tinp", "state", "established"];

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

/// Live collection of network sockets for the active server: combines listening
/// sockets (`ss -tulpn`) with established connections (`ss -tinp state established`).
pub fn collect_sockets_for_server(server: &ServerRecord) -> Vec<SocketUnit> {
    let mut all = collect(server, LIST_SOCKETS, parse_ss_sockets);
    let mut established = collect(server, LIST_ESTABLISHED_SOCKETS, parse_ss_sockets);
    all.append(&mut established);
    all
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
    fn failed_and_not_found_units_keep_their_names() {
        let sample = "\
● apport.service        not-found inactive dead   apport.service
● clamav.service        loaded    failed   failed Clam AntiVirus Daemon
* legacy.service        loaded    failed   failed Legacy (no UTF-8 locale)
  cron.service          loaded    active   running Regular background program processing daemon";
        let units = parse_systemctl_services(sample);
        let names: Vec<&str> = units.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names, ["apport.service", "clamav.service", "legacy.service", "cron.service"]);
        let live = crate::metrics::collector::parse_live_services(sample);
        assert_eq!(live[1].name, "clamav.service");
        assert_eq!(live[1].status, "FAILED");
    }

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
    PID    PPID USER     %CPU %MEM   RSS STAT     TIME COMMAND
1714869 3012 harakiri 28.0  0.7 221360 Ssl+ 00:00:56 crow
   8309 3012 harakiri  3.1  1.8 517520 Sl   07:59:59 brave
    134    2 root      0.0  0.0     0 I<   00:00:00 kworker/R-kblockd";

        let procs = parse_ps_processes(sample);
        assert_eq!(procs.len(), 3);
        assert!(!procs[0].is_kernel && procs[2].is_kernel);
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

    #[test]
    fn test_parse_ss_established_with_traffic() {
        let sample = "\
Netid State  Recv-Q Send-Q Local Address:Port Peer Address:Port Process
tcp   ESTAB  0      0        10.0.0.15:22      10.0.0.2:54321 users:((\"sshd\",pid=1234,fd=4))
\t bbr wscale:7,7 rto:200 rtt:0.1/0.05 bytes_sent:45600 bytes_received:123000
tcp   ESTAB  0      0        10.0.0.15:58920   1.1.1.1:443    users:((\"curl\",pid=5678,fd=3))
\t cubic bytes_sent:800 bytes_received:4200";

        let sockets = parse_ss_sockets(sample);
        assert_eq!(sockets.len(), 2);

        // SSH connection (incoming)
        assert_eq!(sockets[0].protocol, "TCP");
        assert_eq!(sockets[0].state, "ESTAB");
        assert_eq!(sockets[0].local_addr, "10.0.0.15");
        assert_eq!(sockets[0].local_port, "22");
        assert_eq!(sockets[0].peer_addr, "10.0.0.2");
        assert_eq!(sockets[0].peer_port, "54321");
        assert_eq!(sockets[0].process, "sshd");
        assert_eq!(sockets[0].pid, Some(1234));
        assert_eq!(sockets[0].bytes_sent, Some(45600));
        assert_eq!(sockets[0].bytes_recv, Some(123000));

        // Outgoing curl connection
        assert_eq!(sockets[1].protocol, "TCP");
        assert_eq!(sockets[1].state, "ESTAB");
        assert_eq!(sockets[1].local_addr, "10.0.0.15");
        assert_eq!(sockets[1].local_port, "58920");
        assert_eq!(sockets[1].peer_addr, "1.1.1.1");
        assert_eq!(sockets[1].peer_port, "443");
        assert_eq!(sockets[1].process, "curl");
        assert_eq!(sockets[1].pid, Some(5678));
        assert_eq!(sockets[1].bytes_sent, Some(800));
        assert_eq!(sockets[1].bytes_recv, Some(4200));
    }

    #[test]
    fn test_categorize_peer() {
        use super::super::models::PeerCategory;

        // Loopback
        assert_eq!(categorize_peer("127.0.0.1"), PeerCategory::Loopback);
        assert_eq!(categorize_peer("127.0.0.53"), PeerCategory::Loopback);
        assert_eq!(categorize_peer("::1"), PeerCategory::Loopback);

        // Private IPv4 (RFC 1918)
        assert_eq!(categorize_peer("10.0.0.1"), PeerCategory::Private);
        assert_eq!(categorize_peer("192.168.1.100"), PeerCategory::Private);
        assert_eq!(categorize_peer("172.16.0.1"), PeerCategory::Private);
        assert_eq!(categorize_peer("172.31.255.255"), PeerCategory::Private);

        // Link-local
        assert_eq!(categorize_peer("169.254.1.1"), PeerCategory::Private);
        assert_eq!(categorize_peer("fe80::1"), PeerCategory::Private);

        // Public
        assert_eq!(categorize_peer("1.1.1.1"), PeerCategory::Public);
        assert_eq!(categorize_peer("8.8.8.8"), PeerCategory::Public);
        assert_eq!(categorize_peer("142.250.190.46"), PeerCategory::Public);
        assert_eq!(categorize_peer("2607:f8b0:4005:805::200e"), PeerCategory::Public);
    }

    #[test]
    fn test_resolve_connection_map_items() {
        use super::super::models::{ConnectionDirection, SocketUnit};
        use super::super::sockets_map::resolve_connection_map_items;

        let sockets = vec![
            // Listening socket on port 80
            SocketUnit {
                protocol: "TCP".to_string(),
                state: "LISTEN".to_string(),
                local_addr: "0.0.0.0".to_string(),
                local_port: "80".to_string(),
                peer_addr: "*".to_string(),
                peer_port: "*".to_string(),
                process: "nginx".to_string(),
                pid: Some(100),
                is_focused: false,
                bytes_sent: None,
                bytes_recv: None,
            },
            // Established incoming connection to port 80
            SocketUnit {
                protocol: "TCP".to_string(),
                state: "ESTAB".to_string(),
                local_addr: "192.168.1.10".to_string(),
                local_port: "80".to_string(),
                peer_addr: "192.168.1.50".to_string(),
                peer_port: "54321".to_string(),
                process: "nginx".to_string(),
                pid: Some(101),
                is_focused: false,
                bytes_sent: Some(1000),
                bytes_recv: Some(200),
            },
            // Established outgoing connection to remote 1.1.1.1:443
            SocketUnit {
                protocol: "TCP".to_string(),
                state: "ESTAB".to_string(),
                local_addr: "192.168.1.10".to_string(),
                local_port: "49152".to_string(),
                peer_addr: "1.1.1.1".to_string(),
                peer_port: "443".to_string(),
                process: "curl".to_string(),
                pid: Some(200),
                is_focused: false,
                bytes_sent: Some(500),
                bytes_recv: Some(1500),
            },
        ];

        let items = resolve_connection_map_items(&sockets);
        assert_eq!(items.len(), 2);

        // First item is incoming (local port 80 was in listening set)
        assert_eq!(items[0].direction, ConnectionDirection::Incoming);
        assert_eq!(items[0].peer_category, super::super::models::PeerCategory::Private);
        assert_eq!(items[0].socket.peer_addr, "192.168.1.50");

        // Second item is outgoing (local port 49152 was NOT in listening set)
        assert_eq!(items[1].direction, ConnectionDirection::Outgoing);
        assert_eq!(items[1].peer_category, super::super::models::PeerCategory::Public);
        assert_eq!(items[1].socket.peer_addr, "1.1.1.1");
    }
}
