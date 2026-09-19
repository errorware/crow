use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSshKeySummary {
    pub id: String,
    pub name: String,
    pub algorithm: String,
    pub fingerprint: String,
    pub comment: Option<String>,
    pub key_preview: String,
    pub added_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserAccountStatus {
    Active,
    Locked,
    SystemDaemon,
}

impl UserAccountStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Locked => "LOCKED",
            Self::SystemDaemon => "SYSTEM",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUserRecord {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub home_dir: String,
    pub shell: String,
    pub primary_group: String,
    pub groups: Vec<String>,
    pub is_system_user: bool,
    pub is_locked: bool,
    pub authorized_keys: Vec<UserSshKeySummary>,
    pub last_login: Option<String>,
}

impl SystemUserRecord {
    pub fn is_sudoer(&self) -> bool {
        self.groups.iter().any(|g| g == "sudo" || g == "wheel" || g == "admin") || self.username == "root"
    }

    pub fn is_human(&self) -> bool {
        !self.is_system_user || self.username == "root"
    }

    pub fn status(&self) -> UserAccountStatus {
        if self.is_locked {
            UserAccountStatus::Locked
        } else if self.is_system_user {
            UserAccountStatus::SystemDaemon
        } else {
            UserAccountStatus::Active
        }
    }

    /// Serializes this user into the canonical 7-field `/etc/passwd` format:
    /// `username:password:UID:GID:GECOS:directory:shell`
    #[allow(dead_code)]
    pub fn to_passwd_line(&self) -> String {
        let pwd_flag = if self.is_locked { "!" } else { "x" };
        format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.username, pwd_flag, self.uid, self.gid, self.gecos, self.home_dir, self.shell
        )
    }

    #[allow(dead_code)]
    pub fn parse_passwd_line(line: &str) -> Option<(String, String, u32, u32, String, String, String)> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() >= 7 {
            let username = parts[0].to_string();
            let pwd_flag = parts[1].to_string();
            let uid = parts[2].parse::<u32>().ok()?;
            let gid = parts[3].parse::<u32>().ok()?;
            let gecos = parts[4].to_string();
            let home = parts[5].to_string();
            let shell = parts[6].to_string();
            Some((username, pwd_flag, uid, gid, gecos, home, shell))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserFilterTab {
    All,
    Human,
    Sudoers,
    System,
}

impl UserFilterTab {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Accounts",
            Self::Human => "Login Users",
            Self::Sudoers => "Sudoers",
            Self::System => "System Daemons",
        }
    }
}

pub fn default_system_users() -> Vec<SystemUserRecord> {
    vec![
        SystemUserRecord {
            username: "root".to_string(),
            uid: 0,
            gid: 0,
            gecos: "root".to_string(),
            home_dir: "/root".to_string(),
            shell: "/bin/bash".to_string(),
            primary_group: "root".to_string(),
            groups: vec!["root".to_string()],
            is_system_user: false,
            is_locked: false,
            authorized_keys: vec![
                UserSshKeySummary {
                    id: "key_crow_admin".to_string(),
                    name: "crow-fleet-controller".to_string(),
                    algorithm: "ED25519".to_string(),
                    fingerprint: "SHA256:4tM83pP7xKkYnQ2+L8vB9uXz6W".to_string(),
                    comment: Some("crow@control-plane".to_string()),
                    key_preview: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI... crow@control-plane".to_string(),
                    added_at: "2026-03-12 09:15".to_string(),
                },
            ],
            last_login: Some("2026-09-19 14:22:10 via pts/1 (192.168.1.50)".to_string()),
        },
        SystemUserRecord {
            username: "nelson".to_string(),
            uid: 1000,
            gid: 1000,
            gecos: "Nelson Errorware,,,,".to_string(),
            home_dir: "/home/nelson".to_string(),
            shell: "/bin/bash".to_string(),
            primary_group: "nelson".to_string(),
            groups: vec![
                "nelson".to_string(),
                "sudo".to_string(),
                "docker".to_string(),
                "adm".to_string(),
                "systemd-journal".to_string(),
            ],
            is_system_user: false,
            is_locked: false,
            authorized_keys: vec![
                UserSshKeySummary {
                    id: "key_nelson_primary".to_string(),
                    name: "nelson-macbook-pro".to_string(),
                    algorithm: "ED25519".to_string(),
                    fingerprint: "SHA256:7mP09vK3sL2xZ1qW8bY6uT4rE1".to_string(),
                    comment: Some("nelson@mbp-m3.local".to_string()),
                    key_preview: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI7m... nelson@mbp".to_string(),
                    added_at: "2026-01-08 11:30".to_string(),
                },
                UserSshKeySummary {
                    id: "key_nelson_backup".to_string(),
                    name: "nelson-yubikey-fido".to_string(),
                    algorithm: "ED25519-SK".to_string(),
                    fingerprint: "SHA256:9qW8bY6uT4rE17mP09vK3sL2xZ".to_string(),
                    comment: Some("nelson@yubikey-5c".to_string()),
                    key_preview: "sk-ssh-ed25519@openssh.com AAAAGnNrLX...".to_string(),
                    added_at: "2026-02-14 16:45".to_string(),
                },
            ],
            last_login: Some("2026-09-19 15:10:02 via pts/0 (10.0.4.12)".to_string()),
        },
        SystemUserRecord {
            username: "deploy".to_string(),
            uid: 1001,
            gid: 1001,
            gecos: "CI/CD Pipeline Service Account".to_string(),
            home_dir: "/home/deploy".to_string(),
            shell: "/bin/bash".to_string(),
            primary_group: "deploy".to_string(),
            groups: vec!["deploy".to_string(), "docker".to_string()],
            is_system_user: false,
            is_locked: false,
            authorized_keys: vec![
                UserSshKeySummary {
                    id: "key_deploy_ci".to_string(),
                    name: "github-actions-runner".to_string(),
                    algorithm: "RSA-4096".to_string(),
                    fingerprint: "SHA256:1xZ1qW8bY6uT4rE17mP09vK3sL".to_string(),
                    comment: Some("ci-runner@github.internal".to_string()),
                    key_preview: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQD...".to_string(),
                    added_at: "2026-04-02 08:20".to_string(),
                },
            ],
            last_login: Some("2026-09-19 12:44:00 via ssh (10.0.8.201)".to_string()),
        },
        SystemUserRecord {
            username: "postgres".to_string(),
            uid: 1002,
            gid: 1002,
            gecos: "PostgreSQL Database Administrator".to_string(),
            home_dir: "/var/lib/postgresql".to_string(),
            shell: "/bin/bash".to_string(),
            primary_group: "postgres".to_string(),
            groups: vec!["postgres".to_string(), "ssl-cert".to_string()],
            is_system_user: false,
            is_locked: false,
            authorized_keys: vec![],
            last_login: None,
        },
        SystemUserRecord {
            username: "systemd-resolve".to_string(),
            uid: 102,
            gid: 104,
            gecos: "systemd Resolver".to_string(),
            home_dir: "/run/systemd".to_string(),
            shell: "/usr/sbin/nologin".to_string(),
            primary_group: "systemd-resolve".to_string(),
            groups: vec!["systemd-resolve".to_string()],
            is_system_user: true,
            is_locked: false,
            authorized_keys: vec![],
            last_login: None,
        },
        SystemUserRecord {
            username: "nginx".to_string(),
            uid: 103,
            gid: 105,
            gecos: "nginx web server".to_string(),
            home_dir: "/var/www".to_string(),
            shell: "/usr/sbin/nologin".to_string(),
            primary_group: "nginx".to_string(),
            groups: vec!["nginx".to_string(), "www-data".to_string()],
            is_system_user: true,
            is_locked: false,
            authorized_keys: vec![],
            last_login: None,
        },
        SystemUserRecord {
            username: "temp-contractor".to_string(),
            uid: 1003,
            gid: 1003,
            gecos: "Contractor (Expired Project)".to_string(),
            home_dir: "/home/temp-contractor".to_string(),
            shell: "/bin/bash".to_string(),
            primary_group: "temp-contractor".to_string(),
            groups: vec!["temp-contractor".to_string()],
            is_system_user: false,
            is_locked: true,
            authorized_keys: vec![],
            last_login: Some("2026-06-15 17:12:00".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{default_system_users, SystemUserRecord};
    use core::prelude::v1::test;

    #[test]
    fn test_passwd_line_roundtrip() {
        let line = "nelson:x:1000:1000:Nelson Errorware,,,:/home/nelson:/bin/bash";
        let parsed = SystemUserRecord::parse_passwd_line(line).expect("Failed to parse passwd line");
        assert_eq!(parsed.0, "nelson");
        assert_eq!(parsed.1, "x");
        assert_eq!(parsed.2, 1000);
        assert_eq!(parsed.3, 1000);
        assert_eq!(parsed.4, "Nelson Errorware,,,");
        assert_eq!(parsed.5, "/home/nelson");
        assert_eq!(parsed.6, "/bin/bash");
    }

    #[test]
    fn test_user_sudoer_detection() {
        let users = default_system_users();
        let root = users.iter().find(|u| u.username == "root").unwrap();
        let nelson = users.iter().find(|u| u.username == "nelson").unwrap();
        let postgres = users.iter().find(|u| u.username == "postgres").unwrap();

        assert!(root.is_sudoer());
        assert!(nelson.is_sudoer());
        assert!(!postgres.is_sudoer());
    }

    #[test]
    fn test_user_status_and_locking() {
        let users = default_system_users();
        let contractor = users.iter().find(|u| u.username == "temp-contractor").unwrap();
        assert!(contractor.is_locked);
        assert_eq!(contractor.to_passwd_line().chars().nth(16), Some('!'));
    }
}
