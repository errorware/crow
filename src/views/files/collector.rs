use std::collections::HashMap;
use std::path::Path;
use chrono::TimeZone;
use crate::host::{host_for, Host, HostError};
use crate::vault::ServerRecord;
use super::models::FileEntry;

/// `ls -l`'s mode column, setuid/setgid/sticky included (s/S, t/T).
pub fn mode_to_string(mode: u32, is_dir: bool, is_symlink: bool) -> String {
    let file_type = if is_symlink { 'l' } else if is_dir { 'd' } else { '-' };
    let mut s = String::with_capacity(10);
    s.push(file_type);
    // (read, write, exec, special bit, special letter) for user, group, other.
    for (shift, special, letter) in [(6, 0o4000, 's'), (3, 0o2000, 's'), (0, 0o1000, 't')] {
        let bits = (mode >> shift) & 0o7;
        s.push(if bits & 4 != 0 { 'r' } else { '-' });
        s.push(if bits & 2 != 0 { 'w' } else { '-' });
        s.push(match (bits & 1 != 0, mode & special != 0) {
            (true, true) => letter,
            (false, true) => letter.to_ascii_uppercase(),
            (true, false) => 'x',
            (false, false) => '-',
        });
    }
    s
}

/// The most of a file the viewer reads.
pub const PREVIEW_LIMIT: usize = 256 * 1024;

/// The start of a text file for the viewer: up to PREVIEW_LIMIT bytes, and
/// whether there was more. As root (sudo -n) when the login can't read it.
pub fn read_preview(server: &ServerRecord, path: &str) -> Result<(String, bool), String> {
    let host = host_for(server);
    let limit = (PREVIEW_LIMIT + 1).to_string();
    let argv = ["head", "-c", limit.as_str(), "--", path];
    let out = match host.exec(&argv, crate::host::DEFAULT_TIMEOUT) {
        Ok(out) => out,
        Err(HostError::Failed { .. }) => host.exec_privileged(&argv, b"", crate::host::DEFAULT_TIMEOUT).map_err(|e| e.to_string())?,
        Err(e) => return Err(e.to_string()),
    };
    preview_text(out.stdout)
}

/// Text the viewer can show, or why not.
fn preview_text(mut text: String) -> Result<(String, bool), String> {
    let undecodable = text.chars().filter(|c| *c == '\u{FFFD}').count();
    if text.contains('\0') || undecodable * 100 > text.len().max(1) {
        return Err("binary file: nothing to show as text".into());
    }
    let truncated = text.len() > PREVIEW_LIMIT;
    if truncated {
        let mut end = PREVIEW_LIMIT;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    Ok((text, truncated))
}

/// Sets a file's permission bits, as root (sudo -n) when the login can't.
pub fn change_mode(server: &ServerRecord, path: &str, mode: u32) -> Result<(), String> {
    let host = host_for(server);
    let octal = format!("{:04o}", mode & 0o7777);
    let argv = ["chmod", octal.as_str(), "--", path];
    match host.exec(&argv, crate::host::DEFAULT_TIMEOUT) {
        Ok(_) => Ok(()),
        Err(HostError::Failed { .. }) => host.exec_privileged(&argv, b"", crate::host::DEFAULT_TIMEOUT).map(|_| ()).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// What the terminal types to edit `path`: nano (else vim, else vi), through
/// sudoedit when the login can't write the file. None for a path with
/// control characters, which can't be typed safely.
pub fn edit_command(path: &str) -> Option<String> {
    if path.chars().any(char::is_control) {
        return None;
    }
    let f = crate::host::ssh::shell_quote(path);
    Some(format!(
        "f={f}; e=$(command -v nano || command -v vim || command -v vi); if [ -w \"$f\" ] || [ \"$(id -u)\" = 0 ]; then \"$e\" \"$f\"; else SUDO_EDITOR=\"$e\" sudoedit \"$f\"; fi\r"
    ))
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
            mode: e.mode & 0o7777,
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
        assert_eq!(mode_to_string(0o4755, false, false), "-rwsr-xr-x");
        assert_eq!(mode_to_string(0o1777, true, false), "drwxrwxrwt");
        assert_eq!(mode_to_string(0o2640, false, false), "-rw-r-S---");
    }

    #[test]
    fn previews_are_text_and_capped() {
        assert_eq!(preview_text("a\nb\n".into()), Ok(("a\nb\n".into(), false)));
        assert!(preview_text("ELF\0\0".into()).is_err());
        let (text, more) = preview_text("x".repeat(PREVIEW_LIMIT + 1)).unwrap();
        assert!(more);
        assert_eq!(text.len(), PREVIEW_LIMIT);
    }

    #[test]
    fn the_edit_command_quotes_the_path() {
        let cmd = edit_command("/etc/it's here.conf").unwrap();
        assert!(cmd.starts_with("f='/etc/it'\\''s here.conf'; "));
        assert!(cmd.ends_with("fi\r"));
        assert!(cmd.contains("sudoedit"));
        assert_eq!(edit_command("/tmp/a\nb"), None);
    }
}
