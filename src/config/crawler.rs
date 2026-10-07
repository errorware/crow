use std::path::{Path, PathBuf};
use std::fs;
use crate::host::Host;
use super::plugins::editor_for;
use crate::os_detect::DistroFamily;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaKind {
    PgHba,
    Journald,
    Cron,
    Sshd,
    Hosts,
    /// sysctl.conf and /etc/sysctl.d/*.conf.
    Sysctl,
    /// /etc/sudoers and /etc/sudoers.d/*, read and written as root.
    Sudoers,
    /// /etc/fstab.
    Fstab,
    /// logrotate.conf and /etc/logrotate.d/*.
    Logrotate,
    /// systemd units and drop-ins in /etc/systemd/system, and systemd's
    /// own settings files (/etc/systemd/*.conf).
    Systemd,
    /// nginx.conf, conf.d/*.conf and sites-available/*.
    Nginx,
    /// INI files: MySQL/MariaDB (*.cnf) and Samba (smb.conf).
    Ini,
    /// fail2ban's jail.local and jail.d/*.
    Fail2ban,
    /// chrony.conf.
    Chrony,
    /// ntp.conf.
    Ntp,
    /// /etc/resolv.conf.
    Resolv,
    Ufw,
    /// /etc/passwd and /etc/group, owned by the Users screen.
    Accounts,
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
            Self::Sysctl => "procps",
            Self::Sudoers => "sudo",
            Self::Fstab => "util-linux",
            Self::Logrotate => "logrotate",
            Self::Systemd => "systemd",
            Self::Nginx => "nginx",
            Self::Ini => "ini",
            Self::Fail2ban => "fail2ban",
            Self::Chrony => "chrony",
            Self::Ntp => "ntp",
            Self::Resolv => "resolver",
            Self::Ufw => "ufw firewall",
            Self::Accounts => "accounts",
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
            Self::Sysctl => "KEY-VALUE UI",
            Self::Sudoers => "RULE TABLE UI",
            Self::Fstab => "RULE TABLE UI",
            Self::Logrotate => "DIRECTIVE UI",
            Self::Systemd => "DIRECTIVE UI",
            Self::Nginx => "DIRECTIVE UI",
            Self::Ini => "DIRECTIVE UI",
            Self::Fail2ban => "DIRECTIVE UI",
            Self::Chrony => "DIRECTIVE UI",
            Self::Ntp => "DIRECTIVE UI",
            Self::Resolv => "DIRECTIVE UI",
            Self::Ufw => "FIREWALL UI",
            Self::Accounts => "USERS UI",
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
    } else if lower_name == "crontab" {
        // Only the system crontab: the Cron screen edits that one file.
        Some(SchemaKind::Cron)
    } else if lower_name == "sshd_config" || lower_name.starts_with("sshd_config.d") || path_str.contains("ssh/sshd_config") {
        Some(SchemaKind::Sshd)
    } else if lower_name == "hosts" && (path_str == "/etc" || path_str.ends_with("/etc/hosts") || path_str.ends_with("hosts")) {
        Some(SchemaKind::Hosts)
    } else if (lower_name.ends_with(".cnf") && (path_str.starts_with("/etc/mysql/") || path_str.starts_with("/etc/my.cnf"))) || (lower_name == "smb.conf" && path_str.starts_with("/etc/samba/")) {
        Some(SchemaKind::Ini)
    } else if path_str.starts_with("/etc/fail2ban/") && (lower_name == "jail.local" || (path_str.starts_with("/etc/fail2ban/jail.d/") && (lower_name.ends_with(".local") || lower_name.ends_with(".conf")))) {
        Some(SchemaKind::Fail2ban)
    } else if lower_name == "chrony.conf" && path_str.starts_with("/etc/") {
        Some(SchemaKind::Chrony)
    } else if lower_name == "ntp.conf" && path_str.starts_with("/etc/") {
        Some(SchemaKind::Ntp)
    } else if path_str == "/etc/resolv.conf" {
        Some(SchemaKind::Resolv)
    } else if path_str.starts_with("/etc/nginx/") && (lower_name.ends_with(".conf") && !lower_name.ends_with("mime.types") || path_str.contains("/sites-available/") || path_str.contains("/sites-enabled/")) {
        Some(SchemaKind::Nginx)
    } else if path_str.starts_with("/etc/systemd/") && (is_unit_file(&lower_name) || lower_name.ends_with(".conf")) {
        // journald.conf was matched above (its own editor).
        Some(SchemaKind::Systemd)
    } else if lower_name == "logrotate.conf" || path_str.starts_with("/etc/logrotate.d/") {
        Some(SchemaKind::Logrotate)
    } else if lower_name == "fstab" && path_str == "/etc/fstab" {
        Some(SchemaKind::Fstab)
    } else if (lower_name == "sudoers" && path_str.starts_with("/etc/")) || path_str.starts_with("/etc/sudoers.d/") {
        Some(SchemaKind::Sudoers)
    } else if lower_name == "sysctl.conf" || (path_str.contains("/sysctl.d/") && lower_name.ends_with(".conf")) {
        Some(SchemaKind::Sysctl)
    } else if lower_name == "user.rules" || lower_name == "user6.rules" {
        // ufw's own rule store, managed by the Firewall screen. before/after
        // *.rules are raw iptables-restore files: plain text.
        Some(SchemaKind::Ufw)
    } else if (lower_name == "passwd" || lower_name == "group") && path_str.starts_with("/etc/") {
        Some(SchemaKind::Accounts)
    } else if lower_name == "config.toml" && path_str.contains("crow") {
        Some(SchemaKind::Crow)
    } else {
        None
    }
}

/// Crawls a host's well-known configuration paths for real files. `family`
/// picks the directories worth scanning; nothing is listed that isn't there.
/// Returns the files with all schema-mapped ones bumped to the top.
/// The files the Config screen lists: everything found except files another
/// Crow screen owns (see `ConfigEditor::is_listed`).
pub fn crawl_configs(host: &dyn Host, family: DistroFamily) -> Vec<DiscoveredConfigFile> {
    crawl_all_configs(host, family).into_iter().filter(|f| editor_for(f.schema_kind).is_listed()).collect()
}

/// Every config file found, including those owned by other screens (the Cron
/// screen still edits crontab's loaded state, for example).
pub fn crawl_all_configs(host: &dyn Host, family: DistroFamily) -> Vec<DiscoveredConfigFile> {
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
            "/etc/nginx/conf.d",
            "/etc/postgresql",
            "/etc/ufw",
            "/etc/fail2ban",
            "/etc/fail2ban/jail.d",
            "/etc/chrony",
            "/etc/ntpsec",
            "/etc/sysctl.d",
            "/etc/docker",
            "/etc/logrotate.d",
            "/etc/systemd/system",
            "/etc/mysql",
            "/etc/mysql/conf.d",
            "/etc/mysql/mariadb.conf.d",
            "/etc/my.cnf.d",
            "/etc/samba",
        ],
        DistroFamily::RedHat => vec![
            "/etc",
            "/etc/systemd",
            "/etc/ssh",
            "/etc/nginx",
            "/etc/nginx/conf.d",
            "/etc/firewalld",
            "/etc/fail2ban",
            "/etc/fail2ban/jail.d",
            "/etc/chrony",
            "/etc/sysctl.d",
            "/etc/docker",
            "/etc/cron.d",
            "/etc/logrotate.d",
            "/etc/systemd/system",
            "/etc/mysql",
            "/etc/mysql/conf.d",
            "/etc/mysql/mariadb.conf.d",
            "/etc/my.cnf.d",
            "/etc/samba",
        ],
        DistroFamily::Unknown => vec![
            "/etc",
            "/etc/systemd",
            "/etc/ssh",
            "/etc/fail2ban",
            "/etc/fail2ban/jail.d",
            "/etc/chrony",
            "/etc/ntpsec",
            "/etc/sysctl.d",
            "/etc/logrotate.d",
            "/etc/systemd/system",
            "/etc/mysql",
            "/etc/mysql/conf.d",
            "/etc/mysql/mariadb.conf.d",
            "/etc/my.cnf.d",
            "/etc/samba",
            "/etc/nginx",
            "/etc/nginx/conf.d",
            "/etc/nginx/sites-available",
        ],
    };

    // One round trip for every directory.
    let mut push = |discovered: &mut Vec<DiscoveredConfigFile>, name: String, dir: &str, entry: &crate::host::DirEntry| {
        let p = Path::new(dir).join(&entry.name);
        if !seen_paths.insert(p.clone()) {
            return;
        }
        let schema = detect_schema_kind(&entry.name, &p);
        let is_mapped = editor_for(schema).is_crow_ui();
        discovered.push(DiscoveredConfigFile {
            name,
            path_dir: dir.to_string(),
            full_path: p,
            schema_kind: schema,
            is_schema_mapped: is_mapped,
            size_bytes: entry.size_bytes,
            is_readonly: entry.mode & 0o222 == 0,
            pill: if is_mapped { "CROW UI".to_string() } else { "RAW TEXT".to_string() },
            schema_pack: schema.map(|s| s.label()),
        });
    };
    let mut listings = host.list_dirs(&scan_dirs);
    let mut drop_in_dirs: Vec<String> = Vec::new();
    for dir_str in &scan_dirs {
        let Some(entries) = listings.remove(*dir_str) else { continue };
        for entry in entries {
            // /etc/systemd/system: the units written there (most entries are
            // symlinks to the distro's own, which aren't edited here), and
            // the drop-in folders beside them.
            if *dir_str == "/etc/systemd/system" {
                if entry.is_dir && entry.name.ends_with(".d") {
                    drop_in_dirs.push(format!("{dir_str}/{}", entry.name));
                } else if !entry.is_dir && !entry.is_symlink && is_unit_file(&entry.name) {
                    push(&mut discovered, entry.name.clone(), dir_str, &entry);
                }
                continue;
            }
            // nginx sites: sites-available holds the files (no extension);
            // sites-enabled is symlinks to them, not edited twice.
            if dir_str.ends_with("/nginx/sites-available") || dir_str.ends_with("/nginx/sites-enabled") {
                if !entry.is_dir && !entry.is_symlink && !entry.name.starts_with('.') && !entry.name.ends_with('~') {
                    let folder = dir_str.rsplit('/').next().unwrap_or_default();
                    push(&mut discovered, format!("{folder}/{}", entry.name), dir_str, &entry);
                }
                continue;
            }
            // jail.d files are named with their folder, as fail2ban's docs do.
            if dir_str.ends_with("/fail2ban/jail.d") {
                if !entry.is_dir && (entry.name.ends_with(".local") || entry.name.ends_with(".conf")) && !entry.name.starts_with('.') {
                    push(&mut discovered, format!("jail.d/{}", entry.name), dir_str, &entry);
                }
                continue;
            }
            // logrotate.d files are named after their package, no extension.
            let in_logrotate_d = dir_str.ends_with("/logrotate.d") && !entry.name.starts_with('.') && !entry.name.ends_with(['~']) && !entry.name.contains(".dpkg-") && !entry.name.ends_with(".rpmsave") && !entry.name.ends_with(".rpmnew");
            if entry.is_dir || !(is_config_file(&entry.name, entry.size_bytes) || in_logrotate_d) {
                continue;
            }
            push(&mut discovered, entry.name.clone(), dir_str, &entry);
        }
    }
    // Drop-ins (foo.service.d/override.conf): named with their folder, since
    // override.conf is everywhere.
    if !drop_in_dirs.is_empty() {
        let refs: Vec<&str> = drop_in_dirs.iter().map(String::as_str).collect();
        let mut listed = host.list_dirs(&refs);
        for dir in &drop_in_dirs {
            let folder = dir.rsplit('/').next().unwrap_or_default().to_string();
            for entry in listed.remove(dir.as_str()).unwrap_or_default() {
                if !entry.is_dir && !entry.is_symlink && entry.name.ends_with(".conf") {
                    push(&mut discovered, format!("{folder}/{}", entry.name), dir, &entry);
                }
            }
        }
    }

    // Check user's crow config path (~/.config/crow/config.toml) — Crow's own
    // settings, so only when crawling the machine Crow runs on.
    if let Some(home) = dirs::home_dir().filter(|_| host.is_local()) {
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
                priority(&a.name).cmp(&priority(&b.name)).then_with(|| a.full_path.cmp(&b.full_path))
            }
            (false, false) => a.name.cmp(&b.name),
        }
    });

    discovered
}

/// A systemd unit file's name.
fn is_unit_file(name: &str) -> bool {
    [".service", ".socket", ".timer", ".path", ".mount", ".automount", ".target", ".swap", ".slice"].iter().any(|s| name.ends_with(s))
}

fn is_config_file(name: &str, size_bytes: u64) -> bool {
    let n = name.to_lowercase();
    let config_like = n.ends_with(".conf")
        || n.ends_with(".cnf")
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
        assert_eq!(detect_schema_kind("my.cnf", Path::new("/etc/mysql/my.cnf")), Some(SchemaKind::Ini));
        assert_eq!(detect_schema_kind("50-server.cnf", Path::new("/etc/mysql/mariadb.conf.d/50-server.cnf")), Some(SchemaKind::Ini));
        assert_eq!(detect_schema_kind("my.cnf", Path::new("/etc/my.cnf")), Some(SchemaKind::Ini));
        assert_eq!(detect_schema_kind("smb.conf", Path::new("/etc/samba/smb.conf")), Some(SchemaKind::Ini));
        assert_eq!(detect_schema_kind("jail.local", Path::new("/etc/fail2ban/jail.local")), Some(SchemaKind::Fail2ban));
        assert_eq!(detect_schema_kind("sshd.local", Path::new("/etc/fail2ban/jail.d/sshd.local")), Some(SchemaKind::Fail2ban));
        assert_eq!(detect_schema_kind("jail.conf", Path::new("/etc/fail2ban/jail.conf")), None, "the distro's own: overridden in jail.local, not edited");
        assert_eq!(detect_schema_kind("chrony.conf", Path::new("/etc/chrony/chrony.conf")), Some(SchemaKind::Chrony));
        assert_eq!(detect_schema_kind("chrony.conf", Path::new("/etc/chrony.conf")), Some(SchemaKind::Chrony));
        assert_eq!(detect_schema_kind("ntp.conf", Path::new("/etc/ntpsec/ntp.conf")), Some(SchemaKind::Ntp));
        assert_eq!(detect_schema_kind("resolv.conf", Path::new("/etc/resolv.conf")), Some(SchemaKind::Resolv));
        assert_eq!(detect_schema_kind("sysctl.conf", Path::new("/etc/sysctl.conf")), Some(SchemaKind::Sysctl));
        assert_eq!(detect_schema_kind("99-hardening.conf", Path::new("/etc/sysctl.d/99-hardening.conf")), Some(SchemaKind::Sysctl));
        assert_eq!(detect_schema_kind("README", Path::new("/etc/sysctl.d/README")), None);
        assert_eq!(detect_schema_kind("sudoers", Path::new("/etc/sudoers")), Some(SchemaKind::Sudoers));
        assert_eq!(detect_schema_kind("fstab", Path::new("/etc/fstab")), Some(SchemaKind::Fstab));
        assert_eq!(detect_schema_kind("logrotate.conf", Path::new("/etc/logrotate.conf")), Some(SchemaKind::Logrotate));
        assert_eq!(detect_schema_kind("app.service", Path::new("/etc/systemd/system/app.service")), Some(SchemaKind::Systemd));
        assert_eq!(detect_schema_kind("override.conf", Path::new("/etc/systemd/system/app.service.d/override.conf")), Some(SchemaKind::Systemd));
        assert_eq!(detect_schema_kind("logind.conf", Path::new("/etc/systemd/logind.conf")), Some(SchemaKind::Systemd));
        assert_eq!(detect_schema_kind("nginx.conf", Path::new("/etc/nginx/nginx.conf")), Some(SchemaKind::Nginx));
        assert_eq!(detect_schema_kind("default", Path::new("/etc/nginx/sites-available/default")), Some(SchemaKind::Nginx));
        assert_eq!(detect_schema_kind("app.conf", Path::new("/etc/nginx/conf.d/app.conf")), Some(SchemaKind::Nginx));
        assert_eq!(detect_schema_kind("journald.conf", Path::new("/etc/systemd/journald.conf")), Some(SchemaKind::Journald), "keeps its own editor");
        assert_eq!(detect_schema_kind("nginx", Path::new("/etc/logrotate.d/nginx")), Some(SchemaKind::Logrotate));
        assert_eq!(detect_schema_kind("90-cloud-init-users", Path::new("/etc/sudoers.d/90-cloud-init-users")), Some(SchemaKind::Sudoers));
    }

    #[test]
    fn crow_ui_files_come_first_and_screen_owned_files_are_not_listed() {
        let configs = crawl_configs(&LocalHost, DistroFamily::Debian);
        // Every CROW UI file comes before every plain-text file.
        if let Some(first_text) = configs.iter().position(|c| !c.is_schema_mapped) {
            assert!(configs[first_text..].iter().all(|c| !c.is_schema_mapped));
        }
        assert!(configs.iter().all(|c| editor_for(c.schema_kind).is_listed()));
        for owned in ["crontab", "user.rules", "passwd", "group"] {
            assert!(!configs.iter().any(|c| c.name == owned), "{owned} belongs to its own screen");
        }
        // They are still found, for the screens that edit them.
        let all = crawl_all_configs(&LocalHost, DistroFamily::Debian);
        if std::path::Path::new("/etc/crontab").exists() {
            assert!(all.iter().any(|c| c.name == "crontab"));
        }
    }

    /// The remote code path (one scripted exec per batch) reads the same real
    /// /etc as the local one, in two round trips (three with systemd drop-in
    /// folders) instead of one per dir/file.
    #[test]
    fn remote_style_crawl_matches_local_in_two_round_trips() {
        struct Counting(std::sync::atomic::AtomicUsize);
        impl Host for Counting {
            fn label(&self) -> String {
                "counting".into()
            }
            fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: std::time::Duration) -> Result<crate::host::ExecOutput, crate::host::HostError> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                LocalHost.exec_stdin(argv, stdin, timeout)
            }
        }
        let remote = Counting(Default::default());
        let files = crawl_configs(&remote, DistroFamily::Debian);
        let states = load_config_file_states(&remote, &files);
        // One more when /etc/systemd/system has drop-in folders: their names
        // are only known from the first listing.
        let drop_ins = std::fs::read_dir("/etc/systemd/system").map(|d| d.flatten().any(|e| e.file_name().to_string_lossy().ends_with(".d") && e.path().is_dir())).unwrap_or(false);
        assert_eq!(remote.0.load(std::sync::atomic::Ordering::SeqCst), 2 + drop_ins as usize);

        // Crow's own config.toml is only listed for the machine Crow runs on.
        let local_files: Vec<_> = crawl_configs(&LocalHost, DistroFamily::Debian).into_iter().filter(|f| f.name != "config.toml").collect();
        let local_states = load_config_file_states(&LocalHost, &local_files);
        let names = |f: &[DiscoveredConfigFile]| f.iter().map(|f| f.full_path.clone()).collect::<Vec<_>>();
        assert_eq!(names(&files), names(&local_files));
        for f in &files {
            assert_eq!(states[&f.name].current_content, local_states[&f.name].current_content, "{}", f.name);
            assert_eq!(states[&f.name].write_blocked.is_some(), local_states[&f.name].write_blocked.is_some(), "{}", f.name);
        }
        println!("{} files, was {} round trips, now 2", files.len(), 11 + files.len());
    }

    #[test]
    fn test_unknown_distro_fabricates_nothing() {
        let configs = crawl_configs(&LocalHost, DistroFamily::Unknown);
        // symlink_metadata, not exists(): /etc/grub2.cfg links into /boot,
        // which an unprivileged user can't traverse — it's still a real file.
        let missing: Vec<_> = configs.iter().filter(|c| c.full_path.symlink_metadata().is_err()).map(|c| &c.full_path).collect();
        assert!(missing.is_empty(), "every listed file exists on the host, missing: {missing:?}");
    }
}

/// Loads every discovered file's content from `host` in one round trip, as
/// versioned edit states keyed by file name. A file that can't be read is
/// shown empty and marked `write_blocked`, so nothing can be written over
/// the real file blind.
pub fn load_config_file_states(host: &dyn Host, files: &[DiscoveredConfigFile]) -> std::collections::HashMap<String, super::ConfigFileState> {
    let paths: Vec<String> = files.iter().map(|f| f.full_path.to_string_lossy().into_owned()).collect();
    let path_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    let mut contents = host.read_files(&path_refs);
    files
        .iter()
        .zip(&paths)
        .map(|(f, path)| {
            let (content, blocked) = match contents.remove(path.as_str()) {
                Some(Ok(content)) => (content, None),
                Some(Err(e)) => (String::new(), Some(format!("Could not read {} from {}: {}", f.name, host.label(), e))),
                None => (String::new(), Some(format!("Could not read {} from {}", f.name, host.label()))),
            };
            let mut state = super::ConfigFileState::new(f.full_path.clone(), f.name.clone(), content);
            state.read_from_host = blocked.is_none();
            state.write_blocked = blocked;
            (f.name.clone(), state)
        })
        .collect()
}
