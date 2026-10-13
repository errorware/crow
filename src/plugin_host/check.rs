//! The rules a plugin folder must pass before Crow shows it for review, loads
//! it, or runs it (ERR-176), and `crow --check-plugin DIR`, which prints them
//! for the person deciding whether to trust a plugin.
//!
//! Refusals stop the plugin; warnings are shown on the review screen.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::manifest::{self, Action, Effect, Manifest, Runs, Template, Var};

pub const MANIFEST: &str = "plugin.toml";
pub const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
pub const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_FILES: usize = 500;
pub const MAX_DEPTH: usize = 8;

/// One file of a plugin, by its path inside the folder ("/"-separated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginFile {
    pub rel: String,
    pub len: u64,
    /// Not UTF-8 text: can't be read on the review screen.
    pub binary: bool,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub dir: PathBuf,
    pub manifest: Option<Manifest>,
    pub files: Vec<PluginFile>,
    pub refusals: Vec<String>,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.refusals.is_empty() && self.manifest.is_some()
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.len).sum()
    }
}

/// Checks the plugin in `dir`. Reads files; runs nothing.
pub fn check_dir(dir: &Path) -> Report {
    let mut r = Report { dir: dir.to_path_buf(), manifest: None, files: Vec::new(), refusals: Vec::new(), warnings: Vec::new() };
    match std::fs::symlink_metadata(dir) {
        Ok(m) if m.is_dir() => {}
        Ok(_) => {
            r.refusals.push(format!("{}: not a folder", dir.display()));
            return r;
        }
        Err(e) => {
            r.refusals.push(format!("{}: {e}", dir.display()));
            return r;
        }
    }
    walk(dir, "", 0, &mut r);
    if r.files.len() > MAX_FILES {
        r.refusals.push(format!("{} files: at most {MAX_FILES}", r.files.len()));
    }
    if r.total_bytes() > MAX_TOTAL_BYTES {
        r.refusals.push(format!("{} in all: at most {}", human_bytes(r.total_bytes()), human_bytes(MAX_TOTAL_BYTES)));
    }

    let Some(mf) = r.files.iter().find(|f| f.rel == MANIFEST).cloned() else {
        r.refusals.push(format!("no {MANIFEST} in the folder"));
        return r;
    };
    if mf.len > MAX_MANIFEST_BYTES {
        r.refusals.push(format!("{MANIFEST}: {} (at most {})", human_bytes(mf.len), human_bytes(MAX_MANIFEST_BYTES)));
        return r;
    }
    let text = match std::fs::read_to_string(dir.join(MANIFEST)) {
        Ok(t) => t,
        Err(e) => {
            r.refusals.push(format!("{MANIFEST}: {e}"));
            return r;
        }
    };
    let m = match manifest::parse(&text) {
        Ok(m) => m,
        Err(e) => {
            r.refusals.extend(e.0.into_iter().map(|p| format!("{MANIFEST}: {p}")));
            return r;
        }
    };

    let folder = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    if folder != m.plugin.id {
        r.refusals.push(format!("{MANIFEST}: plugin.id \"{}\" must match the folder name \"{folder}\"", m.plugin.id));
    }
    let has_file = |rel: &str| r.files.iter().any(|f| f.rel == rel && !f.binary);
    let mut missing = Vec::new();
    if let Some(s) = m.setup.as_ref().and_then(|s| s.script.as_deref()) {
        if !has_file(s) {
            missing.push(format!("{MANIFEST}: setup: script \"{s}\" isn't a text file in the folder"));
        }
    }
    for a in &m.actions {
        if let Some(s) = a.script.as_deref() {
            if !has_file(s) {
                missing.push(format!("{MANIFEST}: action {}: script \"{s}\" isn't a text file in the folder", a.id));
            }
        }
    }
    r.refusals.extend(missing);

    let commands = m.setup.iter().map(|s| ("setup".to_string(), s.command.as_deref(), &s.env)).chain(m.actions.iter().map(|a| (format!("action {}", a.id), a.command.as_deref(), &a.env)));
    for (at, command, env) in commands {
        if let Some(argv) = command {
            if let Some(why) = shell_from_placeholder(argv) {
                r.refusals.push(format!("{MANIFEST}: {at}: {why}"));
            }
        }
        for (name, value) in env {
            if RISKY_ENV.contains(&name.as_str()) && value.vars().next().is_some() {
                r.refusals.push(format!("{MANIFEST}: {at}: env {name} can't take a placeholder: it changes what runs"));
            }
        }
    }
    for a in &m.actions {
        if a.effect == Effect::Read {
            if let Some(why) = a.command.as_deref().and_then(changes_something) {
                r.refusals.push(format!("{MANIFEST}: action {}: declared read, but {why}: make it effect = \"change\"", a.id));
            }
            if let Some(s) = a.script.as_deref() {
                if let Ok(body) = std::fs::read_to_string(dir.join(s)) {
                    if let Some(word) = script_changes_something(&body) {
                        r.warnings.push(format!("action {}: declared read, but {s} runs `{word}`", a.id));
                    }
                }
            }
        }
    }
    for f in r.files.iter().filter(|f| f.binary) {
        r.warnings.push(format!("{}: binary, so it can't be reviewed here", f.rel));
    }
    r.manifest = Some(m);
    r
}

fn walk(dir: &Path, prefix: &str, depth: usize, r: &mut Report) {
    if depth > MAX_DEPTH {
        r.refusals.push(format!("{prefix}: folders nested more than {MAX_DEPTH} deep"));
        return;
    }
    let mut entries = match std::fs::read_dir(dir) {
        Ok(e) => e.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(e) => {
            r.refusals.push(format!("{}: {e}", if prefix.is_empty() { "." } else { prefix }));
            return;
        }
    };
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        if r.files.len() > MAX_FILES {
            return;
        }
        let Some(name) = e.file_name().to_str().map(str::to_string) else {
            r.refusals.push(format!("{prefix}{}: the name isn't UTF-8", e.file_name().to_string_lossy()));
            continue;
        };
        let rel = format!("{prefix}{name}");
        if let Some(c) = name.chars().find(|c| c.is_control() || is_bidi(*c)) {
            r.refusals.push(format!("{}: the name holds U+{:04X}", rel.escape_debug(), c as u32));
            continue;
        }
        let Ok(meta) = std::fs::symlink_metadata(e.path()) else { continue };
        let kind = meta.file_type();
        if kind.is_symlink() {
            r.refusals.push(format!("{rel}: a symlink; a plugin's files must be inside its folder"));
        } else if kind.is_dir() {
            walk(&e.path(), &format!("{rel}/"), depth + 1, r);
        } else if !kind.is_file() {
            r.refusals.push(format!("{rel}: not a regular file"));
        } else {
            let len = meta.len();
            if len > MAX_FILE_BYTES {
                r.refusals.push(format!("{rel}: {} (at most {})", human_bytes(len), human_bytes(MAX_FILE_BYTES)));
                r.files.push(PluginFile { rel, len, binary: true });
                continue;
            }
            let bytes = std::fs::read(e.path()).unwrap_or_default();
            let binary = match std::str::from_utf8(&bytes) {
                Ok(text) => {
                    if let Some((line, c)) = bad_char(text) {
                        r.refusals.push(format!("{rel}: line {line} holds U+{:04X} ({})", c as u32, if is_bidi(c) { "a direction override, which can make text read differently from what runs" } else { "a control character" }));
                    }
                    false
                }
                Err(_) => true,
            };
            r.files.push(PluginFile { rel, len, binary });
        }
    }
}

/// Characters that reorder text, so what's reviewed isn't what runs.
fn is_bidi(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{061C}')
}

fn bad_char(text: &str) -> Option<(usize, char)> {
    text.lines().enumerate().find_map(|(i, l)| l.chars().find(|&c| is_bidi(c) || (c.is_control() && c != '\t' && c != '\r')).map(|c| (i + 1, c)))
}

/// Environment variables that change which code runs.
const RISKY_ENV: &[&str] = &["PATH", "LD_PRELOAD", "LD_LIBRARY_PATH", "BASH_ENV", "ENV", "IFS", "PS4", "SHELLOPTS", "PROMPT_COMMAND", "PYTHONSTARTUP", "PYTHONPATH", "PERL5OPT", "PERL5LIB", "NODE_OPTIONS", "RUBYOPT"];

/// Programs that run a string as code after a flag (`sh -c TEXT`).
const INLINE_CODE: &[&str] = &["sh", "bash", "dash", "zsh", "ksh", "ash", "mksh", "fish", "busybox", "python", "python2", "python3", "perl", "ruby", "node", "nodejs", "php", "lua", "pwsh", "osascript"];

/// Programs whose first operand is a program of their own, with how many
/// values each of their options takes.
fn script_operand(prog: &str) -> Option<fn(&str) -> usize> {
    match prog {
        "awk" | "gawk" | "mawk" | "nawk" => Some(|f| usize::from(matches!(f, "-v" | "-F" | "-f"))),
        "sed" => Some(|f| usize::from(matches!(f, "-f" | "-l"))),
        "jq" => Some(|f| match f {
            "--arg" | "--argjson" | "--slurpfile" | "--rawfile" => 2,
            "-f" | "--from-file" | "--indent" => 1,
            _ => 0,
        }),
        "xargs" => Some(|f| usize::from(matches!(f, "-I" | "-n" | "-P" | "-d" | "-L" | "-s" | "-E" | "-a"))),
        _ => None,
    }
}

/// Programs that run the command after them.
const WRAPPERS: &[&str] = &["sudo", "doas", "env", "nice", "ionice", "nohup", "timeout", "stdbuf", "chrt", "setsid", "time", "command", "exec"];

/// Where the programs in `argv` are: the first, and each one a wrapper runs.
fn programs(argv: &[Template]) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(prog) = argv.get(i).and_then(basename) {
        out.push((i, prog));
        if !WRAPPERS.contains(&prog) {
            break;
        }
        i += 1;
        // The wrapper's own options and values: -u root, VAR=x, 30s, 10.
        while let Some(a) = argv.get(i).and_then(literal) {
            if a.starts_with('-') {
                i += if matches!(a, "-u" | "-g" | "-n" | "-c" | "-s" | "-k" | "-C") { 2 } else { 1 };
            } else if a.contains('=') || a.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                i += 1;
            } else {
                break;
            }
        }
    }
    out
}

fn basename(t: &Template) -> Option<&str> {
    match t.0.as_slice() {
        [manifest::Part::Text(s)] => Some(s.rsplit('/').next().unwrap_or(s)),
        _ => None,
    }
}

fn literal(t: &Template) -> Option<&str> {
    match t.0.as_slice() {
        [manifest::Part::Text(s)] => Some(s),
        [] => Some(""),
        _ => None,
    }
}

fn templated(t: &Template) -> bool {
    t.vars().next().is_some()
}

/// Why `argv` would turn a placeholder into code, if it would.
pub fn shell_from_placeholder(argv: &[Template]) -> Option<String> {
    for (i, t) in argv.iter().enumerate() {
        let Some(prog) = basename(t) else { continue };
        let rest = &argv[i + 1..];
        if INLINE_CODE.contains(&prog) {
            for (j, a) in rest.iter().enumerate() {
                let Some(flag) = literal(a) else { continue };
                let is_code_flag = matches!(flag, "-c" | "-e" | "-E" | "--eval" | "--command" | "-r")
                    || (flag.starts_with('-') && !flag.starts_with("--") && flag.len() > 2 && flag[1..].chars().all(|c| c.is_ascii_alphabetic()) && flag.contains('c'));
                if is_code_flag && rest.get(j + 1).is_some_and(templated) {
                    return Some(format!("`{prog} {flag}` with a placeholder in its code: pass the value as an argument after the code, or in env"));
                }
            }
        }
        if let Some(takes_value) = script_operand(prog) {
            // awk/sed/jq programs can run commands; xargs runs its operand.
            let mut k = 0;
            while let Some(a) = rest.get(k) {
                match literal(a) {
                    Some(f) if f == "-e" || f == "--expression" => {
                        if rest.get(k + 1).is_some_and(templated) {
                            return Some(format!("`{prog}` with a placeholder in its program: pass the value with -v/--arg, or as an argument"));
                        }
                        k += 2;
                    }
                    Some(f) if f.starts_with('-') && f.len() > 1 => k += 1 + takes_value(f),
                    _ => {
                        if templated(a) {
                            return Some(format!("`{prog}` with a placeholder in its program: pass the value with -v/--arg, or as an argument"));
                        }
                        break;
                    }
                }
            }
        }
        if prog == "find" {
            for (j, a) in rest.iter().enumerate() {
                if literal(a).is_some_and(|f| f == "-exec" || f == "-execdir" || f == "-ok") && rest.get(j + 1).is_some_and(templated) {
                    return Some("`find` with a placeholder as the command it runs".into());
                }
            }
        }
        if prog == "ssh" && rest.iter().any(templated) {
            return Some("`ssh` joins its arguments into shell text on the far side: use a server action, which Crow quotes".into());
        }
        if prog == "eval" && rest.iter().any(templated) {
            return Some("`eval` with a placeholder".into());
        }
    }
    None
}

/// Commands that change a system, with the subcommands that do (empty: any use).
const CHANGES: &[(&str, &[&str])] = &[
    ("rm", &[]), ("rmdir", &[]), ("mv", &[]), ("cp", &[]), ("dd", &[]), ("ln", &[]), ("install", &[]), ("mkdir", &[]), ("touch", &[]), ("truncate", &[]), ("shred", &[]), ("tee", &[]),
    ("chmod", &[]), ("chown", &[]), ("chgrp", &[]), ("chattr", &[]), ("setfacl", &[]),
    ("useradd", &[]), ("userdel", &[]), ("usermod", &[]), ("groupadd", &[]), ("groupdel", &[]), ("passwd", &[]), ("chpasswd", &[]),
    ("reboot", &[]), ("shutdown", &[]), ("poweroff", &[]), ("halt", &[]), ("kill", &[]), ("pkill", &[]), ("killall", &[]),
    ("mkfs", &[]), ("mount", &[]), ("umount", &[]), ("crontab", &[]), ("sysctl", &["-w"]), ("hostnamectl", &["set-hostname"]),
    ("systemctl", &["start", "stop", "restart", "reload", "enable", "disable", "mask", "unmask", "kill", "daemon-reload", "reboot", "poweroff", "isolate", "edit", "set-property"]),
    ("service", &["start", "stop", "restart", "reload"]),
    ("apt", &["install", "remove", "purge", "upgrade", "full-upgrade", "dist-upgrade", "autoremove"]),
    ("apt-get", &["install", "remove", "purge", "upgrade", "dist-upgrade", "autoremove"]),
    ("dnf", &["install", "remove", "erase", "upgrade", "update", "downgrade", "autoremove"]),
    ("yum", &["install", "remove", "erase", "upgrade", "update", "downgrade", "autoremove"]),
    ("zypper", &["install", "in", "remove", "rm", "update", "up", "dist-upgrade", "dup"]),
    ("pacman", &["-S", "-R", "-U", "-Syu", "-Rs", "-Rns"]),
    ("apk", &["add", "del", "upgrade"]),
    ("pip", &["install", "uninstall"]), ("pip3", &["install", "uninstall"]),
    ("docker", &["rm", "rmi", "stop", "kill", "run", "start", "restart", "pull", "exec", "create", "prune"]),
    ("podman", &["rm", "rmi", "stop", "kill", "run", "start", "restart", "pull", "exec", "create", "prune"]),
    ("ufw", &["enable", "disable", "allow", "deny", "reject", "limit", "delete", "insert", "reset", "default"]),
    ("firewall-cmd", &["--add-port", "--remove-port", "--add-service", "--remove-service", "--reload", "--permanent"]),
    ("iptables", &["-A", "-D", "-I", "-R", "-F", "-X", "-P", "-N"]), ("nft", &["add", "delete", "flush", "insert", "replace"]),
    ("restic", &["backup", "forget", "prune", "init", "unlock", "restore", "key", "migrate", "rewrite"]),
];

/// What in `argv` changes the system, if anything does.
pub fn changes_something(argv: &[Template]) -> Option<String> {
    for (i, prog) in programs(argv) {
        let rest: Vec<&str> = argv[i + 1..].iter().filter_map(literal).collect();
        if prog == "sed" && rest.iter().any(|a| *a == "-i" || a.starts_with("-i") || *a == "--in-place") {
            return Some("it runs `sed -i`".into());
        }
        if prog == "ansible-playbook" && !rest.iter().any(|a| *a == "--check" || *a == "-C") {
            return Some("it runs ansible-playbook without --check".into());
        }
        let base = prog.split('.').next().unwrap_or(prog);
        if let Some((_, verbs)) = CHANGES.iter().find(|(p, _)| *p == base) {
            if verbs.is_empty() {
                return Some(format!("it runs `{prog}`"));
            }
            if let Some(v) = rest.iter().find(|a| verbs.contains(a)) {
                return Some(format!("it runs `{prog} {v}`"));
            }
        }
    }
    None
}

/// A command in a read action's script that looks like it changes something.
/// Only a warning: a script can't be read as surely as an argv list.
fn script_changes_something(body: &str) -> Option<String> {
    for line in body.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let words: Vec<&str> = line.split(|c: char| c.is_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '`' | '$' | '{' | '}')).filter(|w| !w.is_empty()).collect();
        for (i, w) in words.iter().enumerate() {
            let base = w.rsplit('/').next().unwrap_or(w);
            if let Some((_, verbs)) = CHANGES.iter().find(|(p, _)| *p == base) {
                if verbs.is_empty() || words.get(i + 1).is_some_and(|v| verbs.contains(v)) {
                    return Some(words[i..(i + 2).min(words.len())].join(" "));
                }
            }
        }
    }
    None
}

fn human_bytes(n: u64) -> String {
    if n >= 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{n} bytes")
    }
}

// ---------------------------------------------------------------------------
// The report

/// An argv list as one line, quoting what needs it.
pub fn command_line(argv: &[Template]) -> String {
    argv.iter()
        .map(|t| {
            let s = t.to_string();
            if s.is_empty() || s.chars().any(|c| c.is_whitespace() || matches!(c, '\'' | '"' | '\\' | '$' | '`' | ';' | '|' | '&' | '<' | '>')) {
                format!("'{}'", s.replace('\'', "'\\''"))
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn effect_word(e: Effect) -> &'static str {
    match e {
        Effect::Read => "read",
        Effect::Change => "CHANGE",
        Effect::Danger => "DANGER",
    }
}

fn where_words(a: &Action) -> &'static str {
    match (a.on, a.root) {
        (Runs::Server, true) => "on each server, as root",
        (Runs::Server, false) => "on each server",
        (Runs::Local, _) => "on this machine",
    }
}

fn runs(command: Option<&[Template]>, script: Option<&str>, interpreter: Option<manifest::Interpreter>, args: &[Template]) -> String {
    match (command, script) {
        (Some(c), _) => command_line(c),
        (None, Some(s)) => {
            let mut line = format!("{} {s}", interpreter.map(|i| i.program()).unwrap_or("?"));
            if !args.is_empty() {
                line.push(' ');
                line.push_str(&command_line(args));
            }
            line
        }
        (None, None) => String::new(),
    }
}

fn env_lines(out: &mut String, env: &std::collections::BTreeMap<String, Template>) {
    for (k, v) in env {
        let secret = v.vars().any(|v| matches!(v, Var::Secret(_)));
        let _ = writeln!(out, "        env {k}={v}{}", if secret { "   (secret, from the vault)" } else { "" });
    }
}

/// What a plugin declares and what was found, for a person to read.
pub fn render(r: &Report) -> String {
    let mut out = String::new();
    let Some(m) = &r.manifest else {
        let _ = writeln!(out, "{}: can't be read as a plugin", r.dir.display());
        for p in &r.refusals {
            let _ = writeln!(out, "  refused  {p}");
        }
        return out;
    };
    let p = &m.plugin;
    let _ = writeln!(out, "{} {} · {} · by {}", p.name, p.version, p.id, p.author);
    let _ = writeln!(out, "{}", p.about);
    let _ = writeln!(out, "For Crow {} · {} files, {}{}", p.crow, r.files.len(), human_bytes(r.total_bytes()), p.license.as_deref().map(|l| format!(" · {l}")).unwrap_or_default());
    let needs: Vec<String> = [("this machine", &m.requires.local), ("servers", &m.requires.server), ("OS", &m.requires.os)].iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| format!("{k}: {}", v.join(", "))).collect();
    if !needs.is_empty() {
        let _ = writeln!(out, "Needs {}", needs.join(" · "));
    }

    if let Some(s) = &m.setup {
        let _ = writeln!(out, "\nSETUP, once, on this machine");
        let _ = writeln!(out, "    {}", runs(s.command.as_deref(), s.script.as_deref(), s.interpreter, &s.args));
        env_lines(&mut out, &s.env);
    }
    let _ = writeln!(out, "\nACTIONS");
    let width = m.actions.iter().map(|a| a.id.len()).max().unwrap_or(0);
    for a in &m.actions {
        let _ = writeln!(out, "  {:width$}  {:6}  {}", a.id, effect_word(a.effect), where_words(a));
        let _ = writeln!(out, "        {}", runs(a.command.as_deref(), a.script.as_deref(), a.interpreter, &a.args));
        env_lines(&mut out, &a.env);
        let mut extras = Vec::new();
        if let Some(d) = &a.dry_run {
            extras.push(format!("dry run first: {d}"));
        }
        if a.effect == Effect::Danger {
            extras.push(format!("typed keyword: {}", a.keyword.as_deref().unwrap_or("CONFIRM")));
        }
        if a.login_check {
            extras.push("Crow proves its login before and after".into());
        }
        for f in &a.inputs {
            extras.push(format!("input {} ({:?}, {:?})", f.key, f.kind, f.charset).to_lowercase());
        }
        if !extras.is_empty() {
            let _ = writeln!(out, "        {}", extras.join(" · "));
        }
    }
    if !m.settings.is_empty() {
        let _ = writeln!(out, "\nSETTINGS");
        for s in &m.settings {
            let _ = writeln!(out, "  {} ({}){}", s.key, format!("{:?}", s.kind).to_lowercase(), if s.kind == manifest::FieldKind::Secret { ", kept in the vault" } else { "" });
        }
    }
    if !m.pages.is_empty() || m.badge.is_some() {
        let _ = writeln!(out, "\nSHOWS");
        for pg in &m.pages {
            let _ = writeln!(out, "  page \"{}\" on {}", pg.label, if pg.place == manifest::Place::Server { "each server" } else { "Fleet configuration" });
        }
        if let Some(b) = &m.badge {
            let _ = writeln!(out, "  a badge from {}, every {}", b.action, b.every);
        }
    }
    if !r.warnings.is_empty() {
        let _ = writeln!(out, "\nWARNINGS");
        for w in &r.warnings {
            let _ = writeln!(out, "  {w}");
        }
    }
    if r.refusals.is_empty() {
        let _ = writeln!(out, "\nNo problems found. Read the commands above before you accept it.");
    } else {
        let _ = writeln!(out, "\nREFUSED: Crow won't load this plugin");
        for p in &r.refusals {
            let _ = writeln!(out, "  {p}");
        }
    }
    out
}

/// `crow --check-plugin DIR`: 0 when it passes, 1 when it's refused, 2 on usage.
pub fn check_dir_main(path: Option<&str>) -> i32 {
    let Some(path) = path else {
        eprintln!("usage: crow --check-plugin ~/.config/crow/plugins/<id>");
        return 2;
    };
    let r = check_dir(Path::new(path));
    print!("{}", render(&r));
    i32::from(!r.ok())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{check_dir, render};

    const BASE: &str = r#"format = 1
[plugin]
id = "demo"
name = "Demo"
version = "0.1.0"
author = "me"
about = "A demo."
crow = ">=0.8.3"

[[action]]
id = "list"
label = "List"
on = "server"
effect = "read"
command = ["ls", "-l"]
"#;

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugin_host/fixtures")
    }

    /// A plugin folder named `id` holding `files`, in a fresh temp dir.
    fn plugin(case: &str, id: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("crow-plugin-check-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join(id);
        for (rel, body) in files {
            let p = dir.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn refused(case: &str, files: &[(&str, &[u8])], want: &str) {
        let r = check_dir(&plugin(case, "demo", files));
        assert!(!r.ok(), "{case}: should be refused");
        let all = r.refusals.join("\n");
        assert!(all.contains(want), "{case}: wanted {want:?} in:\n{all}");
    }

    fn with_command(argv: &str) -> String {
        BASE.replace("command = [\"ls\", \"-l\"]", &format!("command = {argv}"))
    }

    fn with_input(argv: &str) -> String {
        format!("{}\n[[action.input]]\nkey = \"x\"\nlabel = \"X\"\ntype = \"text\"\n", with_command(argv))
    }

    #[test]
    fn the_examples_pass() {
        for id in ["devsec-hardening", "restic-check"] {
            let r = check_dir(&fixtures().join(id));
            assert!(r.ok(), "{id}: {:?}", r.refusals);
            assert!(r.warnings.is_empty(), "{id}: {:?}", r.warnings);
        }
    }

    #[test]
    fn the_report_says_what_runs_where() {
        let text = render(&check_dir(&fixtures().join("restic-check")));
        assert!(text.contains("Restic backups 0.1.0 · restic-check · by Errorware"), "{text}");
        assert!(text.contains("backup     CHANGE  on each server, as root"), "{text}");
        assert!(text.contains("restic backup {input.path}"), "{text}");
        assert!(text.contains("env RESTIC_PASSWORD={secret.password}   (secret, from the vault)"), "{text}");
        assert!(text.contains("No problems found"), "{text}");
        let text = render(&check_dir(&fixtures().join("devsec-hardening")));
        assert!(text.contains("apply   DANGER  on this machine"), "{text}");
        assert!(text.contains("dry run first: plan · typed keyword: HARDEN · Crow proves its login before and after"), "{text}");
        assert!(text.contains("a badge from status, every 6h") && text.contains("roles (choices)"), "{text}");
    }

    #[test]
    fn no_manifest_or_a_broken_one() {
        refused("none", &[("README", b"hi")], "no plugin.toml");
        refused("broken", &[("plugin.toml", b"format = 1\n[plugin\n")], "plugin.toml:");
        refused("unknown-key", &[("plugin.toml", BASE.replace("effect = \"read\"", "effect = \"read\"\nshell = true").as_bytes())], "unknown field `shell`");
    }

    #[test]
    fn the_folder_must_be_named_for_the_id() {
        let dir = plugin("wrong-name", "other", &[("plugin.toml", BASE.as_bytes())]);
        let r = check_dir(&dir);
        assert!(r.refusals.iter().any(|p| p.contains("must match the folder name \"other\"")), "{:?}", r.refusals);
    }

    #[test]
    fn shell_built_from_a_placeholder_is_refused() {
        let cases = [
            ("sh-c", r#"["sh", "-c", "echo {input.x}"]"#, "`sh -c` with a placeholder"),
            ("bash-lc", r#"["/bin/bash", "-lc", "{input.x}"]"#, "`bash -lc` with a placeholder"),
            ("sudo-sh", r#"["sudo", "sh", "-c", "ls {input.x}"]"#, "`sh -c` with a placeholder"),
            ("python-c", r#"["python3", "-c", "print('{input.x}')"]"#, "`python3 -c`"),
            ("perl-e", r#"["perl", "-e", "{input.x}"]"#, "`perl -e`"),
            ("awk", r#"["awk", "{{print \"{input.x}\"}}", "/etc/passwd"]"#, "`awk` with a placeholder in its program"),
            ("sed-e", r#"["sed", "-n", "-e", "s/{input.x}/y/p", "/etc/hosts"]"#, "`sed` with a placeholder in its program"),
            ("find-exec", r#"["find", "/tmp", "-exec", "{input.x}", ";"]"#, "`find` with a placeholder"),
            ("ssh", r#"["ssh", "host", "ls", "{input.x}"]"#, "`ssh` joins its arguments"),
        ];
        for (case, argv, want) in cases {
            refused(case, &[("plugin.toml", with_input(argv).as_bytes())], want);
        }
        // The value as its own argument after the code is fine: "$1" in the script.
        let ok = with_input(r#"["sh", "-c", "ls -- \"$1\"", "sh", "{input.x}"]"#);
        let r = check_dir(&plugin("sh-c-arg", "demo", &[("plugin.toml", ok.as_bytes())]));
        assert!(r.ok(), "{:?}", r.refusals);
        let ok = with_input(r#"["awk", "-v", "x={input.x}", "$1 == x", "/etc/passwd"]"#);
        let r = check_dir(&plugin("awk-v", "demo", &[("plugin.toml", ok.as_bytes())]));
        assert!(r.ok(), "{:?}", r.refusals);
    }

    #[test]
    fn env_that_changes_what_runs_cant_take_a_placeholder() {
        let m = format!("{}env = {{ LD_PRELOAD = \"{{input.x}}\" }}\n[[action.input]]\nkey = \"x\"\nlabel = \"X\"\ntype = \"text\"\n", BASE);
        refused("ld-preload", &[("plugin.toml", m.as_bytes())], "env LD_PRELOAD can't take a placeholder");
    }

    #[test]
    fn a_read_action_that_changes_things_is_refused() {
        let cases = [
            ("rm", r#"["rm", "-rf", "/tmp/x"]"#, "it runs `rm`"),
            ("systemctl", r#"["systemctl", "restart", "nginx"]"#, "it runs `systemctl restart`"),
            ("apt", r#"["apt-get", "install", "-y", "curl"]"#, "it runs `apt-get install`"),
            ("sed-i", r#"["sed", "-i", "s/a/b/", "/etc/hosts"]"#, "it runs `sed -i`"),
            ("ansible", r#"["ansible-playbook", "site.yml"]"#, "without --check"),
            ("restic", r#"["restic", "forget", "--keep-last", "1"]"#, "it runs `restic forget`"),
        ];
        for (case, argv, want) in cases {
            refused(case, &[("plugin.toml", with_command(argv).as_bytes())], want);
        }
        refused("sudo-rm", &[("plugin.toml", with_command(r#"["sudo", "-u", "root", "env", "A=1", "timeout", "30", "rm", "/x"]"#).as_bytes())], "it runs `rm`");
        for ok in [r#"["grep", "rm", "/etc/hosts"]"#, r#"["systemctl", "status", "nginx"]"#, r#"["apt", "list", "--upgradable"]"#, r#"["ansible-playbook", "--check", "site.yml"]"#, r#"["restic", "snapshots"]"#] {
            let r = check_dir(&plugin("read-ok", "demo", &[("plugin.toml", with_command(ok).as_bytes())]));
            assert!(r.ok(), "{ok}: {:?}", r.refusals);
        }
        // The same command declared as a change is fine.
        let m = with_command(r#"["systemctl", "restart", "nginx"]"#).replace("effect = \"read\"", "effect = \"change\"");
        assert!(check_dir(&plugin("change-ok", "demo", &[("plugin.toml", m.as_bytes())])).ok());
    }

    #[test]
    fn a_read_script_that_changes_things_is_a_warning() {
        let m = BASE.replace("command = [\"ls\", \"-l\"]", "script = \"s.sh\"\ninterpreter = \"sh\"");
        let r = check_dir(&plugin("script-warn", "demo", &[("plugin.toml", m.as_bytes()), ("s.sh", b"# rm is mentioned here only\nls /\nsystemctl restart nginx\n")]));
        assert!(r.ok(), "{:?}", r.refusals);
        assert_eq!(r.warnings, vec!["action list: declared read, but s.sh runs `systemctl restart`".to_string()]);
    }

    #[test]
    fn scripts_must_be_text_files_in_the_folder() {
        let m = BASE.replace("command = [\"ls\", \"-l\"]", "script = \"missing.sh\"\ninterpreter = \"sh\"");
        refused("missing-script", &[("plugin.toml", m.as_bytes())], "script \"missing.sh\" isn't a text file in the folder");
        let m = BASE.replace("command = [\"ls\", \"-l\"]", "script = \"/etc/passwd\"\ninterpreter = \"sh\"");
        refused("absolute-script", &[("plugin.toml", m.as_bytes())], "a path inside the plugin's folder");
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_refused() {
        let dir = plugin("symlink", "demo", &[("plugin.toml", BASE.as_bytes())]);
        std::os::unix::fs::symlink("/etc/passwd", dir.join("passwd")).unwrap();
        let r = check_dir(&dir);
        assert!(r.refusals.iter().any(|p| p.contains("passwd: a symlink")), "{:?}", r.refusals);
    }

    #[test]
    fn control_and_bidi_characters_are_refused() {
        let bidi = BASE.replace("about = \"A demo.\"", "about = \"A demo.\u{202E}txt.exe\"");
        refused("bidi", &[("plugin.toml", bidi.as_bytes())], "U+202E (a direction override");
        refused("bidi-script", &[("plugin.toml", BASE.as_bytes()), ("x.sh", "echo ok \u{2066}# hidden\u{2069}\n".as_bytes())], "x.sh: line 1 holds U+2066");
        refused("control", &[("plugin.toml", BASE.as_bytes()), ("x.sh", b"echo \x1b[2Jcleared\n")], "x.sh: line 1 holds U+001B (a control character)");
        refused("bidi-name", &[("plugin.toml", BASE.as_bytes()), ("a\u{202E}hs.txt", b"x")], "the name holds U+202E");
    }

    #[test]
    fn size_limits() {
        let big = vec![b'a'; (super::MAX_FILE_BYTES + 1) as usize];
        refused("big-file", &[("plugin.toml", BASE.as_bytes()), ("big.txt", &big)], "big.txt: 2.0 MB (at most 2.0 MB)");
        let mut manifest = BASE.to_string();
        manifest.push_str(&format!("# {}\n", "x".repeat(super::MAX_MANIFEST_BYTES as usize)));
        refused("big-manifest", &[("plugin.toml", manifest.as_bytes())], "(at most 256.0 KB)");
    }

    #[test]
    fn binaries_are_a_warning() {
        let r = check_dir(&plugin("binary", "demo", &[("plugin.toml", BASE.as_bytes()), ("tool", &[0xff, 0xfe, 0x00, 0x01])]));
        assert!(r.ok(), "{:?}", r.refusals);
        assert_eq!(r.warnings, vec!["tool: binary, so it can't be reviewed here".to_string()]);
    }
}
