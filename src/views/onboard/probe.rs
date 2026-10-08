use std::io::{BufRead, BufReader};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use chrono::Local;
use gpui_kit::Rgba;
use crate::theme::*;
use crate::host::{Host, LocalHost, DEFAULT_TIMEOUT};

#[derive(Clone, Debug)]
pub struct ProbeLog {
    pub timestamp: String,
    pub glyph: String,
    pub color: Rgba,
    pub message: String,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct ProbeResult {
    pub is_reachable: bool,
    pub latency_ms: Option<u64>,
    pub ssh_banner: Option<String>,
    /// SHA256 fingerprint of the host key the server presented (ed25519
    /// preferred), or empty when none could be fetched.
    pub host_key_fingerprint: String,
    /// A key the server presented is already trusted in known_hosts.
    pub is_known_host: bool,
    /// known_hosts has keys for this host, but none match what the server
    /// presented now — possibly a reinstalled server, possibly an attack.
    pub host_key_mismatch: bool,
    /// The server's host keys, as known_hosts lines (from ssh-keyscan).
    pub scanned_keys: Vec<String>,
    #[allow(dead_code)]
    pub error: Option<String>,
}

impl ProbeResult {
    /// Nothing reached, nothing read.
    pub fn empty() -> Self {
        Self { is_reachable: false, latency_ms: None, ssh_banner: None, host_key_fingerprint: String::new(), is_known_host: false, host_key_mismatch: false, scanned_keys: Vec::new(), error: None }
    }
}

#[derive(Clone, Debug)]
pub struct DetectedFacts {
    pub distro: String,
    pub kernel: String,
    pub arch: String,
    pub memory: String,
    pub disk: String,
    pub init: String,
    pub open_ports: String,
    pub firewall: String,
    pub time_sync: String,
    pub schema_packs: Vec<(String, Rgba, Rgba)>,
    /// Where the server lives, from its cloud provider's metadata (ERR-36).
    pub region: Option<crate::region::DetectedRegion>,
}

/// Facts not read yet: shown as unknown, never guessed.
impl Default for DetectedFacts {
    fn default() -> Self {
        let unknown = || "—".to_string();
        Self {
            distro: unknown(),
            kernel: unknown(),
            arch: unknown(),
            memory: unknown(),
            disk: unknown(),
            init: unknown(),
            open_ports: unknown(),
            firewall: unknown(),
            time_sync: unknown(),
            schema_packs: Vec::new(),
            region: None,
        }
    }
}

/// Appends a timestamped line to a probe log.
pub fn log(logs: &mut Vec<ProbeLog>, glyph: &str, color: Rgba, message: String, note: String) {
    logs.push(ProbeLog { timestamp: Local::now().format("%H:%M:%S").to_string(), glyph: glyph.into(), color, message, note });
}

/// Connects to `host:port`, reads the SSH banner, fetches the host keys with
/// `ssh-keyscan`, and compares them with known_hosts. Every log line reports
/// something that actually happened.
pub fn probe_host(host: &str, port: u16) -> (ProbeResult, Vec<ProbeLog>) {
    let mut logs = Vec::new();
    let mut result = ProbeResult {
        is_reachable: false,
        latency_ms: None,
        ssh_banner: None,
        host_key_fingerprint: String::new(),
        is_known_host: false,
        host_key_mismatch: false,
        scanned_keys: Vec::new(),
        error: None,
    };
    if host.is_empty() || host.starts_with('-') {
        result.error = Some(format!("{host:?} is not a valid host"));
        log(&mut logs, "✕", CRIT, format!("{host:?} is not a valid host"), String::new());
        return (result, logs);
    }

    // 1. TCP connect
    let start = Instant::now();
    let addr = match format!("{host}:{port}").to_socket_addrs().ok().and_then(|mut a| a.next()) {
        Some(a) => a,
        None => {
            result.error = Some(format!("could not resolve {host}"));
            log(&mut logs, "✕", CRIT, format!("could not resolve {host}"), "dns".into());
            return (result, logs);
        }
    };
    let stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(3000)) {
        Ok(s) => s,
        Err(e) => {
            result.error = Some(format!("tcp {host}:{port}: {e}"));
            log(&mut logs, "✕", CRIT, format!("tcp connect {host}:{port} failed: {e}"), format!("{}ms", start.elapsed().as_millis()));
            return (result, logs);
        }
    };
    let latency = start.elapsed().as_millis() as u64;
    result.is_reachable = true;
    result.latency_ms = Some(latency);
    log(&mut logs, "✓", OK, format!("tcp connect {host}:{port}"), format!("{latency}ms"));

    // 2. SSH banner
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
    let mut banner = String::new();
    match BufReader::new(&stream).read_line(&mut banner) {
        Ok(_) if banner.starts_with("SSH-") => {
            result.ssh_banner = Some(banner.trim().to_string());
            log(&mut logs, "✓", OK, format!("ssh banner {}", banner.trim()), String::new());
        }
        _ => log(&mut logs, "▲", WARN, "no SSH banner — is this an SSH port?".into(), String::new()),
    }
    drop(stream);

    // 3. Host keys, from the server itself
    let port_s = port.to_string();
    let scan = LocalHost.exec(&["ssh-keyscan", "-T", "5", "-p", &port_s, host], Duration::from_secs(12));
    result.scanned_keys = scan
        .map(|o| o.stdout.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(str::to_string).collect())
        .unwrap_or_default();
    let fingerprints: Vec<(String, String)> = result.scanned_keys.iter().filter_map(|l| key_fingerprint(l)).collect();
    match fingerprints.iter().find(|(alg, _)| alg == "ssh-ed25519").or(fingerprints.first()) {
        Some((alg, fp)) => {
            result.host_key_fingerprint = format!("{fp} ({})", alg.trim_start_matches("ssh-").to_uppercase());
            log(&mut logs, "✓", OK, format!("host key {}", result.host_key_fingerprint), "ssh-keyscan".into());
        }
        None => log(&mut logs, "✕", CRIT, "could not fetch the server's host keys (ssh-keyscan)".into(), String::new()),
    }

    // 4. Compare with known_hosts (ssh-keygen -F handles hashed entries)
    compare_with_known_hosts(&mut result, &mut logs, host, port);
    (result, logs)
}

/// Probes a server behind bastions (ERR-152) the way Crow will reach it:
/// ssh through the bastions' ProxyCommand, with every login method off and
/// a throwaway known_hosts, so it records the key the server presents and
/// stops before logging in. Nothing is trusted here; the key is compared
/// with ~/.ssh/known_hosts like a direct probe's.
pub fn probe_via_bastion(host: &str, port: u16, user: &str, proxy: &str, via: &str) -> (ProbeResult, Vec<ProbeLog>) {
    let mut logs = Vec::new();
    let mut result = ProbeResult::empty();
    if host.is_empty() || host.starts_with('-') {
        result.error = Some(format!("{host:?} is not a valid host"));
        log(&mut logs, "✕", CRIT, format!("{host:?} is not a valid host"), String::new());
        return (result, logs);
    }
    let scratch = std::env::temp_dir().join(format!("crow-probe-{}-{}", std::process::id(), Instant::now().elapsed().as_nanos()));
    let kh = scratch.join("known_hosts");
    if let Err(e) = std::fs::create_dir_all(&scratch).and_then(|_| std::fs::write(&kh, "")) {
        log(&mut logs, "✕", CRIT, format!("couldn't make a scratch known_hosts: {e}"), String::new());
        return (result, logs);
    }
    let port_s = port.to_string();
    let kh_opt = format!("UserKnownHostsFile={}", kh.display());
    let proxy_opt = format!("ProxyCommand={proxy}");
    let mut argv: Vec<&str> = vec!["ssh", "-v", "-o", "BatchMode=yes", "-o", &kh_opt, "-o", "GlobalKnownHostsFile=/dev/null", "-o", "StrictHostKeyChecking=no", "-o", "HashKnownHosts=no", "-o", "UpdateHostKeys=no"];
    for off in ["PubkeyAuthentication=no", "PasswordAuthentication=no", "KbdInteractiveAuthentication=no", "GSSAPIAuthentication=no", "HostbasedAuthentication=no", "ControlMaster=no", "ControlPath=none", "ConnectTimeout=10"] {
        argv.extend(["-o", off]);
    }
    argv.extend(["-o", &proxy_opt, "-p", &port_s]);
    if !user.is_empty() {
        argv.extend(["-l", user]);
    }
    argv.extend(["--", host, "true"]);
    let start = Instant::now();
    let out = crate::host::run_local(&argv, &[], Duration::from_secs(40));
    let elapsed = start.elapsed().as_millis() as u64;
    let stderr = match &out {
        Ok(o) => o.stderr.clone(),
        Err(crate::host::HostError::Failed { stderr, .. }) => stderr.clone(),
        Err(e) => e.to_string(),
    };
    let lines: Vec<String> = std::fs::read_to_string(&kh).unwrap_or_default().lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(str::to_string).collect();
    let _ = std::fs::remove_dir_all(&scratch);
    if let Some(v) = stderr.lines().find_map(|l| l.split("remote software version ").nth(1)) {
        result.ssh_banner = Some(format!("SSH-2.0-{}", v.trim()));
    }
    if lines.is_empty() {
        let why = stderr
            .lines()
            .rev()
            .find(|l| !l.starts_with("debug") && !l.trim().is_empty() && !l.starts_with("OpenSSH_"))
            .unwrap_or("no answer")
            .trim()
            .to_string();
        let why = match crate::host::ssh::classify_hop_failure(&stderr, &[], host) {
            Some(crate::host::ssh::ConnectionState::Unreachable(m)) => m.replacen("the bastion", via, 1),
            _ if why.contains("Connection closed by UNKNOWN") => format!("{via} let Crow in but couldn't reach {host}:{port}, or {via} refused Crow"),
            _ => why,
        };
        result.error = Some(why.clone());
        log(&mut logs, "✕", CRIT, format!("through {via}: {why}"), format!("{elapsed}ms"));
        return (result, logs);
    }
    result.is_reachable = true;
    result.latency_ms = Some(elapsed);
    log(&mut logs, "✓", OK, format!("reached {host}:{port} through {via}"), format!("{elapsed}ms"));
    if let Some(b) = &result.ssh_banner {
        log(&mut logs, "✓", OK, format!("ssh banner {b}"), String::new());
    }
    result.scanned_keys = lines;
    let fingerprints: Vec<(String, String)> = result.scanned_keys.iter().filter_map(|l| key_fingerprint(l)).collect();
    if let Some((alg, fp)) = fingerprints.first() {
        result.host_key_fingerprint = format!("{fp} ({})", alg.trim_start_matches("ssh-").to_uppercase());
        log(&mut logs, "✓", OK, format!("host key {}", result.host_key_fingerprint), format!("through {via}"));
    }
    compare_with_known_hosts(&mut result, &mut logs, host, port);
    (result, logs)
}

fn compare_with_known_hosts(result: &mut ProbeResult, logs: &mut Vec<ProbeLog>, host: &str, port: u16) {
    let known = known_host_keys(host, port);
    let scanned_blobs: Vec<&str> = result.scanned_keys.iter().filter_map(|l| l.split_whitespace().nth(2)).collect();
    result.is_known_host = known.iter().any(|k| scanned_blobs.contains(&k.as_str()));
    result.host_key_mismatch = !known.is_empty() && !result.is_known_host && !scanned_blobs.is_empty();
    if result.is_known_host {
        log(logs, "✓", OK, "host key matches ~/.ssh/known_hosts".into(), String::new());
    } else if result.host_key_mismatch {
        log(logs, "✕", CRIT, "HOST KEY CHANGED: known_hosts has a different key for this host — possible man-in-the-middle; not trusting it".into(), String::new());
    } else {
        log(logs, "▲", WARN, "host key not in known_hosts — verify the fingerprint, then accept".into(), String::new());
    }
}

/// (key type, SHA256 fingerprint) of a known_hosts / ssh-keyscan line.
pub fn key_fingerprint(line: &str) -> Option<(String, String)> {
    let mut parts = line.split_whitespace();
    let _host = parts.next()?;
    let (alg, blob) = (parts.next()?, parts.next()?);
    let key = ssh_key::PublicKey::from_openssh(&format!("{alg} {blob}")).ok()?;
    Some((alg.to_string(), key.fingerprint(ssh_key::HashAlg::Sha256).to_string()))
}

/// Base64 key blobs known_hosts holds for `host:port` (hashed entries included).
fn known_host_keys(host: &str, port: u16) -> Vec<String> {
    let pattern = if port == 22 { host.to_string() } else { format!("[{host}]:{port}") };
    LocalHost
        .exec(&["ssh-keygen", "-F", &pattern], Duration::from_secs(5))
        .map(|o| o.stdout.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_whitespace().nth(2).map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Appends the scanned host keys to `~/.ssh/known_hosts`.
pub fn trust_host_keys(lines: &[String]) -> std::io::Result<()> {
    use std::io::Write;
    let dir = dirs::home_dir().ok_or_else(|| std::io::Error::other("no home directory"))?.join(".ssh");
    std::fs::create_dir_all(&dir)?;
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("known_hosts"))?;
    for line in lines {
        writeln!(file, "{line}")?;
    }
    Ok(())
}

/// Reads read-only facts from a server over its transport in one round trip,
/// including whether passwordless sudo works. Unreadable facts stay "—".
pub fn gather_facts(host: &dyn Host) -> (DetectedFacts, Vec<ProbeLog>) {
    let mut logs = Vec::new();
    let script = r#"PATH="$PATH:/usr/sbin:/sbin"
. /etc/os-release 2>/dev/null && echo "distro=$PRETTY_NAME"
echo "kernel=$(uname -r)"
echo "arch=$(uname -m) · $(nproc 2>/dev/null || getconf _NPROCESSORS_ONLN) vCPU"
awk '/^MemTotal/{printf "memory=%.1f GB\n", $2/1048576}' /proc/meminfo
df -h / 2>/dev/null | awk 'NR==2{print "disk="$2" · "$5" used"}'
echo "init=$(cat /proc/1/comm 2>/dev/null)"
echo "ports=$(ss -tlnH 2>/dev/null | awk '{n=split($4,a,":"); print a[n]}' | sort -un | tr '\n' ',' | sed 's/,$//')"
t=$(timedatectl show -p NTPSynchronized --value 2>/dev/null); [ -n "$t" ] && echo "time=ntp synchronized: $t"
for b in sshd systemctl ufw nginx postgres redis-server docker podman; do command -v "$b" >/dev/null 2>&1 && echo "tool=$b"; done
if [ "$(id -u)" = 0 ]; then echo "sudo=root"; elif sudo -n true 2>/dev/null; then echo "sudo=passwordless"; else echo "sudo=needs password"; fi
true"#;
    // The region probe rides along in the same round trip.
    let script = format!("{script}\n{}", crate::region::REGION_PROBE);
    let out = match host.exec(&["sh", "-c", &script], DEFAULT_TIMEOUT) {
        Ok(o) => o.stdout,
        Err(e) => {
            log(&mut logs, "✕", CRIT, format!("could not read facts over SSH: {e}"), String::new());
            return (DetectedFacts::default(), logs);
        }
    };
    let mut facts = DetectedFacts::default();
    let mut tools = Vec::new();
    for line in out.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim().to_string();
        if v.is_empty() {
            continue;
        }
        match k {
            "distro" => facts.distro = v,
            "kernel" => facts.kernel = v,
            "arch" => facts.arch = v,
            "memory" => facts.memory = v,
            "disk" => facts.disk = v,
            "init" => facts.init = v,
            "ports" => facts.open_ports = v.replace(',', ", "),
            "time" => facts.time_sync = v,
            "tool" => tools.push(v),
            "sudo" => {
                let ok = v != "needs password";
                log(&mut logs, if ok { "✓" } else { "▲" }, if ok { OK } else { WARN }, format!("privileges: {v}"), String::new());
            }
            _ => {}
        }
    }
    facts.region = crate::region::parse_region_probe(&out);
    if let Some(r) = &facts.region {
        let place = match (crate::region::country_name(&r.country), r.city.is_empty()) {
            (Some(c), false) => format!("{}, {c}", r.city),
            (Some(c), true) => c.to_string(),
            (None, _) => "unknown location".to_string(),
        };
        log(&mut logs, "✓", OK, format!("region {place} · {} {}", r.provider, r.code), "cloud metadata".into());
    }
    facts.firewall = if tools.iter().any(|t| t == "ufw") { "ufw installed".into() } else { "—".into() };
    facts.schema_packs = tools.iter().filter(|t| *t != "systemctl").map(|t| (t.clone(), OK, OK_BG)).collect();
    log(&mut logs, "✓", OK, "read os-release, uname, meminfo, df, listening ports".into(), "read-only".into());
    (facts, logs)
}




#[cfg(test)]
mod tests {
    use super::*;

    /// Our fingerprint matches what `ssh-keygen -lf` prints for the same key.
    #[test]
    fn fingerprint_matches_ssh_keygen() {
        let dir = std::env::temp_dir().join(format!("crow-fp-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let key = dir.join("k");
        let gen = std::process::Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", "", "-f"]).arg(&key).status();
        if !gen.is_ok_and(|s| s.success()) {
            return; // no ssh-keygen here
        }
        let public = std::fs::read_to_string(key.with_extension("pub")).unwrap();
        let line = format!("[example]:2222 {}", public.split_whitespace().take(2).collect::<Vec<_>>().join(" "));
        let listed = std::process::Command::new("ssh-keygen").arg("-lf").arg(key.with_extension("pub")).output().unwrap();
        let expected = String::from_utf8_lossy(&listed.stdout).split_whitespace().nth(1).unwrap().to_string();
        assert_eq!(key_fingerprint(&line), Some(("ssh-ed25519".into(), expected)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_option_like_hosts_and_reports_unknown_facts() {
        let (result, logs) = probe_host("-oProxyCommand=x", 22);
        assert!(!result.is_reachable && result.scanned_keys.is_empty());
        assert!(logs[0].message.contains("not a valid host"));
        let facts = DetectedFacts::default();
        assert_eq!(facts.distro, "—");
        assert!(facts.schema_packs.is_empty());
    }

    /// Facts come from the machine itself: this one's kernel is what uname says.
    #[test]
    fn gathers_real_facts_from_a_host() {
        let (facts, _) = gather_facts(&LocalHost);
        let uname = std::process::Command::new("uname").arg("-r").output().unwrap();
        assert_eq!(facts.kernel, String::from_utf8_lossy(&uname.stdout).trim());
        assert_ne!(facts.memory, "—");
    }
}

#[cfg(test)]
mod live {
    /// `CROW_LIVE_PROBE=host:port cargo test live_probe -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_probe() {
        let target = std::env::var("CROW_LIVE_PROBE").expect("CROW_LIVE_PROBE=host:port");
        let (host, port) = target.rsplit_once(':').unwrap();
        let (result, logs) = super::probe_host(host, port.parse().unwrap());
        for l in &logs {
            println!("{} {} {}", l.glyph, l.message, l.note);
        }
        println!("{result:?}");
        assert!(result.is_reachable && !result.scanned_keys.is_empty());
    }
}

/// Probes servers behind a real bastion (the ERR-152 container lab):
///   CROW_BASTION_LAB=/path/with/key+known_hosts+ssh-wrap cargo test live_probe_via_bastion -- --ignored --nocapture
#[cfg(test)]
#[test]
#[ignore]
fn live_probe_via_bastion() {
    use crate::host::ssh::{proxy_command, Hop, SshSettings};
    let lab = std::path::PathBuf::from(std::env::var("CROW_BASTION_LAB").expect("set CROW_BASTION_LAB"));
    let hop = Hop { name: "b1".into(), host: "127.0.0.1".into(), port: 22301, user: "root".into(), key_path: Some(lab.join("key").display().to_string()) };
    let wrap = lab.join("ssh-wrap").display().to_string();
    let proxy = proxy_command(&wrap, &SshSettings::default(), &[hop], "crow-t1", 22).unwrap();
    let (r, logs) = probe_via_bastion("crow-t1", 22, "root", &proxy, "b1");
    for l in &logs {
        eprintln!("{} {} {}", l.glyph, l.message, l.note);
    }
    assert!(r.is_reachable && r.ssh_banner.as_deref().is_some_and(|b| b.starts_with("SSH-2.0-OpenSSH")));
    let t1 = std::fs::read_to_string(lab.join("known_hosts")).unwrap().lines().find(|l| l.starts_with("crow-t1")).unwrap().split_whitespace().nth(2).unwrap().to_string();
    assert!(r.scanned_keys.iter().any(|l| l.starts_with("crow-t1 ") && l.contains(&t1)), "the key t1 really has: {:?}", r.scanned_keys);
    let proxy = proxy_command(&wrap, &SshSettings::default(), &[Hop { name: "b1".into(), host: "127.0.0.1".into(), port: 22301, user: "root".into(), key_path: Some(lab.join("key").display().to_string()) }], "crow-nowhere", 22).unwrap();
    let (r, logs) = probe_via_bastion("crow-nowhere", 22, "root", &proxy, "b1");
    eprintln!("{}", logs.last().unwrap().message);
    assert!(!r.is_reachable && r.error.as_deref().is_some_and(|e| e.contains("b1")));
}
