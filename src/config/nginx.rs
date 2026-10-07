//! nginx on the Config screen (ERR-22): nginx -t can only test the
//! configuration that's installed, so a change is installed, tested, and
//! put back if the test fails.

use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

/// Installs stdin as "$1" as root, then runs nginx -t. The change is kept
/// if the test passes, or if it fails exactly as it already did before the
/// change (so a broken configuration can still be repaired step by step);
/// otherwise the old file is put back (ERR-113). Errors are compared on
/// nginx's own "[emerg]" lines, without timestamps. Temp names end in
/// .crow-*, which nginx's usual include patterns (*.conf, sites-enabled/*
/// symlinks) don't pick up.
pub const INSTALL_NGINX: &str = r#"set -u; f="$1"; export PATH="$PATH:/usr/sbin:/sbin"
command -v nginx >/dev/null 2>&1 || { echo "nginx isn't installed here; nothing to test the change with" >&2; exit 3; }
emerg() { printf '%s\n' "$1" | grep '^nginx: \[emerg\]'; }
before=$(nginx -t 2>&1) && before_ok=1 || before_ok=0
t=$(mktemp "$f.crow-new.XXXXXX") || exit 1
cat > "$t"
if [ -e "$f" ]; then chmod "$(stat -c %a "$f")" "$t"; chown "$(stat -c %u:%g "$f")" "$t" 2>/dev/null || true; fi
b=""; if [ -e "$f" ]; then b=$(mktemp "$f.crow-old.XXXXXX"); cp -p "$f" "$b"; fi
mv -f "$t" "$f"
if ! out=$(nginx -t 2>&1) && { [ "$before_ok" = 1 ] || [ "$(emerg "$out")" != "$(emerg "$before")" ]; }; then
  if [ -n "$b" ]; then mv -f "$b" "$f"; else rm -f "$f"; fi
  why="nginx -t failed"; [ "$before_ok" = 1 ] || why="nginx -t was already failing, and this change made it fail differently"
  echo "not saved, the old file is back: $why: $(emerg "$out" | head -3)" >&2; exit 4
fi
[ -z "$b" ] || rm -f "$b"
"#;

/// Writes an nginx file through the test-and-roll-back guard.
pub fn install_nginx(host: &dyn Host, path: &str, content: &str) -> Result<(), String> {
    match host.exec_privileged(&["sh", "-c", INSTALL_NGINX, "crow-nginx-install", path], content.as_bytes(), DEFAULT_TIMEOUT) {
        Ok(_) => Ok(()),
        Err(HostError::Failed { stderr, .. }) => Err(stderr.trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}
