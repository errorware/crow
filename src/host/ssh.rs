//! Remote servers over the system OpenSSH client.
//!
//! Crow drives `ssh` rather than embedding an SSH library, so it uses exactly
//! what a terminal would: the user's agent, `~/.ssh/config`, jump hosts and
//! `known_hosts` (where onboarding records accepted host keys). A ControlMaster
//! socket per server keeps one connection open, so the 2-second metrics poll
//! reuses it instead of reconnecting.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::Duration;

use super::local::run_command;
use super::{ExecOutput, Host, HostError};
use crate::vault::{ServerRecord, SshKeyRecord};

/// How a server can currently be reached, as seen by its last SSH command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    /// The server rejected our credentials (key, agent) or needs a password.
    AuthFailed(String),
    /// The host key is not in known_hosts, or changed since it was accepted.
    HostKeyRejected(String),
    /// Refused, timed out, DNS failure, no route.
    Unreachable(String),
}

impl ConnectionState {
    pub fn label(&self) -> &'static str {
        match self {
            ConnectionState::Connected => "CONNECTED",
            ConnectionState::AuthFailed(_) => "AUTH FAILED",
            ConnectionState::HostKeyRejected(_) => "HOST KEY",
            ConnectionState::Unreachable(_) => "UNREACHABLE",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            ConnectionState::Connected => None,
            ConnectionState::AuthFailed(d) | ConnectionState::HostKeyRejected(d) | ConnectionState::Unreachable(d) => Some(d),
        }
    }
}

/// Settings → Connection & SSH, as applied to every connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SshSettings {
    /// Seconds to wait for the TCP connection.
    pub connect_timeout: u32,
    /// Seconds between keepalives (two missed ones drop the connection).
    pub keepalive_interval: u32,
    /// Reuse one connection per server (ControlMaster).
    pub control_master: bool,
}

impl Default for SshSettings {
    fn default() -> Self {
        Self { connect_timeout: 8, keepalive_interval: 15, control_master: true }
    }
}

fn ssh_settings() -> &'static RwLock<SshSettings> {
    static SETTINGS: OnceLock<RwLock<SshSettings>> = OnceLock::new();
    SETTINGS.get_or_init(Default::default)
}

/// Applies saved settings to connections opened from now on.
pub fn set_ssh_settings(s: SshSettings) {
    if let Ok(mut current) = ssh_settings().write() {
        *current = s;
    }
}

/// The `-o` options every connection gets. Security ones are fixed;
/// the rest come from Settings → Connection & SSH.
fn base_options(settings: &SshSettings) -> Vec<String> {
    let fixed = [
        "BatchMode=yes".to_string(),
        "StrictHostKeyChecking=yes".to_string(),
        // Crow never needs them; override whatever ~/.ssh/config says.
        "ForwardAgent=no".to_string(),
        "ForwardX11=no".to_string(),
        "ServerAliveCountMax=2".to_string(),
    ];
    let configured = [
        format!("ConnectTimeout={}", settings.connect_timeout.max(1)),
        format!("ServerAliveInterval={}", settings.keepalive_interval.max(1)),
        format!("ControlMaster={}", if settings.control_master { "auto" } else { "no" }),
        format!("ControlPersist={}", if settings.control_master { "600" } else { "no" }),
    ];
    fixed.into_iter().chain(configured).flat_map(|o| ["-o".to_string(), o]).collect()
}

/// Whether the private key at `path` works without an agent: an OpenSSH key
/// with no passphrase. Encrypted, unreadable or unrecognised keys keep the
/// agent (it may hold the unlocked key). Cached by path and mtime, since
/// connections are set up every few seconds.
fn key_usable_without_agent(path: &str) -> bool {
    static CACHE: OnceLock<Mutex<HashMap<String, (std::time::SystemTime, bool)>>> = OnceLock::new();
    let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) else { return false };
    let cache = CACHE.get_or_init(Default::default);
    if let Some((t, ok)) = cache.lock().ok().and_then(|c| c.get(path).copied()) {
        if t == mtime {
            return ok;
        }
    }
    let ok = std::fs::read_to_string(path).ok().is_some_and(|text| key_text_usable_without_agent(&text));
    if let Ok(mut c) = cache.lock() {
        c.insert(path.to_string(), (mtime, ok));
    }
    ok
}

fn key_text_usable_without_agent(text: &str) -> bool {
    ssh_key::PrivateKey::from_openssh(text).is_ok_and(|k| !k.is_encrypted())
}

/// The agent behind `ssh -G` output (or `SSH_AUTH_SOCK` when the config sets
/// none) if it's one that asks you to approve each use of a key.
pub fn approval_agent_name(ssh_g: &str, auth_sock: Option<&str>) -> Option<&'static str> {
    let configured = ssh_g.lines().find_map(|l| l.strip_prefix("identityagent ")).map(str::trim);
    let socket = match configured {
        Some("none") => return None,
        Some("SSH_AUTH_SOCK") | None => auth_sock?,
        Some(path) => path,
    };
    let s = socket.to_lowercase();
    if s.contains("1password") || s.contains("2bua8c4s2c") {
        Some("1Password")
    } else if s.contains("secretive") {
        Some("Secretive")
    } else {
        None
    }
}

/// The approving agent ssh would use for `server` (`ssh -G` with Crow's own
/// options), if any.
pub fn approval_agent_for(server: &ServerRecord) -> Option<&'static str> {
    let host = SshHost::for_server(server);
    let mut argv: Vec<&str> = vec![host.program.as_str(), "-G"];
    argv.extend(host.args.iter().map(String::as_str));
    let out = run_command(&argv, &[], Duration::from_secs(5)).ok()?;
    approval_agent_name(&out.stdout, std::env::var("SSH_AUTH_SOCK").ok().as_deref())
}

/// Turns connection args into ones for a brand-new login: no reused
/// (ControlMaster) connection, keys only. ssh keeps the first value of each
/// option, so the existing ones are replaced, not appended to.
fn fresh_login_args(mut args: Vec<String>) -> Vec<String> {
    for a in args.iter_mut() {
        if a.starts_with("ControlMaster=") {
            *a = "ControlMaster=no".into();
        } else if a.starts_with("ControlPersist=") {
            *a = "ControlPersist=no".into();
        } else if a.starts_with("ControlPath=") {
            *a = "ControlPath=none".into();
        }
    }
    let at = args.iter().position(|a| a == "--").unwrap_or(args.len());
    args.splice(at..at, ["-o".to_string(), "PreferredAuthentications=publickey".to_string()]);
    args
}

/// Logs in to `server` anew, with its key only, and runs `true`: proof that
/// key login works on its own, not just through a connection Crow holds.
pub fn fresh_key_login(server: &ServerRecord) -> Result<(), String> {
    let host = SshHost::for_server(server);
    if let Some(why) = &host.unsupported {
        return Err(why.clone());
    }
    let mut argv: Vec<String> = vec![host.program.clone()];
    argv.extend(fresh_login_args(host.args.clone()));
    argv.push("true".into());
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    run_command(&argv, &[], Duration::from_secs(20)).map(|_| ()).map_err(|e| match e {
        HostError::Failed { stderr, .. } => stderr.trim().lines().last().unwrap_or("login failed").to_string(),
        other => other.to_string(),
    })
}

fn states() -> &'static Mutex<HashMap<String, ConnectionState>> {
    static STATES: OnceLock<Mutex<HashMap<String, ConnectionState>>> = OnceLock::new();
    STATES.get_or_init(Default::default)
}

/// Last known connection state of a server, if Crow has talked to it over SSH.
pub fn connection_state(server_id: &str) -> Option<ConnectionState> {
    states().lock().ok()?.get(server_id).cloned()
}

fn record_state(server_id: &str, state: ConnectionState) {
    if let Ok(mut map) = states().lock() {
        map.insert(server_id.to_string(), state);
    }
}

/// What SSH needs to know beyond a server's own record: key file paths and
/// the other servers (for jump hosts). The app refreshes it whenever servers
/// or keys change; collectors run on worker threads and read it from here.
#[derive(Default)]
struct Directory {
    key_paths: HashMap<String, String>,
    servers: HashMap<String, ServerRecord>,
}

fn directory() -> &'static RwLock<Directory> {
    static DIRECTORY: OnceLock<RwLock<Directory>> = OnceLock::new();
    DIRECTORY.get_or_init(Default::default)
}

pub fn update_directory(servers: &[ServerRecord], keys: &[SshKeyRecord]) {
    if let Ok(mut dir) = directory().write() {
        dir.servers = servers.iter().map(|s| (s.id.clone(), s.clone())).collect();
        dir.key_paths = keys.iter().filter_map(|k| Some((k.id.clone(), k.private_key_path.clone()?))).collect();
    }
}

/// Where ssh keeps its multiplexing sockets. Unix socket paths are short
/// (~104 bytes), so this stays near the root and ssh hashes the rest (%C).
/// Where ssh keeps Crow's ControlMaster sockets: a folder only this user
/// can use (ERR-110). Without a runtime dir it would sit in the shared
/// /tmp, where another user could create it first; the folder is per uid,
/// made 0700, and refused if it exists with another owner, other modes or
/// as a symlink (then the cache dir is tried).
fn control_dir() -> PathBuf {
    #[cfg(unix)]
    {
        let uid = unsafe { libc::geteuid() };
        let name = format!("crow-ssh-{uid}");
        let candidates = [dirs::runtime_dir().unwrap_or_else(std::env::temp_dir), dirs::cache_dir().unwrap_or_else(std::env::temp_dir)];
        for base in &candidates {
            let dir = base.join(&name);
            if private_dir(&dir, uid).is_ok() {
                return dir;
            }
        }
        // Nothing private to be had: ssh refuses the socket with a clear error.
        candidates[1].join(name)
    }
    #[cfg(not(unix))]
    {
        dirs::runtime_dir().unwrap_or_else(std::env::temp_dir).join("crow-ssh")
    }
}

/// Makes `dir` (0700) if it's missing; if it exists, it must be a real
/// directory owned by `uid` that no one else can enter.
#[cfg(unix)]
fn private_dir(dir: &std::path::Path, uid: u32) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    match std::fs::symlink_metadata(dir) {
        Ok(m) if m.file_type().is_dir() && m.uid() == uid && m.mode() & 0o077 == 0 => Ok(()),
        Ok(_) => Err(std::io::Error::other(format!("{} isn't a private folder of this user", dir.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = dir.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::DirBuilder::new().mode(0o700).create(dir)
        }
        Err(e) => Err(e),
    }
}

/// Quotes one argument for the remote POSIX shell ssh hands the command to.
pub fn shell_quote(arg: &str) -> String {
    if !arg.is_empty() && arg.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_./=:@%+,".contains(&b)) {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// One bastion on the way to a server (ERR-152), with its own identity:
/// the jump connection logs in as the bastion, with the bastion's key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hop {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_path: Option<String>,
}

/// Names the way to a server in its control socket: `%C` covers the
/// address, port and user but not a ProxyCommand, so without this a server
/// moved behind a bastion (or out from behind one) would keep using the
/// connection it had, the old way, for up to ControlPersist.
fn route_tag(chain: &[Hop]) -> String {
    use sha2::{Digest, Sha256};
    if chain.is_empty() {
        return "direct".into();
    }
    let mut h = Sha256::new();
    for hop in chain {
        h.update(format!("{}@{}:{}:{}|", hop.user, hop.host, hop.port, hop.key_path.as_deref().unwrap_or("")));
    }
    h.finalize()[..4].iter().map(|b| format!("{b:02x}")).collect()
}

/// How many bastions deep a server may sit.
pub const MAX_HOPS: usize = 8;

/// The bastions between Crow and `server`, nearest to Crow first, from
/// each server's `jump_host_id`. A loop, a bastion that's gone, or one Crow
/// can't log in to with a key is an error naming it.
pub fn chain_for(server: &ServerRecord, servers: &HashMap<String, ServerRecord>, key_paths: &HashMap<String, String>) -> Result<Vec<Hop>, String> {
    let mut chain = Vec::new();
    let mut seen = vec![server.id.clone()];
    let mut next = server.jump_host_id.clone().filter(|j| !j.is_empty());
    while let Some(id) = next {
        if seen.contains(&id) {
            return Err(format!("the bastions loop back to {id}: fix one of them in its settings"));
        }
        if chain.len() >= MAX_HOPS {
            return Err(format!("more than {MAX_HOPS} bastions deep"));
        }
        let b = servers.get(&id).ok_or_else(|| format!("its bastion ({id}) is no longer in the fleet"))?;
        if b.host.trim().is_empty() || b.host.starts_with('-') {
            return Err(format!("bastion {} has an invalid address", b.name));
        }
        let key_path = match b.auth_method.as_str() {
            "password" => return Err(format!("bastion {} logs in with a password; Crow goes through bastions with keys", b.name)),
            "publickey" => match b.key_id.as_ref() {
                Some(k) => Some(key_paths.get(k).cloned().ok_or_else(|| format!("bastion {}'s key has no private key file", b.name))?),
                None => None,
            },
            _ => None,
        };
        chain.push(Hop { name: b.name.clone(), host: b.host.clone(), port: if b.port == 0 { 22 } else { b.port }, user: b.login_user.clone(), key_path });
        seen.push(id);
        next = b.jump_host_id.clone().filter(|j| !j.is_empty());
    }
    chain.reverse();
    Ok(chain)
}

/// `host:port` for `ssh -W`, bracketing an IPv6 address.
fn forward_target(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

/// The `ProxyCommand` that reaches `host:port` through `chain` (nearest
/// first): an ssh to the last bastion that forwards stdio (`-W`), itself
/// going through the bastions before it. Every hop gets Crow's fixed
/// security options and its own user and key; ssh's `-J` would use only
/// what ~/.ssh/config says. No `%` anywhere: ssh would expand it.
pub fn proxy_command(program: &str, settings: &SshSettings, chain: &[Hop], host: &str, port: u16) -> Result<String, String> {
    let Some((last, before)) = chain.split_last() else { return Err("no bastion".into()) };
    let mut argv: Vec<String> = vec![program.to_string()];
    for a in base_options(settings) {
        argv.push(match a.as_str() {
            a if a.starts_with("ControlMaster=") => "ControlMaster=no".into(),
            a if a.starts_with("ControlPersist=") => "ControlPersist=no".into(),
            _ => a,
        });
    }
    argv.extend(["-o".into(), "ControlPath=none".into(), "-p".into(), last.port.to_string()]);
    if !last.user.is_empty() {
        argv.extend(["-l".into(), last.user.clone()]);
    }
    if let Some(key) = &last.key_path {
        let key = expand_home(key);
        argv.extend(["-i".into(), key.clone(), "-o".into(), "IdentitiesOnly=yes".into()]);
        if key_usable_without_agent(&key) {
            argv.extend(["-o".into(), "IdentityAgent=none".into()]);
        }
    }
    if !before.is_empty() {
        argv.extend(["-o".into(), format!("ProxyCommand={}", proxy_command(program, settings, before, &last.host, last.port)?)]);
    }
    argv.extend(["-W".into(), forward_target(host, port), "--".into(), last.host.clone()]);
    if let Some(bad) = argv.iter().find(|a| a.contains('%') && !a.starts_with("ProxyCommand=")) {
        return Err(format!("{bad:?} holds a '%', which ssh would read as a token"));
    }
    Ok(argv.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" "))
}

/// The bastions on the way to `server`, from what the app has synced.
pub fn chain_of(server: &ServerRecord) -> Result<Vec<Hop>, String> {
    let dir = directory().read().map_err(|_| "the server directory is busy".to_string())?;
    chain_for(server, &dir.servers, &dir.key_paths)
}

/// The `ProxyCommand` option value for `server`, if it sits behind a bastion.
pub fn proxy_for(server: &ServerRecord) -> Result<Option<String>, String> {
    let chain = chain_of(server)?;
    if chain.is_empty() {
        return Ok(None);
    }
    let settings = ssh_settings().read().map(|s| *s).unwrap_or_default();
    proxy_command("ssh", &settings, &chain, &server.host, if server.port == 0 { 22 } else { server.port }).map(Some)
}

/// Says which hop failed: a bastion that can't be reached, refused Crow,
/// or can't reach the server behind it. `None` when it's about the server.
pub fn classify_hop_failure(stderr: &str, chain: &[Hop], target: &str) -> Option<ConnectionState> {
    let last_line = |pred: &dyn Fn(&str) -> bool| stderr.lines().rev().find(|l| pred(l)).map(|l| l.trim().to_string());
    if let Some(line) = last_line(&|l: &str| l.contains("open failed")).or_else(|| last_line(&|l: &str| l.contains("stdio forwarding failed"))) {
        let via = chain.last().map(|h| h.name.as_str()).unwrap_or("the bastion");
        let why = line.rsplit(": ").next().unwrap_or(&line).trim().to_string();
        return Some(ConnectionState::Unreachable(format!("bastion {via} can't reach {target}: {why}")));
    }
    for hop in chain {
        let mentions = |l: &str| l.contains(&format!("host {} ", hop.host)) || l.contains(&format!("@{}:", hop.host)) || l.contains(&format!(" {} port", hop.host)) || l.starts_with(&format!("{}:", hop.host));
        if let Some(line) = last_line(&|l: &str| mentions(l)) {
            let lower = line.to_lowercase();
            return Some(if lower.contains("permission denied") {
                ConnectionState::AuthFailed(format!("bastion {} refused Crow's key: {line}", hop.name))
            } else if lower.contains("host key") {
                ConnectionState::HostKeyRejected(format!("bastion {}: {line}", hop.name))
            } else {
                ConnectionState::Unreachable(format!("bastion {}: {line}", hop.name))
            });
        }
    }
    None
}

pub struct SshHost {
    server_id: String,
    /// The bastions on the way, nearest first (ERR-152).
    chain: Vec<Hop>,
    label: String,
    /// The server's address and port (the `-W` target of the last hop).
    dest: (String, u16),
    /// ssh arguments up to and including the destination.
    args: Vec<String>,
    /// Why this server can't be used at all (e.g. password auth), if so.
    unsupported: Option<String>,
    program: String,
}

impl SshHost {
    pub fn for_server(server: &ServerRecord) -> Self {
        let dir = directory().read().ok();
        let key_path = server.key_id.as_ref().and_then(|id| dir.as_ref()?.key_paths.get(id).cloned());
        let chain = match dir.as_ref() {
            Some(d) => chain_for(server, &d.servers, &d.key_paths),
            None => Ok(Vec::new()),
        };
        drop(dir);
        let mut host = Self::new(server, key_path, chain.clone().unwrap_or_default(), control_dir());
        if let Err(why) = chain {
            host.unsupported = Some(why);
        }
        host
    }

    pub(crate) fn new(server: &ServerRecord, key_path: Option<String>, chain: Vec<Hop>, control_dir: PathBuf) -> Self {
        let port = if server.port == 0 { 22 } else { server.port };
        let settings = ssh_settings().read().map(|s| *s).unwrap_or_default();
        let mut args = base_options(&settings);
        args.push("-o".into());
        args.push(format!("ControlPath={}/%C-{}", control_dir.display(), route_tag(&chain)));
        args.extend(["-p".into(), port.to_string()]);
        if !server.login_user.is_empty() {
            args.extend(["-l".into(), server.login_user.clone()]);
        }
        let mut unsupported = None;
        match server.auth_method.as_str() {
            "password" => {
                unsupported = Some("This server is set to password login; Crow connects with keys, installed once with the password.".to_string());
            }
            "publickey" => match &key_path {
                Some(path) => {
                    args.extend(["-i".into(), expand_home(path), "-o".into(), "IdentitiesOnly=yes".into()]);
                    // A passphrase-free key file needs no agent. Keeping ssh
                    // away from it stops agents that ask you to approve every
                    // use (1Password, Secretive) from stalling Crow (ERR-87).
                    if key_usable_without_agent(&expand_home(path)) {
                        args.extend(["-o".into(), "IdentityAgent=none".into()]);
                    }
                }
                None if server.key_id.is_some() => {
                    unsupported = Some("the enrolled key for this server has no private key file".to_string());
                }
                None => {}
            },
            _ => {} // "agent": ssh uses the agent and ~/.ssh/config by default
        }
        // The ProxyCommand (bastions) is added in `rebuild`, with the program.
        // The destination comes from the vault; a value starting with '-' would
        // be read by ssh as an option (e.g. -oProxyCommand=...). Refuse it, and
        // end option parsing before the destination regardless.
        if server.host.starts_with('-') || server.host.trim().is_empty() {
            unsupported = Some(format!("invalid host address {:?}", server.host));
        }
        let via = chain.last().map(|h| format!(" via {}", h.name)).unwrap_or_default();
        let mut host = Self {
            server_id: server.id.clone(),
            chain,
            dest: (server.host.clone(), port),
            label: format!("ssh:{}@{}{via}", server.login_user, server.host),
            args,
            unsupported,
            program: "ssh".into(),
        };
        host.finish_args();
        host
    }

    /// Ends the arguments with the bastions' ProxyCommand (built with this
    /// host's ssh program, so every hop runs the same client) and `-- host`.
    fn finish_args(&mut self) {
        if let Some(at) = self.args.iter().position(|a| a == "--") {
            self.args.truncate(at);
            // The ProxyCommand pair sits right before `--`.
            if self.args.len() >= 2 && self.args[self.args.len() - 1].starts_with("ProxyCommand=") {
                self.args.truncate(self.args.len() - 2);
            }
        }
        if !self.chain.is_empty() {
            let settings = ssh_settings().read().map(|s| *s).unwrap_or_default();
            match proxy_command(&self.program, &settings, &self.chain, &self.dest.0, self.dest.1) {
                Ok(cmd) => self.args.extend(["-o".into(), format!("ProxyCommand={cmd}")]),
                Err(e) => self.unsupported = Some(e),
            }
        }
        self.args.push("--".into());
        self.args.push(self.dest.0.clone());
    }

    /// The bastions on the way to this server, nearest first.
    pub fn chain(&self) -> &[Hop] {
        &self.chain
    }

    /// After a failure through bastions that ssh didn't explain (a shared
    /// connection hides the hops' own errors): logs in to each bastion in
    /// turn, fresh, and names the first one that fails.
    fn diagnose_chain(&self) -> Option<ConnectionState> {
        for (i, hop) in self.chain.iter().enumerate() {
            let record = ServerRecord {
                id: format!("{}#hop", self.server_id),
                name: hop.name.clone(),
                host: hop.host.clone(),
                port: hop.port,
                login_user: hop.user.clone(),
                auth_method: if hop.key_path.is_some() { "publickey".into() } else { "agent".into() },
                ..ServerRecord::default()
            };
            let mut probe = SshHost::new(&record, hop.key_path.clone(), self.chain[..i].to_vec(), PathBuf::from("/nonexistent"));
            probe.program = self.program.clone();
            probe.args = fresh_login_args(probe.args);
            probe.finish_args();
            let mut argv: Vec<&str> = vec![probe.program.as_str()];
            argv.extend(probe.args.iter().map(String::as_str));
            argv.push("true");
            if let Err(HostError::Failed { status: 255, stderr }) = run_command(&argv, &[], Duration::from_secs(20)) {
                let state = classify_hop_failure(&stderr, &self.chain[..i], &hop.host).unwrap_or_else(|| classify_ssh_failure(&stderr));
                let prefix = format!("bastion {}: ", hop.name);
                return Some(match state {
                    ConnectionState::AuthFailed(d) => ConnectionState::AuthFailed(format!("bastion {} refused Crow's key: {d}", hop.name)),
                    ConnectionState::HostKeyRejected(d) if !d.starts_with("bastion ") => ConnectionState::HostKeyRejected(prefix + &d),
                    ConnectionState::Unreachable(d) if !d.starts_with("bastion ") => ConnectionState::Unreachable(prefix + &d),
                    other => other,
                });
            }
        }
        // Every bastion lets Crow in: try the server itself, fresh.
        let mut argv: Vec<String> = vec![self.program.clone()];
        argv.extend(fresh_login_args(self.args.clone()));
        argv.push("true".into());
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        match run_command(&argv, &[], Duration::from_secs(20)) {
            Err(HostError::Failed { status: 255, stderr }) => classify_hop_failure(&stderr, &self.chain, &self.dest.0),
            _ => None,
        }
    }

    /// ssh with a terminal for an interactive shell (ERR-93): the same
    /// options every Crow connection uses (shared connection, strict host
    /// keys, the enrolled key, the jump host), plus -tt.
    pub fn interactive_command(&self) -> Result<(String, Vec<String>), String> {
        if let Some(why) = &self.unsupported {
            return Err(why.clone());
        }
        let mut args = vec!["-tt".to_string()];
        args.extend(self.args.iter().cloned());
        Ok((self.program.clone(), args))
    }

    #[cfg(test)]
    pub(crate) fn with_program(mut self, program: &str) -> Self {
        self.program = program.to_string();
        self.finish_args();
        self
    }
}

fn expand_home(path: &str) -> String {
    match (path.strip_prefix("~/"), dirs::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest).display().to_string(),
        _ => path.to_string(),
    }
}

/// Maps an ssh client failure (exit 255) to a connection state.
pub fn classify_ssh_failure(stderr: &str) -> ConnectionState {
    let s = stderr.to_lowercase();
    let first_line = stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("ssh failed").trim().to_string();
    if s.contains("host key verification failed") || s.contains("remote host identification has changed") || s.contains("no matching host key") {
        ConnectionState::HostKeyRejected(first_line)
    } else if s.contains("permission denied") || s.contains("too many authentication failures") || s.contains("no more authentication methods") {
        ConnectionState::AuthFailed(first_line)
    } else {
        ConnectionState::Unreachable(first_line)
    }
}

impl Host for SshHost {
    fn label(&self) -> String {
        self.label.clone()
    }

    /// Closes the multiplexed connection this server's ControlMaster holds
    /// (ERR-32): `ssh -O exit` with the same options resolves to the same `%C`
    /// control socket that `exec_stdin` opened.
    fn close_connection(&self) -> bool {
        let mut argv: Vec<&str> = vec![self.program.as_str(), "-O", "exit"];
        argv.extend(self.args.iter().map(String::as_str));
        run_command(&argv, &[], Duration::from_secs(5)).is_ok()
    }

    fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
        if let Some(why) = &self.unsupported {
            record_state(&self.server_id, ConnectionState::AuthFailed(why.clone()));
            return Err(HostError::Unreachable(why.clone()));
        }
        let remote = argv.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" ");
        let mut full: Vec<&str> = vec![self.program.as_str()];
        // args end with `-- <destination>`; everything after is the command.
        full.extend(self.args.iter().map(String::as_str));
        full.push(&remote);
        match run_command(&full, stdin, timeout) {
            // ssh exits 255 for its own failures; anything else is the remote command's status.
            Err(HostError::Failed { status: 255, stderr }) => {
                let state = classify_hop_failure(&stderr, &self.chain, &self.dest.0)
                    .or_else(|| (!self.chain.is_empty() && stderr.contains("Connection closed by UNKNOWN")).then(|| self.diagnose_chain()).flatten())
                    .unwrap_or_else(|| classify_ssh_failure(&stderr));
                let message = format!("{}: {}", state.label().to_lowercase(), state.detail().unwrap_or_default());
                record_state(&self.server_id, state);
                Err(HostError::Unreachable(message))
            }
            Err(HostError::Timeout) => {
                record_state(&self.server_id, ConnectionState::Unreachable("timed out".into()));
                Err(HostError::Timeout)
            }
            other => {
                record_state(&self.server_id, ConnectionState::Connected);
                other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::FileTypeExt;

    fn server(auth: &str, key_id: Option<&str>) -> ServerRecord {
        ServerRecord {
            id: "srv-1".into(),
            name: "db-01".into(),
            host: "10.0.4.12".into(),
            port: 2222,
            login_user: "ops".into(),
            auth_method: auth.into(),
            key_id: key_id.map(Into::into),
            ..ServerRecord::default()
        }
    }

    #[test]
    fn each_route_to_a_server_gets_its_own_connection() {
        let control = |chain: Vec<Hop>| SshHost::new(&server("publickey", Some("k1")), Some("/keys/id".into()), chain, "/run/crow".into()).args.into_iter().find(|a| a.starts_with("ControlPath=")).unwrap();
        let hop = |host: &str| Hop { name: "b".into(), host: host.into(), port: 22, user: "root".into(), key_path: Some("/keys/b".into()) };
        let direct = control(vec![]);
        let via_a = control(vec![hop("a.lan")]);
        let via_b = control(vec![hop("b.lan")]);
        assert_eq!(direct, "ControlPath=/run/crow/%C-direct");
        assert!(via_a != direct && via_b != direct && via_a != via_b, "{via_a} {via_b}");
        assert_eq!(via_a, control(vec![hop("a.lan")]), "the same route shares a connection");
        assert!(!via_a.contains("a.lan"), "no addresses in socket names");
    }

    #[test]
    fn builds_batch_multiplexed_args_with_key_and_jump() {
        let hop = Hop { name: "bastion".into(), host: "bastion.lan".into(), port: 22, user: "root".into(), key_path: Some("/keys/bastion".into()) };
        let h = SshHost::new(&server("publickey", Some("k1")), Some("/keys/id".into()), vec![hop], "/run/crow".into());
        let a = h.args.join(" ");
        assert!(a.contains("BatchMode=yes") && a.contains("StrictHostKeyChecking=yes"));
        assert!(a.contains("ControlMaster=auto") && a.contains("ControlPath=/run/crow/%C-") && !a.contains("%C-direct"));
        assert!(a.contains("-p 2222") && a.contains("-l ops"));
        assert!(a.contains("-i /keys/id -o IdentitiesOnly=yes"));
        let proxy = h.args.iter().find(|a| a.starts_with("ProxyCommand=")).expect("a bastion means a ProxyCommand");
        assert!(proxy.contains("-l root") && proxy.contains("-i /keys/bastion") && proxy.contains("-W 10.0.4.12:2222 -- bastion.lan"), "{proxy}");
        assert!(proxy.contains("BatchMode=yes") && proxy.contains("StrictHostKeyChecking=yes") && proxy.contains("ControlPath=none"), "the hop keeps Crow's security options: {proxy}");
        assert!(!a.contains(" -J "), "no -J: it would drop the hop's options");
        assert!(h.label().ends_with("via bastion"));
        assert_eq!(&h.args[h.args.len() - 2..], ["--", "10.0.4.12"]);
        assert!(h.unsupported.is_none());
    }

    fn rec(id: &str, host: &str, jump: Option<&str>, auth: &str, key: Option<&str>) -> ServerRecord {
        ServerRecord { id: id.into(), name: id.into(), host: host.into(), port: 22, login_user: "ops".into(), auth_method: auth.into(), key_id: key.map(Into::into), jump_host_id: jump.map(Into::into), ..ServerRecord::default() }
    }

    #[test]
    fn bastions_chain_nearest_first_and_nest() {
        let servers: HashMap<String, ServerRecord> = [
            rec("b1", "1.1.1.1", None, "publickey", Some("k1")),
            rec("b2", "10.0.0.2", Some("b1"), "publickey", Some("k1")),
            rec("app", "10.1.0.3", Some("b2"), "publickey", Some("k1")),
        ]
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();
        let keys: HashMap<String, String> = [("k1".to_string(), "/keys/crow".to_string())].into();
        let chain = chain_for(&servers["app"], &servers, &keys).unwrap();
        assert_eq!(chain.iter().map(|h| h.name.as_str()).collect::<Vec<_>>(), ["b1", "b2"]);
        let cmd = proxy_command("ssh", &SshSettings::default(), &chain, "10.1.0.3", 22).unwrap();
        // The outer hop (b2) forwards to the server; inside it, b1 forwards to b2.
        assert!(cmd.ends_with("-W 10.1.0.3:22 -- 10.0.0.2"), "{cmd}");
        assert!(cmd.contains("-W 10.0.0.2:22 -- 1.1.1.1"), "{cmd}");
        assert!(!cmd.contains('%'));
        // Through a shell, as ssh runs it: the quoting holds.
        let out = std::process::Command::new("sh").arg("-c").arg(format!("set -- {cmd}; printf '%s\\n' \"$@\"")).output().unwrap();
        let words = String::from_utf8(out.stdout).unwrap();
        let inner = words.lines().find(|l| l.starts_with("ProxyCommand=")).unwrap();
        assert!(inner.ends_with("-W 10.0.0.2:22 -- 1.1.1.1"), "{inner}");
        assert_eq!(forward_target("fe80::1", 22), "[fe80::1]:22");
    }

    #[test]
    fn a_bastion_crow_cant_use_is_refused_by_name() {
        let keys = HashMap::new();
        let looped: HashMap<String, ServerRecord> = [rec("a", "1.1.1.1", Some("b"), "agent", None), rec("b", "2.2.2.2", Some("a"), "agent", None)].into_iter().map(|s| (s.id.clone(), s)).collect();
        assert!(chain_for(&looped["a"], &looped, &keys).unwrap_err().contains("loop"));
        let pw: HashMap<String, ServerRecord> = [rec("b", "1.1.1.1", None, "password", None), rec("t", "2.2.2.2", Some("b"), "agent", None)].into_iter().map(|s| (s.id.clone(), s)).collect();
        assert!(chain_for(&pw["t"], &pw, &keys).unwrap_err().contains("bastion b logs in with a password"));
        let gone: HashMap<String, ServerRecord> = [rec("t", "2.2.2.2", Some("ghost"), "agent", None)].into_iter().map(|s| (s.id.clone(), s)).collect();
        assert!(chain_for(&gone["t"], &gone, &keys).unwrap_err().contains("no longer in the fleet"));
        let nokey: HashMap<String, ServerRecord> = [rec("b", "1.1.1.1", None, "publickey", Some("k9")), rec("t", "2.2.2.2", Some("b"), "agent", None)].into_iter().map(|s| (s.id.clone(), s)).collect();
        assert!(chain_for(&nokey["t"], &nokey, &keys).unwrap_err().contains("no private key file"));
    }

    #[test]
    fn a_failure_names_the_hop_it_happened_at() {
        let chain = vec![Hop { name: "edge".into(), host: "1.1.1.1".into(), port: 22, user: "ops".into(), key_path: None }];
        let down = classify_hop_failure("ssh: connect to host 1.1.1.1 port 22: Connection refused\nConnection closed by UNKNOWN port 65535\n", &chain, "10.0.0.5");
        assert!(matches!(down, Some(ConnectionState::Unreachable(m)) if m.starts_with("bastion edge:")));
        let denied = classify_hop_failure("ops@1.1.1.1: Permission denied (publickey).\n", &chain, "10.0.0.5");
        assert!(matches!(denied, Some(ConnectionState::AuthFailed(m)) if m.contains("bastion edge refused")));
        let behind = classify_hop_failure("channel 0: open failed: connect failed: No route to host\nstdio forwarding failed\n", &chain, "10.0.0.5");
        assert!(matches!(&behind, Some(ConnectionState::Unreachable(m)) if m.contains("edge can't reach 10.0.0.5")), "{behind:?}");
        assert_eq!(classify_hop_failure("ops@10.0.0.5: Permission denied (publickey).\n", &chain, "10.0.0.5"), None, "the server's own refusal");
    }

    #[test]
    fn a_host_that_looks_like_an_option_is_refused() {
        let mut srv = server("agent", None);
        srv.id = "srv-evil".into();
        srv.host = "-oProxyCommand=touch /tmp/crow-pwned".into();
        let h = SshHost::new(&srv, None, Vec::new(), "/run/crow".into());
        assert!(h.unsupported.as_deref().unwrap().contains("invalid host"));
        assert!(h.exec(&["true"], Duration::from_secs(1)).is_err());
        assert!(!std::path::Path::new("/tmp/crow-pwned").exists());
    }

    #[test]
    fn password_auth_is_reported_not_attempted() {
        let h = SshHost::new(&server("password", None), None, Vec::new(), "/run/crow".into());
        let err = h.exec(&["true"], Duration::from_secs(1)).unwrap_err();
        assert!(matches!(err, HostError::Unreachable(m) if m.contains("password")));
        assert!(matches!(connection_state("srv-1"), Some(ConnectionState::AuthFailed(_))));
    }

    #[test]
    fn quoting_survives_the_remote_shell() {
        assert_eq!(shell_quote("journalctl"), "journalctl");
        assert_eq!(shell_quote("_PID=42"), "_PID=42");
        assert_eq!(shell_quote("it's here"), "'it'\\''s here'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn ssh_failures_are_classified() {
        assert!(matches!(classify_ssh_failure("ops@10.0.4.12: Permission denied (publickey).\n"), ConnectionState::AuthFailed(_)));
        assert!(matches!(classify_ssh_failure("Host key verification failed.\n"), ConnectionState::HostKeyRejected(_)));
        assert!(matches!(classify_ssh_failure("ssh: connect to host 10.0.4.12 port 22: Connection refused\n"), ConnectionState::Unreachable(_)));
    }

    /// Runs SshHost against a stand-in `ssh` that executes the remote command
    /// with a local shell — the argv building, quoting and every exec-based
    /// file operation go through the same path a real server would.
    #[test]
    fn end_to_end_through_a_fake_ssh_client() {
        let dir = std::env::temp_dir().join(format!("crow-fake-ssh-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("ssh");
        // Last argument is the remote command string; everything before is ssh options.
        std::fs::write(&fake, "#!/bin/sh\nfor last; do :; done\nexec sh -c \"$last\"\n").unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut srv = server("agent", None);
        srv.id = "srv-fake".into();
        let h = SshHost::new(&srv, None, Vec::new(), dir.clone()).with_program(fake.to_str().unwrap());

        // Executing a just-written script can hit ETXTBSY when a parallel test
        // forks while the file is still open for writing; retry briefly.
        let mut attempt = 0;
        let out = loop {
            match h.exec(&["printf", "%s|%s", "it's", "a b"], Duration::from_secs(5)) {
                Err(HostError::Io(e)) if e.contains("busy") && attempt < 20 => {
                    attempt += 1;
                    std::thread::sleep(Duration::from_millis(50));
                }
                other => break other.unwrap(),
            }
        };
        assert_eq!(out.stdout, "it's|a b");
        assert_eq!(connection_state("srv-fake"), Some(ConnectionState::Connected));

        let work = dir.join("work dir");
        let w = work.to_str().unwrap();
        h.create_dir(w).unwrap();
        let file = format!("{w}/app.conf");
        h.write_file_atomic(&file, "key = 'value'\n").unwrap();
        assert_eq!(h.read_file(&file).unwrap(), "key = 'value'\n");
        assert_eq!(h.list_dir(w).unwrap().iter().map(|e| e.name.clone()).collect::<Vec<_>>(), vec!["app.conf"]);

        // A failing remote command is the command's failure, not a connection problem.
        let err = h.exec(&["sh", "-c", "exit 3"], Duration::from_secs(5)).unwrap_err();
        assert!(matches!(err, HostError::Failed { status: 3, .. }));
        assert_eq!(connection_state("srv-fake"), Some(ConnectionState::Connected));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Live test against a real sshd. Opt-in:
    ///   CROW_LIVE_SSH="127.0.0.1:22222:root:/path/to/key" \
    ///   CROW_LIVE_KNOWN_HOSTS=/path/to/known_hosts cargo test live_ssh -- --ignored
    /// (a known_hosts file for that endpoint, e.g. from `ssh-keyscan -p 22222 127.0.0.1`).
    #[test]
    #[ignore]
    fn live_ssh_round_trip() {
        let spec = std::env::var("CROW_LIVE_SSH").expect("set CROW_LIVE_SSH=host:port:user:key");
        let known_hosts = std::env::var("CROW_LIVE_KNOWN_HOSTS").expect("set CROW_LIVE_KNOWN_HOSTS");
        let parts: Vec<&str> = spec.splitn(4, ':').collect();
        let dir = std::env::temp_dir().join(format!("crow-live-ssh-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Real ssh, pointed at the test known_hosts instead of the user's.
        let wrapper = |kh: &str| {
            let path = dir.join(format!("ssh-{}", kh.len()));
            std::fs::write(&path, format!("#!/bin/sh\nexec ssh -o UserKnownHostsFile={kh} \"$@\"\n")).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            path.display().to_string()
        };
        let srv = ServerRecord {
            id: "live".into(),
            host: parts[0].into(),
            port: parts[1].parse().unwrap(),
            login_user: parts[2].into(),
            auth_method: "publickey".into(),
            key_id: Some("k".into()),
            ..ServerRecord::default()
        };
        let h = SshHost::new(&srv, Some(parts[3].into()), Vec::new(), dir.clone()).with_program(&wrapper(&known_hosts));

        let out = h.exec(&["uname", "-s"], Duration::from_secs(20)).unwrap();
        assert_eq!(out.stdout.trim(), "Linux");
        assert_eq!(connection_state("live"), Some(ConnectionState::Connected));
        // The ControlMaster socket keeps the connection for the next command.
        assert!(std::fs::read_dir(&dir).unwrap().flatten().any(|e| e.file_type().map(|t| t.is_socket()).unwrap_or(false)));
        let started = std::time::Instant::now();
        h.exec(&["true"], Duration::from_secs(5)).unwrap();
        assert!(started.elapsed() < Duration::from_millis(500), "multiplexed exec took {:?}", started.elapsed());

        assert_eq!(h.exec_privileged(&["id", "-u"], &[], Duration::from_secs(5)).unwrap().stdout.trim(), "0");
        let f = "/tmp/crow live test.conf";
        h.write_file_privileged(f, "a = 'b'\n").unwrap();
        assert_eq!(h.read_file(f).unwrap(), "a = 'b'\n");
        assert!(h.list_dir("/tmp").unwrap().iter().any(|e| e.name == "crow live test.conf"));
        h.remove(f, false).unwrap();
        assert!(matches!(h.exec(&["sh", "-c", "exit 7"], Duration::from_secs(5)), Err(HostError::Failed { status: 7, .. })));

        // An unknown host key is rejected, not silently accepted.
        let empty_kh = dir.join("empty_known_hosts");
        std::fs::write(&empty_kh, "").unwrap();
        let mut strict = srv.clone();
        strict.id = "live-strict".into();
        let h2 = SshHost::new(&strict, Some(parts[3].into()), Vec::new(), dir.join("sub")).with_program(&wrapper(empty_kh.to_str().unwrap()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        assert!(h2.exec(&["true"], Duration::from_secs(20)).is_err());
        assert!(matches!(connection_state("live-strict"), Some(ConnectionState::HostKeyRejected(_))));

        // A closed port is unreachable.
        let mut closed = srv.clone();
        closed.id = "live-closed".into();
        closed.port = 1;
        let h3 = SshHost::new(&closed, Some(parts[3].into()), Vec::new(), dir.join("sub")).with_program(&wrapper(&known_hosts));
        assert!(h3.exec(&["true"], Duration::from_secs(20)).is_err());
        assert!(matches!(connection_state("live-closed"), Some(ConnectionState::Unreachable(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_reach_the_ssh_options_and_security_ones_stay_fixed() {
        let args = base_options(&SshSettings { connect_timeout: 20, keepalive_interval: 30, control_master: false }).join(" ");
        assert!(args.contains("ConnectTimeout=20") && args.contains("ServerAliveInterval=30"));
        assert!(args.contains("ControlMaster=no") && args.contains("ControlPersist=no"));
        for fixed in ["BatchMode=yes", "StrictHostKeyChecking=yes", "ForwardAgent=no", "ForwardX11=no"] {
            assert!(args.contains(fixed), "{fixed} is always set");
        }
        let default = base_options(&SshSettings::default()).join(" ");
        assert!(default.contains("ControlMaster=auto") && default.contains("ControlPersist=600"));
    }

    #[test]
    fn fresh_login_never_reuses_a_connection_and_uses_keys_only() {
        let args = fresh_login_args(vec!["-o".into(), "ControlMaster=auto".into(), "-o".into(), "ControlPersist=600".into(), "-o".into(), "ControlPath=/run/crow/%C".into(), "--".into(), "10.0.0.1".into()]);
        let joined = args.join(" ");
        assert!(joined.contains("ControlMaster=no") && joined.contains("ControlPersist=no") && joined.contains("ControlPath=none"));
        assert!(!joined.contains("ControlMaster=auto"));
        let dashdash = args.iter().position(|a| a == "--").unwrap();
        assert_eq!(args[dashdash - 1], "PreferredAuthentications=publickey", "before the destination");
    }

    #[test]
    fn passphrase_free_key_files_skip_the_agent_and_encrypted_ones_keep_it() {
        let dir = std::env::temp_dir().join(format!("crow-err87-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let keygen = |name: &str, passphrase: &str| {
            let path = dir.join(name);
            let ok = std::process::Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", passphrase, "-f"]).arg(&path).status().is_ok_and(|s| s.success());
            ok.then(|| path.to_string_lossy().into_owned())
        };
        let (Some(plain), Some(locked)) = (keygen("plain", ""), keygen("locked", "a passphrase")) else {
            eprintln!("ssh-keygen not available; skipping");
            return;
        };
        assert!(key_text_usable_without_agent(&std::fs::read_to_string(&plain).unwrap()));
        assert!(!key_text_usable_without_agent(&std::fs::read_to_string(&locked).unwrap()), "an encrypted key may live unlocked in the agent");
        assert!(!key_text_usable_without_agent("-----BEGIN RSA PRIVATE KEY-----\nnot openssh\n"), "unrecognised: keep the agent");

        let srv = ServerRecord { id: "s".into(), host: "10.0.0.1".into(), login_user: "root".into(), auth_method: "publickey".into(), key_id: Some("k".into()), ..Default::default() };
        let args = |key: &str| SshHost::new(&srv, Some(key.to_string()), Vec::new(), dir.clone()).args.join(" ");
        assert!(args(&plain).contains("IdentityAgent=none"), "Crow's kind of key: no agent, no prompts");
        assert!(!args(&locked).contains("IdentityAgent"), "a passphrase key keeps the agent");
        let agent = ServerRecord { auth_method: "agent".into(), key_id: None, ..srv.clone() };
        assert!(!SshHost::new(&agent, None, Vec::new(), dir.clone()).args.join(" ").contains("IdentityAgent"), "agent logins keep the agent");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn recognises_agents_that_ask_for_approval() {
        let one = "identityagent /Users/me/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock\n";
        assert_eq!(approval_agent_name(one, None), Some("1Password"));
        assert_eq!(approval_agent_name("user root\n", Some("/home/me/.1password/agent.sock")), Some("1Password"), "from SSH_AUTH_SOCK");
        assert_eq!(approval_agent_name("identityagent SSH_AUTH_SOCK\n", Some("/Users/me/Library/Containers/com.maxgoedjen.Secretive.SecretAgent/Data/socket.ssh")), Some("Secretive"));
        assert_eq!(approval_agent_name("identityagent none\n", Some("/home/me/.1password/agent.sock")), None, "Crow turned the agent off");
        assert_eq!(approval_agent_name("user root\n", Some("/run/user/1000/keyring/ssh")), None);
    }
    /// Real sshd containers: a bastion on 127.0.0.1:22301, a second one only
    /// it can reach, and a server behind each (see the ERR-152 notes):
    ///   CROW_BASTION_LAB=/path/with/key+known_hosts cargo test live_bastion_chain -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_bastion_chain() {
        let lab = std::path::PathBuf::from(std::env::var("CROW_BASTION_LAB").expect("set CROW_BASTION_LAB"));
        let key = lab.join("key").display().to_string();
        let wrapper = |kh: &std::path::Path, name: &str| {
            let path = lab.join(name);
            std::fs::write(&path, format!("#!/bin/sh\nexec ssh -F /dev/null -o UserKnownHostsFile={} \"$@\"\n", kh.display())).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            path.display().to_string()
        };
        let ssh = wrapper(&lab.join("known_hosts"), "ssh-wrap");
        let mut b1 = rec("b1", "127.0.0.1", None, "publickey", Some("k"));
        b1.port = 22301;
        b1.login_user = "root".into();
        let servers: HashMap<String, ServerRecord> = [
            b1,
            ServerRecord { login_user: "root".into(), ..rec("b2", "crow-b2", Some("b1"), "publickey", Some("k")) },
            ServerRecord { login_user: "root".into(), ..rec("t1", "crow-t1", Some("b1"), "publickey", Some("k")) },
            ServerRecord { login_user: "root".into(), ..rec("t2", "crow-t2", Some("b2"), "publickey", Some("k")) },
        ]
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();
        let keys: HashMap<String, String> = [("k".to_string(), key.clone())].into();
        let control = std::env::temp_dir().join(format!("crow-bl-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&control);
        let host = |id: &str, program: &str, keys: &HashMap<String, String>| {
            let chain = chain_for(&servers[id], &servers, keys).unwrap();
            SshHost::new(&servers[id], Some(key.clone()), chain, control.clone()).with_program(program)
        };
        for (id, expect) in [("t1", "crow-t1"), ("t2", "crow-t2"), ("b2", "crow-b2")] {
            let h = host(id, &ssh, &keys);
            let out = h.exec(&["hostname"], Duration::from_secs(30)).unwrap_or_else(|e| panic!("{id}: {e}"));
            eprintln!("{id}: {} via {:?}", out.stdout.trim(), h.chain().iter().map(|c| &c.name).collect::<Vec<_>>());
            assert!(!out.stdout.trim().is_empty() && expect.starts_with("crow-"));
            h.close_connection();
        }
        // The hop really uses the bastion's own key: give b1 a key the bastion doesn't know.
        let other = lab.join("other_key");
        if !other.exists() {
            std::process::Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", "", "-f", other.to_str().unwrap()]).status().unwrap();
        }
        let mut bad_servers = servers.clone();
        bad_servers.get_mut("b1").unwrap().key_id = Some("other".into());
        let bad_keys: HashMap<String, String> = [("k".to_string(), key.clone()), ("other".to_string(), other.display().to_string())].into();
        let chain = chain_for(&bad_servers["t1"], &bad_servers, &bad_keys).unwrap();
        let h = SshHost::new(&bad_servers["t1"], Some(key.clone()), chain, control.clone()).with_program(&ssh);
        let err = h.exec(&["true"], Duration::from_secs(30)).unwrap_err().to_string();
        eprintln!("wrong bastion key: {err}");
        assert!(err.contains("bastion b1 refused"), "{err}");
        // A server whose host key isn't pinned is refused, through the bastion too.
        let partial = lab.join("known_hosts_no_t1");
        std::fs::write(&partial, std::fs::read_to_string(lab.join("known_hosts")).unwrap().lines().filter(|l| !l.starts_with("crow-t1")).collect::<Vec<_>>().join("\n") + "\n").unwrap();
        let strict = wrapper(&partial, "ssh-wrap-strict");
        let err = host("t1", &strict, &keys).exec(&["true"], Duration::from_secs(30)).unwrap_err().to_string();
        eprintln!("unpinned server: {err}");
        assert!(err.contains("host key"), "{err}");
        // A server the bastion can't reach says so.
        let mut lost = servers.clone();
        lost.get_mut("t1").unwrap().host = "crow-nowhere".into();
        let chain = chain_for(&lost["t1"], &lost, &keys).unwrap();
        let err = SshHost::new(&lost["t1"], Some(key.clone()), chain, control.clone()).with_program(&ssh).exec(&["true"], Duration::from_secs(30)).unwrap_err().to_string();
        eprintln!("unreachable behind the bastion: {err}");
        assert!(err.contains("b1 can't reach crow-nowhere"), "{err}");
    }
}

#[cfg(all(test, unix))]
mod control_dir_tests {
    use super::private_dir;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn only_a_private_folder_of_this_user_is_used() {
        let uid = unsafe { libc::geteuid() };
        let base = std::env::temp_dir().join(format!("crow-ctl-test-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let fresh = base.join("fresh");
        assert!(private_dir(&fresh, uid).is_ok(), "a missing folder is made");
        assert_eq!(std::fs::metadata(&fresh).unwrap().permissions().mode() & 0o777, 0o700);
        assert!(private_dir(&fresh, uid).is_ok(), "and reused");
        let open = base.join("open");
        std::fs::create_dir(&open).unwrap();
        std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(private_dir(&open, uid).is_err(), "others can enter it");
        let link = base.join("link");
        std::os::unix::fs::symlink(&fresh, &link).unwrap();
        assert!(private_dir(&link, uid).is_err(), "a symlink, even to a good folder");
        assert!(private_dir(&fresh, uid + 1).is_err(), "someone else's");
        std::fs::remove_dir_all(&base).unwrap();
    }

}
