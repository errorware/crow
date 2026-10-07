//! What a server's kernel runs now, for the keys in a sysctl file (ERR-22).
//! sysctl files only take effect at boot or on `sysctl --system`, so the
//! editor shows where the file and the running kernel disagree.

use std::collections::HashMap;

use crate::host::{Host, DEFAULT_TIMEOUT};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveValue {
    /// The running value, whitespace as the kernel prints it.
    Value(String),
    /// This kernel has no such parameter.
    Missing,
    /// The parameter exists but can't be read (write-only, or root-only).
    Unreadable,
}

/// Prints `key<TAB>=value`, `key<TAB>!` (no such key) or `key<TAB>?`
/// (unreadable) per key. Keys are the script's arguments, never code. As
/// sysctl reads them: a key whose first separator is `/` is a path as is;
/// otherwise a `.` is a `/` in /proc/sys and a `/` is a `.`.
const READ_LIVE: &str = r#"for k; do s=${k%%[./]*}; case "${k#"$s"}" in /*) p="$k" ;; *) p=$(printf '%s' "$k" | tr ./ /.) ;; esac; f="/proc/sys/$p"; if [ ! -e "$f" ]; then printf '%s\t!\n' "$k"; elif v=$(tr '\n\t' '  ' < "$f" 2>/dev/null); then printf '%s\t=%s\n' "$k" "$v"; else printf '%s\t?\n' "$k"; fi; done"#;

/// Reads the running value of each key. Keys with a glob are skipped.
pub fn read_live(host: &dyn Host, keys: &[String]) -> HashMap<String, LiveValue> {
    let keys: Vec<&str> = keys.iter().map(String::as_str).filter(|k| !k.contains('*')).collect();
    if keys.is_empty() {
        return HashMap::new();
    }
    let mut argv = vec!["sh", "-c", READ_LIVE, "crow-sysctl"];
    argv.extend(keys.iter().copied());
    match host.exec(&argv, DEFAULT_TIMEOUT) {
        Ok(out) => parse_live(&out.stdout),
        // Couldn't ask: record every key as unreadable, so it isn't retried
        // on every frame.
        Err(_) => keys.iter().map(|k| (k.to_string(), LiveValue::Unreadable)).collect(),
    }
}

fn parse_live(stdout: &str) -> HashMap<String, LiveValue> {
    stdout
        .lines()
        .filter_map(|l| {
            let (key, rest) = l.split_once('\t')?;
            let live = match rest {
                "!" => LiveValue::Missing,
                "?" => LiveValue::Unreadable,
                v => LiveValue::Value(v.strip_prefix('=')?.trim().to_string()),
            };
            Some((key.to_string(), live))
        })
        .collect()
}

/// Whether a value written in a file is what the kernel runs: compared
/// word by word, since files and /proc separate numbers with any whitespace.
pub fn same_value(file: &str, live: &str) -> bool {
    file.split_whitespace().eq(live.split_whitespace())
}

/// What to say under a sysctl row, if anything: a later line that overrides
/// it, a value the kernel isn't running, or a key the kernel doesn't have.
/// `later_line` is where the same key is set again further down the file.
pub fn row_note(value: &str, live: Option<&LiveValue>, later_line: Option<usize>) -> Option<(String, bool)> {
    if let Some(line) = later_line {
        return Some((format!("set again on line {line}; that value wins"), false));
    }
    match live? {
        LiveValue::Value(v) if !same_value(value, v) => Some((format!("running now: {v} · this value applies at boot or with sysctl --system"), true)),
        LiveValue::Missing => Some(("this kernel has no such parameter".to_string(), false)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_scripts_output() {
        let live = parse_live("net.ipv4.ip_forward\t=1\nno.such\t!\nvm.drop_caches\t?\nnet.ipv4.ip_local_port_range\t=32768 60999 \n");
        assert_eq!(live["net.ipv4.ip_forward"], LiveValue::Value("1".into()));
        assert_eq!(live["no.such"], LiveValue::Missing);
        assert_eq!(live["vm.drop_caches"], LiveValue::Unreadable);
        assert_eq!(live["net.ipv4.ip_local_port_range"], LiveValue::Value("32768 60999".into()));
    }

    #[test]
    fn notes_say_what_matters() {
        let v = |s: &str| LiveValue::Value(s.into());
        assert_eq!(row_note("32768\t60999", Some(&v("32768 60999")), None), None, "same numbers, different spacing");
        assert!(row_note("1", Some(&v("0")), None).is_some_and(|(n, warn)| warn && n.contains("running now: 0")));
        assert!(row_note("1", Some(&LiveValue::Missing), None).is_some_and(|(_, warn)| !warn));
        assert_eq!(row_note("1", Some(&LiveValue::Unreadable), None), None);
        assert!(row_note("1", Some(&v("0")), Some(12)).is_some_and(|(n, _)| n.contains("line 12")), "an overridden line isn't compared");
        assert_eq!(row_note("1", None, None), None, "not read yet");
    }
}
