use std::collections::HashMap;
use std::path::Path;
use chrono::TimeZone;
use crate::host::{host_for, Host, HostError};
use crate::vault::ServerRecord;
use super::models::FileEntry;

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

/// Parses a `name:...:id:...`-shaped file (/etc/passwd or /etc/group) into an
/// id -> name lookup. Malformed lines are skipped.
fn parse_id_map(content: &str) -> HashMap<u32, String> {
    content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            Some((parts.get(2)?.parse::<u32>().ok()?, parts.first()?.to_string()))
        })
        .collect()
}

/// Lists a directory on the server.
pub fn list_directory_for_server(server: &ServerRecord, path: &str) -> Result<Vec<FileEntry>, String> {
    list_directory(host_for(server).as_ref(), path).map_err(|e| e.to_string())
}

/// Lists `path` on `host`, directories first, resolving owners through the
/// host's own /etc/passwd and /etc/group (best-effort: numeric ids otherwise).
pub fn list_directory(host: &dyn Host, path: &str) -> Result<Vec<FileEntry>, HostError> {
    let entries = host.list_dir(path)?;
    let users = host.read_file("/etc/passwd").map(|c| parse_id_map(&c)).unwrap_or_default();
    let groups = host.read_file("/etc/group").map(|c| parse_id_map(&c)).unwrap_or_default();

    let mut out: Vec<FileEntry> = entries
        .into_iter()
        .map(|e| FileEntry {
            mode_str: mode_to_string(e.mode, e.is_dir, e.is_symlink),
            owner: users.get(&e.uid).cloned().unwrap_or_else(|| e.uid.to_string()),
            group: groups.get(&e.gid).cloned().unwrap_or_else(|| e.gid.to_string()),
            modified: chrono::Local
                .timestamp_opt(e.mtime, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default(),
            name: e.name,
            is_dir: e.is_dir,
            is_symlink: e.is_symlink,
            size_bytes: e.size_bytes,
        })
        .collect();

    out.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(out)
}

pub fn create_directory(server: &ServerRecord, parent: &str, name: &str) -> Result<(), String> {
    if name.trim().is_empty() || name.contains('/') {
        return Err("Invalid folder name".to_string());
    }
    let host = host_for(server);
    host.create_dir(&join_path(parent, name)).map_err(|e| e.to_string())
}

/// Deletes a single file, or an EMPTY directory only — deliberately refuses a
/// non-empty directory rather than recursing, mirroring how a real FTP
/// server's RMD behaves. No recursive delete exists in this tool.
pub fn delete_entry(server: &ServerRecord, parent: &str, name: &str, is_dir: bool) -> Result<(), String> {
    let host = host_for(server);
    host.remove(&join_path(parent, name), is_dir).map_err(|e| e.to_string())
}

fn join_path(parent: &str, name: &str) -> String {
    Path::new(parent).join(name).to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_map_skips_malformed_lines() {
        let map = parse_id_map("root:x:0:0:root:/root:/bin/bash\nbroken\nnhc:x:1000:1000::/home/nhc:/bin/zsh\n");
        assert_eq!(map.get(&0).map(String::as_str), Some("root"));
        assert_eq!(map.get(&1000).map(String::as_str), Some("nhc"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn mode_strings() {
        assert_eq!(mode_to_string(0o755, true, false), "drwxr-xr-x");
        assert_eq!(mode_to_string(0o640, false, false), "-rw-r-----");
        assert_eq!(mode_to_string(0o777, false, true), "lrwxrwxrwx");
    }
}
