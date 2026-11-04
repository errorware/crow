//! Dokku on a server (ERR-203..ERR-210): apps, Procfile processes, backing
//! services (Postgres, Redis, Valkey, MySQL, MariaDB, Mongo, RabbitMQ),
//! staged environment variables, domains, proxy ports, Let's Encrypt TLS,
//! SSH deploy keys and server bootstrap, over SSH with no agent.
//!
//! One round trip (`SCAN_SCRIPT`) reads Dokku's version, global domain,
//! installed plugins, SSH deploy keys, Let's Encrypt cron, every backing
//! service and its linked apps, and every app's processes, scale, domains,
//! ports, TLS certificate, git revision, maintenance state and storage
//! mounts. Environment variable values are read on demand when an app's
//! Env & Secrets tab opens, and never stored on disk.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::host::{Host, HostError};

pub const SCAN_TIMEOUT: Duration = Duration::from_secs(45);
pub const ACTION_TIMEOUT: Duration = Duration::from_secs(180);

/// Official Dokku backing-service plugins Crow knows how to install, scan,
/// create, link and export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ServiceKind {
    Postgres,
    Redis,
    Valkey,
    Mysql,
    Mariadb,
    Mongo,
    Rabbitmq,
}

impl ServiceKind {
    pub const ALL: [ServiceKind; 7] = [
        Self::Postgres,
        Self::Redis,
        Self::Valkey,
        Self::Mysql,
        Self::Mariadb,
        Self::Mongo,
        Self::Rabbitmq,
    ];

    pub fn cmd(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Redis => "redis",
            Self::Valkey => "valkey",
            Self::Mysql => "mysql",
            Self::Mariadb => "mariadb",
            Self::Mongo => "mongo",
            Self::Rabbitmq => "rabbitmq",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Postgres => "PostgreSQL",
            Self::Redis => "Redis",
            Self::Valkey => "Valkey",
            Self::Mysql => "MySQL",
            Self::Mariadb => "MariaDB",
            Self::Mongo => "MongoDB",
            Self::Rabbitmq => "RabbitMQ",
        }
    }

    /// Short badge prefix ("pg:main-db", "redis:queue").
    pub fn short(self) -> &'static str {
        match self {
            Self::Postgres => "pg",
            Self::Redis => "redis",
            Self::Valkey => "valkey",
            Self::Mysql => "mysql",
            Self::Mariadb => "mariadb",
            Self::Mongo => "mongo",
            Self::Rabbitmq => "amqp",
        }
    }

    /// The default environment variable Dokku sets when linking this service.
    pub fn default_env_var(self) -> &'static str {
        match self {
            Self::Postgres => "DATABASE_URL",
            Self::Redis => "REDIS_URL",
            Self::Valkey => "VALKEY_URL",
            Self::Mysql => "DATABASE_URL",
            Self::Mariadb => "MARIADB_URL",
            Self::Mongo => "MONGO_URL",
            Self::Rabbitmq => "RABBITMQ_URL",
        }
    }

    /// Official Dokku plugin git URL.
    pub fn plugin_url(self) -> &'static str {
        match self {
            Self::Postgres => "https://github.com/dokku/dokku-postgres.git",
            Self::Redis => "https://github.com/dokku/dokku-redis.git",
            Self::Valkey => "https://github.com/dokku/dokku-valkey.git",
            Self::Mysql => "https://github.com/dokku/dokku-mysql.git",
            Self::Mariadb => "https://github.com/dokku/dokku-mariadb.git",
            Self::Mongo => "https://github.com/dokku/dokku-mongo.git",
            Self::Rabbitmq => "https://github.com/dokku/dokku-rabbitmq.git",
        }
    }

    pub fn id(self) -> &'static str {
        self.cmd()
    }

    pub fn env_var(self) -> &'static str {
        self.default_env_var()
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.trim() {
            "postgres" => Some(Self::Postgres),
            "redis" => Some(Self::Redis),
            "valkey" => Some(Self::Valkey),
            "mysql" => Some(Self::Mysql),
            "mariadb" => Some(Self::Mariadb),
            "mongo" => Some(Self::Mongo),
            "rabbitmq" => Some(Self::Rabbitmq),
            _ => None,
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        Self::from_name(s)
    }
}

/// Official optional plugins (services + letsencrypt + maintenance + clone).
pub struct CatalogPlugin {
    pub id: &'static str,
    pub label: &'static str,
    pub about: &'static str,
    pub url: &'static str,
}

pub const PLUGIN_CATALOG: &[CatalogPlugin] = &[
    CatalogPlugin {
        id: "postgres",
        label: "PostgreSQL",
        about: "Managed Postgres containers with DATABASE_URL linking and pg_dump exports",
        url: "https://github.com/dokku/dokku-postgres.git",
    },
    CatalogPlugin {
        id: "redis",
        label: "Redis",
        about: "Managed Redis caches and background job queues (Sidekiq, BullMQ)",
        url: "https://github.com/dokku/dokku-redis.git",
    },
    CatalogPlugin {
        id: "valkey",
        label: "Valkey",
        about: "Open-source Redis-compatible in-memory datastore and queue",
        url: "https://github.com/dokku/dokku-valkey.git",
    },
    CatalogPlugin {
        id: "mysql",
        label: "MySQL",
        about: "Managed MySQL database containers with DATABASE_URL linking",
        url: "https://github.com/dokku/dokku-mysql.git",
    },
    CatalogPlugin {
        id: "mariadb",
        label: "MariaDB",
        about: "Managed MariaDB database containers",
        url: "https://github.com/dokku/dokku-mariadb.git",
    },
    CatalogPlugin {
        id: "mongo",
        label: "MongoDB",
        about: "Managed MongoDB document database containers",
        url: "https://github.com/dokku/dokku-mongo.git",
    },
    CatalogPlugin {
        id: "rabbitmq",
        label: "RabbitMQ",
        about: "Managed RabbitMQ message broker containers",
        url: "https://github.com/dokku/dokku-rabbitmq.git",
    },
    CatalogPlugin {
        id: "letsencrypt",
        label: "Let's Encrypt",
        about: "Automatic ACME TLS certificates and cron renewal per app",
        url: "https://github.com/dokku/dokku-letsencrypt.git",
    },
    CatalogPlugin {
        id: "maintenance",
        label: "Maintenance mode",
        about: "Custom maintenance page while running database migrations",
        url: "https://github.com/dokku/dokku-maintenance.git",
    },
];

pub fn catalog_plugin(id: &str) -> Option<&'static CatalogPlugin> {
    PLUGIN_CATALOG.iter().find(|p| p.id == id)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstalledPlugin {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub description: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeployKey {
    pub name: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackingService {
    pub kind: ServiceKind,
    pub name: String,
    /// "running", "stopped", "missing", ...
    pub status: String,
    pub version: String,
    pub linked_apps: Vec<String>,
}

impl BackingService {
    pub fn key(&self) -> String {
        format!("{}:{}", self.kind.cmd(), self.name)
    }

    pub fn badge_label(&self) -> String {
        format!("{}:{}", self.kind.short(), self.name)
    }

    pub fn is_running(&self) -> bool {
        self.status == "running"
    }
}

/// One Procfile process type (`web`, `worker`, `release`, `clock`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcScale {
    pub proc_type: String,
    pub desired: u32,
    pub running: u32,
}

/// One proxy port mapping (`http:80:5000`, `https:443:5000`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PortMapping {
    pub scheme: String,
    pub host_port: u16,
    pub container_port: u16,
}

impl PortMapping {
    pub fn spec(&self) -> String {
        format!("{}:{}:{}", self.scheme, self.host_port, self.container_port)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DokkuApp {
    pub name: String,
    pub deployed: bool,
    pub running: bool,
    pub restore: bool,
    pub proc_total: u32,
    pub processes: Vec<ProcScale>,
    /// Status per container instance ("web.1" -> "running (CID: abc)").
    pub instances: BTreeMap<String, String>,
    pub domains_enabled: bool,
    pub domains: Vec<String>,
    pub proxy_enabled: bool,
    pub proxy_type: String,
    pub ports: Vec<PortMapping>,
    pub ssl_enabled: bool,
    pub ssl_expires_at: Option<i64>,
    pub letsencrypt_active: bool,
    pub letsencrypt_expiry_days: Option<i64>,
    pub git_sha: String,
    pub git_branch: String,
    pub last_updated: String,
    pub deploy_source_image: String,
    pub maintenance: bool,
    pub mounts: Vec<String>,
    /// Linked backing services as `("postgres", "main-db")`.
    pub linked_services: Vec<(ServiceKind, String)>,
}

impl DokkuApp {
    /// Summary of process counts: "web 2/2 · worker 1/1", or "not deployed".
    pub fn process_summary(&self) -> String {
        if !self.processes.is_empty() {
            self.processes
                .iter()
                .filter(|p| p.desired > 0 || p.running > 0)
                .map(|p| format!("{} {}/{}", p.proc_type, p.running, p.desired))
                .collect::<Vec<_>>()
                .join(" · ")
        } else if self.deployed {
            format!("{} running", self.proc_total)
        } else {
            String::new()
        }
    }

    /// True if the app is deployed and any desired process is not running.
    pub fn degraded(&self) -> bool {
        if !self.deployed {
            return false;
        }
        if self.processes.iter().any(|p| p.running < p.desired) {
            return true;
        }
        self.instances.values().any(|s| s.contains("exited") || s.contains("restarting") || s.contains("dead"))
    }

    pub fn primary_domain(&self) -> Option<&str> {
        self.domains.first().map(String::as_str)
    }

    pub fn total_desired(&self) -> u32 {
        self.processes.iter().map(|p| p.desired).sum()
    }
}

pub type DokkuScan = Scan;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scan {
    pub installed: bool,
    pub version: String,
    pub os_id: String,
    pub mem_mb: u64,
    pub docker_ok: bool,
    pub global_domains: Vec<String>,
    pub letsencrypt_email: String,
    pub letsencrypt_cron: bool,
    pub plugins: Vec<InstalledPlugin>,
    pub deploy_keys: Vec<DeployKey>,
    pub services: Vec<BackingService>,
    pub apps: Vec<DokkuApp>,
}

impl Scan {
    pub fn find_app(&self, name: &str) -> Option<&DokkuApp> {
        self.apps.iter().find(|a| a.name == name)
    }

    pub fn find_service(&self, kind: ServiceKind, name: &str) -> Option<&BackingService> {
        self.services.iter().find(|s| s.kind == kind && s.name == name)
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.iter().any(|p| p.name == name && p.enabled)
    }

    pub fn can_bootstrap(&self) -> bool {
        matches!(self.os_id.as_str(), "ubuntu" | "debian")
    }
}

/// One SSH round trip reads Dokku's version, global settings, plugins,
/// deploy keys, backing services and every app's state.
pub const SCAN_SCRIPT: &str = r#"
export PATH="$PATH:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
os_id=""
[ -f /etc/os-release ] && os_id=$(. /etc/os-release 2>/dev/null && printf '%s' "$ID")
mem_mb=$(awk '/MemTotal:/ { printf "%d", $2 / 1024 }' /proc/meminfo 2>/dev/null)
docker_ok=0
command -v docker >/dev/null 2>&1 && docker version --format '{{.Server.Version}}' >/dev/null 2>&1 && docker_ok=1
echo "@@host os=$os_id mem_mb=${mem_mb:-0} docker=$docker_ok"

if ! command -v dokku >/dev/null 2>&1; then
  echo "@@missing"
  exit 0
fi

uid=$(id -u)
dk() {
  if [ "$uid" = 0 ]; then
    dokku --quiet "$@"
  elif sudo -n true 2>/dev/null; then
    sudo -n -- dokku --quiet "$@"
  else
    dokku --quiet "$@"
  fi
}

ver=$(dk version 2>/dev/null | head -1)
echo "@@version $ver"

echo "@@global_domains"
dk domains:report --global 2>/dev/null

echo "@@global_letsencrypt"
# letsencrypt:set --global keeps it in the plugin's own report; older
# setups had it as a global env var.
le_email=$(dk letsencrypt:report --global --letsencrypt-global-email 2>/dev/null | head -1)
[ -n "$le_email" ] || le_email=$(dk config:get --global DOKKU_LETSENCRYPT_EMAIL 2>/dev/null | head -1)
echo "email: $le_email"
if crontab -l -u dokku 2>/dev/null | grep -q 'letsencrypt' || [ -f /etc/cron.d/dokku-letsencrypt ]; then
  echo "cron: true"
fi

echo "@@plugins"
dk plugin:list 2>/dev/null

echo "@@keys"
dk ssh-keys:list 2>/dev/null

echo "@@letsencrypt_list"
dk letsencrypt:list 2>/dev/null

for svc in postgres redis valkey mysql mariadb mongo rabbitmq; do
  if [ -d "/var/lib/dokku/plugins/enabled/$svc" ]; then
    for name in $(dk "$svc:list" 2>/dev/null | grep -v '^====='); do
      [ -z "$name" ] && continue
      echo "@@service $svc $name"
      dk "$svc:info" "$name" 2>/dev/null
      echo "@@service_links"
      dk "$svc:links" "$name" 2>/dev/null
    done
  fi
done

for app in $(dk apps:list 2>/dev/null | grep -v '^====='); do
  [ -z "$app" ] && continue
  echo "@@app $app"
  echo "@@sub ps"
  dk ps:report "$app" 2>/dev/null
  echo "@@sub scale"
  dk ps:scale "$app" 2>/dev/null
  echo "@@sub domains"
  dk domains:report "$app" 2>/dev/null
  echo "@@sub ports"
  dk ports:report "$app" 2>/dev/null
  echo "@@sub proxy"
  dk proxy:report "$app" 2>/dev/null
  echo "@@sub certs"
  dk certs:report "$app" 2>/dev/null
  echo "@@sub git"
  dk git:report "$app" 2>/dev/null
  if [ -d /var/lib/dokku/plugins/enabled/maintenance ]; then
    echo "@@sub maintenance"
    dk maintenance:report "$app" 2>/dev/null
  fi
  echo "@@sub storage"
  dk storage:list "$app" 2>/dev/null
done
echo "@@done"
"#;

fn report_kv(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.starts_with("=====>") || trimmed.starts_with("----->") {
        return None;
    }
    let (k, v) = trimmed.split_once(':')?;
    Some((k.trim(), v.trim()))
}

fn parse_bool(v: &str) -> bool {
    matches!(v.trim().to_lowercase().as_str(), "true" | "enabled" | "yes" | "on")
}

/// Parses `dokku plugin:list` lines:
/// `  00_dokku-standard    0.35.15 enabled    dokku core standard plugin`
pub fn parse_plugin_line(line: &str) -> Option<InstalledPlugin> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("=====") || trimmed.starts_with("-----") {
        return None;
    }
    let mut parts = trimmed.split_whitespace();
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    let status = parts.next()?;
    let enabled = status == "enabled";
    let description = parts.collect::<Vec<_>>().join(" ");
    Some(InstalledPlugin { name, version, enabled, description })
}

/// Parses `dokku ssh-keys:list` lines:
/// `SHA256:abc... NAME="admin" ...` or `SHA256:abc... admin`
pub fn parse_deploy_key_line(line: &str) -> Option<DeployKey> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("=====") {
        return None;
    }
    let mut parts = trimmed.split_whitespace();
    let fp = parts.next()?.to_string();
    if !fp.starts_with("SHA256:") && !fp.starts_with("MD5:") && !fp.contains(':') {
        return None;
    }
    let rest = parts.collect::<Vec<_>>().join(" ");
    let name = if let Some(after) = rest.split("NAME=\"").nth(1) {
        after.split('"').next().unwrap_or("").to_string()
    } else {
        rest.split_whitespace().next().unwrap_or("default").trim_matches('"').to_string()
    };
    if name.is_empty() {
        return None;
    }
    Some(DeployKey { name, fingerprint: fp })
}

/// Parses `dokku ps:scale <app>` table lines:
/// `proctype: qty` or `web:  2`
pub fn parse_scale_line(line: &str) -> Option<(String, u32)> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("-----")
        || trimmed.starts_with("=====")
        || trimmed.starts_with("proctype")
    {
        return None;
    }
    let (proc_type, qty) = trimmed.split_once(':')?;
    let proc_type = proc_type.trim();
    if proc_type.is_empty() || proc_type.contains(' ') {
        return None;
    }
    let n: u32 = qty.trim().parse().ok()?;
    Some((proc_type.to_string(), n))
}

/// Parses `"http:80:5000 https:443:5000"` into `PortMapping`s.
pub fn parse_port_mappings(s: &str) -> Vec<PortMapping> {
    s.split_whitespace()
        .filter_map(|word| {
            let mut parts = word.split(':');
            let scheme = parts.next()?.trim().to_string();
            let host_port: u16 = parts.next()?.trim().parse().ok()?;
            let container_port: u16 = parts.next()?.trim().parse().ok()?;
            Some(PortMapping { scheme, host_port, container_port })
        })
        .collect()
}

/// Parses `SCAN_SCRIPT`'s output into a structured [`Scan`].
pub fn parse_scan(out: &str) -> Scan {
    let mut scan = Scan { installed: true, ..Default::default() };
    let mut section = "";
    let mut app_sub = "";
    let mut current_app: Option<DokkuApp> = None;
    let mut current_svc: Option<BackingService> = None;
    let mut svc_links_mode = false;
    let mut le_apps: BTreeMap<String, Option<i64>> = BTreeMap::new();

    let flush_app = |current_app: &mut Option<DokkuApp>, scan: &mut Scan| {
        if let Some(mut app) = current_app.take() {
            // Reconcile running counts per proc_type from instances (`Status web 1: running...`).
            for p in &mut app.processes {
                let prefix = format!("{}.", p.proc_type);
                let count = app
                    .instances
                    .iter()
                    .filter(|(k, v)| k.starts_with(&prefix) && v.starts_with("running"))
                    .count() as u32;
                p.running = count;
            }
            // If `ps:scale` had no rows yet but instances exist, synthesize processes from instances.
            if app.processes.is_empty() && !app.instances.is_empty() {
                let mut by_proc: BTreeMap<String, (u32, u32)> = BTreeMap::new();
                for (inst, status) in &app.instances {
                    if let Some((proc_type, _)) = inst.split_once('.') {
                        let entry = by_proc.entry(proc_type.to_string()).or_default();
                        entry.0 += 1;
                        if status.starts_with("running") {
                            entry.1 += 1;
                        }
                    }
                }
                for (proc_type, (desired, running)) in by_proc {
                    app.processes.push(ProcScale { proc_type, desired, running });
                }
            }
            scan.apps.push(app);
        }
    };

    let flush_svc = |current_svc: &mut Option<BackingService>, scan: &mut Scan| {
        if let Some(svc) = current_svc.take() {
            scan.services.push(svc);
        }
    };

    for raw_line in out.lines() {
        let line = raw_line.trim_end();
        if let Some(rest) = line.strip_prefix("@@host ") {
            for kv in rest.split_whitespace() {
                if let Some((k, v)) = kv.split_once('=') {
                    match k {
                        "os" => scan.os_id = v.to_string(),
                        "mem_mb" => scan.mem_mb = v.parse().unwrap_or(0),
                        "docker" => scan.docker_ok = v == "1",
                        _ => {}
                    }
                }
            }
            continue;
        }
        if line == "@@missing" {
            scan.installed = false;
            return scan;
        }
        if let Some(v) = line.strip_prefix("@@version ") {
            scan.version = v.trim().trim_start_matches("dokku version ").trim().to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix("@@service ") {
            flush_svc(&mut current_svc, &mut scan);
            svc_links_mode = false;
            let mut parts = rest.split_whitespace();
            if let (Some(kind_str), Some(name)) = (parts.next(), parts.next()) {
                if let Some(kind) = ServiceKind::from_name(kind_str) {
                    current_svc = Some(BackingService {
                        kind,
                        name: name.to_string(),
                        status: "unknown".into(),
                        version: String::new(),
                        linked_apps: Vec::new(),
                    });
                }
            }
            section = "service";
            continue;
        }
        if line == "@@service_links" {
            svc_links_mode = true;
            continue;
        }
        if let Some(app_name) = line.strip_prefix("@@app ") {
            flush_svc(&mut current_svc, &mut scan);
            flush_app(&mut current_app, &mut scan);
            current_app = Some(DokkuApp {
                name: app_name.trim().to_string(),
                domains_enabled: true,
                proxy_enabled: true,
                proxy_type: "nginx".into(),
                ..Default::default()
            });
            section = "app";
            app_sub = "";
            continue;
        }
        if let Some(sub) = line.strip_prefix("@@sub ") {
            app_sub = match sub.trim() {
                "ps" => "ps",
                "scale" => "scale",
                "domains" => "domains",
                "ports" => "ports",
                "proxy" => "proxy",
                "certs" => "certs",
                "git" => "git",
                "maintenance" => "maintenance",
                "storage" => "storage",
                _ => "",
            };
            continue;
        }
        if let Some(sec) = line.strip_prefix("@@") {
            flush_svc(&mut current_svc, &mut scan);
            flush_app(&mut current_app, &mut scan);
            section = match sec.trim() {
                "global_domains" => "global_domains",
                "global_letsencrypt" => "global_letsencrypt",
                "plugins" => "plugins",
                "keys" => "keys",
                "letsencrypt_list" => "letsencrypt_list",
                _ => "",
            };
            continue;
        }

        match section {
            "global_domains" => {
                if let Some((k, v)) = report_kv(line) {
                    if k.eq_ignore_ascii_case("Domains global vhosts") && !v.is_empty() {
                        scan.global_domains = v.split_whitespace().map(String::from).collect();
                    }
                }
            }
            "global_letsencrypt" => {
                if let Some((k, v)) = report_kv(line) {
                    match k {
                        "email" => scan.letsencrypt_email = v.to_string(),
                        "cron" => scan.letsencrypt_cron = parse_bool(v),
                        _ => {}
                    }
                }
            }
            "plugins" => {
                if let Some(p) = parse_plugin_line(line) {
                    scan.plugins.push(p);
                }
            }
            "keys" => {
                if let Some(k) = parse_deploy_key_line(line) {
                    scan.deploy_keys.push(k);
                }
            }
            "letsencrypt_list" => {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("----") && !trimmed.starts_with("App name") {
                    let mut parts = trimmed.split_whitespace();
                    if let Some(app_name) = parts.next() {
                        le_apps.insert(app_name.to_string(), expiry_days(&parts.collect::<Vec<_>>()));
                    }
                }
            }
            "service" => {
                let Some(svc) = current_svc.as_mut() else { continue };
                if svc_links_mode {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("=====") && !trimmed.starts_with("-----") {
                        for app in trimmed.split_whitespace() {
                            if !svc.linked_apps.iter().any(|a| a == app) {
                                svc.linked_apps.push(app.to_string());
                            }
                        }
                    }
                } else if let Some((k, v)) = report_kv(line) {
                    match k.to_lowercase().as_str() {
                        "status" => svc.status = v.to_lowercase(),
                        "version" => svc.version = v.to_string(),
                        "links" if !v.is_empty() && v != "-" => {
                            for app in v.split_whitespace() {
                                if !svc.linked_apps.iter().any(|a| a == app) {
                                    svc.linked_apps.push(app.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            "app" => {
                let Some(app) = current_app.as_mut() else { continue };
                match app_sub {
                    "ps" => {
                        if let Some((k, v)) = report_kv(line) {
                            let kl = k.to_lowercase();
                            if kl == "deployed" {
                                app.deployed = parse_bool(v);
                            } else if kl == "running" {
                                app.running = parse_bool(v);
                            } else if kl == "restore" {
                                app.restore = parse_bool(v);
                            } else if kl == "processes" {
                                app.proc_total = v.parse().unwrap_or(0);
                            } else if kl == "deploy source" && v.starts_with("docker-image:") {
                                app.deploy_source_image = v.trim_start_matches("docker-image:").trim().to_string();
                            } else if let Some(inst) = kl.strip_prefix("status ") {
                                let inst_key = inst.trim().replace(' ', ".");
                                app.instances.insert(inst_key, v.to_string());
                            }
                        }
                    }
                    "scale" => {
                        if let Some((proc_type, desired)) = parse_scale_line(line) {
                            app.processes.push(ProcScale { proc_type, desired, running: 0 });
                        }
                    }
                    "domains" => {
                        if let Some((k, v)) = report_kv(line) {
                            let kl = k.to_lowercase();
                            if kl == "domains app enabled" {
                                app.domains_enabled = parse_bool(v);
                            } else if kl == "domains app vhosts" && !v.is_empty() {
                                app.domains = v.split_whitespace().map(String::from).collect();
                            }
                        }
                    }
                    "ports" => {
                        if let Some((k, v)) = report_kv(line) {
                            let kl = k.to_lowercase();
                            if (kl == "port map" || kl == "ports map") && !v.is_empty() {
                                app.ports = parse_port_mappings(v);
                            } else if kl == "ports map detected" && app.ports.is_empty() {
                                // 0.35: an app nobody set ports for lists
                                // what Dokku picked (from EXPOSE) here only.
                                app.ports = parse_port_mappings(v);
                            }
                        }
                    }
                    "proxy" => {
                        if let Some((k, v)) = report_kv(line) {
                            let kl = k.to_lowercase();
                            if kl == "proxy enabled" {
                                app.proxy_enabled = parse_bool(v);
                            } else if kl == "proxy type" && !v.is_empty() {
                                app.proxy_type = v.to_string();
                            }
                        }
                    }
                    "certs" => {
                        if let Some((k, v)) = report_kv(line) {
                            let kl = k.to_lowercase();
                            if kl == "ssl enabled" {
                                app.ssl_enabled = parse_bool(v);
                            } else if kl == "ssl expires at" && !v.is_empty() {
                                app.ssl_expires_at = chrono::DateTime::parse_from_rfc3339(v)
                                    .ok()
                                    .map(|dt| dt.timestamp());
                            }
                        }
                    }
                    "git" => {
                        if let Some((k, v)) = report_kv(line) {
                            let kl = k.to_lowercase();
                            if kl == "git sha" && !v.is_empty() {
                                app.git_sha = v.chars().take(10).collect();
                            } else if kl == "git deploy branch" && !v.is_empty() {
                                app.git_branch = v.to_string();
                            } else if kl == "git last updated at" && !v.is_empty() {
                                app.last_updated = v.to_string();
                            }
                        }
                    }
                    "maintenance" => {
                        if let Some((k, v)) = report_kv(line) {
                            if k.to_lowercase().contains("maintenance enabled") {
                                app.maintenance = parse_bool(v);
                            }
                        }
                    }
                    "storage" => {
                        let trimmed = line.trim();
                        if trimmed.contains(':') && !trimmed.starts_with("=====") && !trimmed.starts_with("-----") && !trimmed.ends_with("bind-mounts:") {
                            app.mounts.push(trimmed.to_string());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    flush_svc(&mut current_svc, &mut scan);
    flush_app(&mut current_app, &mut scan);

    // Populate linked_services and Let's Encrypt status on each app.
    for svc in &scan.services {
        for linked in &svc.linked_apps {
            if let Some(app) = scan.apps.iter_mut().find(|a| &a.name == linked) {
                if !app.linked_services.iter().any(|(k, n)| *k == svc.kind && n == &svc.name) {
                    app.linked_services.push((svc.kind, svc.name.clone()));
                }
            }
        }
    }
    for (app_name, days) in le_apps {
        if let Some(app) = scan.apps.iter_mut().find(|a| a.name == app_name) {
            app.letsencrypt_active = true;
            app.letsencrypt_expiry_days = days;
        }
    }

    scan.apps.sort_by(|a, b| a.name.cmp(&b.name));
    scan.services.sort_by(|a, b| (a.kind, &a.name).cmp(&(b.kind, &b.name)));
    scan
}

/// Days to expiry from a `letsencrypt:list` row after the app name:
/// `2027-02-02 21:29:22  89d, 23h, 1m, 6s  59d, ...` (the first `Nd` is
/// the time before expiry), or `... 89 days ...`.
fn expiry_days(cols: &[&str]) -> Option<i64> {
    cols.iter().enumerate().find_map(|(i, c)| {
        let c = c.trim_end_matches(',');
        match c.strip_suffix('d') {
            Some(n) => n.parse().ok(),
            None if c == "days" => cols.get(i.wrapping_sub(1)).and_then(|n| n.parse().ok()),
            None => None,
        }
    })
}

/// Runs `SCAN_SCRIPT` on `host`.
pub fn scan(host: &dyn Host) -> Result<Scan, String> {
    let argv = ["sh", "-c", SCAN_SCRIPT];
    let out = host
        .exec_privileged(&argv, &[], SCAN_TIMEOUT)
        .or_else(|_| host.exec(&argv, SCAN_TIMEOUT))
        .map_err(|e| e.to_string())?;
    Ok(parse_scan(&out.stdout))
}

// ==========================================
// Validation & Command Builders
// ==========================================

/// Validates a Dokku app or service name: lowercase ASCII letters, digits,
/// and dashes, starting with a letter or digit, 1..=40 chars.
pub fn validate_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Name cannot be empty.".into());
    }
    if trimmed.len() > 40 {
        return Err("Name must be 40 characters or fewer.".into());
    }
    let mut chars = trimmed.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err("Name must start with a lowercase letter or digit.".into());
    }
    if !trimmed.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err("Use lowercase letters, digits and hyphens only.".into());
    }
    Ok(trimmed.to_string())
}

/// Validates a domain name (`app.example.com`, `*.example.com`).
pub fn validate_domain(domain: &str) -> Result<String, String> {
    let d = domain.trim().to_lowercase();
    if d.is_empty() || d.len() > 253 {
        return Err("Enter a valid domain name.".into());
    }
    let rest = d.strip_prefix("*.").unwrap_or(&d);
    if !rest.contains('.')
        || rest.starts_with('.')
        || rest.ends_with('.')
        || !rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    {
        return Err("Enter a valid domain name (e.g. app.example.com).".into());
    }
    Ok(d)
}

/// Validates a proxy port spec (`http:80:5000`, `https:443:3000`).
pub fn validate_port_spec(spec: &str) -> Result<PortMapping, String> {
    let mut parts = spec.trim().split(':');
    let scheme = parts.next().unwrap_or("").trim().to_lowercase();
    if !matches!(scheme.as_str(), "http" | "https" | "tcp" | "udp" | "grpc" | "grpcs") {
        return Err("Scheme must be http, https, tcp, udp, grpc or grpcs (e.g. http:80:5000).".into());
    }
    let host_port: u16 = parts
        .next()
        .and_then(|p| p.trim().parse().ok())
        .filter(|&p| p > 0)
        .ok_or_else(|| "Invalid host port (e.g. http:80:5000).".to_string())?;
    let container_port: u16 = parts
        .next()
        .and_then(|p| p.trim().parse().ok())
        .filter(|&p| p > 0)
        .ok_or_else(|| "Invalid container port (e.g. http:80:5000).".to_string())?;
    if parts.next().is_some() {
        return Err("Port mapping takes scheme:host_port:container_port (e.g. http:80:5000).".into());
    }
    Ok(PortMapping { scheme, host_port, container_port })
}

/// Validates a host storage bind mount (`/var/lib/dokku/data/storage/app:/app/storage`).
pub fn validate_mount_spec(spec: &str) -> Result<(String, String), String> {
    let (host_dir, container_dir) = spec
        .trim()
        .split_once(':')
        .ok_or_else(|| "Format as /host/path:/container/path.".to_string())?;
    let (h, c) = (host_dir.trim(), container_dir.trim());
    if !h.starts_with('/') || !c.starts_with('/') || h.contains(' ') || c.contains(' ') || h == "/" || c == "/" {
        return Err("Both paths must be absolute (not /) without spaces.".into());
    }
    Ok((h.to_string(), c.to_string()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    Start,
    Stop,
    Restart,
    Rebuild,
    MaintenanceOn,
    MaintenanceOff,
    LetsEncryptEnable,
    LetsEncryptRenew,
}

impl AppAction {
    pub fn argv(self, app: &str) -> Vec<String> {
        match self {
            Self::Start => vec!["dokku".into(), "ps:start".into(), app.into()],
            Self::Stop => vec!["dokku".into(), "ps:stop".into(), app.into()],
            Self::Restart => vec!["dokku".into(), "ps:restart".into(), app.into()],
            Self::Rebuild => vec!["dokku".into(), "ps:rebuild".into(), app.into()],
            Self::MaintenanceOn => vec!["dokku".into(), "maintenance:enable".into(), app.into()],
            Self::MaintenanceOff => vec!["dokku".into(), "maintenance:disable".into(), app.into()],
            Self::LetsEncryptEnable => vec!["dokku".into(), "letsencrypt:enable".into(), app.into()],
            Self::LetsEncryptRenew => vec!["dokku".into(), "letsencrypt:auto-renew".into(), app.into()],
        }
    }

    pub fn kind(self) -> &'static str {
        match self {
            Self::Start => "dokku.ps.start",
            Self::Stop => "dokku.ps.stop",
            Self::Restart => "dokku.ps.restart",
            Self::Rebuild => "dokku.ps.rebuild",
            Self::MaintenanceOn => "dokku.maintenance.enable",
            Self::MaintenanceOff => "dokku.maintenance.disable",
            Self::LetsEncryptEnable => "dokku.letsencrypt.enable",
            Self::LetsEncryptRenew => "dokku.letsencrypt.renew",
        }
    }
}

pub fn scale_argv(app: &str, changes: &BTreeMap<String, u32>) -> Vec<String> {
    let mut argv = vec!["dokku".to_string(), "ps:scale".to_string(), app.to_string()];
    for (proc_type, qty) in changes {
        argv.push(format!("{proc_type}={qty}"));
    }
    argv
}

pub fn link_service_argv(kind: ServiceKind, service: &str, app: &str, no_restart: bool) -> Vec<String> {
    let mut argv = vec!["dokku".to_string(), format!("{}:link", kind.cmd()), service.to_string(), app.to_string()];
    if no_restart {
        argv.push("--no-restart".into());
    }
    argv
}

pub fn unlink_service_argv(kind: ServiceKind, service: &str, app: &str, no_restart: bool) -> Vec<String> {
    let mut argv = vec!["dokku".to_string(), format!("{}:unlink", kind.cmd()), service.to_string(), app.to_string()];
    if no_restart {
        argv.push("--no-restart".into());
    }
    argv
}

// ==========================================
// Logs & Environment Config (`dokku config`)
// ==========================================

/// Reads the last 200 log lines of `app` (optionally filtered to `proc_type`),
/// newest first (matching `containers::logs`).
pub fn app_logs(host: &dyn Host, app: &str, proc_type: Option<&str>) -> Result<String, String> {
    let mut argv = vec!["dokku", "logs", app, "--num", "200"];
    if let Some(p) = proc_type.filter(|p| !p.is_empty()) {
        argv.push("--ps");
        argv.push(p);
    }
    let out = host
        .exec_privileged(&argv, &[], Duration::from_secs(15))
        .or_else(|_| host.exec(&argv, Duration::from_secs(15)))
        .map_err(|e| match e {
            HostError::Failed { stderr, .. } if !stderr.trim().is_empty() => stderr.trim().to_string(),
            other => other.to_string(),
        })?;
    let text = if out.stdout.trim().is_empty() { &out.stderr } else { &out.stdout };
    Ok(text.lines().rev().collect::<Vec<_>>().join("\n"))
}

/// Reads the last 150 log lines of a backing service (`dokku <kind>:logs <name>`).
pub fn service_logs(host: &dyn Host, kind: ServiceKind, name: &str) -> Result<String, String> {
    let cmd = format!("{}:logs", kind.cmd());
    let argv = ["dokku", &cmd, name, "--tail", "150"];
    let out = host
        .exec_privileged(&argv, &[], Duration::from_secs(15))
        .or_else(|_| host.exec(&argv, Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    let text = if out.stdout.trim().is_empty() { &out.stderr } else { &out.stdout };
    Ok(text.lines().rev().collect::<Vec<_>>().join("\n"))
}

/// Keys whose values are non-secret operational settings and can be shown
/// unmasked by default. Everything else starts masked (`••••••••`).
pub fn is_public_env_key(key: &str) -> bool {
    let u = key.to_uppercase();
    if u.contains("SECRET")
        || u.contains("KEY")
        || u.contains("TOKEN")
        || u.contains("PASSWORD")
        || u.contains("PASS")
        || u.contains("URL")
        || u.contains("PRIVATE")
        || u.contains("AUTH")
        || u.contains("CREDENTIAL")
    {
        return false;
    }
    matches!(
        u.as_str(),
        "RAILS_ENV"
            | "RACK_ENV"
            | "NODE_ENV"
            | "PORT"
            | "HOST"
            | "BIND"
            | "TZ"
            | "LANG"
            | "LC_ALL"
            | "RAILS_LOG_TO_STDOUT"
            | "RAILS_SERVE_STATIC_FILES"
            | "RAILS_MAX_THREADS"
            | "WEB_CONCURRENCY"
            | "MALLOC_ARENA_MAX"
            | "RUST_LOG"
            | "GO_ENV"
    ) || u.starts_with("DOKKU_")
        || u.starts_with("GIT_REV")
}

/// True if `key` looks like one managed by a linked Dokku backing service.
pub fn is_service_managed_key(key: &str) -> bool {
    matches!(
        key,
        "DATABASE_URL"
            | "REDIS_URL"
            | "VALKEY_URL"
            | "MYSQL_URL"
            | "MARIADB_URL"
            | "MONGO_URL"
            | "RABBITMQ_URL"
            | "AMQP_URL"
    ) || key.ends_with("_DATABASE_URL")
        || key.ends_with("_REDIS_URL")
}

/// Validates an environment variable key (`A-Z`, `0-9`, `_`, starting with a letter or `_`).
pub fn validate_env_key(key: &str) -> Result<String, String> {
    let k = key.trim();
    if k.is_empty() || k.len() > 128 {
        return Err("Environment variable key cannot be empty.".into());
    }
    let first = k.chars().next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Use letters, digits and underscores (e.g. RAILS_MASTER_KEY).".into());
    }
    Ok(k.to_string())
}

/// Parses `dokku config:export --format json <app>`.
pub fn parse_config_json(json: &str) -> Result<BTreeMap<String, String>, String> {
    let trimmed = json.trim();
    if trimmed.is_empty() {
        return Ok(BTreeMap::new());
    }
    let val: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| format!("invalid config JSON: {e}"))?;
    let obj = val.as_object().ok_or_else(|| "config export was not a JSON object".to_string())?;
    let mut out = BTreeMap::new();
    for (k, v) in obj {
        let s = match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        out.insert(k.clone(), s);
    }
    Ok(out)
}

/// Reads an app's environment variables on demand.
pub fn read_app_config(host: &dyn Host, app: &str) -> Result<BTreeMap<String, String>, String> {
    let argv = ["dokku", "config:export", "--format", "json", app];
    let out = host
        .exec_privileged(&argv, &[], Duration::from_secs(15))
        .or_else(|_| host.exec(&argv, Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    parse_config_json(&out.stdout)
}

/// Parses a pasted `.env` block (`KEY=VALUE`, ignoring `#` comments and optional `export `).
pub fn parse_dotenv(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map(str::trim).unwrap_or(line);
        let Some((k, v)) = line.split_once('=') else {
            return Err(format!("Line {}: expected KEY=VALUE", idx + 1));
        };
        let key = validate_env_key(k).map_err(|e| format!("Line {}: {e}", idx + 1))?;
        let val = v.trim();
        let unquoted = if (val.starts_with('"') && val.ends_with('"') && val.len() >= 2)
            || (val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2)
        {
            &val[1..val.len() - 1]
        } else {
            val
        };
        out.insert(key, unquoted.to_string());
    }
    Ok(out)
}

/// Standard base64 encoder (RFC 4648) for `dokku config:set --encoded`, so
/// values with spaces, quotes, newlines or `$` are passed without shell
/// escaping issues.
pub fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Builds the `dokku config:set --encoded` and `dokku config:unset` commands
/// for a batch of staged environment changes, plus a redacted audit summary
/// that never records secret values.
pub fn build_config_commands(
    app: &str,
    set: &BTreeMap<String, String>,
    unset: &[String],
    restart: bool,
) -> (Vec<Vec<String>>, String) {
    let mut cmds = Vec::new();
    let mut summary_parts = Vec::new();

    if !set.is_empty() {
        let mut argv = vec!["dokku".to_string(), "config:set".to_string(), "--encoded".to_string()];
        // If we also have unsets, hold the restart until the unset command.
        if !restart || !unset.is_empty() {
            argv.push("--no-restart".into());
        }
        argv.push(app.to_string());
        for (k, v) in set {
            argv.push(format!("{k}={}", base64_encode(v.as_bytes())));
        }
        cmds.push(argv);
        let keys: Vec<&str> = set.keys().map(String::as_str).collect();
        summary_parts.push(format!("dokku config:set {app} {} (•••)", keys.join(" ")));
    }

    if !unset.is_empty() {
        let mut argv = vec!["dokku".to_string(), "config:unset".to_string()];
        if !restart {
            argv.push("--no-restart".into());
        }
        argv.push(app.to_string());
        for k in unset {
            argv.push(k.clone());
        }
        cmds.push(argv);
        summary_parts.push(format!("dokku config:unset {app} {}", unset.join(" ")));
    }

    (cmds, summary_parts.join(" && "))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SCAN: &str = r#"@@host os=ubuntu mem_mb=4096 docker=1
@@version dokku version 0.35.15
@@global_domains
=====> Global domains information
       Domains global enabled: true
       Domains global vhosts:  paas.example.com
@@global_letsencrypt
email: ops@example.com
cron: true
@@plugins
  00_dokku-standard    0.35.15 enabled    dokku core standard plugin
  20_events            0.35.15 enabled    dokku core events logging plugin
  letsencrypt          0.19.2  enabled    dokku letsencrypt plugin
  maintenance          0.10.0  enabled    dokku maintenance plugin
  postgres             1.41.0  enabled    dokku postgres service plugin
  redis                1.35.0  enabled    dokku redis service plugin
@@keys
SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8 NAME="deploy-laptop" SSHCOMMAND_ALLOWED_KEYS="no-agent-forwarding"
@@letsencrypt_list
-----> App name           Certificate Expiry        Time before expiry        Time before renewal
rails-shop                2026-12-25 12:00:00       51d 14h                   21d 14h
@@service postgres shop-db
=====> shop-db postgres service information
       Status:              running
       Version:             postgres:16.4
       Links:               rails-shop
@@service_links
rails-shop
@@service redis shop-queue
=====> shop-queue redis service information
       Status:              running
       Version:             redis:7.4.1
       Links:               rails-shop
@@service_links
rails-shop
@@app rails-shop
@@sub ps
=====> rails-shop ps information
       Deployed:            true
       Processes:           3
       Ps can scale:        true
       Restore:             true
       Running:             true
       Status web 1:        running (CID: a1b2c3d4e5f)
       Status web 2:        running (CID: f5e4d3c2b1a)
       Status worker 1:     running (CID: 99887766554)
@@sub scale
-----> Scaling for rails-shop
proctype: qty
--------: ---
web:      2
worker:   1
release:  0
@@sub domains
=====> rails-shop domains information
       Domains app enabled: true
       Domains app vhosts:  shop.example.com www.shop.example.com
@@sub ports
=====> rails-shop ports information
       Port map:            http:80:3000 https:443:3000
@@sub proxy
=====> rails-shop proxy information
       Proxy enabled:       true
       Proxy type:          nginx
@@sub certs
=====> rails-shop ssl information
       Ssl enabled:         true
       Ssl expires at:      2026-12-25T12:00:00Z
@@sub git
=====> rails-shop git information
       Git deploy branch:   main
       Git last updated at: 2026-11-04T19:00:00Z
       Git sha:             7f3a91c82b1044e
@@sub maintenance
=====> rails-shop maintenance information
       Maintenance enabled: false
@@sub storage
/var/lib/dokku/data/storage/rails-shop:/rails/storage
@@done
"#;

    #[test]
    fn letsencrypt_expiry_reads_as_the_plugin_prints_it() {
        // dokku-letsencrypt 0.19 on Dokku 0.35.15.
        let row = "2027-02-02 21:29:22       89d, 23h, 1m, 6s          59d, 23h, 1m, 6s";
        assert_eq!(expiry_days(&row.split_whitespace().collect::<Vec<_>>()), Some(89));
        assert_eq!(expiry_days(&["2026-12-25", "12:00:00", "51d", "14h"]), Some(51));
        assert_eq!(expiry_days(&["2026-12-25", "12:00:00", "12", "days"]), Some(12));
        assert_eq!(expiry_days(&["expired"]), None);
    }

    #[test]
    fn ports_dokku_detected_count_when_none_were_set() {
        // As Dokku 0.35.15 prints it for an app deployed from a Dockerfile.
        let out = "@@app quorum\n@@sub ports\n=====> quorum ports information\n       Ports map:                     \n       Ports map detected:            http:80:80\n@@done\n";
        let ports = &parse_scan(out).apps[0].ports;
        assert_eq!(ports.iter().map(|p| p.spec()).collect::<Vec<_>>(), vec!["http:80:80"]);
        let set = "@@app quorum\n@@sub ports\n       Ports map:                     http:80:5000\n       Ports map detected:            http:80:80\n@@done\n";
        assert_eq!(parse_scan(set).apps[0].ports.iter().map(|p| p.spec()).collect::<Vec<_>>(), vec!["http:80:5000"], "what was set wins");
    }

    #[test]
    fn parses_full_rails_scan() {
        let scan = parse_scan(SAMPLE_SCAN);
        assert!(scan.installed);
        assert_eq!(scan.os_id, "ubuntu");
        assert_eq!(scan.mem_mb, 4096);
        assert!(scan.docker_ok);
        assert_eq!(scan.version, "0.35.15");
        assert_eq!(scan.global_domains, vec!["paas.example.com"]);
        assert_eq!(scan.letsencrypt_email, "ops@example.com");
        assert!(scan.letsencrypt_cron);
        assert!(scan.has_plugin("postgres") && scan.has_plugin("redis") && scan.has_plugin("letsencrypt"));
        assert_eq!(scan.deploy_keys.len(), 1);
        assert_eq!(scan.deploy_keys[0].name, "deploy-laptop");

        assert_eq!(scan.services.len(), 2);
        let pg = scan.find_service(ServiceKind::Postgres, "shop-db").unwrap();
        assert!(pg.is_running());
        assert_eq!(pg.version, "postgres:16.4");
        assert_eq!(pg.linked_apps, vec!["rails-shop"]);

        assert_eq!(scan.apps.len(), 1);
        let app = scan.find_app("rails-shop").unwrap();
        assert!(app.deployed && app.running && !app.degraded());
        assert_eq!(app.process_summary(), "web 2/2 · worker 1/1");
        assert_eq!(app.domains, vec!["shop.example.com", "www.shop.example.com"]);
        assert_eq!(app.ports.len(), 2);
        assert_eq!(app.ports[0].spec(), "http:80:3000");
        assert!(app.ssl_enabled && app.letsencrypt_active);
        assert_eq!(app.git_branch, "main");
        assert_eq!(app.git_sha, "7f3a91c82b");
        assert_eq!(app.mounts, vec!["/var/lib/dokku/data/storage/rails-shop:/rails/storage"]);
        assert_eq!(
            app.linked_services,
            vec![
                (ServiceKind::Postgres, "shop-db".into()),
                (ServiceKind::Redis, "shop-queue".into())
            ]
        );
    }

    #[test]
    fn parses_missing_dokku_with_host_facts() {
        let scan = parse_scan("@@host os=debian mem_mb=2048 docker=0\n@@missing\n");
        assert!(!scan.installed);
        assert!(scan.can_bootstrap());
        assert_eq!(scan.mem_mb, 2048);
        assert!(!scan.docker_ok);
    }

    #[test]
    fn validates_names_domains_ports_and_mounts() {
        assert!(validate_name("rails-shop-1").is_ok());
        assert!(validate_name("Bad_Name").is_err());
        assert!(validate_name("-leading").is_err());

        assert_eq!(validate_domain("Shop.Example.com").unwrap(), "shop.example.com");
        assert!(validate_domain("*.example.com").is_ok());
        assert!(validate_domain("not-a-domain").is_err());

        let pm = validate_port_spec("http:80:3000").unwrap();
        assert_eq!((pm.scheme.as_str(), pm.host_port, pm.container_port), ("http", 80, 3000));
        assert!(validate_port_spec("ftp:21:21").is_err());

        assert!(validate_mount_spec("/var/lib/dokku/data/storage/app:/rails/storage").is_ok());
        assert!(validate_mount_spec("relative:/rails/storage").is_err());
    }

    #[test]
    fn builds_encoded_config_commands_and_redacts_summary() {
        let mut set = BTreeMap::new();
        set.insert("RAILS_MASTER_KEY".into(), "deadbeef1234".into());
        set.insert("RAILS_ENV".into(), "production".into());
        let unset = vec!["OLD_FLAG".to_string()];

        let (cmds, summary) = build_config_commands("rails-shop", &set, &unset, false);
        assert_eq!(cmds.len(), 2);
        assert_eq!(
            cmds[0],
            vec![
                "dokku",
                "config:set",
                "--encoded",
                "--no-restart",
                "rails-shop",
                "RAILS_ENV=cHJvZHVjdGlvbg==",
                "RAILS_MASTER_KEY=ZGVhZGJlZWYxMjM0",
            ]
        );
        assert_eq!(cmds[1], vec!["dokku", "config:unset", "--no-restart", "rails-shop", "OLD_FLAG"]);
        assert!(!summary.contains("deadbeef1234"));
        assert!(summary.contains("(•••)"));
    }

    #[test]
    fn parses_dotenv_lines() {
        let env = parse_dotenv("# comment\nexport RAILS_ENV=production\nSECRET_KEY_BASE=\"my secret=value\"\n").unwrap();
        assert_eq!(env.get("RAILS_ENV").map(String::as_str), Some("production"));
        assert_eq!(env.get("SECRET_KEY_BASE").map(String::as_str), Some("my secret=value"));
    }
}
