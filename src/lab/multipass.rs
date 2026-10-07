//! Canonical Multipass as a source of local VMs (ERR-119). Crow drives the
//! `multipass` CLI on this machine; every VM it launches or imports becomes
//! an ordinary SSH server in the fleet, logged in with Crow's own key, its
//! host key read through Multipass (a channel Crow already trusts) rather
//! than taken on first use.

use std::time::Duration;

use serde::Deserialize;

use crate::host::{Host, HostError, LocalHost};
use crate::vault::{ServerRecord, SshKeyRecord};

const QUICK: Duration = Duration::from_secs(20);
/// A launch downloads the image the first time and boots the VM.
const LAUNCH: Duration = Duration::from_secs(15 * 60);
/// Start, stop, restart, suspend and delete wait for the VM.
const LIFECYCLE: Duration = Duration::from_secs(3 * 60);

/// Whether Multipass can be used here, and what to do when it can't.
#[derive(Clone, Debug, PartialEq)]
pub enum MultipassStatus {
    /// No `multipass` command: how to install it on this system.
    Missing { steps: Vec<InstallStep> },
    /// The client runs but its daemon doesn't answer.
    DaemonDown { client: String, detail: String, steps: Vec<InstallStep> },
    /// Running. `fixes` is what still stops a VM from getting a network.
    Ready { client: String, daemon: String, fixes: Vec<InstallStep> },
}

impl MultipassStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

/// One thing to do to get Multipass going: why, and the command.
#[derive(Clone, Debug, PartialEq)]
pub struct InstallStep {
    pub why: String,
    pub command: String,
}

fn step(why: &str, command: &str) -> InstallStep {
    InstallStep { why: why.into(), command: command.into() }
}

/// The client and daemon versions from `multipass version --format json`;
/// the daemon's is missing when it isn't running.
pub fn parse_version(json: &str) -> Option<(String, Option<String>)> {
    #[derive(Deserialize)]
    struct V {
        multipass: String,
        multipassd: Option<String>,
    }
    let v: V = serde_json::from_str(json).ok()?;
    Some((v.multipass, v.multipassd.filter(|d| !d.is_empty())))
}

/// What this machine is, for the install steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Platform {
    MacOs,
    /// Fedora Atomic and its kin (Silverblue, Kinoite, Bazzite): no snap
    /// until it's layered in, and system users only in /usr/lib/passwd.
    LinuxOstree,
    Linux,
    Other,
}

pub fn platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(target_os = "linux") {
        if std::path::Path::new("/run/ostree-booted").exists() {
            Platform::LinuxOstree
        } else {
            Platform::Linux
        }
    } else {
        Platform::Other
    }
}

/// How to install Multipass. `has_snap` and `has_nobody` are what this
/// machine has now, so done steps aren't repeated.
pub fn install_steps(platform: Platform, has_snap: bool, has_nobody: bool) -> Vec<InstallStep> {
    match platform {
        Platform::MacOs => vec![step("Install Multipass with Homebrew", "brew install --cask multipass")],
        Platform::Other => vec![step("Install Multipass from Canonical", "https://canonical.com/multipass/install")],
        Platform::Linux | Platform::LinuxOstree => {
            let mut steps = Vec::new();
            if !has_snap {
                if platform == Platform::LinuxOstree {
                    steps.push(step("Multipass ships as a snap; add snapd to the system image, then reboot", "rpm-ostree install snapd && systemctl reboot"));
                } else {
                    steps.push(step("Multipass ships as a snap; install snapd with your package manager", "sudo dnf install snapd   # or: sudo apt install snapd"));
                }
                steps.push(step("Start snapd", "sudo systemctl enable --now snapd.socket"));
            }
            steps.extend(daemon_fixes(platform, has_nobody));
            steps.push(step("Install Multipass", "sudo snap install multipass"));
            steps
        }
    }
}

/// What keeps the daemon from starting where it's installed. On an ostree
/// system the snap's dnsmasq can't see `nobody` (it's in /usr/lib/passwd,
/// which only Fedora's own altfiles lookup reads), so the daemon dies.
fn daemon_fixes(platform: Platform, has_nobody: bool) -> Vec<InstallStep> {
    if platform == Platform::LinuxOstree && !has_nobody {
        vec![step(
            "Let Multipass's DNS server find the nobody user (system users live in /usr/lib on this system)",
            "grep '^nobody:' /usr/lib/passwd | sudo tee -a /etc/passwd; grep '^nobody:' /usr/lib/group | sudo tee -a /etc/group",
        )]
    } else {
        Vec::new()
    }
}

/// firewalld drops a VM's DHCP and DNS on Multipass's bridge unless the
/// bridge is in a zone that lets them through: the VM boots with no address
/// and `multipass launch` times out. `zone` is the bridge's zone, `None`
/// when it's in none (so the default zone applies).
pub fn bridge_fix(firewalld_active: bool, zone: Option<&str>) -> Option<InstallStep> {
    (firewalld_active && zone != Some("trusted")).then(|| {
        step(
            "Let VMs get an address: firewalld blocks DHCP and DNS on Multipass's private bridge",
            &format!("sudo firewall-cmd --permanent --zone=trusted --add-interface={BRIDGE} && sudo firewall-cmd --reload"),
        )
    })
}

/// Multipass's bridge on Linux (the qemu driver).
const BRIDGE: &str = "mpqemubr0";

fn network_fixes() -> Vec<InstallStep> {
    if !cfg!(target_os = "linux") || !has_command("firewall-cmd") {
        return Vec::new();
    }
    let active = LocalHost.exec(&["firewall-cmd", "--state"], QUICK).is_ok();
    let zone = LocalHost.exec(&["firewall-cmd", &format!("--get-zone-of-interface={BRIDGE}")], QUICK).ok().map(|o| o.stdout.trim().to_string());
    bridge_fix(active, zone.as_deref()).into_iter().collect()
}

fn has_nobody_in_etc() -> bool {
    let has = |f: &str| std::fs::read_to_string(f).is_ok_and(|s| s.lines().any(|l| l.starts_with("nobody:")));
    has("/etc/passwd") && has("/etc/group")
}

fn has_command(name: &str) -> bool {
    LocalHost.exec(&["sh", "-c", "command -v \"$1\"", "crow-which", name], QUICK).is_ok()
}

/// Checks for the client and its daemon.
pub fn detect() -> MultipassStatus {
    let p = platform();
    if !has_command("multipass") {
        return MultipassStatus::Missing { steps: install_steps(p, has_command("snap"), has_nobody_in_etc()) };
    }
    let out = LocalHost.exec(&["multipass", "version", "--format", "json"], QUICK);
    let (stdout, err) = match out {
        Ok(o) => (o.stdout, String::new()),
        Err(HostError::Failed { stderr, .. }) => (String::new(), stderr),
        Err(e) => (String::new(), e.to_string()),
    };
    match parse_version(&stdout) {
        Some((client, Some(daemon))) => MultipassStatus::Ready { client, daemon, fixes: network_fixes() },
        parsed => {
            let mut steps = daemon_fixes(p, has_nobody_in_etc());
            if matches!(p, Platform::Linux | Platform::LinuxOstree) {
                steps.push(step("Restart the Multipass daemon", "sudo snap restart multipass"));
            }
            let detail = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("the daemon didn't answer").trim().to_string();
            MultipassStatus::DaemonDown { client: parsed.map(|p| p.0).unwrap_or_default(), detail, steps }
        }
    }
}

/// What a new VM gets.
#[derive(Clone, Debug, PartialEq)]
pub struct LaunchSpec {
    pub name: String,
    pub cpus: u8,
    pub memory_gb: u16,
    pub disk_gb: u16,
    /// An image alias Multipass knows: `lts` (the newest LTS), `24.04`, `noble`.
    pub image: String,
}

impl Default for LaunchSpec {
    fn default() -> Self {
        Self { name: String::new(), cpus: 1, memory_gb: 1, disk_gb: 10, image: "lts".into() }
    }
}

/// Multipass's own rule for instance names: a letter, then letters,
/// digits and hyphens, not ending in a hyphen.
pub fn check_name(name: &str) -> Result<(), String> {
    let ok = name.len() <= 63
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && !name.ends_with('-')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(format!("{name:?}: a VM name is a letter, then letters, digits and hyphens (not ending in one)"))
    }
}

impl LaunchSpec {
    pub fn check(&self) -> Result<(), String> {
        check_name(&self.name)?;
        if self.cpus == 0 || self.memory_gb == 0 || self.disk_gb < 5 {
            return Err("a VM needs at least 1 CPU, 1 GB of memory and 5 GB of disk".into());
        }
        if self.image.is_empty() || !self.image.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':')) {
            return Err(format!("{:?} isn't an image name Multipass knows", self.image));
        }
        Ok(())
    }

    pub fn argv(&self) -> Vec<String> {
        vec![
            "multipass".into(),
            "launch".into(),
            "--name".into(),
            self.name.clone(),
            "--cpus".into(),
            self.cpus.to_string(),
            "--memory".into(),
            format!("{}G", self.memory_gb),
            "--disk".into(),
            format!("{}G", self.disk_gb),
            "--cloud-init".into(),
            // On stdin: the snap can't read files outside the user's home,
            // and there's no temp file to leave behind.
            "-".into(),
            self.image.clone(),
        ]
    }
}

/// cloud-init user-data that lets Crow's key log in as the image's default
/// user (`ubuntu`, with passwordless sudo) from the first boot.
pub fn cloud_init(public_key: &str) -> Result<String, String> {
    let key = public_key.trim();
    ssh_key::PublicKey::from_openssh(key).map_err(|_| "Crow's public key isn't a valid OpenSSH key".to_string())?;
    // A YAML double-quoted scalar; an OpenSSH key line has no quotes or
    // backslashes, but the comment is escaped all the same.
    let quoted = key.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!("#cloud-config\nssh_authorized_keys:\n  - \"{quoted}\"\n"))
}

/// Launches a VM with Crow's key in its cloud-init. Blocks until the VM is
/// up (minutes the first time, while the image downloads).
pub fn launch(spec: &LaunchSpec, public_key: &str) -> Result<(), String> {
    spec.check()?;
    let user_data = cloud_init(public_key)?;
    let argv = spec.argv();
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    LocalHost.exec_stdin(&argv, user_data.as_bytes(), LAUNCH).map(|_| ()).map_err(cli_error)
}

/// A Multipass error, as one line (its progress spinner and blank lines dropped).
fn cli_error(e: HostError) -> String {
    match e {
        HostError::Failed { stderr, .. } => stderr.split(['\r', '\n']).map(str::trim).filter(|l| !l.is_empty()).last().unwrap_or("multipass failed").to_string(),
        other => other.to_string(),
    }
}

/// A Multipass VM as `multipass list` / `info` report it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instance {
    pub name: String,
    /// Running, Stopped, Suspended, Starting, Deleted, Unknown…
    pub state: String,
    pub ipv4: Vec<String>,
    /// "Ubuntu 26.04 LTS" (list) or "Ubuntu 26.04.1 LTS" (info).
    pub release: String,
    /// From `info` only (empty from `list`).
    pub cpus: String,
    pub memory_total: Option<u64>,
    pub disk_total: Option<u64>,
}

impl Instance {
    pub fn is_running(&self) -> bool {
        self.state == "Running"
    }

    /// The address Crow reaches it on: its first IPv4 (the bridge's).
    pub fn address(&self) -> Option<&str> {
        self.ipv4.first().map(String::as_str)
    }
}

/// `multipass list --format json`.
pub fn parse_list(json: &str) -> Result<Vec<Instance>, String> {
    #[derive(Deserialize)]
    struct Item {
        name: String,
        state: String,
        #[serde(default)]
        ipv4: Vec<String>,
        #[serde(default)]
        release: String,
    }
    #[derive(Deserialize)]
    struct List {
        list: Vec<Item>,
    }
    let l: List = serde_json::from_str(json).map_err(|e| format!("unexpected `multipass list` output: {e}"))?;
    Ok(l.list.into_iter().map(|i| Instance { name: i.name, state: i.state, ipv4: i.ipv4, release: i.release, ..Default::default() }).collect())
}

/// `multipass info <name> --format json`. Sizes are numbers for memory and
/// strings for disks; both are missing while a VM isn't running.
pub fn parse_info(json: &str, name: &str) -> Result<Instance, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("unexpected `multipass info` output: {e}"))?;
    let i = v.get("info").and_then(|i| i.get(name)).ok_or_else(|| format!("multipass has no VM named {name}"))?;
    let text = |k: &str| i.get(k).and_then(|x| x.as_str()).unwrap_or_default().to_string();
    let number = |x: Option<&serde_json::Value>| x.and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse().ok())));
    Ok(Instance {
        name: name.to_string(),
        state: text("state"),
        ipv4: i.get("ipv4").and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default(),
        release: text("release"),
        cpus: text("cpu_count"),
        memory_total: number(i.get("memory").and_then(|m| m.get("total"))),
        disk_total: i.get("disks").and_then(|d| d.as_object()).map(|d| d.values().filter_map(|x| number(x.get("total"))).sum()).filter(|t| *t > 0),
    })
}

pub fn list() -> Result<Vec<Instance>, String> {
    let out = LocalHost.exec(&["multipass", "list", "--format", "json"], QUICK).map_err(cli_error)?;
    parse_list(&out.stdout)
}

pub fn info(name: &str) -> Result<Instance, String> {
    check_name(name)?;
    let out = LocalHost.exec(&["multipass", "info", name, "--format", "json"], QUICK).map_err(cli_error)?;
    parse_info(&out.stdout, name)
}

/// Runs `script` in the VM as its default user, with `args` as $1, $2…
fn exec_in(name: &str, script: &str, args: &[&str]) -> Result<String, String> {
    check_name(name)?;
    let mut argv = vec!["multipass", "exec", name, "--", "sh", "-c", script, "crow"];
    argv.extend_from_slice(args);
    LocalHost.exec(&argv, QUICK).map(|o| o.stdout).map_err(cli_error)
}

/// The VM's SSH host keys as known_hosts lines for `address`, read through
/// Multipass rather than from the network, so Crow pins them without
/// trusting a first connection.
pub fn host_key_lines(name: &str, address: &str) -> Result<Vec<String>, String> {
    let out = exec_in(name, "cat /etc/ssh/ssh_host_*_key.pub", &[])?;
    let lines = known_hosts_lines(&out, address);
    if lines.is_empty() {
        return Err(format!("{name} has no SSH host keys yet; is it still booting?"));
    }
    Ok(lines)
}

/// `alg blob [comment]` lines → `address alg blob`, valid keys only.
pub fn known_hosts_lines(pub_files: &str, address: &str) -> Vec<String> {
    pub_files
        .lines()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let (alg, blob) = (parts.next()?, parts.next()?);
            ssh_key::PublicKey::from_openssh(&format!("{alg} {blob}")).ok()?;
            Some(format!("{address} {alg} {blob}"))
        })
        .collect()
}

/// Adds `public_key` to the default user's authorized_keys in the VM
/// (once), so an existing VM can be imported.
pub fn authorize_key(name: &str, public_key: &str) -> Result<(), String> {
    let key = public_key.trim();
    ssh_key::PublicKey::from_openssh(key).map_err(|_| "not a valid OpenSSH public key".to_string())?;
    exec_in(name, r#"umask 077; mkdir -p "$HOME/.ssh"; f="$HOME/.ssh/authorized_keys"; touch "$f"; grep -qxF -- "$1" "$f" || printf '%s
' "$1" >> "$f""#, &[key]).map(|_| ())
}

/// What can be done to a VM from Crow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Start,
    Stop,
    Restart,
    Suspend,
    /// `delete --purge`: the VM and its disk are gone for good.
    Purge,
}

impl Lifecycle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
            Self::Suspend => "Suspend",
            Self::Purge => "Delete for good",
        }
    }

    pub fn argv(self, name: &str) -> Vec<String> {
        let verb: &[&str] = match self {
            Self::Start => &["start"],
            Self::Stop => &["stop"],
            Self::Restart => &["restart"],
            Self::Suspend => &["suspend"],
            Self::Purge => &["delete", "--purge"],
        };
        std::iter::once("multipass").chain(verb.iter().copied()).chain(std::iter::once(name)).map(str::to_string).collect()
    }
}

pub fn run(action: Lifecycle, name: &str) -> Result<(), String> {
    check_name(name)?;
    let argv = action.argv(name);
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    LocalHost.exec(&argv, LIFECYCLE).map(|_| ()).map_err(cli_error)
}

/// Marks a fleet server as a Multipass VM; `provider_instance` is the VM's
/// name, so it survives the server being renamed in Crow.
pub const PROVIDER: &str = "multipass";

/// The VM name behind a fleet server, when it is one.
pub fn vm_of(server: &ServerRecord) -> Option<&str> {
    (server.provider_account == PROVIDER && !server.provider_instance.is_empty()).then_some(server.provider_instance.as_str())
}

/// Waits for a VM to be running with an address and SSH host keys.
pub fn wait_until_up(name: &str, timeout: Duration) -> Result<(Instance, Vec<String>), String> {
    let deadline = std::time::Instant::now() + timeout;
    let mut last = String::from("not running yet");
    while std::time::Instant::now() < deadline {
        match info(name) {
            Ok(i) if i.is_running() => match i.address().map(str::to_string) {
                Some(ip) => match host_key_lines(name, &ip) {
                    Ok(keys) => return Ok((i, keys)),
                    Err(e) => last = e,
                },
                None => last = "no address yet".into(),
            },
            Ok(i) => last = format!("state {}", i.state),
            Err(e) => last = e,
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(format!("{name} didn't come up in {}s ({last})", timeout.as_secs()))
}

/// The fleet record for a VM that's up: reached on its bridge address as
/// the image's default user with Crow's key, its host key pinned.
pub fn server_record(inst: &Instance, host_keys: &[String], key: &SshKeyRecord) -> ServerRecord {
    let now = chrono::Utc::now().to_rfc3339();
    let fingerprints: Vec<(String, String)> = host_keys.iter().filter_map(|l| crate::views::onboard::probe::key_fingerprint(l)).collect();
    let host_key_fingerprint = fingerprints
        .iter()
        .find(|(alg, _)| alg == "ssh-ed25519")
        .or(fingerprints.first())
        .map(|(alg, fp)| format!("{fp} ({})", alg.trim_start_matches("ssh-").to_uppercase()));
    ServerRecord {
        id: format!("multipass-{}", inst.name),
        name: inst.name.clone(),
        host: inst.address().unwrap_or_default().to_string(),
        port: 22,
        login_user: "ubuntu".into(),
        auth_method: "publickey".into(),
        key_id: Some(key.id.clone()),
        env: "LAB".into(),
        role: "multipass vm".into(),
        group_name: "local-lab".into(),
        tags: vec!["local".into(), PROVIDER.into()],
        host_key_fingerprint,
        os_distro: inst.release.clone(),
        status: "online".into(),
        created_at: now.clone(),
        last_seen_at: Some(now),
        provider_account: PROVIDER.into(),
        provider_instance: inst.name.clone(),
        ..Default::default()
    }
}

/// Pins `lines` for their address in ~/.ssh/known_hosts. The keys came
/// from inside the VM, so an older entry for the same bridge address with
/// other keys (a deleted VM's, as Multipass hands addresses out again) is
/// replaced; one that already matches is left alone.
pub fn pin_host_keys(lines: &[String]) -> Result<(), String> {
    let Some(address) = lines.first().and_then(|l| l.split_whitespace().next()) else { return Err("no host keys to pin".into()) };
    let known = LocalHost.exec(&["ssh-keygen", "-F", address], QUICK).map(|o| o.stdout).unwrap_or_default();
    let blobs: Vec<&str> = lines.iter().filter_map(|l| l.split_whitespace().nth(2)).collect();
    let known_blobs: Vec<&str> = known.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_whitespace().nth(2)).collect();
    if !known_blobs.is_empty() && known_blobs.iter().all(|b| blobs.contains(b)) {
        return Ok(());
    }
    if !known_blobs.is_empty() {
        LocalHost.exec(&["ssh-keygen", "-R", address], QUICK).map_err(|e| format!("couldn't drop the old host key for {address}: {e}"))?;
    }
    crate::views::onboard::probe::trust_host_keys(lines).map_err(|e| format!("couldn't write ~/.ssh/known_hosts: {e}"))
}

/// Brings a running VM into the fleet: waits for it, pins its host keys,
/// logs in with Crow's key and reads its facts. Blocking; run it off the
/// UI thread.
pub fn enroll(name: &str, key: &SshKeyRecord) -> Result<ServerRecord, String> {
    let (inst, host_keys) = wait_until_up(name, Duration::from_secs(180))?;
    pin_host_keys(&host_keys)?;
    let mut record = server_record(&inst, &host_keys, key);
    let host = crate::host::host_for(&record);
    host.exec(&["true"], QUICK).map_err(|e| format!("Crow's key doesn't log in to {name} as ubuntu: {e}"))?;
    let (facts, _) = crate::views::onboard::probe::gather_facts(host.as_ref());
    let known = |v: &str| (v != "—").then(|| v.to_string());
    record.os_distro = known(&facts.distro).unwrap_or(record.os_distro);
    record.os_kernel = known(&facts.kernel).unwrap_or_default();
    record.arch = known(&facts.arch).unwrap_or_default();
    record.memory_total = known(&facts.memory).unwrap_or_default();
    record.disk_total = known(&facts.disk).unwrap_or_default();
    Ok(record)
}

/// Launches a VM with Crow's key and enrolls it.
pub fn launch_and_enroll(spec: &LaunchSpec, key: &SshKeyRecord) -> Result<ServerRecord, String> {
    launch(spec, &key.public_key)?;
    enroll(&spec.name, key)
}

/// Adds Crow's key to a VM Crow didn't make, then enrolls it.
pub fn import(name: &str, key: &SshKeyRecord) -> Result<ServerRecord, String> {
    authorize_key(name, &key.public_key)?;
    enroll(name, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vm_becomes_an_ssh_server_marked_as_multipass() {
        let inst = parse_info(INFO, "crow-probe").unwrap();
        let key = SshKeyRecord { id: "k1".into(), name: "crow".into(), ..Default::default() };
        let keys = vec!["10.5.210.53 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID6IeWJtxICF/halpN1E+KtZi88x2yIZCTlt4CjfYRyr".to_string()];
        let r = server_record(&inst, &keys, &key);
        assert_eq!((r.host.as_str(), r.port, r.login_user.as_str(), r.key_id.as_deref()), ("10.5.210.53", 22, "ubuntu", Some("k1")));
        assert!(r.host_key_fingerprint.as_deref().is_some_and(|f| f.starts_with("SHA256:") && f.ends_with("(ED25519)")));
        assert_eq!(vm_of(&r), Some("crow-probe"));
        assert!(!r.tags.iter().any(|t| t == "test-node"), "test-node would route it through podman exec");
    }

    // Captured from multipass 1.16.4 on 2026-10-07.
    const LIST: &str = "{\n    \"list\": [\n        {\n            \"ipv4\": [\n                \"10.5.210.53\"\n            ],\n            \"name\": \"crow-probe\",\n            \"release\": \"Ubuntu 26.04 LTS\",\n            \"state\": \"Running\"\n        }\n    ]\n}\n";
    const INFO: &str = r#"{"errors":[],"info":{"crow-probe":{"cpu_count":"1","disks":{"sda1":{"total":"5120138752","used":"2179761664"}},"image_hash":"88","image_release":"26.04 LTS","ipv4":["10.5.210.53"],"load":[0.2,0.12,0.04],"memory":{"total":996909056,"used":199127040},"mounts":{},"release":"Ubuntu 26.04.1 LTS","snapshot_count":"0","state":"Running"}}}"#;
    const INFO_UNKNOWN: &str = r#"{"errors":[],"info":{"crow-probe":{"cpu_count":"","disks":{"sda1":{}},"image_hash":"88","image_release":"26.04 LTS","ipv4":[],"load":[],"memory":{},"mounts":{},"release":"","snapshot_count":"0","state":"Unknown"}}}"#;

    #[test]
    fn list_and_info_read_the_real_json() {
        let l = parse_list(LIST).unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].name.as_str(), l[0].address(), l[0].is_running()), ("crow-probe", Some("10.5.210.53"), true));
        assert!(parse_list(r#"{"list": []}"#).unwrap().is_empty());
        let i = parse_info(INFO, "crow-probe").unwrap();
        assert_eq!((i.cpus.as_str(), i.memory_total, i.disk_total, i.release.as_str()), ("1", Some(996909056), Some(5120138752), "Ubuntu 26.04.1 LTS"));
        let u = parse_info(INFO_UNKNOWN, "crow-probe").unwrap();
        assert_eq!((u.address(), u.memory_total, u.disk_total, u.state.as_str()), (None, None, None, "Unknown"));
        assert!(parse_info(INFO, "other").is_err());
    }

    #[test]
    fn host_keys_become_known_hosts_lines_for_the_vm_address() {
        let pubs = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID6IeWJtxICF/halpN1E+KtZi88x2yIZCTlt4CjfYRyr root@crow-probe\nnot a key\n";
        assert_eq!(known_hosts_lines(pubs, "10.5.210.53"), ["10.5.210.53 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID6IeWJtxICF/halpN1E+KtZi88x2yIZCTlt4CjfYRyr"]);
    }

    #[test]
    fn firewalld_needs_the_bridge_trusted() {
        assert!(bridge_fix(true, None).is_some(), "no zone: the default zone blocks DHCP");
        assert!(bridge_fix(true, Some("FedoraWorkstation")).is_some());
        assert!(bridge_fix(true, Some("trusted")).is_none());
        assert!(bridge_fix(false, None).is_none());
    }

    #[test]
    fn purge_is_delete_purge_and_names_are_last() {
        assert_eq!(Lifecycle::Purge.argv("lab"), ["multipass", "delete", "--purge", "lab"]);
        assert_eq!(Lifecycle::Suspend.argv("lab"), ["multipass", "suspend", "lab"]);
    }

    #[test]
    fn versions_come_from_the_json_and_a_silent_daemon_shows() {
        assert_eq!(parse_version("{\n    \"multipass\": \"1.16.4\",\n    \"multipassd\": \"1.16.4\"\n}\n"), Some(("1.16.4".into(), Some("1.16.4".into()))));
        assert_eq!(parse_version("{\"multipass\": \"1.16.4\"}"), Some(("1.16.4".into(), None)));
        assert_eq!(parse_version("[error] [client] Caught an unhandled exception"), None);
    }

    #[test]
    fn install_steps_fit_the_system_and_skip_what_is_done() {
        let ostree = install_steps(Platform::LinuxOstree, false, false);
        assert!(ostree[0].command.starts_with("rpm-ostree install snapd"));
        assert!(ostree.iter().any(|s| s.command.contains("/usr/lib/passwd")), "the nobody fix this machine needed");
        assert_eq!(ostree.last().unwrap().command, "sudo snap install multipass");
        let done = install_steps(Platform::LinuxOstree, true, true);
        assert_eq!(done.len(), 1, "only the install left: {done:?}");
        assert!(install_steps(Platform::Linux, true, false).iter().all(|s| !s.command.contains("nobody")), "only ostree systems hide nobody");
        assert_eq!(install_steps(Platform::MacOs, false, false)[0].command, "brew install --cask multipass");
    }

    #[test]
    fn names_follow_multipass_rules() {
        assert!(check_name("crow-lab-1").is_ok());
        for bad in ["", "1abc", "-x", "a-", "a_b", "a b", "--help"] {
            assert!(check_name(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn cloud_init_carries_only_a_valid_key() {
        let key = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIPvq+8GxB7FzEsul3Rsu3zxLyEI01HX64+XvMOGJM4M8 crow@host";
        assert_eq!(cloud_init(key).unwrap(), format!("#cloud-config\nssh_authorized_keys:\n  - \"{key}\"\n"));
        assert!(cloud_init("ssh-ed25519 notakey\nruncmd: [reboot]").is_err());
    }

    #[test]
    fn launch_reads_cloud_init_from_stdin() {
        let spec = LaunchSpec { name: "lab".into(), ..Default::default() };
        assert!(spec.check().is_ok());
        let argv = spec.argv();
        let at = argv.iter().position(|a| a == "--cloud-init").unwrap();
        assert_eq!(argv[at + 1], "-");
        assert_eq!(argv.last().unwrap(), "lts");
        assert!(LaunchSpec { image: "lts; rm".into(), ..spec.clone() }.check().is_err());
        assert!(LaunchSpec { disk_gb: 2, ..spec }.check().is_err());
    }

    /// Real Multipass, end to end: launch with a key in cloud-init, wait for
    /// the VM, pin its host keys (read through Multipass) in a test
    /// known_hosts, log in over SSH, read facts, import a second key, stop,
    /// start and purge. Needs a working `multipass`; leaves nothing behind
    /// and never touches ~/.ssh:
    ///   CROW_LIVE_MULTIPASS=1 cargo test live_multipass -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_multipass_launch_enroll_import_lifecycle() {
        assert!(std::env::var("CROW_LIVE_MULTIPASS").is_ok(), "set CROW_LIVE_MULTIPASS=1");
        assert!(detect().is_ready(), "multipass isn't ready: {:?}", detect());
        let dir = std::env::temp_dir().join(format!("crow-live-mp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let keygen = |n: &str| {
            let p = dir.join(n);
            LocalHost.exec(&["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", n, "-f", &p.display().to_string()], QUICK).unwrap();
            (p.display().to_string(), std::fs::read_to_string(p.with_extension("pub")).unwrap())
        };
        let (key_path, public_key) = keygen("launch");
        let key = SshKeyRecord { id: "live".into(), name: "crow".into(), public_key: public_key.clone(), ..Default::default() };
        let name = format!("crow-live-{}", std::process::id());
        let spec = LaunchSpec { name: name.clone(), ..Default::default() };
        let cleanup = || {
            let _ = run(Lifecycle::Purge, &name);
            let _ = std::fs::remove_dir_all(&dir);
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            launch(&spec, &public_key).expect("launch");
            let (inst, host_keys) = wait_until_up(&name, Duration::from_secs(180)).expect("up");
            eprintln!("{name} up at {:?}, {} host keys", inst.address(), host_keys.len());
            let known_hosts = dir.join("known_hosts");
            std::fs::write(&known_hosts, host_keys.join("\n") + "\n").unwrap();
            let wrapper = dir.join("ssh");
            std::fs::write(&wrapper, format!("#!/bin/sh\nexec ssh -o UserKnownHostsFile={} \"$@\"\n", known_hosts.display())).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
            let login = |record: &ServerRecord, key_path: &str, sub: &str| {
                std::fs::create_dir_all(dir.join(sub)).unwrap();
                crate::host::SshHost::new(record, Some(key_path.into()), None, dir.join(sub)).with_program(&wrapper.display().to_string())
            };

            let record = server_record(&inst, &host_keys, &key);
            assert!(record.host_key_fingerprint.is_some());
            let host = login(&record, &key_path, "a");
            let me = host.exec(&["sh", "-c", "id -un; sudo -n true && echo sudo"], QUICK).expect("cloud-init key logs in");
            assert_eq!(me.stdout.split_whitespace().collect::<Vec<_>>(), ["ubuntu", "sudo"]);
            let (facts, _) = crate::views::onboard::probe::gather_facts(&host);
            assert!(facts.distro.contains("Ubuntu"), "{}", facts.distro);

            // Import: a key Crow didn't launch with, added over multipass exec.
            let (import_path, import_pub) = keygen("import");
            authorize_key(&name, &import_pub).unwrap();
            authorize_key(&name, &import_pub).unwrap();
            let count = exec_in(&name, "grep -c crow-import-marker-never ~/.ssh/authorized_keys; grep -cF -- \"$1\" ~/.ssh/authorized_keys", &[import_pub.trim()]).unwrap_or_default();
            assert_eq!(count.lines().last(), Some("1"), "added once");
            assert!(login(&record, &import_path, "b").exec(&["true"], QUICK).is_ok(), "the imported key logs in");

            run(Lifecycle::Stop, &name).expect("stop");
            assert_eq!(info(&name).unwrap().state, "Stopped");
            run(Lifecycle::Start, &name).expect("start");
            let (again, _) = wait_until_up(&name, Duration::from_secs(120)).expect("up again");
            eprintln!("after restart: {:?}", again.address());
        }));
        cleanup();
        assert!(!list().unwrap_or_default().iter().any(|i| i.name == name), "purged");
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}
