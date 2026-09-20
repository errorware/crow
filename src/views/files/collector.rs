use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use chrono::TimeZone;
use crate::vault::ServerRecord;
use super::models::FileEntry;

pub fn is_localhost_server(server: &ServerRecord) -> bool {
    server.host == "127.0.0.1"
        || server.host == "localhost"
        || server.host == "::1"
        || server.name.to_lowercase() == "localhost"
        || server.tags.iter().any(|t| t == "localhost" || t == "local")
}

fn mode_to_string(mode: u32, is_dir: bool, is_symlink: bool) -> String {
    let file_type = if is_symlink { 'l' } else if is_dir { 'd' } else { '-' };
    let bits: [(u32, char); 9] = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    let mut s = String::with_capacity(10);
    s.push(file_type);
    for (bit, ch) in bits {
        s.push(if mode & bit != 0 { ch } else { '-' });
    }
    s
}

/// Loads a `name:...:id:...`-shaped file (/etc/passwd or /etc/group) into an
/// id -> name lookup. Best-effort: an unreadable or malformed file just
/// yields numeric ids in the listing instead of names.
fn load_id_map(path: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(id) = parts[2].parse::<u32>() {
                    map.insert(id, parts[0].to_string());
                }
            }
        }
    }
    map
}

/// Lists a directory. Returns the entries and whether the listing is
/// simulated — this is Crow's own machine, there is no remote transport yet
/// (see CROW.md's crow-ssh gap), so anything not localhost gets a clearly
/// labeled simulated listing rather than a silent guess.
pub fn list_directory_for_server(server: &ServerRecord, path: &str) -> (Vec<FileEntry>, bool) {
    if is_localhost_server(server) {
        if let Some(entries) = list_local_directory(path) {
            return (entries, false);
        }
    }
    (simulated_directory(path), true)
}

fn list_local_directory(path: &str) -> Option<Vec<FileEntry>> {
    let dir = fs::read_dir(path).ok()?;
    let users = load_id_map("/etc/passwd");
    let groups = load_id_map("/etc/group");

    let mut out = Vec::new();
    for entry in dir.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_symlink = file_type.is_symlink();
        let is_dir = file_type.is_dir();
        let modified = meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .and_then(|d| chrono::Local.timestamp_opt(d.as_secs() as i64, 0).single())
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();

        out.push(FileEntry {
            name: file_name,
            is_dir,
            is_symlink,
            size_bytes: meta.size(),
            mode_str: mode_to_string(meta.mode(), is_dir, is_symlink),
            owner: users.get(&meta.uid()).cloned().unwrap_or_else(|| meta.uid().to_string()),
            group: groups.get(&meta.gid()).cloned().unwrap_or_else(|| meta.gid().to_string()),
            modified,
        });
    }

    out.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Some(out)
}

fn seed(name: &str, is_dir: bool, size: u64, mode: &str, owner: &str) -> FileEntry {
    FileEntry {
        name: name.to_string(),
        is_dir,
        is_symlink: false,
        size_bytes: size,
        mode_str: mode.to_string(),
        owner: owner.to_string(),
        group: owner.to_string(),
        modified: "2026-09-01 00:00".to_string(),
    }
}

/// A plausible-but-fake root-filesystem shape for hosts Crow can't actually
/// reach yet — never presented as anything other than simulated by the UI.
fn simulated_directory(path: &str) -> Vec<FileEntry> {
    match path {
        "/" => vec![
            seed("bin", true, 0, "drwxr-xr-x", "root"),
            seed("boot", true, 0, "drwxr-xr-x", "root"),
            seed("etc", true, 0, "drwxr-xr-x", "root"),
            seed("home", true, 0, "drwxr-xr-x", "root"),
            seed("lib", true, 0, "drwxr-xr-x", "root"),
            seed("root", true, 0, "drwx------", "root"),
            seed("srv", true, 0, "drwxr-xr-x", "root"),
            seed("tmp", true, 0, "drwxrwxrwt", "root"),
            seed("usr", true, 0, "drwxr-xr-x", "root"),
            seed("var", true, 0, "drwxr-xr-x", "root"),
        ],
        "/var" => vec![
            seed("log", true, 0, "drwxr-xr-x", "root"),
            seed("lib", true, 0, "drwxr-xr-x", "root"),
            seed("www", true, 0, "drwxr-xr-x", "www-data"),
        ],
        "/etc" => vec![
            seed("ssh", true, 0, "drwxr-xr-x", "root"),
            seed("systemd", true, 0, "drwxr-xr-x", "root"),
            seed("hosts", false, 340, "-rw-r--r--", "root"),
            seed("hostname", false, 12, "-rw-r--r--", "root"),
        ],
        "/home" => vec![
            seed("operator", true, 0, "drwxr-xr-x", "operator"),
        ],
        _ => vec![
            seed(".keep", false, 0, "-rw-r--r--", "root"),
        ],
    }
}

pub fn create_directory(server: &ServerRecord, parent: &str, name: &str) -> Result<(), String> {
    if !is_localhost_server(server) {
        return Err("Not supported on a simulated/remote host yet".to_string());
    }
    if name.trim().is_empty() || name.contains('/') {
        return Err("Invalid folder name".to_string());
    }
    fs::create_dir(Path::new(parent).join(name)).map_err(|e| e.to_string())
}

/// Deletes a single file, or an EMPTY directory only — deliberately refuses a
/// non-empty directory rather than recursing, mirroring how a real FTP
/// server's RMD behaves. No recursive delete exists in this tool.
pub fn delete_entry(server: &ServerRecord, parent: &str, name: &str, is_dir: bool) -> Result<(), String> {
    if !is_localhost_server(server) {
        return Err("Not supported on a simulated/remote host yet".to_string());
    }
    let full = Path::new(parent).join(name);
    if is_dir {
        fs::remove_dir(&full).map_err(|e| format!("{} (directory must be empty)", e))
    } else {
        fs::remove_file(&full).map_err(|e| e.to_string())
    }
}
