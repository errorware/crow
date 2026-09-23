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

#[cfg(test)]
mod tests {
    use super::SystemUserRecord;
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

}
