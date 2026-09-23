use std::path::{Path, PathBuf};
use std::fs;
use crate::host::Host;
use crate::os_detect::DistroFamily;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaKind {
    PgHba,
    Journald,
    Cron,
    Sshd,
    Hosts,
    Ufw,
    Crow,
}

impl SchemaKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PgHba => "postgres 16",
            Self::Journald => "systemd 255",
            Self::Cron => "vixie-cron",
            Self::Sshd => "openssh 9.6",
            Self::Hosts => "linux-net",
            Self::Ufw => "ufw firewall",
            Self::Crow => "crow core",
        }
    }

    pub fn editor_type(&self) -> &'static str {
        match self {
            Self::PgHba => "RULE TABLE UI",
            Self::Journald => "CRASH-SAFE UI",
            Self::Cron => "SCHEDULE UI",
            Self::Sshd => "DIRECTIVE UI",
            Self::Hosts => "KEY-VALUE UI",
            Self::Ufw => "FIREWALL UI",
            Self::Crow => "LOSSLESS TOML",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredConfigFile {
    pub name: String,
    pub path_dir: String,
    pub full_path: PathBuf,
    pub schema_kind: Option<SchemaKind>,
    pub is_schema_mapped: bool,
    pub size_bytes: u64,
    pub is_readonly: bool,
    pub pill: String,
    pub schema_pack: Option<&'static str>,
    /// True for a baseline-seed placeholder (a well-known config Crow expects
    /// for this distro family but did not actually find on disk) — never a
    /// real discovery. The UI must never present this with the same
    /// confidence as a genuinely found file.
    pub is_synthetic: bool,
}

impl DiscoveredConfigFile {
    pub fn is_mapped(&self) -> bool {
        self.is_schema_mapped
    }

    pub fn display_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Detects if a filename or path matches one of Crow's mapped schema plugins.
/// Extensible: Any format supported by crow-config-schemas or Crow core is mapped here.
pub fn detect_schema_kind(name: &str, path: &Path) -> Option<SchemaKind> {
    let lower_name = name.to_lowercase();
    let path_str = path.to_string_lossy().to_lowercase();

    if lower_name == "pg_hba.conf" || path_str.contains("pg_hba") {
        Some(SchemaKind::PgHba)
    } else if lower_name == "journald.conf" || path_str.contains("journald.conf") {
        Some(SchemaKind::Journald)
    } else if lower_name == "crontab" || lower_name.ends_with(".cron") || path_str.contains("cron.d") || path_str.contains("crontab") {
        Some(SchemaKind::Cron)
    } else if lower_name == "sshd_config" || lower_name.starts_with("sshd_config.d") || path_str.contains("ssh/sshd_config") {
        Some(SchemaKind::Sshd)
    } else if lower_name == "hosts" && (path_str == "/etc" || path_str.ends_with("/etc/hosts") || path_str.ends_with("hosts")) {
        Some(SchemaKind::Hosts)
    } else if lower_name.ends_with(".rules") && (path_str.contains("ufw") || lower_name == "user.rules") {
        Some(SchemaKind::Ufw)
    } else if lower_name == "config.toml" && path_str.contains("crow") {
        Some(SchemaKind::Crow)
    } else {
        None
    }
}

/// Crawls a host's well-known configuration paths (none when `host` is `None`,
/// i.e. a server Crow cannot reach yet — only placeholders are listed then).
/// Returns a list of configuration files, with all schema-mapped files bumped to the top.
/// `family` narrows both which directories get scanned and which baseline
/// placeholders (see below) are plausible for this host — on an unrecognized
/// distro, Crow scans a generic path set and fabricates no placeholders at
/// all, rather than presenting Debian-shaped guesses as if they were real.
pub fn crawl_configs(host: Option<&dyn Host>, family: DistroFamily) -> Vec<DiscoveredConfigFile> {
    let mut discovered: Vec<DiscoveredConfigFile> = Vec::new();
    let mut seen_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    // Standard directories to inspect for actual machine files — the flat,
    // non-nested locations for this distro family's real package layout.
    let scan_dirs: Vec<&str> = match family {
        DistroFamily::Debian => vec![
            "/etc",
            "/etc/systemd",
            "/etc/ssh",
            "/etc/nginx",
            "/etc/nginx/sites-available",
            "/etc/nginx/sites-enabled",
            "/etc/postgresql",
            "/etc/ufw",
            "/etc/fail2ban",
            "/etc/sysctl.d",
            "/etc/docker",
        ],
        DistroFamily::RedHat => vec![
            "/etc",
            "/etc/systemd",
            "/etc/ssh",
            "/etc/nginx",
            "/etc/nginx/conf.d",
            "/etc/firewalld",
            "/etc/fail2ban",
            "/etc/sysctl.d",
            "/etc/docker",
            "/etc/cron.d",
        ],
        DistroFamily::Unknown => vec![
            "/etc",
            "/etc/systemd",
            "/etc/ssh",
            "/etc/sysctl.d",
        ],
    };

    for dir_str in host.map(|_| scan_dirs.as_slice()).unwrap_or(&[]) {
        let Ok(entries) = host.expect("scan dirs only with a host").list_dir(dir_str) else { continue };
        for entry in entries {
            if entry.is_dir || !is_config_file(&entry.name, entry.size_bytes) {
                continue;
            }
            let p = Path::new(dir_str).join(&entry.name);
            if !seen_paths.insert(p.clone()) {
                continue;
            }
            let schema = detect_schema_kind(&entry.name, &p);
            let is_mapped = schema.is_some();
            discovered.push(DiscoveredConfigFile {
                name: entry.name.clone(),
                path_dir: dir_str.to_string(),
                full_path: p,
                schema_kind: schema,
                is_schema_mapped: is_mapped,
                size_bytes: entry.size_bytes,
                is_readonly: entry.mode & 0o222 == 0,
                pill: if is_mapped { "CROW UI".to_string() } else { "RAW TEXT".to_string() },
                schema_pack: schema.map(|s| s.label()),
                is_synthetic: false,
            });
        }
    }

    // Check user's crow config path (~/.config/crow/config.toml) — Crow's own
    // settings, so only when crawling the machine Crow runs on.
    if let Some(home) = dirs::home_dir().filter(|_| host.is_some_and(|h| h.is_local())) {
        let crow_cfg = home.join(".config").join("crow").join("config.toml");
        if crow_cfg.exists() && seen_paths.insert(crow_cfg.clone()) {
            let size = fs::metadata(&crow_cfg).map(|m| m.len()).unwrap_or(1200);
            discovered.push(DiscoveredConfigFile {
                name: "config.toml".to_string(),
                path_dir: format!("{}/.config/crow", home.display()),
                full_path: crow_cfg,
                schema_kind: Some(SchemaKind::Crow),
                is_schema_mapped: true,
                size_bytes: size,
                is_readonly: false,
                pill: "CROW UI".to_string(),
                schema_pack: Some("crow core"),
                is_synthetic: false,
            });
        }
    }

    // Baseline Seeded Known Configs — plausible-for-this-family placeholders so
    // the mapped schema tools stay browsable even if e.g. postgres isn't
    // actually installed locally. Every entry here is marked `is_synthetic`;
    // the UI must show that distinction. On an unrecognized distro, Crow
    // fabricates none of these at all — a real discovery or nothing.
    let baseline_seeds: &[(&str, &str, Option<SchemaKind>, u64, &str, &str)] = match family {
        DistroFamily::Debian => &[
            ("journald.conf", "/etc/systemd", Some(SchemaKind::Journald), 2840, "CRASH-SAFE", "systemd 255"),
            ("pg_hba.conf", "/etc/postgresql/16/main", Some(SchemaKind::PgHba), 4510, "EDITED", "postgres 16"),
            ("sshd_config", "/etc/ssh", Some(SchemaKind::Sshd), 3240, "OK", "openssh 9.6"),
            ("hosts", "/etc", Some(SchemaKind::Hosts), 820, "OK", "linux-net"),
            ("user.rules", "/etc/ufw", Some(SchemaKind::Ufw), 1840, "EDITED", "ufw firewall"),
            ("postgresql.conf", "/etc/postgresql/16/main", None, 28900, "OK", "postgres 16"),
            ("nginx.conf", "/etc/nginx", None, 1480, "OK", "web"),
            ("sites-enabled/api", "/etc/nginx", None, 920, "OK", "web"),
            ("authorized_keys", "/root/.ssh", None, 1024, "DRIFT", "security"),
            ("fail2ban/jail.local", "/etc/fail2ban", None, 2150, "OK", "security"),
            ("sysctl.d/99-tuning", "/etc", None, 640, "OK", "kernel"),
            ("crontab", "/etc", Some(SchemaKind::Cron), 1180, "OK", "vixie-cron"),
            ("passwd", "/etc", None, 2140, "OK", "accounts"),
            ("group", "/etc", None, 980, "OK", "accounts"),
            ("resolv.conf", "/etc", None, 340, "LOCKED", "dns"),
            ("docker/daemon.json", "/etc", None, 580, "OK", "containers"),
        ],
        DistroFamily::RedHat => &[
            ("journald.conf", "/etc/systemd", Some(SchemaKind::Journald), 2840, "CRASH-SAFE", "systemd 255"),
            ("pg_hba.conf", "/var/lib/pgsql/16/data", Some(SchemaKind::PgHba), 4510, "EDITED", "postgres 16"),
            ("sshd_config", "/etc/ssh", Some(SchemaKind::Sshd), 3240, "OK", "openssh 9.6"),
            ("hosts", "/etc", Some(SchemaKind::Hosts), 820, "OK", "linux-net"),
            ("postgresql.conf", "/var/lib/pgsql/16/data", None, 28900, "OK", "postgres 16"),
            ("nginx.conf", "/etc/nginx", None, 1480, "OK", "web"),
            ("default.conf", "/etc/nginx/conf.d", None, 920, "OK", "web"),
            ("authorized_keys", "/root/.ssh", None, 1024, "DRIFT", "security"),
            ("fail2ban/jail.local", "/etc/fail2ban", None, 2150, "OK", "security"),
            ("sysctl.d/99-tuning", "/etc", None, 640, "OK", "kernel"),
            ("crontab", "/etc", Some(SchemaKind::Cron), 1180, "OK", "vixie-cron"),
            ("passwd", "/etc", None, 2140, "OK", "accounts"),
            ("group", "/etc", None, 980, "OK", "accounts"),
            ("resolv.conf", "/etc", None, 340, "LOCKED", "dns"),
            ("docker/daemon.json", "/etc", None, 580, "OK", "containers"),
        ],
        DistroFamily::Unknown => &[],
    };

    for (name, path, schema, size, pill, pack) in baseline_seeds {
        let pb = PathBuf::from(format!("{}/{}", path, name));
        let already_present = discovered.iter().any(|d| d.name == *name || d.full_path == pb);
        if !already_present {
            let is_mapped = schema.is_some();
            let final_pill = if is_mapped && *pill == "OK" {
                "CROW UI".to_string()
            } else {
                pill.to_string()
            };
            discovered.push(DiscoveredConfigFile {
                name: name.to_string(),
                path_dir: path.to_string(),
                full_path: pb,
                schema_kind: *schema,
                is_schema_mapped: is_mapped,
                size_bytes: *size,
                is_readonly: *pill == "LOCKED",
                pill: final_pill,
                schema_pack: Some(pack),
                is_synthetic: true,
            });
        }
    }

    // Sort:
    // 1. Schema-mapped files first (CROW UI), ordered deterministically
    // 2. Unmapped/raw system configs second, alphabetically
    discovered.sort_by(|a, b| {
        match (a.is_schema_mapped, b.is_schema_mapped) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => {
                let priority = |name: &str| -> usize {
                    match name {
                        "pg_hba.conf" => 0,
                        "journald.conf" => 1,
                        "crontab" => 2,
                        "sshd_config" => 3,
                        "hosts" => 4,
                        "user.rules" => 5,
                        "config.toml" => 6,
                        _ => 10,
                    }
                };
                priority(&a.name).cmp(&priority(&b.name))
            }
            (false, false) => a.name.cmp(&b.name),
        }
    });

    discovered
}

fn is_config_file(name: &str, size_bytes: u64) -> bool {
    let n = name.to_lowercase();
    let config_like = n.ends_with(".conf")
        || n.ends_with(".toml")
        || n.ends_with(".yaml")
        || n.ends_with(".yml")
        || n.ends_with(".json")
        || n.ends_with(".rules")
        || n.ends_with(".ini")
        || n.ends_with(".cfg")
        || n == "hosts"
        || n == "passwd"
        || n == "group"
        || n == "crontab"
        || n == "fstab"
        || n == "resolv.conf"
        || n == "sshd_config"
        || n.starts_with("sshd_config.d");
    // Avoid huge files or binaries
    config_like && size_bytes < 2 * 1024 * 1024
}

pub fn sample_config_content(filename: &str) -> String {
    let lower = filename.to_lowercase();
    if lower == "pg_hba.conf" {
        r#"# PostgreSQL Client Authentication Configuration File
# ===================================================
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             postgres                                peer
local   all             all                                     peer
host    all             all             127.0.0.1/32            scram-sha-256
host    all             all             ::1/128                 scram-sha-256
host    acme_prod       acme_app        10.0.4.19/32            scram-sha-256
host    acme_prod       acme_app        10.0.4.22/32            scram-sha-256
host    acme_prod       analyst         10.0.9.0/24             ldap
host    all             all             0.0.0.0/0               md5
host    all             all             10.0.4.0/24             trust
host    replication     repl            10.0.4.11/32            trust
hostssl metrics         prom            10.0.4.31/32            cert
host    template1       all             10.0.4.0/24             reject
"#.to_string()
    } else if lower.contains("journald") {
        r#"# /etc/systemd/journald.conf
# Managed by Crow Fleet Manager
[Journal]
Storage=persistent
Compress=yes
SystemMaxUse=4096M
SystemKeepFree=1024M
MaxRetentionSec=30day
MaxFileSec=1month
ForwardToSyslog=no
"#.to_string()
    } else if lower.contains("nginx") {
        r#"user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log notice;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    sendfile on;
    keepalive_timeout 65;

    server {
        listen 80 default_server;
        server_name _;
        root /usr/share/nginx/html;

        location / {
            try_files $uri $uri/ =404;
        }
    }
}
"#.to_string()
    } else if lower.contains("sshd") {
        r#"# OpenSSH Server Configuration
Port 22
AddressFamily any
ListenAddress 0.0.0.0

# Authentication
PermitRootLogin prohibit-password
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
PasswordAuthentication no
KbdInteractiveAuthentication no

# Hardening
X11Forwarding no
MaxAuthTries 3
ClientAliveInterval 300
ClientAliveCountMax 2
Subsystem sftp /usr/lib/openssh/sftp-server
"#.to_string()
    } else if lower == "hosts" {
        r#"127.0.0.1   localhost localhost.localdomain
::1         localhost6 localhost6.localdomain6
10.0.4.12   db-primary-01.internal db-primary-01
10.0.4.13   db-replica-01.internal db-replica-01
10.0.4.20   api-gateway.internal api-01
"#.to_string()
    } else if lower == "passwd" {
        r#"root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
bin:x:2:2:bin:/bin:/usr/sbin/nologin
sys:x:3:3:sys:/dev:/usr/sbin/nologin
systemd-resolve:x:102:104:systemd Resolver:/run/systemd:/usr/sbin/nologin
nginx:x:103:105:nginx web server:/var/www:/usr/sbin/nologin
nelson:x:1000:1000:Nelson Errorware,,,,:/home/nelson:/bin/bash
deploy:x:1001:1001:CI/CD Pipeline Service Account:/home/deploy:/bin/bash
postgres:x:1002:1002:PostgreSQL Database Administrator:/var/lib/postgresql:/bin/bash
temp-contractor:!:1003:1003:Contractor (Expired Project):/home/temp-contractor:/bin/bash
"#.to_string()
    } else if lower.contains("user.rules") || lower.contains("ufw") {
        r#"*filter
:ufw-user-input - [0:0]
:ufw-user-output - [0:0]
:ufw-user-forward - [0:0]
-A ufw-user-input -p tcp --dport 22 -j ACCEPT -m comment --comment "OpenSSH"
-A ufw-user-input -p tcp --dport 80 -j ACCEPT -m comment --comment "HTTP"
-A ufw-user-input -p tcp --dport 443 -j ACCEPT -m comment --comment "HTTPS"
-A ufw-user-input -p tcp --dport 5432 -s 10.0.4.0/24 -j ACCEPT -m comment --comment "PostgreSQL"
COMMIT
"#.to_string()
    } else if lower.contains("resolv.conf") {
        r#"# Generated by NetworkManager
nameserver 1.1.1.1
nameserver 8.8.8.8
search internal.net
options edns0 trust-ad
"#.to_string()
    } else if lower.contains("sysctl") {
        r#"# Kernel sysctl parameters
net.ipv4.ip_forward = 1
net.ipv4.tcp_syncookies = 1
net.core.somaxconn = 4096
vm.swappiness = 10
fs.file-max = 2097152
"#.to_string()
    } else if lower.contains("daemon.json") {
        r#"{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "50m",
    "max-file": "3"
  },
  "live-restore": true,
  "storage-driver": "overlay2"
}
"#.to_string()
    } else if lower.contains("crontab") {
        r#"# m h dom mon dow user command
17 * * * * root cd / && run-parts --report /etc/cron.hourly
25 6 * * * root test -x /usr/sbin/anacron || ( cd / && run-parts --report /etc/cron.daily )
47 6 * * 7 root test -x /usr/sbin/anacron || ( cd / && run-parts --report /etc/cron.weekly )
52 6 1 * * root test -x /usr/sbin/anacron || ( cd / && run-parts --report /etc/cron.monthly )
0 3 * * * root /usr/local/bin/crow-backup.sh >/dev/null 2>&1
"#.to_string()
    } else {
        format!(
            "# Configuration file: {}\n# Managed by Crow Fleet Manager\nenabled = true\nlog_level = \"info\"\nmax_retries = 5\ntimeout_seconds = 30\n",
            filename
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::LocalHost;

    #[test]
    fn test_schema_kind_detection() {
        assert_eq!(detect_schema_kind("pg_hba.conf", Path::new("/etc/postgresql/16/main/pg_hba.conf")), Some(SchemaKind::PgHba));
        assert_eq!(detect_schema_kind("journald.conf", Path::new("/etc/systemd/journald.conf")), Some(SchemaKind::Journald));
        assert_eq!(detect_schema_kind("sshd_config", Path::new("/etc/ssh/sshd_config")), Some(SchemaKind::Sshd));
        assert_eq!(detect_schema_kind("hosts", Path::new("/etc/hosts")), Some(SchemaKind::Hosts));
        assert_eq!(detect_schema_kind("user.rules", Path::new("/etc/ufw/user.rules")), Some(SchemaKind::Ufw));
        assert_eq!(detect_schema_kind("nginx.conf", Path::new("/etc/nginx/nginx.conf")), None);
    }

    #[test]
    fn test_crawl_machine_configs_sorting() {
        // Debian family is deterministic regardless of the host running the
        // test: its baseline seeds guarantee mapped entries exist.
        let configs = crawl_configs(Some(&LocalHost), DistroFamily::Debian);
        assert!(!configs.is_empty());

        // First configs must be schema mapped
        assert!(configs[0].is_schema_mapped);
        assert!(configs.iter().take(4).all(|c| c.is_schema_mapped));

        // Unmapped configs must follow later
        let first_unmapped = configs.iter().position(|c| !c.is_schema_mapped);
        assert!(first_unmapped.is_some());
        assert!(first_unmapped.unwrap() > 3);
    }

    #[test]
    fn test_unknown_distro_fabricates_nothing() {
        let configs = crawl_configs(Some(&LocalHost), DistroFamily::Unknown);
        assert!(configs.iter().all(|c| !c.is_synthetic));
    }
}

/// Loads a discovered file's content from `host` into a versioned edit state.
/// Placeholders, unreadable files and servers with no transport get sample
/// content and are marked `write_blocked`, so illustrative text can never be
/// written over a real file.
pub fn load_config_file_state(host: Option<&dyn Host>, f: &DiscoveredConfigFile) -> super::ConfigFileState {
    let (content, blocked) = match host {
        _ if f.is_synthetic => (
            sample_config_content(&f.name),
            Some(format!("{} is a placeholder — it does not exist on this server", f.name)),
        ),
        None => (sample_config_content(&f.name), Some("This server is not connected — Crow has no transport to it yet".to_string())),
        Some(h) => match h.read_file(&f.full_path.to_string_lossy()) {
            Ok(content) => (content, None),
            Err(e) => (sample_config_content(&f.name), Some(format!("Could not read {} from {}: {}", f.name, h.label(), e))),
        },
    };
    let mut state = super::ConfigFileState::new(f.full_path.clone(), f.name.clone(), content);
    state.write_blocked = blocked;
    state
}
