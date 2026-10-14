//! Running a plugin's action on a server (ERR-178), and reading what it
//! printed.
//!
//! An action reaches the server as a small `sh` program on stdin, never as
//! a command line: env values (secrets among them) are `export`ed inside it,
//! so they don't show in `ps` there, and every argument is single-quoted.
//! A script is sent as a quoted here-document, so nothing is left on disk.
//! The program always exits 0 and writes the action's own exit code on its
//! last stderr line, so what the action printed is kept even when it fails.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use crate::host::{Host, HostError};

use super::manifest::{Action, Field, FieldKind, Interpreter, Output, OutputFormat, Part, Template, Var};

/// What an action may print before Crow stops keeping it.
pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

/// The values placeholders are filled from, already checked and rendered.
#[derive(Debug, Clone, Default)]
pub struct Values {
    pub inputs: BTreeMap<String, String>,
    pub settings: BTreeMap<String, String>,
    pub secrets: BTreeMap<String, String>,
    pub server: Option<ServerValues>,
    /// For actions on this machine and setup.
    pub local: Option<super::local::LocalPaths>,
}

#[derive(Debug, Clone, Default)]
pub struct ServerValues {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
}

/// A value for `f`, checked against its type, charset, choices and range,
/// rendered as it goes into a command: a list of choices is comma-joined.
pub fn check_value(f: &Field, raw: &str) -> Result<String, String> {
    let raw = raw.trim_end_matches(['\n', '\r']);
    if raw.is_empty() {
        return if f.required { Err(format!("{}: needed", f.label)) } else { Ok(String::new()) };
    }
    if raw.chars().any(|c| c.is_control()) {
        return Err(format!("{}: control characters aren't allowed", f.label));
    }
    match f.kind {
        FieldKind::Text => match raw.chars().find(|c| !f.charset.allows(*c)) {
            Some(c) => Err(format!("{}: \"{c}\" isn't allowed here", f.label)),
            None => Ok(raw.to_string()),
        },
        FieldKind::Number => {
            let n: f64 = raw.parse().map_err(|_| format!("{}: not a number", f.label))?;
            if f.min.is_some_and(|lo| n < lo) || f.max.is_some_and(|hi| n > hi) {
                return Err(format!("{}: out of range", f.label));
            }
            Ok(if n.fract() == 0.0 { format!("{}", n as i64) } else { n.to_string() })
        }
        FieldKind::Bool => match raw {
            "true" | "false" => Ok(raw.to_string()),
            _ => Err(format!("{}: true or false", f.label)),
        },
        FieldKind::Choice => f.choices.iter().find(|c| *c == raw).cloned().ok_or_else(|| format!("{}: not one of the choices", f.label)),
        FieldKind::Choices => {
            let picked: Vec<&str> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
            match picked.iter().find(|p| !f.choices.iter().any(|c| c == *p)) {
                Some(p) => Err(format!("{}: \"{p}\" isn't one of the choices", f.label)),
                None => Ok(picked.join(",")),
            }
        }
        FieldKind::Secret => Ok(raw.to_string()),
    }
}

/// A field's default, rendered like a typed value.
pub fn default_value(f: &Field) -> Option<String> {
    let d = f.default.as_ref()?;
    Some(match d {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(x) => x.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(a) => a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","),
        _ => return None,
    })
}

/// Settings split into plain and secret values, plus what's missing or
/// wrong as (key, why).
pub type SettingsValues = (BTreeMap<String, String>, BTreeMap<String, String>, Vec<(String, String)>);

/// The settings an action can use: what's stored, else the defaults. A
/// required setting with neither is listed with its label.
pub fn settings_values(settings: &[Field], stored: &BTreeMap<String, String>) -> SettingsValues {
    let (mut plain, mut secret, mut missing) = (BTreeMap::new(), BTreeMap::new(), Vec::new());
    for f in settings {
        let raw = stored.get(&f.key).cloned().or_else(|| default_value(f));
        match raw.map(|r| check_value(f, &r)) {
            Some(Ok(v)) if !v.is_empty() => {
                if f.kind == FieldKind::Secret {
                    secret.insert(f.key.clone(), v);
                } else {
                    plain.insert(f.key.clone(), v);
                }
            }
            Some(Err(e)) => missing.push((f.key.clone(), e)),
            _ if f.required => missing.push((f.key.clone(), f.label.clone())),
            _ => {}
        }
    }
    (plain, secret, missing)
}

/// One template filled in. A placeholder with no value is an error, never
/// an empty string standing in for it.
pub fn fill(t: &Template, v: &Values) -> Result<String, String> {
    let mut out = String::new();
    for part in &t.0 {
        match part {
            Part::Text(s) => out.push_str(s),
            Part::Var(var) => {
                let value = match var {
                    Var::Input(k) => v.inputs.get(k).cloned(),
                    Var::Setting(k) => v.settings.get(k).cloned(),
                    Var::Secret(k) => v.secrets.get(k).cloned(),
                    Var::Server(f) => v.server.as_ref().map(|s| match f {
                        super::manifest::ServerField::Name => s.name.clone(),
                        super::manifest::ServerField::Host => s.host.clone(),
                        super::manifest::ServerField::User => s.user.clone(),
                        super::manifest::ServerField::Port => s.port.to_string(),
                    }),
                    Var::PluginDir => v.local.as_ref().map(|l| l.plugin_dir.display().to_string()),
                    Var::DataDir => v.local.as_ref().map(|l| l.data_dir.display().to_string()),
                    Var::Inventory => v.local.as_ref().map(|l| l.inventory.display().to_string()),
                    Var::SshConfig => v.local.as_ref().map(|l| l.ssh_config.display().to_string()),
                    Var::Results => v.local.as_ref().map(|l| l.results.display().to_string()),
                };
                out.push_str(&value.ok_or_else(|| format!("{var} has no value"))?);
            }
        }
    }
    Ok(out)
}

/// Single quotes around anything; a quote inside becomes '\''.
pub fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn exit_marker(nonce: &str) -> String {
    format!("CROW_EXIT_{nonce}=")
}

/// The `sh` program that runs `a` on a server. `plugin_dir` is where its
/// script is read from.
pub fn remote_program(a: &Action, requires: &[String], plugin_dir: &Path, v: &Values, nonce: &str) -> Result<String, String> {
    let mut p = String::from("# Crow plugin action\n");
    // What the plugin needs on the server, named before anything runs.
    let marker = quote(&exit_marker(nonce));
    for c in requires {
        let c = quote(c);
        p.push_str(&format!("command -v {c} >/dev/null 2>&1 || {{ printf 'needs %s on this server\\n' {c} >&2; printf '%s%s\\n' {marker} 127 >&2; exit 0; }}\n"));
    }
    for (k, t) in &a.env {
        p.push_str(&format!("export {k}={}\n", quote(&fill(t, v)?)));
    }
    // The action runs in its own session (setsid, where there is one), so
    // STOP can end it and everything it started; its pid is reported first.
    p.push_str("if command -v setsid >/dev/null 2>&1; then crow_own=setsid; else crow_own=; fi\n");
    match (&a.command, &a.script) {
        (Some(argv), _) => {
            let words = argv.iter().map(|t| fill(t, v).map(|s| quote(&s))).collect::<Result<Vec<_>, _>>()?;
            // stdin is this program; the action mustn't read the rest of it.
            p.push_str(&format!("$crow_own {} </dev/null &\n", words.join(" ")));
        }
        (None, Some(script)) => {
            let body = std::fs::read_to_string(plugin_dir.join(script)).map_err(|e| format!("{script}: {e}"))?;
            let delim = format!("CROW_SCRIPT_{nonce}");
            if body.lines().any(|l| l.trim() == delim) {
                return Err(format!("{script} holds the line {delim}"));
            }
            let args = a.args.iter().map(|t| fill(t, v).map(|s| quote(&s))).collect::<Result<Vec<_>, _>>()?;
            let run = match a.interpreter.unwrap_or(Interpreter::Sh) {
                Interpreter::Sh => "sh -s --",
                Interpreter::Bash => "bash -s --",
                Interpreter::Python3 => "python3 -",
            };
            let args = if args.is_empty() { String::new() } else { format!(" {}", args.join(" ")) };
            p.push_str(&format!("$crow_own {run}{args} <<'{delim}' &\n{body}{}{delim}\n", if body.ends_with('\n') { "" } else { "\n" }));
        }
        (None, None) => return Err("nothing to run".into()),
    }
    p.push_str(&format!("crow_pid=$!\nprintf '%s%s\\n' {} \"$crow_pid\" >&2\nwait \"$crow_pid\"\ncrow_rc=$?\nprintf '%s%s\\n' {} \"$crow_rc\" >&2\nexit 0\n", quote(&pid_marker(nonce)), quote(&exit_marker(nonce))));
    Ok(p)
}

pub fn pid_marker(nonce: &str) -> String {
    format!("CROW_PID_{nonce}=")
}

/// The `sh` program that ends a run started with pid `pid`: its whole
/// session if it has one, else the process, then a kill if it lingers.
pub fn stop_program(pid: u32) -> String {
    format!("kill -TERM -- -{pid} 2>/dev/null || kill -TERM {pid} 2>/dev/null\nsleep 2\nkill -KILL -- -{pid} 2>/dev/null || kill -KILL {pid} 2>/dev/null\nexit 0\n")
}

/// Splits the markers off stderr: (stderr without them, the exit code).
/// Only the last exit marker counts; one the action printed itself can't
/// end its run early.
pub fn split_exit(stderr: &str, nonce: &str) -> (String, Option<i32>) {
    let (exit, pid) = (exit_marker(nonce), pid_marker(nonce));
    let lines: Vec<&str> = stderr.trim_end().lines().collect();
    let code = match lines.last() {
        Some(last) if last.starts_with(&exit) => last[exit.len()..].trim().parse().ok(),
        _ => None,
    };
    let keep = if code.is_some() { &lines[..lines.len() - 1] } else { &lines[..] };
    (keep.iter().filter(|l| !l.starts_with(&pid)).copied().collect::<Vec<_>>().join("\n"), code)
}

/// Whether a stderr line is one of Crow's own markers.
pub fn is_marker(line: &str, nonce: &str) -> bool {
    line.starts_with(&exit_marker(nonce)) || line.starts_with(&pid_marker(nonce))
}

/// How one run ended.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunResult {
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
    /// Couldn't run at all: unreachable, sudo, a value missing.
    pub error: Option<String>,
    pub started_at: i64,
    pub finished_at: i64,
    /// Ended by STOP.
    pub stopped: bool,
    /// A local action's per-server results, from `{results}`.
    pub per_host: Vec<serde_json::Value>,
}

impl RunResult {
    pub fn failed(error: String) -> RunResult {
        let now = chrono::Utc::now().timestamp();
        RunResult { error: Some(error), started_at: now, finished_at: now, ..Default::default() }
    }

    pub fn output_bytes(&self) -> u64 {
        (self.stdout.len() + self.stderr.len()) as u64
    }
}

fn cap(mut s: String) -> (String, bool) {
    if s.len() <= MAX_OUTPUT_BYTES {
        return (s, false);
    }
    let mut end = MAX_OUTPUT_BYTES;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
    (s, true)
}

/// Runs `program` on `host`, handing each line it prints to `sink` (Crow's
/// markers left out). Setting `stop` ends it there and on the server.
/// Blocking: call it off the UI thread.
pub fn run_streaming(host: &dyn Host, program: &str, root: bool, timeout: Duration, nonce: &str, sink: &mut dyn FnMut(bool, &str), stop: &std::sync::atomic::AtomicBool) -> RunResult {
    let started_at = chrono::Utc::now().timestamp();
    let mut pid: Option<u32> = None;
    let pid_m = pid_marker(nonce);
    // What arrived, kept for a run that ends early (stopped, timed out).
    let (mut so_far_out, mut so_far_err) = (String::new(), String::new());
    let mut on_line = |is_err: bool, line: &str| {
        if is_err && line.starts_with(&pid_m) {
            pid = line[pid_m.len()..].trim().parse().ok();
        }
        if !(is_err && is_marker(line, nonce)) {
            sink(is_err, line);
            let into = if is_err { &mut so_far_err } else { &mut so_far_out };
            if into.len() < MAX_OUTPUT_BYTES {
                into.push_str(line);
                into.push('\n');
            }
        }
    };
    let result = if root {
        host.exec_privileged_streaming(&["sh", "-s"], program.as_bytes(), timeout, &mut on_line, stop)
    } else {
        host.exec_streaming(&["sh", "-s"], program.as_bytes(), timeout, &mut on_line, stop)
    };
    let stopped = stop.load(std::sync::atomic::Ordering::Relaxed) || matches!(result, Err(HostError::Timeout));
    // Ending the connection may leave the action running there: end it too.
    if stopped {
        if let Some(pid) = pid {
            let kill = stop_program(pid);
            let _ = if root { host.exec_privileged(&["sh", "-s"], kill.as_bytes(), Duration::from_secs(15)) } else { host.exec_stdin(&["sh", "-s"], kill.as_bytes(), Duration::from_secs(15)) };
        }
    }
    let finished_at = chrono::Utc::now().timestamp();
    match result {
        Ok(out) => {
            let (stderr, exit_code) = split_exit(&out.stderr, nonce);
            let (stdout, t1) = cap(out.stdout);
            let (stderr, t2) = cap(stderr);
            let error = exit_code.is_none().then(|| "the action didn't report how it ended".to_string());
            RunResult { ok: exit_code == Some(0), exit_code, stdout, stderr, truncated: t1 || t2, error, started_at, finished_at, ..Default::default() }
        }
        Err(e) => {
            let error = match e {
                _ if stop.load(std::sync::atomic::Ordering::Relaxed) => "stopped".to_string(),
                HostError::Failed { stderr, .. } => split_exit(&stderr, nonce).0.trim().to_string(),
                HostError::Unreachable(why) => why,
                HostError::Timeout => format!("timed out after {}s, and was stopped", timeout.as_secs()),
                other => other.to_string(),
            };
            let (stdout, t1) = cap(so_far_out);
            let (stderr, t2) = cap(so_far_err);
            RunResult { error: Some(error), stdout, stderr, truncated: t1 || t2, started_at, finished_at, stopped, ..Default::default() }
        }
    }
}

/// `run_streaming` with nobody watching the lines.
pub fn run_program(host: &dyn Host, program: &str, root: bool, timeout: Duration, nonce: &str) -> RunResult {
    run_streaming(host, program, root, timeout, nonce, &mut |_, _| {}, &std::sync::atomic::AtomicBool::new(false))
}

/// `login_check` (ERR-170): Crow's own key login to each server, made anew
/// (no connection it already holds). Lab containers and this machine have
/// no login to lose and are skipped.
pub fn prove_logins(servers: &[crate::vault::ServerRecord]) -> Result<(), String> {
    for s in servers {
        if crate::host::transport_kind(s) != crate::host::TransportKind::Ssh {
            continue;
        }
        crate::host::ssh::fresh_key_login(s).map_err(|e| format!("{}: {e}", s.name))?;
    }
    Ok(())
}

/// Runs `job` between two `prove_logins`: refused when Crow can't log in
/// before; failed, with how to get back in, when it can't after.
pub fn guarded(servers: &[crate::vault::ServerRecord], job: impl FnOnce() -> RunResult) -> RunResult {
    if let Err(e) = prove_logins(servers) {
        return RunResult::failed(t!("plugin-run-login-before", why = e));
    }
    let mut r = job();
    if let Err(e) = prove_logins(servers) {
        r.ok = false;
        r.error = Some(t!("plugin-run-login-after", why = e));
    }
    r
}

/// A fresh nonce for one run.
pub fn nonce() -> String {
    use rand::RngExt;
    let mut b = [0u8; 8];
    rand::rng().fill(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// ---------------------------------------------------------------------------
// Reading the output

/// The run's one-line summary: the line the plugin named, else the first
/// line printed, else why it failed.
pub fn summary(out: &Output, r: &RunResult) -> String {
    let line = out
        .summary_prefix
        .as_deref()
        .and_then(|p| r.stdout.lines().find(|l| l.trim_start().starts_with(p)))
        .or_else(|| (out.format == OutputFormat::Lines).then(|| r.stdout.lines().find(|l| !l.trim().is_empty())).flatten())
        .map(|l| l.trim().to_string());
    let last_err = r.stderr.lines().rev().find(|l| !l.trim().is_empty()).map(|l| l.trim().to_string());
    let text = match (&r.error, line) {
        (Some(e), _) => e.lines().next().unwrap_or_default().to_string(),
        // A failure is better told by what it said went wrong.
        (None, _) if !r.ok && out_named_no_line(out, r) && last_err.is_some() => last_err.unwrap_or_default(),
        (None, Some(l)) => l,
        (None, None) if r.ok => String::new(),
        (None, None) => r.stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or_default().trim().to_string(),
    };
    text.chars().take(200).collect()
}

/// The plugin didn't name a summary line that appears in this output.
fn out_named_no_line(out: &Output, r: &RunResult) -> bool {
    out.summary_prefix.as_deref().is_none_or(|p| !r.stdout.lines().any(|l| l.trim_start().starts_with(p)))
}

fn at_path<'a>(v: &'a serde_json::Value, path: Option<&str>) -> Option<&'a serde_json::Value> {
    match path {
        None => Some(v),
        Some(p) => p.split('.').try_fold(v, |v, k| v.get(k)),
    }
}

/// A JSON value as one cell of text.
pub fn cell(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(a) => a.iter().map(cell).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

pub type Table = (Vec<String>, Vec<Vec<String>>);

/// Rows from json or table output, with the declared columns (or all).
pub fn table(out: &Output, stdout: &str) -> Result<Table, String> {
    match out.format {
        OutputFormat::Json => {
            let v: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|e| format!("not JSON: {e}"))?;
            let rows = at_path(&v, out.rows.as_deref()).and_then(|r| r.as_array()).ok_or("no list of rows in the output")?;
            let keys: Vec<(String, String)> = if out.columns.is_empty() {
                let mut ks: Vec<String> = Vec::new();
                for r in rows.iter().filter_map(|r| r.as_object()) {
                    for k in r.keys() {
                        if !ks.contains(k) {
                            ks.push(k.clone());
                        }
                    }
                }
                ks.into_iter().map(|k| (k.clone(), k)).collect()
            } else {
                out.columns.iter().map(|c| (c.key.clone(), c.label.clone())).collect()
            };
            let body = rows.iter().map(|r| keys.iter().map(|(k, _)| r.get(k).map(cell).unwrap_or_default()).collect()).collect();
            Ok((keys.into_iter().map(|(_, l)| l).collect(), body))
        }
        OutputFormat::Table => {
            let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
            let header: Vec<String> = lines.next().ok_or("no header line")?.split_whitespace().map(str::to_string).collect();
            let rows: Vec<Vec<String>> = lines
                .map(|l| {
                    // Aligned columns; the last one takes the rest of the line.
                    let words: Vec<&str> = l.split_whitespace().collect();
                    let n = header.len().max(1);
                    let mut cells: Vec<String> = words.iter().take(n - 1).map(|w| w.to_string()).collect();
                    if words.len() >= n {
                        cells.push(words[n - 1..].join(" "));
                    }
                    cells
                })
                .collect();
            if out.columns.is_empty() {
                return Ok((header, rows));
            }
            let idx: Vec<Option<usize>> = out.columns.iter().map(|c| header.iter().position(|h| *h == c.key)).collect();
            let body = rows.iter().map(|r| idx.iter().map(|i| i.and_then(|i| r.get(i).cloned()).unwrap_or_default()).collect()).collect();
            Ok((out.columns.iter().map(|c| c.label.clone()).collect(), body))
        }
        _ => Err("this action's output isn't json or a table".into()),
    }
}

/// Key and value pairs from a JSON object.
pub fn kv(out: &Output, stdout: &str) -> Result<Vec<(String, String)>, String> {
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|e| format!("not JSON: {e}"))?;
    let obj = at_path(&v, out.rows.as_deref()).and_then(|o| o.as_object()).ok_or("not a JSON object")?;
    Ok(obj.iter().map(|(k, v)| (k.clone(), cell(v))).collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Ok,
    Warn,
    Bad,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    pub label: String,
}

/// A status: `{"state": "ok" | "warn" | "bad" | "unknown", "label": "…"}`.
pub fn status(stdout: &str) -> Result<Status, String> {
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|e| format!("not JSON: {e}"))?;
    let state = match v.get("state").and_then(|s| s.as_str()) {
        Some("ok") => State::Ok,
        Some("warn") => State::Warn,
        Some("bad") => State::Bad,
        Some("unknown") => State::Unknown,
        _ => return Err("no \"state\": ok, warn, bad or unknown".into()),
    };
    let label: String = v.get("label").and_then(|s| s.as_str()).unwrap_or_default().chars().filter(|c| !c.is_control()).take(80).collect();
    Ok(Status { state, label })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::{check_value, fill, kv, quote, remote_program, settings_values, split_exit, status, summary, table, RunResult, ServerValues, State, Values};
    use crate::plugin_host::manifest::{self, Template};

    fn manifest(actions: &str) -> manifest::Manifest {
        let text = format!("format = 1\n[plugin]\nid = \"demo\"\nname = \"D\"\nversion = \"0.1.0\"\nauthor = \"a\"\nabout = \"b\"\ncrow = \">=0.8\"\n\n[[setting]]\nkey = \"token\"\nlabel = \"Token\"\ntype = \"secret\"\n{actions}");
        manifest::parse(&text).unwrap_or_else(|e| panic!("{e}"))
    }

    /// Runs `program` with a local `sh -s`, as a server would.
    fn run_sh(program: &str) -> (String, String) {
        use std::io::Write;
        let mut child = std::process::Command::new("sh").arg("-s").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(program.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        (String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
    }

    #[test]
    fn hostile_values_stay_one_argument() {
        let m = manifest("[[action]]\nid = \"echo\"\nlabel = \"E\"\non = \"server\"\neffect = \"read\"\ncommand = [\"printf\", \"%s|\", \"{input.x}\", \"--{input.x}--\"]\nenv = { TOKEN = \"{secret.token}\" }\n[[action.input]]\nkey = \"x\"\nlabel = \"X\"\ntype = \"text\"\ncharset = \"any\"\n");
        let a = &m.actions[0];
        let canary = std::env::temp_dir().join(format!("crow-plugin-pwned-{}", std::process::id()));
        let hostile = format!("'; touch {} ; echo '$(id)`id`\"\\", canary.display());
        let v = Values { inputs: BTreeMap::from([("x".into(), hostile.clone())]), secrets: BTreeMap::from([("token".into(), "s3cr'et".into())]), ..Default::default() };
        let program = remote_program(a, &[], Path::new("/nonexistent"), &v, "n1").unwrap();
        assert!(!program.contains("s3cr'et\n") && program.contains("export TOKEN='s3cr'\\''et'"), "{program}");
        let (stdout, stderr) = run_sh(&program);
        assert_eq!(stdout, format!("{hostile}|--{hostile}--|"));
        assert!(!canary.exists(), "the input ran as a command");
        assert_eq!(split_exit(&stderr, "n1"), (String::new(), Some(0)));
    }

    #[test]
    fn secrets_reach_the_env_not_the_command_line() {
        let m = manifest("[[action]]\nid = \"env\"\nlabel = \"E\"\non = \"server\"\neffect = \"read\"\ncommand = [\"sh\", \"-c\", \"printf %s \\\"$TOKEN\\\"\"]\nenv = { TOKEN = \"{secret.token}\" }\n");
        let v = Values { secrets: BTreeMap::from([("token".into(), "hunter2".into())]), ..Default::default() };
        let program = remote_program(&m.actions[0], &[], Path::new("/nonexistent"), &v, "n2").unwrap();
        let command_line = program.lines().find(|l| l.contains("'sh' '-c'")).unwrap();
        assert!(!command_line.contains("hunter2"), "{command_line}");
        assert_eq!(run_sh(&program).0, "hunter2");
    }

    #[test]
    fn scripts_go_as_a_here_document_with_their_args() {
        let dir = std::env::temp_dir().join(format!("crow-plugin-run-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("s.sh"), "printf '%s,' \"$@\"\necho \"[$(cat)]\"").unwrap();
        let m = manifest("[[action]]\nid = \"ss\"\nlabel = \"S\"\non = \"server\"\neffect = \"read\"\nscript = \"s.sh\"\ninterpreter = \"sh\"\nargs = [\"a b\", \"{server.name}\"]\n");
        let v = Values { server: Some(ServerValues { name: "web 1".into(), ..Default::default() }), ..Default::default() };
        let program = remote_program(&m.actions[0], &[], &dir, &v, "n3").unwrap();
        let (stdout, stderr) = run_sh(&program);
        // The script's own stdin is empty: it can't read Crow's program.
        assert_eq!(stdout, "a b,web 1,[]\n");
        assert_eq!(split_exit(&stderr, "n3").1, Some(0));
        std::fs::write(dir.join("s.sh"), "echo hi\nCROW_SCRIPT_n4\nrm -rf /\n").unwrap();
        assert!(remote_program(&m.actions[0], &[], &dir, &v, "n4").unwrap_err().contains("holds the line"));
    }

    #[test]
    fn failures_keep_their_output_and_exit_code() {
        let m = manifest("[[action]]\nid = \"ff\"\nlabel = \"F\"\non = \"server\"\neffect = \"read\"\ncommand = [\"sh\", \"-c\", \"echo out; echo err >&2; exit 3\"]\n");
        let program = remote_program(&m.actions[0], &[], Path::new("/x"), &Values::default(), "n5").unwrap();
        let (stdout, stderr) = run_sh(&program);
        assert_eq!(stdout, "out\n");
        assert_eq!(split_exit(&stderr, "n5"), ("err".to_string(), Some(3)));
        // A missing requirement stops it before it runs, with a reason.
        let program = remote_program(&m.actions[0], &["crow-no-such-tool".to_string()], Path::new("/x"), &Values::default(), "n6").unwrap();
        let (stdout, stderr) = run_sh(&program);
        assert_eq!(stdout, "");
        assert_eq!(split_exit(&stderr, "n6"), ("needs crow-no-such-tool on this server".to_string(), Some(127)));
        // A marker with another nonce isn't believed.
        assert_eq!(split_exit("CROW_EXIT_zz=0\n", "n5").1, None);
    }

    /// Processes whose command line is exactly `argv`.
    fn running(argv: &[&str]) -> usize {
        let want = argv.join("\0") + "\0";
        std::fs::read_dir("/proc").unwrap().filter_map(Result::ok).filter(|e| std::fs::read(e.path().join("cmdline")).is_ok_and(|c| c == want.as_bytes())).count()
    }

    #[test]
    fn stop_ends_the_action_and_what_it_started() {
        use std::sync::atomic::{AtomicBool, Ordering};
        // A parent that started a child: both must go.
        let m = manifest("[[action]]\nid = \"slow\"\nlabel = \"S\"\non = \"server\"\neffect = \"change\"\ncommand = [\"sh\", \"-c\", \"sleep 299.5 & sleep 299.6\"]\n");
        let program = remote_program(&m.actions[0], &[], Path::new("/x"), &Values::default(), "n7").unwrap();
        let stop = AtomicBool::new(false);
        let host = crate::host::LocalHost;
        let watcher = std::thread::scope(|scope| {
            let stop = &stop;
            scope.spawn(move || {
                for _ in 0..100 {
                    if running(&["sleep", "299.6"]) == 1 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                stop.store(true, Ordering::Relaxed);
            });
            super::run_streaming(&host, &program, false, std::time::Duration::from_secs(60), "n7", &mut |_, _| {}, stop)
        });
        assert!(watcher.stopped, "{watcher:?}");
        assert!(watcher.stdout.is_empty(), "nothing printed before STOP: {watcher:?}");
        assert_eq!(watcher.error.as_deref(), Some("stopped"));
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert_eq!((running(&["sleep", "299.5"]), running(&["sleep", "299.6"])), (0, 0), "the action outlived STOP");
    }

    #[test]
    fn missing_values_are_errors_not_blanks() {
        let t = Template::parse("{input.path}").unwrap();
        assert!(fill(&t, &Values::default()).unwrap_err().contains("{input.path} has no value"));
    }

    #[test]
    fn values_are_checked_against_their_field() {
        let m = manifest("[[action]]\nid = \"vv\"\nlabel = \"V\"\non = \"server\"\neffect = \"read\"\ncommand = [\"true\"]\n[[action.input]]\nkey = \"n\"\nlabel = \"N\"\ntype = \"text\"\n[[action.input]]\nkey = \"p\"\nlabel = \"P\"\ntype = \"text\"\ncharset = \"path\"\nrequired = true\n[[action.input]]\nkey = \"k\"\nlabel = \"K\"\ntype = \"number\"\nmin = 1\nmax = 10\n[[action.input]]\nkey = \"c\"\nlabel = \"C\"\ntype = \"choices\"\nchoices = [\"a\", \"b\"]\n");
        let f = |k: &str| m.actions[0].inputs.iter().find(|f| f.key == k).unwrap().clone();
        assert_eq!(check_value(&f("n"), "web-1.prod"), Ok("web-1.prod".into()));
        assert!(check_value(&f("n"), "a/b").is_err());
        assert!(check_value(&f("n"), "a;b").is_err());
        assert_eq!(check_value(&f("p"), "/var/lib/x"), Ok("/var/lib/x".into()));
        assert!(check_value(&f("p"), "").unwrap_err().contains("needed"));
        assert!(check_value(&f("p"), "/x\n/y").is_err());
        assert_eq!(check_value(&f("k"), "4"), Ok("4".into()));
        assert!(check_value(&f("k"), "11").is_err());
        assert_eq!(check_value(&f("c"), "b, a"), Ok("b,a".into()));
        assert!(check_value(&f("c"), "a,z").is_err());
    }

    #[test]
    fn settings_come_from_storage_then_defaults() {
        let m = manifest("[[setting]]\nkey = \"hours\"\nlabel = \"Hours\"\ntype = \"number\"\ndefault = 26\n[[setting]]\nkey = \"repo\"\nlabel = \"Repo\"\ntype = \"text\"\ncharset = \"path\"\nrequired = true\n");
        let (plain, secret, missing) = settings_values(&m.settings, &BTreeMap::new());
        assert_eq!(plain.get("hours").map(String::as_str), Some("26"));
        assert!(secret.is_empty());
        assert_eq!(missing, vec![("repo".to_string(), "Repo".to_string())]);
        let stored = BTreeMap::from([("repo".to_string(), "/srv/restic".to_string()), ("token".to_string(), "t".to_string())]);
        let (plain, secret, missing) = settings_values(&m.settings, &stored);
        assert_eq!((plain.get("repo").map(String::as_str), secret.get("token").map(String::as_str), missing.len()), (Some("/srv/restic"), Some("t"), 0));
    }

    #[test]
    fn json_tables_and_kv_and_status() {
        let restic = include_str!("../../plugins/restic-check/plugin.toml");
        let m = manifest::parse(restic).unwrap();
        let snapshots = m.actions.iter().find(|a| a.id == "snapshots").unwrap();
        let out = r#"[{"time":"2026-10-13T02:00:00Z","hostname":"web-1","paths":["/etc","/srv"],"short_id":"4f2a","tags":null}]"#;
        let (cols, rows) = table(&snapshots.output, out).unwrap();
        assert_eq!(cols, vec!["Time", "Host", "Paths", "ID"]);
        assert_eq!(rows, vec![vec!["2026-10-13T02:00:00Z", "web-1", "/etc, /srv", "4f2a"]]);
        assert!(table(&snapshots.output, "not json").unwrap_err().contains("not JSON"));

        let mut whitespace = snapshots.output.clone();
        whitespace.format = manifest::OutputFormat::Table;
        whitespace.columns.clear();
        let (cols, rows) = table(&whitespace, "Filesystem Size Mounted\n/dev/sda1      20G  /\ntmpfs  1G   /run/user 1000\n").unwrap();
        assert_eq!(cols, vec!["Filesystem", "Size", "Mounted"]);
        assert_eq!(rows[1], vec!["tmpfs", "1G", "/run/user 1000"]);

        assert_eq!(kv(&snapshots.output, r#"{"os":"Ubuntu","n":3,"up":true}"#).unwrap(), vec![("os".into(), "Ubuntu".into()), ("n".into(), "3".into()), ("up".into(), "true".into())]);
        assert_eq!(status(r#"{"state":"warn","label":"last backup 30h ago"}"#).unwrap().state, State::Warn);
        assert!(status(r#"{"state":"great"}"#).is_err());
    }

    #[test]
    fn summaries() {
        let m = manifest("[[action]]\nid = \"cc\"\nlabel = \"C\"\non = \"server\"\neffect = \"read\"\ncommand = [\"true\"]\n[action.output]\nsummary_prefix = \"no errors\"\n");
        let r = RunResult { ok: true, exit_code: Some(0), stdout: "checking\nno errors were found\n".into(), ..Default::default() };
        assert_eq!(summary(&m.actions[0].output, &r), "no errors were found");
        let r = RunResult { error: Some("host unreachable: timed out\nmore".into()), ..Default::default() };
        assert_eq!(summary(&m.actions[0].output, &r), "host unreachable: timed out");
        let r = RunResult { exit_code: Some(1), stderr: "Fatal: wrong password\n".into(), ..Default::default() };
        assert_eq!(summary(&m.actions[0].output, &r), "Fatal: wrong password");
        let lines = manifest::Output::default();
        let r = RunResult { exit_code: Some(1), stdout: "making /etc/motd.d\n".into(), stderr: "mkdir: can't create directory '/etc/motd.d': File exists\n".into(), ..Default::default() };
        assert_eq!(summary(&lines, &r), "mkdir: can't create directory '/etc/motd.d': File exists");
        assert_eq!(quote("it's"), "'it'\\''s'");
    }
}

