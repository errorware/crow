//! sudoers on the Config screen (ERR-22): the files are root-only, so Crow
//! reads them in one root command, and installs them with a guard that puts
//! the old file back if the change would lock the login user out of sudo.

use std::path::PathBuf;

use super::crawler::{DiscoveredConfigFile, SchemaKind};
use super::ConfigFileState;
use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

/// Prints `CROW-FILE <bytes> <path>` then the file, for /etc/sudoers and
/// each file sudo reads from /etc/sudoers.d (it skips names with a `.` or
/// ending in `~`).
const READ_SUDOERS: &str = r#"for f in /etc/sudoers /etc/sudoers.d/*; do [ -f "$f" ] || continue; case "$f" in /etc/sudoers.d/*) case "${f##*/}" in *.*|*~) continue ;; esac ;; esac; printf 'CROW-FILE %s %s\n' "$(wc -c < "$f" | tr -d ' ')" "$f"; cat "$f"; done"#;

/// Installs stdin as "$1", as root. Checked with `visudo -cf` first; after
/// the swap, sudo's whole configuration must still parse and, if "$2" (the
/// login user) could use sudo before, it still must. Otherwise the old file
/// is put back. Temp and backup names contain a `.`, so sudo never reads
/// them from sudoers.d.
const INSTALL_SUDOERS: &str = r#"set -u; f="$1"; u="$2"; export LC_ALL=C
can() { [ -z "$u" ] || sudo -l -U "$u" 2>/dev/null | grep -q "may run the following commands"; }
had=0; can && had=1
t=$(mktemp "$f.crow.XXXXXX") || exit 1
cat > "$t"
if [ -e "$f" ]; then chmod "$(stat -c %a "$f")" "$t"; chown "$(stat -c %u:%g "$f")" "$t"; else chmod 0440 "$t"; chown 0:0 "$t"; fi
if ! out=$(visudo -cf "$t" 2>&1); then rm -f "$t"; echo "visudo rejected it: $(printf '%s' "$out" | sed "s|$t|$f|g")" >&2; exit 3; fi
b=""; if [ -e "$f" ]; then b=$(mktemp "$f.crow-old.XXXXXX"); cp -p "$f" "$b"; fi
mv -f "$t" "$f"
why=""
visudo -c >/dev/null 2>&1 || why="sudo's configuration as a whole no longer parses"
if [ -z "$why" ] && [ "$had" = 1 ] && ! can; then why="$u would lose sudo, and Crow with it"; fi
if [ -n "$why" ]; then if [ -n "$b" ]; then mv -f "$b" "$f"; else rm -f "$f"; fi; echo "not saved, the old file is back: $why" >&2; exit 4; fi
[ -z "$b" ] || rm -f "$b"
"#;

/// Splits the reader's output into (path, content).
fn parse_bundle(stdout: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = stdout;
    while let Some(after) = rest.strip_prefix("CROW-FILE ") {
        let Some((header, body)) = after.split_once('\n') else { break };
        let Some((len, path)) = header.split_once(' ') else { break };
        let Ok(len) = len.parse::<usize>() else { break };
        let Some(content) = body.get(..len) else { break };
        out.push((path.to_string(), content.to_string()));
        rest = &body[len..];
    }
    out
}

fn discovered(path: &str, size: u64) -> DiscoveredConfigFile {
    let full = PathBuf::from(path);
    DiscoveredConfigFile {
        name: full.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path_dir: full.parent().map(|p| p.display().to_string()).unwrap_or_default(),
        full_path: full,
        schema_kind: Some(SchemaKind::Sudoers),
        is_schema_mapped: true,
        size_bytes: size,
        is_readonly: false,
        pill: "CROW UI".to_string(),
        schema_pack: Some(SchemaKind::Sudoers.label()),
    }
}

/// The host's sudoers files, read as root. If root isn't available without
/// a password, /etc/sudoers is listed read-only with the reason.
pub fn load_sudoers(host: &dyn Host) -> Vec<(DiscoveredConfigFile, ConfigFileState)> {
    match host.exec_privileged(&["sh", "-c", READ_SUDOERS, "crow-sudoers"], &[], DEFAULT_TIMEOUT) {
        Ok(out) => parse_bundle(&out.stdout)
            .into_iter()
            .map(|(path, content)| {
                let file = discovered(&path, content.len() as u64);
                let mut state = ConfigFileState::new(file.full_path.clone(), file.name.clone(), content);
                state.read_from_host = true;
                (file, state)
            })
            .collect(),
        Err(e) if host.exists("/etc/sudoers") => {
            let file = discovered("/etc/sudoers", 0);
            let mut state = ConfigFileState::new(file.full_path.clone(), file.name.clone(), String::new());
            state.write_blocked = Some(format!("sudoers is root-only, and Crow couldn't read it as root: {e}"));
            vec![(file, state)]
        }
        Err(_) => Vec::new(),
    }
}

/// Writes a sudoers file through the guard. `login_user` is who Crow logs in
/// as; root (or empty) skips the keep-sudo check.
pub fn install_sudoers(host: &dyn Host, path: &str, content: &str, login_user: &str) -> Result<(), String> {
    let user = if login_user == "root" { "" } else { login_user };
    match host.exec_privileged(&["sh", "-c", INSTALL_SUDOERS, "crow-sudoers-install", path, user], content.as_bytes(), DEFAULT_TIMEOUT) {
        Ok(_) => Ok(()),
        Err(HostError::Failed { stderr, .. }) => Err(stderr.trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundles_split_by_length_even_without_trailing_newlines() {
        let out = "CROW-FILE 19 /etc/sudoers\nroot ALL=(ALL) ALL\nCROW-FILE 10 /etc/sudoers.d/ci\nci ALL=ALLCROW-FILE 0 /etc/sudoers.d/empty\n";
        let files = parse_bundle(out);
        assert_eq!(files, vec![
            ("/etc/sudoers".to_string(), "root ALL=(ALL) ALL\n".to_string()),
            ("/etc/sudoers.d/ci".to_string(), "ci ALL=ALL".to_string()),
            ("/etc/sudoers.d/empty".to_string(), String::new()),
        ]);
    }

    #[test]
    fn listed_by_file_name_in_its_directory() {
        let f = discovered("/etc/sudoers.d/90-cloud-init-users", 10);
        assert_eq!((f.name.as_str(), f.path_dir.as_str()), ("90-cloud-init-users", "/etc/sudoers.d"));
        assert_eq!(f.schema_kind, Some(SchemaKind::Sudoers));
    }
}
