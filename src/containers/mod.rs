//! Containers on a server (ERR-140): Docker and Podman read over SSH, as
//! the login user or through `sudo -n`, with no agent. One round trip
//! lists every runtime the server has, its containers and their compose
//! stacks; actions and logs run the runtime's own CLI the same way.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::Value;

use crate::host::{Host, HostError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Runtime {
    Docker,
    Podman,
}

impl Runtime {
    pub fn cmd(self) -> &'static str {
        match self {
            Runtime::Docker => "docker",
            Runtime::Podman => "podman",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Runtime::Docker => "Docker",
            Runtime::Podman => "Podman",
        }
    }
}

/// How Crow reaches a runtime: as the login user, or as root (through
/// `sudo -n` unless it logs in as root).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scope {
    User,
    Root,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeOutcome {
    Ok { version: String, compose: Option<String> },
    /// Installed, but Crow can't use it: why, and what fixes it.
    Problem { summary: String, fix: Option<String> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeReport {
    pub runtime: Runtime,
    pub scope: Scope,
    pub outcome: RuntimeOutcome,
}

impl RuntimeReport {
    /// "Docker 29.8.2 · through sudo", "Podman 5.8.7 · rootless".
    pub fn describe(&self, login_is_root: bool) -> String {
        let how = match (self.runtime, self.scope, login_is_root) {
            (_, Scope::Root, true) => "as root",
            (Runtime::Docker, Scope::User, _) => "docker group",
            (Runtime::Podman, Scope::User, _) => "rootless",
            (_, Scope::Root, false) => "through sudo",
        };
        match &self.outcome {
            RuntimeOutcome::Ok { version, compose } => {
                let compose = compose.as_ref().map(|c| format!(" · compose {c}")).unwrap_or_default();
                format!("{} {version} · {how}{compose}", self.runtime.label())
            }
            RuntimeOutcome::Problem { summary, .. } => format!("{}: {summary}", self.runtime.label()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PortMap {
    /// "" or "0.0.0.0" / "::" mean every address.
    pub host_ip: String,
    /// "8080", or a range "8000-8001".
    pub host_port: String,
    /// "80/tcp".
    pub container: String,
}

impl PortMap {
    /// Published on every address, so reachable from any network the
    /// server's firewall lets in.
    pub fn exposed(&self) -> bool {
        matches!(self.host_ip.as_str(), "" | "0.0.0.0" | "::" | "[::]")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Container {
    pub runtime: Runtime,
    pub scope: Scope,
    pub id: String,
    pub name: String,
    pub image: String,
    /// "running", "exited", "paused", "created", "restarting".
    pub state: String,
    /// The runtime's words: "Up 10 seconds (healthy)", "Exited (1) 7 seconds ago".
    pub status: String,
    /// "healthy", "unhealthy", "starting"; `None` without a health check.
    pub health: Option<String>,
    pub ports: Vec<PortMap>,
    pub restarts: Option<u32>,
    pub labels: BTreeMap<String, String>,
}

impl Container {
    /// Unique on the server: the same id can't belong to two runtimes, but
    /// the scope keeps rootless and root Podman apart.
    pub fn key(&self) -> String {
        format!("{}:{:?}:{}", self.runtime.cmd(), self.scope, self.id)
    }

    pub fn short_id(&self) -> &str {
        &self.id[..self.id.len().min(12)]
    }

    pub fn is_running(&self) -> bool {
        self.state == "running" || self.state == "restarting"
    }

    pub fn project(&self) -> Option<&str> {
        self.labels.get("com.docker.compose.project").or_else(|| self.labels.get("io.podman.compose.project")).map(String::as_str).filter(|p| !p.is_empty())
    }

    pub fn service(&self) -> Option<&str> {
        self.labels.get("com.docker.compose.service").or_else(|| self.labels.get("io.podman.compose.service")).map(String::as_str)
    }

    /// Published ports, one per (host port, container port): an IPv4 and an
    /// IPv6 binding of the same port count once, exposed if either is.
    pub fn published(&self) -> Vec<PortMap> {
        let mut out: Vec<PortMap> = Vec::new();
        for p in &self.ports {
            match out.iter_mut().find(|o| o.host_port == p.host_port && o.container == p.container) {
                Some(o) => {
                    if p.exposed() {
                        o.host_ip = p.host_ip.clone();
                    }
                }
                None => out.push(p.clone()),
            }
        }
        out
    }
}

/// A compose stack, from its containers' labels.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {
    pub runtime: Runtime,
    pub scope: Scope,
    pub project: String,
    pub working_dir: Option<String>,
    pub config_files: Vec<String>,
    /// Keys of its containers.
    pub containers: Vec<String>,
    pub running: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scan {
    pub login_is_root: bool,
    pub runtimes: Vec<RuntimeReport>,
    /// Runtimes the server doesn't have.
    pub missing: Vec<Runtime>,
    pub containers: Vec<Container>,
}

impl Scan {
    pub fn stacks(&self) -> Vec<Stack> {
        let mut stacks: Vec<Stack> = Vec::new();
        for c in &self.containers {
            let Some(project) = c.project() else { continue };
            let i = match stacks.iter().position(|s| s.project == project && s.runtime == c.runtime && s.scope == c.scope) {
                Some(i) => i,
                None => {
                    stacks.push(Stack {
                        runtime: c.runtime,
                        scope: c.scope,
                        project: project.to_string(),
                        working_dir: c.labels.get("com.docker.compose.project.working_dir").cloned().filter(|d| !d.is_empty()),
                        config_files: c.labels.get("com.docker.compose.project.config_files").map(|f| f.split(',').map(str::trim).filter(|f| !f.is_empty()).map(String::from).collect()).unwrap_or_default(),
                        containers: Vec::new(),
                        running: 0,
                    });
                    stacks.len() - 1
                }
            };
            stacks[i].containers.push(c.key());
            stacks[i].running += usize::from(c.is_running());
        }
        stacks.sort_by(|a, b| a.project.cmp(&b.project));
        stacks
    }

    pub fn find(&self, key: &str) -> Option<&Container> {
        self.containers.iter().find(|c| c.key() == key)
    }

    /// The runtime reports Crow could use.
    pub fn usable(&self) -> impl Iterator<Item = &RuntimeReport> {
        self.runtimes.iter().filter(|r| matches!(r.outcome, RuntimeOutcome::Ok { .. }))
    }
}

/// One round trip: for each runtime, its version, compose, `ps` and (for
/// Docker) restart counts, as the user and, where that's how it's reached,
/// through `sudo -n`.
pub const SCAN_SCRIPT: &str = r#"
uid=$(id -u)
echo "@@uid $uid"
run() { sc=$1; shift; if [ "$sc" = root ] && [ "$uid" != 0 ]; then sudo -n -- "$@"; else "$@"; fi; }
scan() {
  rt=$1; sc=$2
  echo "@@begin $rt $sc"
  if [ "$rt" = docker ]; then v=$(run "$sc" docker version --format '{{.Server.Version}}' 2>&1); else v=$(run "$sc" podman version --format '{{.Client.Version}}' 2>&1); fi
  if [ $? -ne 0 ] || [ -z "$v" ]; then
    echo "@@error $(printf '%s\n' "$v" | grep -v '^[[:space:]]*$' | tail -1)"
    echo "@@end"
    return 1
  fi
  echo "@@version $v"
  c=$(run "$sc" "$rt" compose version --short 2>/dev/null | head -1)
  [ -n "$c" ] && echo "@@compose $c"
  echo "@@ps"
  if [ "$rt" = docker ]; then
    run "$sc" docker ps -a --no-trunc --format '{{json .}}'
    ids=$(run "$sc" docker ps -aq --no-trunc)
    if [ -n "$ids" ]; then echo "@@restarts"; run "$sc" docker inspect --format '{{.Id}} {{.RestartCount}}' $ids; fi
  else
    run "$sc" podman ps -a --format json
  fi
  echo "@@end"
}
for rt in docker podman; do
  if ! command -v "$rt" >/dev/null 2>&1; then echo "@@missing $rt"; continue; fi
  if [ "$uid" = 0 ]; then
    scan "$rt" root
  elif [ "$rt" = docker ]; then
    scan docker user || { sudo -n true 2>/dev/null && scan docker root; }
  else
    scan podman user
    sudo -n true 2>/dev/null && scan podman root
  fi
done
echo "@@done"
"#;

const SCAN_TIMEOUT: Duration = Duration::from_secs(40);
const ACTION_TIMEOUT: Duration = Duration::from_secs(120);

fn problem_for(runtime: Runtime, error: &str) -> RuntimeOutcome {
    let lower = error.to_lowercase();
    let (summary, fix) = if lower.contains("permission denied") {
        (
            "the login user can't reach the Docker daemon, and has no passwordless sudo".to_string(),
            Some("sudo usermod -aG docker $USER   # then log in again; the docker group is as good as root".to_string()),
        )
    } else if lower.contains("cannot connect to the docker daemon") || lower.contains("is the docker daemon running") {
        ("installed, but its daemon isn't running".to_string(), Some("sudo systemctl enable --now docker".to_string()))
    } else if error.trim().is_empty() {
        ("gave no answer".to_string(), None)
    } else {
        (error.trim().to_string(), None)
    };
    let _ = runtime;
    RuntimeOutcome::Problem { summary, fix }
}

fn parse_labels_str(s: &str) -> BTreeMap<String, String> {
    // "k=v,k2=v2", where a value may itself hold commas: a piece without
    // '=' belongs to the value before it.
    let mut out = BTreeMap::new();
    let mut last: Option<String> = None;
    for piece in s.split(',') {
        match piece.split_once('=') {
            Some((k, v)) if !k.is_empty() && !k.contains(' ') => {
                out.insert(k.to_string(), v.to_string());
                last = Some(k.to_string());
            }
            _ => {
                if let Some(k) = &last {
                    if let Some(v) = out.get_mut(k) {
                        v.push(',');
                        v.push_str(piece);
                    }
                }
            }
        }
    }
    out
}

/// "0.0.0.0:8080->80/tcp, [::]:8080->80/tcp, 9000/tcp": only published ones.
pub fn parse_docker_ports(s: &str) -> Vec<PortMap> {
    s.split(", ")
        .filter_map(|part| {
            let (host, container) = part.trim().split_once("->")?;
            let (ip, port) = host.rsplit_once(':')?;
            Some(PortMap { host_ip: ip.trim_matches(|c| c == '[' || c == ']').to_string(), host_port: port.to_string(), container: container.to_string() })
        })
        .collect()
}

/// "Up 10 seconds (healthy)" → "healthy"; "(health: starting)" → "starting".
pub fn health_from_status(status: &str) -> Option<String> {
    let inner = status.rsplit_once('(')?.1.strip_suffix(')')?;
    let h = inner.strip_prefix("health: ").unwrap_or(inner);
    matches!(h, "healthy" | "unhealthy" | "starting").then(|| h.to_string())
}

fn s(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or_default().to_string()
}

pub fn parse_docker_line(line: &str, scope: Scope) -> Option<Container> {
    let v: Value = serde_json::from_str(line).ok()?;
    let status = s(&v, "Status");
    let health = match v.get("HealthStatus").and_then(Value::as_str) {
        Some("none") | Some("") => None,
        Some(h) => Some(h.to_string()),
        None => health_from_status(&status),
    };
    Some(Container {
        runtime: Runtime::Docker,
        scope,
        id: s(&v, "ID"),
        name: s(&v, "Names").split(',').next().unwrap_or_default().to_string(),
        image: s(&v, "Image"),
        state: s(&v, "State"),
        status,
        health,
        ports: parse_docker_ports(&s(&v, "Ports")),
        restarts: None,
        labels: parse_labels_str(&s(&v, "Labels")),
    })
}

pub fn parse_podman_json(json: &str, scope: Scope) -> Vec<Container> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(json) else { return Vec::new() };
    items
        .iter()
        .filter(|v| !v.get("IsInfra").and_then(Value::as_bool).unwrap_or(false))
        .map(|v| {
            let status = s(v, "Status");
            let ports = v
                .get("Ports")
                .and_then(Value::as_array)
                .map(|ps| {
                    ps.iter()
                        .map(|p| {
                            let host = p.get("host_port").and_then(Value::as_u64).unwrap_or(0);
                            let cont = p.get("container_port").and_then(Value::as_u64).unwrap_or(0);
                            let range = p.get("range").and_then(Value::as_u64).unwrap_or(1).max(1);
                            let span = |start: u64| if range > 1 { format!("{start}-{}", start + range - 1) } else { start.to_string() };
                            PortMap { host_ip: s(p, "host_ip"), host_port: span(host), container: format!("{}/{}", span(cont), p.get("protocol").and_then(Value::as_str).unwrap_or("tcp")) }
                        })
                        .collect()
                })
                .unwrap_or_default();
            let labels = v.get("Labels").and_then(Value::as_object).map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string())).collect()).unwrap_or_default();
            Container {
                runtime: Runtime::Podman,
                scope,
                id: s(v, "Id"),
                name: v.get("Names").and_then(Value::as_array).and_then(|n| n.first()).and_then(Value::as_str).unwrap_or_default().to_string(),
                image: s(v, "Image"),
                state: s(v, "State"),
                health: health_from_status(&status),
                status,
                ports,
                restarts: v.get("Restarts").and_then(Value::as_u64).map(|r| r as u32),
                labels,
            }
        })
        .collect()
}

/// Reads `SCAN_SCRIPT`'s output.
pub fn parse_scan(out: &str) -> Scan {
    let mut scan = Scan::default();
    let mut lines = out.lines().peekable();
    // A Docker the user couldn't reach, kept in case sudo can't either.
    let mut docker_user_error: Option<String> = None;
    while let Some(line) = lines.next() {
        if let Some(uid) = line.strip_prefix("@@uid ") {
            scan.login_is_root = uid.trim() == "0";
        } else if let Some(rt) = line.strip_prefix("@@missing ") {
            match rt.trim() {
                "docker" => scan.missing.push(Runtime::Docker),
                "podman" => scan.missing.push(Runtime::Podman),
                _ => {}
            }
        } else if let Some(rest) = line.strip_prefix("@@begin ") {
            let mut parts = rest.split_whitespace();
            let runtime = match parts.next() {
                Some("docker") => Runtime::Docker,
                Some("podman") => Runtime::Podman,
                _ => continue,
            };
            let scope = if parts.next() == Some("root") { Scope::Root } else { Scope::User };
            let (mut version, mut compose, mut error) = (None, None, None);
            let mut body = String::new();
            let mut restarts: Vec<(String, u32)> = Vec::new();
            let mut section = "";
            for l in lines.by_ref() {
                if l == "@@end" {
                    break;
                } else if let Some(v) = l.strip_prefix("@@version ") {
                    version = Some(v.trim().to_string());
                } else if let Some(c) = l.strip_prefix("@@compose ") {
                    compose = Some(c.trim().trim_start_matches('v').to_string());
                } else if let Some(e) = l.strip_prefix("@@error") {
                    error = Some(e.trim().to_string());
                } else if l == "@@ps" {
                    section = "ps";
                } else if l == "@@restarts" {
                    section = "restarts";
                } else if section == "ps" {
                    body.push_str(l);
                    body.push('\n');
                } else if section == "restarts" {
                    if let Some((id, n)) = l.trim().split_once(' ') {
                        if let Ok(n) = n.trim().parse() {
                            restarts.push((id.trim_start_matches("sha256:").to_string(), n));
                        }
                    }
                }
            }
            if let Some(e) = error {
                if runtime == Runtime::Docker && scope == Scope::User {
                    docker_user_error = Some(e);
                } else if !(runtime == Runtime::Podman && scope == Scope::Root) {
                    scan.runtimes.push(RuntimeReport { runtime, scope, outcome: problem_for(runtime, &e) });
                }
                continue;
            }
            if runtime == Runtime::Docker {
                docker_user_error = None;
            }
            let mut containers: Vec<Container> = match runtime {
                Runtime::Docker => body.lines().filter_map(|l| parse_docker_line(l, scope)).collect(),
                Runtime::Podman => parse_podman_json(&body, scope),
            };
            for c in &mut containers {
                if let Some((_, n)) = restarts.iter().find(|(id, _)| *id == c.id) {
                    c.restarts = Some(*n);
                }
            }
            // Root Podman with nothing in it isn't worth a line.
            if runtime == Runtime::Podman && scope == Scope::Root && containers.is_empty() && scan.runtimes.iter().any(|r| r.runtime == Runtime::Podman) {
                continue;
            }
            scan.containers.extend(containers);
            scan.runtimes.push(RuntimeReport { runtime, scope, outcome: RuntimeOutcome::Ok { version: version.unwrap_or_default(), compose } });
        }
    }
    if let Some(e) = docker_user_error {
        scan.runtimes.push(RuntimeReport { runtime: Runtime::Docker, scope: Scope::User, outcome: problem_for(Runtime::Docker, &e) });
    }
    scan.runtimes.sort_by_key(|r| (r.runtime, r.scope));
    scan.containers.sort_by(|a, b| a.name.cmp(&b.name));
    scan
}

/// Lists the server's containers. Blocking: run it off the UI thread.
pub fn scan(host: &dyn Host) -> Result<Scan, String> {
    match host.exec(&["sh", "-c", SCAN_SCRIPT], SCAN_TIMEOUT) {
        // A runtime failing shows in the output; the script ends in an echo,
        // so it only fails when the server can't be reached.
        Ok(out) => Ok(parse_scan(&out.stdout)),
        Err(e) => Err(e.to_string()),
    }
}

fn run_in_scope(host: &dyn Host, scope: Scope, argv: &[&str], timeout: Duration) -> Result<String, String> {
    let out = match scope {
        Scope::User => host.exec(argv, timeout),
        Scope::Root => host.exec_privileged(argv, &[], timeout),
    };
    out.map(|o| o.stdout).map_err(|e| match e {
        HostError::Failed { stderr, status } => {
            stderr.lines().rev().find(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()).unwrap_or_else(|| format!("failed (exit {status})"))
        }
        other => other.to_string(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    Restart,
}

impl Action {
    pub fn verb(self) -> &'static str {
        match self {
            Action::Start => "start",
            Action::Stop => "stop",
            Action::Restart => "restart",
        }
    }
}

pub fn action_argv(c: &Container, action: Action) -> Vec<String> {
    vec![c.runtime.cmd().into(), action.verb().into(), c.id.clone()]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackAction {
    Up,
    Restart,
    Down,
}

impl StackAction {
    pub fn verb(self) -> &'static str {
        match self {
            StackAction::Up => "up",
            StackAction::Restart => "restart",
            StackAction::Down => "down",
        }
    }
}

/// The compose command for a stack. `up` needs its files (and runs where
/// they are); `restart` and `down` find the stack by its project name.
pub fn stack_argv(stack: &Stack, action: StackAction) -> Result<Vec<String>, String> {
    let rt = stack.runtime.cmd().to_string();
    let mut argv: Vec<String> = Vec::new();
    match action {
        StackAction::Up => {
            let dir = stack.working_dir.clone().ok_or("the stack's directory isn't known (it wasn't started with compose v2)")?;
            if stack.config_files.is_empty() {
                return Err("the stack's compose files aren't known".into());
            }
            argv.extend(["sh".into(), "-c".into(), "cd \"$1\" && shift && exec \"$@\"".into(), "crow-compose".into(), dir]);
            argv.extend([rt, "compose".into(), "-p".into(), stack.project.clone()]);
            for f in &stack.config_files {
                argv.extend(["-f".into(), f.clone()]);
            }
            argv.extend(["up".into(), "-d".into()]);
        }
        StackAction::Restart | StackAction::Down => argv.extend([rt, "compose".into(), "-p".into(), stack.project.clone(), action.verb().into()]),
    }
    Ok(argv)
}

/// Runs a container or stack command the way the runtime is reached.
pub fn run(host: &dyn Host, scope: Scope, argv: &[String]) -> Result<(), String> {
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    run_in_scope(host, scope, &argv, ACTION_TIMEOUT).map(|_| ())
}

pub const LOG_LINES: usize = 400;

/// The container's last lines, stdout and stderr together, oldest first.
pub fn logs(host: &dyn Host, c: &Container) -> Result<String, String> {
    let tail = LOG_LINES.to_string();
    let argv = ["sh", "-c", "exec \"$@\" 2>&1", "crow-logs", c.runtime.cmd(), "logs", "--tail", &tail, "--timestamps", &c.id];
    run_in_scope(host, c.scope, &argv, SCAN_TIMEOUT)
}

/// Environment values aren't shown: they're where passwords and tokens live.
const SHOWN_ENV: [&str; 5] = ["PATH", "HOME", "LANG", "TZ", "HOSTNAME"];

/// `inspect`'s JSON with environment values masked, pretty-printed.
pub fn mask_inspect(json: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<Value>(json) else { return json.to_string() };
    let items = match &mut v {
        Value::Array(items) => items.iter_mut().collect::<Vec<_>>(),
        other => vec![other],
    };
    for item in items {
        if let Some(Value::Array(env)) = item.pointer_mut("/Config/Env") {
            for e in env.iter_mut() {
                if let Some(text) = e.as_str() {
                    if let Some((k, _)) = text.split_once('=') {
                        if !SHOWN_ENV.contains(&k) {
                            *e = Value::String(format!("{k}=•••• (hidden)"));
                        }
                    }
                }
            }
        }
    }
    serde_json::to_string_pretty(&v).unwrap_or_else(|_| json.to_string())
}

pub fn inspect(host: &dyn Host, c: &Container) -> Result<String, String> {
    run_in_scope(host, c.scope, &[c.runtime.cmd(), "inspect", &c.id], SCAN_TIMEOUT).map(|j| mask_inspect(&j))
}

#[cfg(test)]
mod tests {
    use super::{action_argv, health_from_status, mask_inspect, parse_docker_ports, parse_scan, stack_argv, Action, Runtime, RuntimeOutcome, Scope, StackAction};

    // Real output, Docker 29.8.2 with compose 5.6.0 (2026-10-08), trimmed.
    const DOCKER_PS: &str = r#"{"Command":"\"false\"","CreatedAt":"2026-10-08 08:08:19 +0000 UTC","HealthStatus":"none","ID":"2a28cd00442ea1a784adc856b83d353496b5dd969fcb890c9ad4fa111207d985","Image":"alpine","Labels":"","LocalVolumes":"0","Mounts":"","Names":"exited-one","Networks":"bridge","Ports":"","RunningFor":"7 seconds ago","State":"exited","Status":"Exited (1) 7 seconds ago"}
{"Command":"\"/docker-entrypoint.sh nginx -g 'daemon off;'\"","HealthStatus":"healthy","ID":"61d5a50a6cba9012a8ede2ee4aa1b4d81096fb7a42a40eae59f97266256fc692","Image":"nginx:alpine","Labels":"com.docker.compose.oneoff=False,com.docker.compose.project.config_files=/root/stack/compose.yaml,com.docker.compose.project.working_dir=/root/stack,com.docker.compose.project=stack,com.docker.compose.service=web,maintainer=NGINX Docker Maintainers <docker-maint@nginx.com>, and friends","Names":"stack-web-1","Ports":"0.0.0.0:8080->80/tcp, [::]:8080->80/tcp","State":"running","Status":"Up 10 seconds (healthy)"}
{"HealthStatus":"none","ID":"ded20e45de6f82b1726673c575b35f5c1db58a49be2f071fddcaa9f3a1c66b74","Image":"redis:alpine","Labels":"com.docker.compose.project.config_files=/root/stack/compose.yaml,com.docker.compose.project.working_dir=/root/stack,com.docker.compose.project=stack,com.docker.compose.service=cache","Names":"stack-cache-1","Ports":"127.0.0.1:6379->6379/tcp","State":"running","Status":"Up 10 seconds"}"#;

    // Real output, rootless Podman 5.8.7, trimmed.
    const PODMAN_PS: &str = r#"[
  {"AutoRemove": true, "Id": "db85def7d45d3256620c91554182982e94ae7a3efd9a371b88f639faacef606e", "Image": "docker.io/library/alpine:latest", "IsInfra": false,
   "Labels": {"com.docker.compose.project": "demo", "com.docker.compose.service": "web"}, "Names": ["crow-ps-probe"],
   "Ports": [{"host_ip": "127.0.0.1", "container_port": 80, "host_port": 18080, "range": 1, "protocol": "tcp"}],
   "Restarts": 2, "State": "running", "Status": "Up Less than a second (starting)"},
  {"Id": "infra0", "IsInfra": true, "Names": ["pod-infra"], "State": "running", "Status": "Up"}
]"#;

    fn scan_output(docker_user: &str) -> String {
        format!(
            "@@uid 1000\n{docker_user}@@begin docker root\n@@version 29.8.2\n@@compose v5.6.0\n@@ps\n{DOCKER_PS}\n@@restarts\n61d5a50a6cba9012a8ede2ee4aa1b4d81096fb7a42a40eae59f97266256fc692 3\n@@end\n@@begin podman user\n@@version 5.8.7\n@@ps\n{PODMAN_PS}\n@@end\n@@begin podman root\n@@version 5.8.7\n@@ps\n[]\n@@end\n@@done\n"
        )
    }

    #[test]
    fn a_scan_reads_every_runtime_and_its_containers() {
        let scan = parse_scan(&scan_output("@@begin docker user\n@@error permission denied while trying to connect to the docker API at unix:///var/run/docker.sock\n@@end\n"));
        assert!(!scan.login_is_root);
        // Docker through sudo (the user error dropped), rootless Podman; empty root Podman left out.
        assert_eq!(scan.runtimes.len(), 2, "{:?}", scan.runtimes);
        assert_eq!((scan.runtimes[0].runtime, scan.runtimes[0].scope), (Runtime::Docker, Scope::Root));
        assert_eq!(scan.runtimes[0].describe(false), "Docker 29.8.2 · through sudo · compose 5.6.0");
        assert_eq!(scan.runtimes[1].describe(false), "Podman 5.8.7 · rootless");
        assert_eq!(scan.containers.len(), 4, "the pod's infra container is left out");
        let web = scan.containers.iter().find(|c| c.name == "stack-web-1").unwrap();
        assert_eq!((web.health.as_deref(), web.restarts), (Some("healthy"), Some(3)));
        assert_eq!(web.published().len(), 1, "IPv4 and IPv6 of 8080 count once");
        assert!(web.published()[0].exposed());
        assert!(web.labels["maintainer"].ends_with("and friends"), "a comma in a label value stays in it");
        let cache = scan.containers.iter().find(|c| c.name == "stack-cache-1").unwrap();
        assert!(!cache.published()[0].exposed(), "127.0.0.1 only");
        let probe = scan.containers.iter().find(|c| c.name == "crow-ps-probe").unwrap();
        assert_eq!((probe.health.as_deref(), probe.restarts, probe.ports[0].host_port.as_str()), (Some("starting"), Some(2), "18080"));
        let stacks = scan.stacks();
        assert_eq!(stacks.len(), 2);
        let stack = stacks.iter().find(|s| s.project == "stack").unwrap();
        assert_eq!((stack.containers.len(), stack.running, stack.working_dir.as_deref()), (2, 2, Some("/root/stack")));
        assert_eq!(stack.config_files, vec!["/root/stack/compose.yaml"]);
    }

    #[test]
    fn docker_nobody_can_reach_says_how_to_fix_it() {
        let out = "@@uid 1000\n@@begin docker user\n@@error permission denied while trying to connect to the docker API at unix:///var/run/docker.sock\n@@end\n@@missing podman\n@@done\n";
        let scan = parse_scan(out);
        assert_eq!(scan.missing, vec![Runtime::Podman]);
        match &scan.runtimes[0].outcome {
            RuntimeOutcome::Problem { summary, fix } => {
                assert!(summary.contains("can't reach"));
                assert!(fix.as_deref().unwrap().contains("usermod -aG docker"));
            }
            other => panic!("{other:?}"),
        }
        let down = parse_scan("@@uid 0\n@@begin docker root\n@@error Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?\n@@end\n@@done\n");
        assert!(matches!(&down.runtimes[0].outcome, RuntimeOutcome::Problem { fix: Some(f), .. } if f.contains("systemctl enable --now docker")));
    }

    #[test]
    fn ports_and_health_read_as_the_runtime_writes_them() {
        let p = parse_docker_ports("0.0.0.0:8000-8001->8000-8001/tcp, 9000/tcp, [::]:53->53/udp");
        assert_eq!(p.len(), 2, "9000/tcp isn't published");
        assert_eq!((p[0].host_port.as_str(), p[0].container.as_str()), ("8000-8001", "8000-8001/tcp"));
        assert_eq!(p[1].host_ip, "::");
        assert_eq!(health_from_status("Up 3 minutes (unhealthy)").as_deref(), Some("unhealthy"));
        assert_eq!(health_from_status("Up 3 minutes (health: starting)").as_deref(), Some("starting"));
        assert_eq!(health_from_status("Exited (1) 7 seconds ago"), None);
    }

    #[test]
    fn commands_target_the_container_and_stack() {
        let scan = parse_scan(&scan_output(""));
        let web = scan.containers.iter().find(|c| c.name == "stack-web-1").unwrap();
        assert_eq!(action_argv(web, Action::Restart), vec!["docker", "restart", &web.id]);
        let stack = scan.stacks().into_iter().find(|s| s.project == "stack").unwrap();
        let up = stack_argv(&stack, StackAction::Up).unwrap();
        assert_eq!(&up[4..], ["/root/stack", "docker", "compose", "-p", "stack", "-f", "/root/stack/compose.yaml", "up", "-d"]);
        assert_eq!(stack_argv(&stack, StackAction::Down).unwrap(), vec!["docker", "compose", "-p", "stack", "down"]);
        let mut bare = stack.clone();
        bare.working_dir = None;
        assert!(stack_argv(&bare, StackAction::Up).is_err());
        assert!(stack_argv(&bare, StackAction::Restart).is_ok());
    }

    #[test]
    fn inspect_hides_environment_values() {
        let json = r#"[{"Id":"x","Config":{"Env":["REDIS_PASSWORD=hunter2-secret","PATH=/usr/bin","EMPTY="]}}]"#;
        let shown = mask_inspect(json);
        assert!(!shown.contains("hunter2"), "{shown}");
        assert!(shown.contains("REDIS_PASSWORD=•••• (hidden)") && shown.contains("PATH=/usr/bin"));
    }
}

/// Scans a real server and acts on it:
///   CROW_CONTAINER_HOST=podman:crow-dind-probe cargo test live_container_scan -- --ignored --nocapture
/// (a throwaway docker:dind under Podman), or with no variable, this machine.
#[cfg(test)]
#[test]
#[ignore]
fn live_container_scan() {
    let host: Box<dyn Host> = match std::env::var("CROW_CONTAINER_HOST").ok().and_then(|v| v.split_once(':').map(|(e, n)| (e.to_string(), n.to_string()))) {
        Some((engine, name)) => Box::new(crate::host::ContainerHost::new(&engine, &name)),
        None => Box::new(crate::host::LocalHost),
    };
    let scan = scan(host.as_ref()).expect("scan");
    for r in &scan.runtimes {
        eprintln!("runtime: {}", r.describe(scan.login_is_root));
    }
    for c in &scan.containers {
        eprintln!("{:24} {:20} {:9} {:?} {:?} {:?}", c.name, c.image, c.state, c.health, c.restarts, c.published());
    }
    for s in scan.stacks() {
        eprintln!("stack {} {:?} {:?} {}/{}", s.project, s.working_dir, s.config_files, s.running, s.containers.len());
    }
    // Only throwaway containers made for this test are touched.
    if let Some(c) = scan.containers.iter().find(|c| c.is_running() && (c.name.starts_with("crow-") || c.name.starts_with("lone-"))) {
        let l = logs(host.as_ref(), c).expect("logs");
        eprintln!("logs of {}: {} lines, last: {:?}", c.name, l.lines().count(), l.lines().last());
        let i = inspect(host.as_ref(), c).expect("inspect");
        eprintln!("inspect: {} bytes, env hidden: {}", i.len(), i.contains("(hidden)"));
        run(host.as_ref(), c.scope, &action_argv(c, Action::Restart)).expect("restart");
        eprintln!("restarted {}", c.name);
    }
}
