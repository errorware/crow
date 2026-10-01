pub mod deploy;

use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey, PublicKey};

pub use crate::vault::{SshKeyGroup, SshKeyRecord, SshScanPath};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAlgorithm {
    Ed25519,
    Rsa4096,
}

impl KeyAlgorithm {
    pub fn display_name(&self) -> &'static str {
        match self {
            KeyAlgorithm::Ed25519 => "Ed25519 (Recommended)",
            KeyAlgorithm::Rsa4096 => "RSA 4096-bit",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            KeyAlgorithm::Ed25519 => "ED25519",
            KeyAlgorithm::Rsa4096 => "RSA-4096",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredKey {
    pub file_path: String,
    pub public_key: String,
    pub fingerprint: String,
    pub algorithm: String,
    pub comment: Option<String>,
    pub has_private_key: bool,
    pub is_enrolled: bool,
    pub enrolled_id: Option<String>,
    pub suggested_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyGenFieldFocus {
    Name,
    Comment,
    Directory,
}

#[derive(Clone, Debug)]
pub struct KeyGenModalState {
    pub name_input: String,
    pub algo: KeyAlgorithm,
    pub group_id: String,
    pub comment_input: String,
    pub custom_dir_input: String,
    pub active_focus: KeyGenFieldFocus,
    pub error_message: Option<String>,
    pub generated_public_key: Option<String>,
    pub generated_priv_path: Option<String>,
    pub generated_fingerprint: Option<String>,
}

impl Default for KeyGenModalState {
    fn default() -> Self {
        let default_dir = dirs::home_dir()
            .map(|h| h.join(".ssh").display().to_string())
            .unwrap_or_else(|| "~/.ssh".to_string());
        Self {
            name_input: String::new(),
            algo: KeyAlgorithm::Ed25519,
            group_id: "fleet".to_string(),
            comment_input: String::new(),
            custom_dir_input: default_dir,
            active_focus: KeyGenFieldFocus::Name,
            error_message: None,
            generated_public_key: None,
            generated_priv_path: None,
            generated_fingerprint: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NewGroupModalState {
    pub name_input: String,
    pub color_input: String,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct AddScanPathModalState {
    pub path_input: String,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EditKeyModalState {
    pub key_id: String,
    pub name_input: String,
    pub group_id: String,
    pub attached_servers: Vec<String>,
    pub error_message: Option<String>,
}

/// Expands a leading '~' into the user's home directory.
pub fn expand_tilde(path_str: &str) -> PathBuf {
    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path_str)
}

/// Contracts an absolute path starting with the user's home directory to '~/' for display.
pub fn contract_tilde(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~/{}", rel.display());
        }
    }
    path.display().to_string()
}

#[cfg(unix)]
pub fn set_private_key_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
pub fn set_private_key_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn format_algorithm_label(algo: &Algorithm) -> String {
    match algo {
        Algorithm::Ed25519 => "Ed25519".to_string(),
        Algorithm::Rsa { .. } => "RSA".to_string(),
        Algorithm::Ecdsa { curve } => format!("ECDSA-{:?}", curve),
        Algorithm::Dsa => "DSA (Legacy)".to_string(),
        Algorithm::SkEd25519 => "Ed25519 (FIDO/U2F)".to_string(),
        other => other.to_string(),
    }
}

/// Scans a given directory for SSH public and private keys.
pub fn scan_directory(dir_path: &Path, enrolled_keys: &[SshKeyRecord]) -> Vec<DiscoveredKey> {
    let mut results: Vec<DiscoveredKey> = Vec::new();
    let mut seen_fps: std::collections::HashSet<String> = std::collections::HashSet::new();

    let read_dir = match std::fs::read_dir(dir_path) {
        Ok(rd) => rd,
        Err(_) => return results,
    };

    let entries: Vec<PathBuf> = read_dir
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();

    // 1. Process all .pub files first
    for path in entries.iter().filter(|p| p.extension().map_or(false, |ext| ext == "pub")) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let line = content.lines().find(|l| !l.trim().is_empty() && !l.starts_with('#')).unwrap_or("").trim();
            if let Ok(pub_key) = PublicKey::from_openssh(line) {
                let fingerprint = pub_key.fingerprint(HashAlg::Sha256).to_string();
                if seen_fps.contains(&fingerprint) {
                    continue;
                }
                seen_fps.insert(fingerprint.clone());

                let algo_str = format_algorithm_label(&pub_key.algorithm());
                let comment = if pub_key.comment().is_empty() {
                    None
                } else {
                    Some(pub_key.comment().to_string())
                };

                let priv_path = path.with_extension("");
                let has_private_key = priv_path.exists();

                let enrolled = enrolled_keys.iter().find(|k| k.fingerprint == fingerprint);
                let is_enrolled = enrolled.is_some();
                let enrolled_id = enrolled.map(|k| k.id.clone());

                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("key");
                let suggested_name = if let Some(ref c) = comment {
                    if c.contains('@') {
                        c.clone()
                    } else {
                        file_stem.to_string()
                    }
                } else {
                    file_stem.to_string()
                };

                results.push(DiscoveredKey {
                    file_path: priv_path.display().to_string(),
                    public_key: line.to_string(),
                    fingerprint,
                    algorithm: algo_str,
                    comment,
                    has_private_key,
                    is_enrolled,
                    enrolled_id,
                    suggested_name,
                });
            }
        }
    }

    // 2. Process private key files without .pub counterpart
    for path in entries.iter().filter(|p| p.extension().map_or(true, |ext| ext != "pub")) {
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if file_name.starts_with("id_") || file_name.ends_with(".pem") || file_name.ends_with(".key") {
            if let Ok(priv_key) = PrivateKey::read_openssh_file(path) {
                let pub_key = priv_key.public_key();
                let fingerprint = pub_key.fingerprint(HashAlg::Sha256).to_string();
                if seen_fps.contains(&fingerprint) {
                    continue;
                }
                seen_fps.insert(fingerprint.clone());

                let algo_str = format_algorithm_label(&pub_key.algorithm());
                let comment = if priv_key.comment().is_empty() {
                    None
                } else {
                    Some(priv_key.comment().to_string())
                };

                let pub_key_str = pub_key.to_openssh().unwrap_or_default();
                let enrolled = enrolled_keys.iter().find(|k| k.fingerprint == fingerprint);
                let is_enrolled = enrolled.is_some();
                let enrolled_id = enrolled.map(|k| k.id.clone());

                let suggested_name = file_name.to_string();

                results.push(DiscoveredKey {
                    file_path: path.display().to_string(),
                    public_key: pub_key_str,
                    fingerprint,
                    algorithm: algo_str,
                    comment,
                    has_private_key: true,
                    is_enrolled,
                    enrolled_id,
                    suggested_name,
                });
            }
        }
    }

    // Sort: unimported first, then by name
    results.sort_by(|a, b| {
        match (a.is_enrolled, b.is_enrolled) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.suggested_name.cmp(&b.suggested_name),
        }
    });

    results
}

/// Generates a new SSH keypair and saves it to disk with strict permissions.
pub fn generate_keypair(
    name: &str,
    algo: KeyAlgorithm,
    comment: Option<&str>,
    group_id: &str,
    target_dir: &Path,
    filename_override: Option<&str>,
) -> Result<(SshKeyRecord, String, PathBuf, PathBuf), String> {
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create destination directory '{}': {}", target_dir.display(), e))?;
    }

    let mut rng = ssh_key::rand_core::OsRng;
    let mut priv_key = match algo {
        KeyAlgorithm::Ed25519 => PrivateKey::random(&mut rng, Algorithm::Ed25519)
            .map_err(|e| format!("Failed to generate Ed25519 key: {}", e))?,
        KeyAlgorithm::Rsa4096 => {
            let rsa_pair = ssh_key::private::RsaKeypair::random(&mut rng, 4096)
                .map_err(|e| format!("Failed to generate RSA 4096 key: {}", e))?;
            PrivateKey::from(rsa_pair)
        }
    };

    let comment_str = comment.unwrap_or("").trim();
    if !comment_str.is_empty() {
        priv_key.set_comment(comment_str);
    }

    let pub_key = priv_key.public_key();
    let fingerprint = pub_key.fingerprint(HashAlg::Sha256).to_string();
    let pub_key_openssh = pub_key.to_openssh()
        .map_err(|e| format!("Failed to format public key: {}", e))?;
    let priv_key_openssh = priv_key.to_openssh(LineEnding::LF)
        .map_err(|e| format!("Failed to format private key: {}", e))?;

    let base_name = if let Some(custom) = filename_override.filter(|s| !s.trim().is_empty()) {
        custom.trim().to_string()
    } else {
        let slug = name.to_lowercase().replace(' ', "-").replace('_', "-");
        match algo {
            KeyAlgorithm::Ed25519 => format!("id_ed25519_{}", slug),
            KeyAlgorithm::Rsa4096 => format!("id_rsa_{}", slug),
        }
    };

    let priv_path = target_dir.join(&base_name);
    let pub_path = target_dir.join(format!("{}.pub", base_name));

    // Ensure we don't accidentally overwrite existing keys
    if priv_path.exists() {
        return Err(format!("Private key file already exists at '{}'", priv_path.display()));
    }
    if pub_path.exists() {
        return Err(format!("Public key file already exists at '{}'", pub_path.display()));
    }

    // Write private key file
    std::fs::write(&priv_path, priv_key_openssh.as_bytes())
        .map_err(|e| format!("Failed to write private key to '{}': {}", priv_path.display(), e))?;
    set_private_key_permissions(&priv_path)
        .map_err(|e| format!("Failed to set permissions (0600) on private key: {}", e))?;

    // Write public key file
    std::fs::write(&pub_path, pub_key_openssh.as_bytes())
        .map_err(|e| format!("Failed to write public key to '{}': {}", pub_path.display(), e))?;

    let record = SshKeyRecord {
        id: format!("key-{}", &fingerprint.replace("SHA256:", "").chars().take(12).collect::<String>()),
        name: name.to_string(),
        group_id: group_id.to_string(),
        public_key: pub_key_openssh.trim().to_string(),
        fingerprint,
        algorithm: match algo {
            KeyAlgorithm::Ed25519 => "Ed25519".to_string(),
            KeyAlgorithm::Rsa4096 => "RSA 4096".to_string(),
        },
        comment: if comment_str.is_empty() { None } else { Some(comment_str.to_string()) },
        private_key_path: Some(contract_tilde(&priv_path)),
        attached_servers: Vec::new(),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };

    Ok((record, pub_key_openssh.trim().to_string(), priv_path, pub_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_generation_and_fingerprint() {
        let temp_dir = std::env::temp_dir().join("crow_test_ed25519");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let (record, pub_str, priv_path, pub_path) = generate_keypair(
            "Test Key",
            KeyAlgorithm::Ed25519,
            Some("test@crow"),
            "fleet",
            &temp_dir,
            Some("id_test_ed25519"),
        ).unwrap();

        assert!(priv_path.exists());
        assert!(pub_path.exists());
        assert!(pub_str.starts_with("ssh-ed25519 "));
        assert!(record.fingerprint.starts_with("SHA256:"));
        assert_eq!(record.name, "Test Key");
        assert_eq!(record.group_id, "fleet");

        // Scan the directory and verify it finds the generated key
        let discovered = scan_directory(&temp_dir, &[record]);
        assert_eq!(discovered.len(), 1);
        assert!(discovered[0].is_enrolled);
        assert!(discovered[0].has_private_key);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tilde_helpers() {
        let expanded = expand_tilde("~/.ssh");
        assert!(expanded.is_absolute());
        let contracted = contract_tilde(&expanded);
        assert_eq!(contracted, "~/.ssh");
    }
}
