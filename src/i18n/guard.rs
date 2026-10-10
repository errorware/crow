//! Keeps English from creeping back into the screens: text written straight
//! into a view (`.child("Delete server")`) instead of a message fails the
//! build's tests. What is meant to stay as it is (a path, a command, a file
//! name) is listed in `untranslated.txt`.

use std::path::Path;

const SINKS: &[&str] = &[".child(", ".placeholder(", ".tooltip(", "Label::new("];

/// The literal text of the strings passed straight to a sink in `src`,
/// before any `#[cfg(test)]`.
fn literals_in_sinks(src: &str) -> Vec<String> {
    let src = src.split("#[cfg(test)]").next().unwrap_or(src);
    let mut out = Vec::new();
    for sink in SINKS {
        let mut rest = src;
        while let Some(at) = rest.find(sink) {
            rest = &rest[at + sink.len()..];
            let mut s = rest.trim_start();
            if let Some(after) = s.strip_prefix("format!(") {
                s = after.trim_start();
            }
            let Some(body) = s.strip_prefix('"') else { continue };
            let mut text = String::new();
            let mut chars = body.chars();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => {
                        // Kept as written, so the allow-list matches the source.
                        text.push('\\');
                        text.extend(chars.next());
                    }
                    '"' => break,
                    c => text.push(c),
                }
            }
            out.push(text);
        }
    }
    out
}

/// Whether a string reads as words: three letters in a row once the format
/// placeholders are taken out.
fn has_words(text: &str) -> bool {
    let mut plain = String::new();
    let mut depth = 0;
    for c in text.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            c if depth == 0 => plain.push(c),
            _ => {}
        }
    }
    let mut run = 0;
    for c in plain.chars() {
        run = if c.is_ascii_alphabetic() { run + 1 } else { 0 };
        if run >= 3 {
            return true;
        }
    }
    false
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn screens_take_their_text_from_messages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let allowed: Vec<(String, String)> = include_str!("untranslated.txt")
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once(" | ").map(|(p, t)| (p.trim().to_string(), t.to_string())))
        .collect();
    let mut files = Vec::new();
    for dir in ["src/views", "src/components", "src/app"] {
        walk(&root.join(dir), &mut files);
    }
    files.sort();
    let mut found = Vec::new();
    for path in files {
        let rel = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();
        let src = std::fs::read_to_string(&path).unwrap();
        for text in literals_in_sinks(&src) {
            if has_words(&text) && !allowed.iter().any(|(p, t)| *p == rel && *t == text) {
                found.push(format!("{rel} | {text}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "English written straight into a screen: move it into assets/i18n/en.ftl and use t!(), \
         or, if it must stay as it is, add it to src/i18n/untranslated.txt:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_guard_sees_what_it_should() {
    let src = r#"div().child("Delete server").child(t!("x")).child(format!("{n} servers")).child("—").placeholder("e.g. web-01")"#;
    let lits = literals_in_sinks(src);
    assert_eq!(lits, ["Delete server", "{n} servers", "—", "e.g. web-01"]);
    assert!(has_words("Delete server") && has_words("{n} servers") && !has_words("—") && !has_words("{count} · {n}"));
}
