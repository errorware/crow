//! Linux `/proc` process-tree socket witness (`ERR-212`).
//!
//! Independently of Crow's application-level Wire Gatekeeper, this module asks
//! the Linux kernel which TCP and UDP sockets are open right now inside `crow`
//! and every descendant process (`ssh`, `curl`, `ssh-keyscan`, plugins) by:
//! 1. Walking `/proc/<crow_pid>/task/*/children` recursively.
//! 2. Reading `/proc/<pid>/fd/*` symlinks (`socket:[<inode>]`).
//! 3. Joining those inodes against `/proc/net/tcp`, `/proc/net/tcp6`,
//!    `/proc/net/udp`, and `/proc/net/udp6`.
//! 4. Attributing each open socket to a `WireEvent` in the ledger, an SSH
//!    ControlMaster (`[mux]`), a loopback socket, or flagging it as
//!    `Unaccounted` (`CRIT`).

use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::WireEvent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SocketAttribution {
    /// Matches an active or recent `WireEvent` (`#seq caller -> target_label`).
    Accounted(String),
    /// An idle OpenSSH ControlMaster multiplexing process kept open by Crow.
    ControlMaster(String),
    /// Local loopback (`127.0.0.1` / `::1`) socket.
    Loopback,
    /// Non-loopback socket in Crow's process tree with no matching WireEvent!
    Unaccounted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSocket {
    pub pid: u32,
    pub comm: String,
    pub cmdline: String,
    pub protocol: &'static str,
    pub local_ip: IpAddr,
    pub local_port: u16,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub state: &'static str,
    pub inode: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributedSocket {
    pub socket: LiveSocket,
    pub attribution: SocketAttribution,
}

impl AttributedSocket {
    pub fn is_loopback(&self) -> bool {
        self.socket.remote_ip.is_loopback() || self.socket.local_ip.is_loopback()
    }

    pub fn is_unaccounted(&self) -> bool {
        self.attribution == SocketAttribution::Unaccounted
    }
}

#[derive(Clone, Debug)]
pub struct KernelSocketRow {
    pub protocol: &'static str,
    pub local_ip: IpAddr,
    pub local_port: u16,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub state: &'static str,
    pub inode: u64,
}

/// Collects `root_pid` and all recursive descendant PIDs from `/proc`.
pub fn collect_process_tree(root_pid: u32) -> Vec<u32> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![root_pid];
    while let Some(pid) = stack.pop() {
        if !visited.insert(pid) {
            continue;
        }
        result.push(pid);
        let task_dir = format!("/proc/{pid}/task");
        if let Ok(tasks) = std::fs::read_dir(&task_dir) {
            for t in tasks.flatten() {
                let children_path = t.path().join("children");
                if let Ok(text) = std::fs::read_to_string(children_path) {
                    for tok in text.split_whitespace() {
                        if let Ok(child_pid) = tok.parse::<u32>() {
                            stack.push(child_pid);
                        }
                    }
                }
            }
        }
    }
    // Also discover detached `ssh` ControlMaster `[mux]` processes whose
    // ControlPath lives in Crow's private `crow-ssh-<uid>` directory, since
    // OpenSSH daemonizes ControlMaster (`PPID = 1`).
    #[cfg(target_os = "linux")]
    if let Ok(entries) = std::fs::read_dir("/proc") {
        let uid = unsafe { libc::geteuid() };
        let marker = format!("crow-ssh-{uid}");
        for entry in entries.flatten() {
            let Some(pid) = entry.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else {
                continue;
            };
            if visited.contains(&pid) {
                continue;
            }
            let cmdline = read_cmdline(pid);
            if cmdline.contains(&marker) {
                visited.insert(pid);
                result.push(pid);
            }
        }
    }
    result
}

#[cfg(not(windows))]
fn read_comm(pid: u32) -> String {
    std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(not(windows))]
fn read_cmdline(pid: u32) -> String {
    std::fs::read(format!("/proc/{pid}/cmdline"))
        .map(|bytes| {
            bytes
                .split(|&b| b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

#[cfg(not(windows))]
fn socket_inodes_for_pid(pid: u32) -> Vec<u64> {
    let mut out = Vec::new();
    let Ok(fds) = std::fs::read_dir(format!("/proc/{pid}/fd")) else {
        return out;
    };
    for fd in fds.flatten() {
        if let Ok(target) = std::fs::read_link(fd.path()) {
            if let Some(s) = target.to_str() {
                if let Some(rest) = s.strip_prefix("socket:[") {
                    if let Some(num) = rest.strip_suffix(']') {
                        if let Ok(inode) = num.parse::<u64>() {
                            out.push(inode);
                        }
                    }
                }
            }
        }
    }
    out
}

pub fn tcp_state_label(code: u8) -> &'static str {
    match code {
        0x01 => "ESTABLISHED",
        0x02 => "SYN_SENT",
        0x03 => "SYN_RECV",
        0x04 => "FIN_WAIT1",
        0x05 => "FIN_WAIT2",
        0x06 => "TIME_WAIT",
        0x07 => "CLOSE",
        0x08 => "CLOSE_WAIT",
        0x09 => "LAST_ACK",
        0x0A => "LISTEN",
        0x0B => "CLOSING",
        _ => "UNKNOWN",
    }
}

pub fn parse_ipv4_hex(hex_addr: &str) -> Option<(IpAddr, u16)> {
    let (ip_hex, port_hex) = hex_addr.split_once(':')?;
    let raw_ip = u32::from_str_radix(ip_hex, 16).ok()?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    let octets = raw_ip.to_ne_bytes();
    Some((IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])), port))
}

pub fn parse_ipv6_hex(hex_addr: &str) -> Option<(IpAddr, u16)> {
    let (ip_hex, port_hex) = hex_addr.split_once(':')?;
    if ip_hex.len() != 32 {
        return None;
    }
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    let mut octets = [0u8; 16];
    for i in 0..4 {
        let word = u32::from_str_radix(&ip_hex[i * 8..(i + 1) * 8], 16).ok()?;
        let word_bytes = word.to_ne_bytes();
        octets[i * 4..i * 4 + 4].copy_from_slice(&word_bytes);
    }
    let v6 = Ipv6Addr::from(octets);
    // Unwrap IPv4-mapped IPv6 (`::ffff:a.b.c.d`) so matching against IPv4 hosts works seamlessly.
    if let Some(v4) = v6.to_ipv4_mapped() {
        Some((IpAddr::V4(v4), port))
    } else {
        Some((IpAddr::V6(v6), port))
    }
}

pub fn parse_proc_net_table(content: &str, protocol: &'static str, is_v6: bool) -> HashMap<u64, KernelSocketRow> {
    let mut map = HashMap::new();
    for line in content.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 10 {
            continue;
        }
        let addr_parser = if is_v6 { parse_ipv6_hex } else { parse_ipv4_hex };
        let Some((local_ip, local_port)) = addr_parser(cols[1]) else { continue };
        let Some((remote_ip, remote_port)) = addr_parser(cols[2]) else { continue };
        let state_code = u8::from_str_radix(cols[3], 16).unwrap_or(0);
        let Ok(inode) = cols[9].parse::<u64>() else { continue };
        if inode == 0 || remote_port == 0 {
            continue;
        }
        map.insert(
            inode,
            KernelSocketRow {
                protocol,
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                state: tcp_state_label(state_code),
                inode,
            },
        );
    }
    map
}

/// Scans live outbound sockets belonging to Crow and its descendant processes,
/// and attributes each against `events`.
/// - Linux: walks `/proc/<pid>/task/*/children`, `/proc/<pid>/fd/*`, and `/proc/net/*`.
/// - Windows: walks process tree via `CreateToolhelp32Snapshot` and queries TCP/UDP
///   owner-PID tables via IP Helper API (`GetExtendedTcpTable` / `GetExtendedUdpTable`).
pub fn scan_live_sockets(events: &[WireEvent]) -> Vec<AttributedSocket> {
    #[cfg(windows)]
    {
        return scan_live_sockets_windows(events);
    }
    #[cfg(not(windows))]
    {
        let root_pid = std::process::id();
        let pids = collect_process_tree(root_pid);

        let mut kernel_table = HashMap::new();
        for (path, proto, is_v6) in [
            ("/proc/net/tcp", "TCP", false),
            ("/proc/net/tcp6", "TCP6", true),
            ("/proc/net/udp", "UDP", false),
            ("/proc/net/udp6", "UDP6", true),
        ] {
            if let Ok(content) = std::fs::read_to_string(path) {
                kernel_table.extend(parse_proc_net_table(&content, proto, is_v6));
            }
        }

        let mut out = Vec::new();
        let mut seen_inodes = HashSet::new();
        for pid in pids {
            let inodes = socket_inodes_for_pid(pid);
            if inodes.is_empty() {
                continue;
            }
            let comm = read_comm(pid);
            let cmdline = read_cmdline(pid);
            for inode in inodes {
                if !seen_inodes.insert(inode) {
                    continue;
                }
                let Some(row) = kernel_table.get(&inode) else { continue };
                let attribution = attribute_socket(&comm, &cmdline, row, events);
                out.push(AttributedSocket {
                    socket: LiveSocket {
                        pid,
                        comm: comm.clone(),
                        cmdline: cmdline.clone(),
                        protocol: row.protocol,
                        local_ip: row.local_ip,
                        local_port: row.local_port,
                        remote_ip: row.remote_ip,
                        remote_port: row.remote_port,
                        state: row.state,
                        inode: row.inode,
                    },
                    attribution,
                });
            }
        }
        out.sort_by(|a, b| a.socket.pid.cmp(&b.socket.pid).then_with(|| a.socket.remote_port.cmp(&b.socket.remote_port)));
        out
    }
}

#[cfg(windows)]
fn windows_tcp_state_label(state: u32) -> &'static str {
    match state {
        1 => "CLOSE",
        2 => "LISTEN",
        3 => "SYN_SENT",
        4 => "SYN_RECV",
        5 => "ESTABLISHED",
        6 => "FIN_WAIT1",
        7 => "FIN_WAIT2",
        8 => "CLOSE_WAIT",
        9 => "CLOSING",
        10 => "LAST_ACK",
        11 => "TIME_WAIT",
        12 => "DELETE_TCB",
        _ => "UNKNOWN",
    }
}

#[cfg(windows)]
fn collect_windows_process_tree(root_pid: u32) -> HashMap<u32, String> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut names: HashMap<u32, String> = HashMap::new();

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return names;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                names.insert(entry.th32ProcessID, name);
                children.entry(entry.th32ParentProcessID).or_default().push(entry.th32ProcessID);
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }

    let mut tree = HashMap::new();
    let mut stack = vec![root_pid];
    while let Some(pid) = stack.pop() {
        if tree.contains_key(&pid) {
            continue;
        }
        let comm = names.get(&pid).cloned().unwrap_or_else(|| "unknown".into());
        tree.insert(pid, comm);
        if let Some(kids) = children.get(&pid) {
            for &kid in kids {
                stack.push(kid);
            }
        }
    }
    tree
}

#[cfg(windows)]
fn scan_live_sockets_windows(events: &[WireEvent]) -> Vec<AttributedSocket> {
    use windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        MIB_UDP6ROW_OWNER_PID, MIB_UDPROW_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
        UDP_TABLE_OWNER_PID,
    };
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

    let pid_tree = collect_windows_process_tree(std::process::id());
    if pid_tree.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut synth_inode: u64 = 1;

    let mut push_socket = |pid: u32, protocol: &'static str, local_ip: IpAddr, local_port: u16, remote_ip: IpAddr, remote_port: u16, state: &'static str| {
        let Some(comm) = pid_tree.get(&pid) else { return };
        let inode = synth_inode;
        synth_inode += 1;
        let row = KernelSocketRow { protocol, local_ip, local_port, remote_ip, remote_port, state, inode };
        let attribution = attribute_socket(comm, comm, &row, events);
        out.push(AttributedSocket {
            socket: LiveSocket {
                pid,
                comm: comm.clone(),
                cmdline: comm.clone(),
                protocol,
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                state,
                inode,
            },
            attribution,
        });
    };

    // TCP IPv4
    unsafe {
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0);
        if size > 0 {
            let mut buf = vec![0u8; size as usize];
            let mut res = GetExtendedTcpTable(buf.as_mut_ptr().cast(), &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0);
            if res == ERROR_INSUFFICIENT_BUFFER && size > 0 {
                buf.resize(size as usize, 0);
                res = GetExtendedTcpTable(buf.as_mut_ptr().cast(), &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0);
            }
            if res == 0 && buf.len() >= 4 {
                let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                let row_size = std::mem::size_of::<MIB_TCPROW_OWNER_PID>();
                for i in 0..count {
                    let offset = 4 + i * row_size;
                    if offset + row_size > buf.len() {
                        break;
                    }
                    let r: MIB_TCPROW_OWNER_PID = std::ptr::read_unaligned(buf.as_ptr().add(offset).cast());
                    let remote_port = u16::from_be((r.dwRemotePort & 0xFFFF) as u16);
                    if remote_port == 0 {
                        continue;
                    }
                    let local_ip = IpAddr::V4(Ipv4Addr::from(r.dwLocalAddr.to_ne_bytes()));
                    let local_port = u16::from_be((r.dwLocalPort & 0xFFFF) as u16);
                    let remote_ip = IpAddr::V4(Ipv4Addr::from(r.dwRemoteAddr.to_ne_bytes()));
                    push_socket(r.dwOwningPid, "TCP", local_ip, local_port, remote_ip, remote_port, windows_tcp_state_label(r.dwState));
                }
            }
        }
    }

    // TCP IPv6
    unsafe {
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, AF_INET6 as u32, TCP_TABLE_OWNER_PID_ALL, 0);
        if size > 0 {
            let mut buf = vec![0u8; size as usize];
            let mut res = GetExtendedTcpTable(buf.as_mut_ptr().cast(), &mut size, 0, AF_INET6 as u32, TCP_TABLE_OWNER_PID_ALL, 0);
            if res == ERROR_INSUFFICIENT_BUFFER && size > 0 {
                buf.resize(size as usize, 0);
                res = GetExtendedTcpTable(buf.as_mut_ptr().cast(), &mut size, 0, AF_INET6 as u32, TCP_TABLE_OWNER_PID_ALL, 0);
            }
            if res == 0 && buf.len() >= 4 {
                let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                let row_size = std::mem::size_of::<MIB_TCP6ROW_OWNER_PID>();
                for i in 0..count {
                    let offset = 4 + i * row_size;
                    if offset + row_size > buf.len() {
                        break;
                    }
                    let r: MIB_TCP6ROW_OWNER_PID = std::ptr::read_unaligned(buf.as_ptr().add(offset).cast());
                    let remote_port = u16::from_be((r.dwRemotePort & 0xFFFF) as u16);
                    if remote_port == 0 {
                        continue;
                    }
                    let unwrap_v6 = |bytes: [u8; 16]| {
                        let v6 = Ipv6Addr::from(bytes);
                        v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(v6))
                    };
                    let local_ip = unwrap_v6(r.ucLocalAddr);
                    let local_port = u16::from_be((r.dwLocalPort & 0xFFFF) as u16);
                    let remote_ip = unwrap_v6(r.ucRemoteAddr);
                    push_socket(r.dwOwningPid, "TCP6", local_ip, local_port, remote_ip, remote_port, windows_tcp_state_label(r.dwState));
                }
            }
        }
    }

    // UDP IPv4 / IPv6 (bound endpoints owned by Crow / child processes)
    unsafe {
        let mut size: u32 = 0;
        let _ = GetExtendedUdpTable(std::ptr::null_mut(), &mut size, 0, AF_INET as u32, UDP_TABLE_OWNER_PID, 0);
        if size > 0 {
            let mut buf = vec![0u8; size as usize];
            if GetExtendedUdpTable(buf.as_mut_ptr().cast(), &mut size, 0, AF_INET as u32, UDP_TABLE_OWNER_PID, 0) == 0 && buf.len() >= 4 {
                let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                let row_size = std::mem::size_of::<MIB_UDPROW_OWNER_PID>();
                for i in 0..count {
                    let offset = 4 + i * row_size;
                    if offset + row_size > buf.len() {
                        break;
                    }
                    let r: MIB_UDPROW_OWNER_PID = std::ptr::read_unaligned(buf.as_ptr().add(offset).cast());
                    let local_port = u16::from_be((r.dwLocalPort & 0xFFFF) as u16);
                    if local_port == 0 {
                        continue;
                    }
                    let local_ip = IpAddr::V4(Ipv4Addr::from(r.dwLocalAddr.to_ne_bytes()));
                    if local_ip.is_unspecified() {
                        continue;
                    }
                    push_socket(r.dwOwningPid, "UDP", local_ip, local_port, local_ip, local_port, "BOUND");
                }
            }
        }

        let mut size6: u32 = 0;
        let _ = GetExtendedUdpTable(std::ptr::null_mut(), &mut size6, 0, AF_INET6 as u32, UDP_TABLE_OWNER_PID, 0);
        if size6 > 0 {
            let mut buf = vec![0u8; size6 as usize];
            if GetExtendedUdpTable(buf.as_mut_ptr().cast(), &mut size6, 0, AF_INET6 as u32, UDP_TABLE_OWNER_PID, 0) == 0 && buf.len() >= 4 {
                let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                let row_size = std::mem::size_of::<MIB_UDP6ROW_OWNER_PID>();
                for i in 0..count {
                    let offset = 4 + i * row_size;
                    if offset + row_size > buf.len() {
                        break;
                    }
                    let r: MIB_UDP6ROW_OWNER_PID = std::ptr::read_unaligned(buf.as_ptr().add(offset).cast());
                    let local_port = u16::from_be((r.dwLocalPort & 0xFFFF) as u16);
                    if local_port == 0 {
                        continue;
                    }
                    let v6 = Ipv6Addr::from(r.ucLocalAddr);
                    if v6.is_unspecified() {
                        continue;
                    }
                    let local_ip = v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(v6));
                    push_socket(r.dwOwningPid, "UDP6", local_ip, local_port, local_ip, local_port, "BOUND");
                }
            }
        }
    }

    out.sort_by(|a, b| a.socket.pid.cmp(&b.socket.pid).then_with(|| a.socket.remote_port.cmp(&b.socket.remote_port)));
    out
}

fn attribute_socket(
    comm: &str,
    cmdline: &str,
    row: &KernelSocketRow,
    events: &[WireEvent],
) -> SocketAttribution {
    if row.remote_ip.is_loopback() || row.local_ip.is_loopback() {
        return SocketAttribution::Loopback;
    }
    let remote_ip_str = row.remote_ip.to_string();

    // 1. Direct match by remote IP/port or hostname in recent WireEvents
    if let Some(ev) = events.iter().find(|e| {
        e.remote_port == row.remote_port
            && (e.remote_host == remote_ip_str || cmdline.contains(&e.remote_host))
    }) {
        return SocketAttribution::Accounted(format!(
            "#{} {} -> {}",
            ev.seq, ev.caller, ev.target_label
        ));
    }

    // 2. OpenSSH ControlMaster `[mux]` or active ssh connection
    if comm.eq_ignore_ascii_case("ssh") || comm.eq_ignore_ascii_case("ssh.exe") || cmdline.contains("[mux]") || cmdline.contains("crow-ssh-") {
        if let Some(ev) = events.iter().find(|e| e.remote_port == row.remote_port) {
            return SocketAttribution::ControlMaster(ev.target_label.clone());
        }
        return SocketAttribution::ControlMaster(format!("{}:{}", row.remote_ip, row.remote_port));
    }

    // 3. Match by port for active curl/check/ssh-keyscan children
    if let Some(ev) = events.iter().find(|e| e.remote_port == row.remote_port) {
        return SocketAttribution::Accounted(format!(
            "#{} {} -> {}",
            ev.seq, ev.caller, ev.target_label
        ));
    }

    SocketAttribution::Unaccounted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_proc_net_tcp_and_attributes_sockets() {
        let sample = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:1F90 0100007F:C350 01 00000000:00000000 00:00000000 00000000  1000        0 11111 1 0000000000000000 20 4 30 10 -1
   1: 0A00000A:D431 050000CB:0016 01 00000000:00000000 02:000A7214 00000000  1000        0 22222 1 0000000000000000 20 4 30 10 -1
";
        let table = parse_proc_net_table(sample, "TCP", false);
        assert_eq!(table.len(), 2);
        let s_loop = &table[&11111];
        assert!(s_loop.local_ip.is_loopback());
        let s_ssh = &table[&22222];
        assert_eq!(s_ssh.remote_ip.to_string(), "203.0.0.5");
        assert_eq!(s_ssh.remote_port, 22);
        assert_eq!(s_ssh.state, "ESTABLISHED");
    }

    #[cfg(windows)]
    #[test]
    fn windows_socket_witness_sees_live_loopback_tcp_socket() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _client = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        let _server = listener.accept().unwrap();

        let sockets = scan_live_sockets(&[]);
        let found = sockets
            .iter()
            .find(|s| s.socket.pid == std::process::id() && s.socket.remote_port == port);
        assert!(found.is_some(), "expected live loopback socket on port {port} in {sockets:?}");
        assert_eq!(found.unwrap().attribution, SocketAttribution::Loopback);
    }
}
