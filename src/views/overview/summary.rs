//! At-a-glance numbers for the Overview dashboard, derived from the same
//! services / processes / sockets tables the dedicated pages show.

use super::models::{ProcessUnit, ServiceUnit, SocketUnit};

/// How many items each "top" list on the dashboard shows.
pub const TOP_N: usize = 5;

#[derive(Debug, Default, PartialEq)]
pub struct ServiceSummary {
    pub total: usize,
    pub active: usize,
    pub failed: usize,
    pub degraded: usize,
    pub inactive: usize,
    /// Failed units first, then degraded ones — what needs attention.
    pub attention: Vec<(String, String)>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ProcessSummary {
    pub total: usize,
    /// Zombie (Z) processes: exited but not reaped by their parent.
    pub zombies: usize,
    /// Processes in uninterruptible sleep (D), usually blocked on I/O.
    pub blocked: usize,
    /// (pid, command, cpu %), highest CPU first.
    pub top_cpu: Vec<(u32, String, f32)>,
    /// (pid, command, mem %, rss), highest memory first.
    pub top_mem: Vec<(u32, String, f32, String)>,
}

/// Where a listening socket can be reached from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exposure {
    /// Bound to every interface (0.0.0.0, *, ::): reachable from any network.
    AllInterfaces,
    /// Bound to one interface or address (a bridge, a LAN IP, multicast).
    Interface,
    /// Loopback only.
    Local,
}

impl Exposure {
    pub fn label(&self) -> &'static str {
        match self {
            Exposure::AllInterfaces => "ALL INTERFACES",
            Exposure::Interface => "INTERFACE",
            Exposure::Local => "LOCAL",
        }
    }
}

/// A listening socket as the dashboard shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct ListeningPort {
    /// Stable id the Sockets page uses to focus this row.
    pub socket_id: String,
    pub protocol: String,
    pub address: String,
    pub port: String,
    pub process: String,
    pub exposure: Exposure,
}

#[derive(Debug, Default, PartialEq)]
pub struct SocketSummary {
    pub listening: usize,
    pub all_interfaces: usize,
    pub interface: usize,
    pub local_only: usize,
    /// Widest exposure first, then by port number.
    pub ports: Vec<ListeningPort>,
}

pub fn summarize_services(services: &[ServiceUnit]) -> ServiceSummary {
    let mut s = ServiceSummary { total: services.len(), ..Default::default() };
    for svc in services {
        match svc.status.as_str() {
            "ACTIVE" => s.active += 1,
            "FAILED" => s.failed += 1,
            "DEGRADED" => s.degraded += 1,
            _ => s.inactive += 1,
        }
    }
    for status in ["FAILED", "DEGRADED"] {
        s.attention.extend(services.iter().filter(|svc| svc.status == status).map(|svc| (svc.name.clone(), svc.status.clone())));
    }
    s
}

/// Crow samples processes with `ps`, which always shows up at the top of its
/// own snapshot (its CPU % is measured over a lifetime of milliseconds).
fn is_sampler(p: &ProcessUnit) -> bool {
    p.command == "ps"
}

pub fn summarize_processes(processes: &[ProcessUnit]) -> ProcessSummary {
    let mut by_cpu: Vec<&ProcessUnit> = processes.iter().filter(|p| !is_sampler(p)).collect();
    by_cpu.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));
    let mut by_mem: Vec<&ProcessUnit> = processes.iter().collect();
    by_mem.sort_by(|a, b| b.mem.total_cmp(&a.mem));
    ProcessSummary {
        total: processes.len(),
        zombies: processes.iter().filter(|p| p.stat.starts_with('Z')).count(),
        blocked: processes.iter().filter(|p| p.stat.starts_with('D')).count(),
        top_cpu: by_cpu.iter().take(TOP_N).map(|p| (p.pid, p.command.clone(), p.cpu)).collect(),
        top_mem: by_mem.iter().take(TOP_N).map(|p| (p.pid, p.command.clone(), p.mem, p.rss.clone())).collect(),
    }
}

/// The id the Sockets page uses for a row (`protocol:port:index`).
pub fn socket_id(sock: &SocketUnit, index: usize) -> String {
    format!("{}:{}:{}", sock.protocol, sock.local_port, index)
}

/// A socket `ss -l` lists as bound for incoming traffic: TCP in LISTEN, and
/// UDP, which `ss` shows as UNCONN.
pub fn is_listening(sock: &SocketUnit) -> bool {
    sock.state == "LISTEN" || (sock.state == "UNCONN" && sock.protocol.starts_with("UDP"))
}

pub fn exposure(addr: &str) -> Exposure {
    let a = addr.trim_start_matches('[').trim_end_matches(']');
    // `ss` appends %iface to addresses bound to one device (e.g. 0.0.0.0%virbr0).
    let (ip, iface) = match a.split_once('%') {
        Some((ip, iface)) => (ip, Some(iface)),
        None => (a, None),
    };
    if ip.starts_with("127.") || ip == "::1" || ip == "localhost" || iface == Some("lo") {
        Exposure::Local
    } else if iface.is_none() && matches!(ip, "0.0.0.0" | "*" | "::" | "") {
        Exposure::AllInterfaces
    } else {
        Exposure::Interface
    }
}

pub fn summarize_sockets(sockets: &[SocketUnit]) -> SocketSummary {
    let mut ports: Vec<ListeningPort> = sockets
        .iter()
        .enumerate()
        .filter(|(_, s)| is_listening(s))
        .map(|(i, s)| ListeningPort {
            socket_id: socket_id(s, i),
            protocol: s.protocol.clone(),
            address: s.local_addr.clone(),
            port: s.local_port.clone(),
            process: s.process.clone(),
            exposure: exposure(&s.local_addr),
        })
        .collect();
    ports.sort_by(|a, b| {
        a.exposure
            .cmp(&b.exposure)
            .then_with(|| a.port.parse::<u32>().unwrap_or(u32::MAX).cmp(&b.port.parse::<u32>().unwrap_or(u32::MAX)))
    });
    let count = |e: Exposure| ports.iter().filter(|p| p.exposure == e).count();
    SocketSummary {
        listening: ports.len(),
        all_interfaces: count(Exposure::AllInterfaces),
        interface: count(Exposure::Interface),
        local_only: count(Exposure::Local),
        ports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc(name: &str, status: &str) -> ServiceUnit {
        ServiceUnit {
            name: name.into(),
            status: status.into(),
            status_color_hex: 0,
            pid: String::new(),
            cpu: String::new(),
            mem: String::new(),
            rss: String::new(),
            uptime: String::new(),
            description: String::new(),
            is_focused: false,
            show_confirm: false,
        }
    }

    fn proc(pid: u32, cpu: f32, mem: f32, stat: &str) -> ProcessUnit {
        ProcessUnit {
            pid,
            user: "root".into(),
            cpu,
            mem,
            rss: format!("{pid}M"),
            stat: stat.into(),
            time: String::new(),
            command: format!("cmd{pid}"),
            is_focused: false,
            show_confirm: false,
        }
    }

    fn sock(state: &str, addr: &str, port: &str) -> SocketUnit {
        SocketUnit {
            protocol: "TCP".into(),
            state: state.into(),
            local_addr: addr.into(),
            local_port: port.into(),
            peer_addr: "0.0.0.0".into(),
            peer_port: "*".into(),
            process: "p".into(),
            pid: None,
            is_focused: false,
        }
    }

    #[test]
    fn services_are_counted_and_failures_listed_first() {
        let s = summarize_services(&[
            svc("a.service", "ACTIVE"),
            svc("b.service", "DEGRADED"),
            svc("c.service", "FAILED"),
            svc("d.service", "INACTIVE"),
        ]);
        assert_eq!((s.total, s.active, s.failed, s.degraded, s.inactive), (4, 1, 1, 1, 1));
        assert_eq!(s.attention.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), vec!["c.service", "b.service"]);
    }

    #[test]
    fn processes_rank_by_cpu_and_memory() {
        let procs: Vec<ProcessUnit> = (1..=8).map(|i| proc(i, i as f32, 10.0 - i as f32, if i == 3 { "Z" } else if i == 4 { "D+" } else { "S" })).collect();
        let s = summarize_processes(&procs);
        assert_eq!(s.total, 8);
        assert_eq!((s.zombies, s.blocked), (1, 1));
        assert_eq!(s.top_cpu.len(), TOP_N);
        assert_eq!(s.top_cpu[0].0, 8);
        assert_eq!(s.top_mem[0].0, 1);
    }

    #[test]
    fn sockets_are_grouped_by_exposure() {
        let mut udp = sock("UNCONN", "192.168.122.1", "53");
        udp.protocol = "UDP".into();
        let mut udp_client = sock("UNCONN", "0.0.0.0", "57074");
        udp_client.protocol = "UDP".into();
        let s = summarize_sockets(&[
            sock("LISTEN", "127.0.0.1", "5432"),
            sock("LISTEN", "0.0.0.0", "443"),
            sock("LISTEN", "[::1]", "631"),
            sock("LISTEN", "*", "22"),
            sock("LISTEN", "0.0.0.0%virbr0", "67"),
            sock("LISTEN", "127.0.0.53%lo", "53"),
            sock("ESTAB", "10.0.0.2", "50123"),
            udp,
            udp_client,
        ]);
        assert_eq!((s.listening, s.all_interfaces, s.interface, s.local_only), (8, 3, 2, 3));
        assert_eq!(s.ports[0].port, "22");
        assert_eq!(s.ports[0].exposure, Exposure::AllInterfaces);
        // Ids match what the Sockets page uses to focus the row.
        assert_eq!(s.ports[0].socket_id, "TCP:22:3");
    }

    #[test]
    fn the_ps_sampler_is_not_a_top_process() {
        let mut sampler = proc(99, 300.0, 0.1, "R");
        sampler.command = "ps".into();
        let s = summarize_processes(&[sampler, proc(1, 5.0, 1.0, "S")]);
        assert_eq!(s.top_cpu[0].0, 1);
    }
}
