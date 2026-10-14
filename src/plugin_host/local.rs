//! Plugin actions that run on this machine (ERR-180): a setup step, and
//! local actions that reach servers themselves (Ansible).
//!
//! A local action runs in the plugin's data folder with a cleared
//! environment: PATH, a HOME and TMPDIR inside the data folder, and what the
//! action declares. It gets the servers it's for as an Ansible-style
//! inventory plus an `ssh_config` carrying Crow's own options, key file,
//! users, ports and bastions; the key's contents never leave Crow. Its
//! files go in a run folder that's removed afterwards.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::host::{HostError, SshHost};
use crate::vault::ServerRecord;

use super::manifest::{Interpreter, Template};
use super::run::{fill, RunResult, Values, MAX_OUTPUT_BYTES};

/// Where a plugin may write: its venv, caches, a HOME of its own.
pub fn data_dir(id: &str) -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".local/share")).join("crow").join("plugins").join(id)
}

/// The paths a local action's placeholders stand for.
#[derive(Debug, Clone, Default)]
pub struct LocalPaths {
    pub plugin_dir: PathBuf,
    pub data_dir: PathBuf,
    pub inventory: PathBuf,
    pub ssh_config: PathBuf,
    pub results: PathBuf,
}

/// The name a server goes by in the inventory and `ssh_config`.
pub fn alias(s: &ServerRecord) -> String {
    let ok = !s.name.is_empty() && s.name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if ok { s.name.clone() } else { format!("crow-{}", s.id) }
}

/// Writes `inventory` and `ssh_config` for `servers` into `run_dir`.
pub fn write_targets(run_dir: &Path, servers: &[ServerRecord]) -> Result<(PathBuf, PathBuf), String> {
    let ssh_config = run_dir.join("ssh_config");
    let inventory = run_dir.join("inventory");
    let mut config = String::from("# Written by Crow for one run; removed afterwards.\n");
    let mut hosts = String::from("[crow]\n");
    for s in servers {
        if s.tags.iter().any(|t| t == "test-node") {
            return Err(t!("plugin-local-lab-container", name = s.name));
        }
        let a = alias(s);
        let block = SshHost::for_server(s).ssh_config_block(&a).map_err(|e| format!("{}: {e}", s.name))?;
        config.push_str(&block);
        // A bastion: other servers are reached through it (a plugin that
        // hardens ssh must keep its forwarding on).
        let bastion = if crate::host::ssh::is_jump_target(&s.id) { "yes" } else { "no" };
        hosts.push_str(&format!("{a} ansible_host={a} ansible_user={} crow_server_id={} crow_server_name={} crow_bastion={bastion}\n", ini_value(&s.login_user), ini_value(&s.id), ini_value(&s.name)));
    }
    hosts.push_str(&format!("\n[all:vars]\nansible_ssh_common_args=\"-F {}\"\n", ssh_config.display()));
    write_private(&ssh_config, &config)?;
    write_private(&inventory, &hosts)?;
    Ok((inventory, ssh_config))
}

fn ini_value(v: &str) -> String {
    if v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@')) { v.to_string() } else { format!("\"{}\"", v.replace('"', "")) }
}

fn write_private(path: &Path, text: &str) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    f.write_all(text.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))
}

/// The commands in `requires` that aren't on PATH here.
pub fn missing_here(requires: &[String]) -> Vec<String> {
    let path = std::env::var("PATH").unwrap_or_default();
    requires
        .iter()
        .filter(|c| !std::env::split_paths(&path).any(|d| {
            let p = d.join(c);
            std::fs::metadata(&p).is_ok_and(|m| m.is_file() && std::os::unix::fs::PermissionsExt::mode(&m.permissions()) & 0o111 != 0)
        }))
        .cloned()
        .collect()
}

/// A local action's environment: nothing of Crow's own but PATH and LANG.
pub fn environment(data_dir: &Path, run_dir: &Path, declared: &BTreeMap<String, Template>, v: &Values) -> Result<Vec<(String, String)>, String> {
    let mut env = vec![
        ("PATH".to_string(), std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".into())),
        ("HOME".to_string(), data_dir.display().to_string()),
        ("TMPDIR".to_string(), run_dir.join("tmp").display().to_string()),
        ("LANG".to_string(), "C.UTF-8".to_string()),
        ("TERM".to_string(), "dumb".to_string()),
        // Ansible prints for people, not colours for a terminal.
        ("ANSIBLE_NOCOLOR".to_string(), "1".to_string()),
        ("ANSIBLE_FORCE_COLOR".to_string(), "0".to_string()),
        ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
        // Ansible's ssh sockets go in Crow's own short, private folder: under
        // HOME (the data folder) the path can pass the 108-byte limit.
        ("ANSIBLE_SSH_CONTROL_PATH_DIR".to_string(), crate::host::ssh::control_dir().join("ansible").display().to_string()),
    ];
    for (k, t) in declared {
        env.retain(|(name, _)| name != k);
        env.push((k.clone(), fill(t, v)?));
    }
    Ok(env)
}

/// The argv of a local action or setup: its command, or its script's
/// interpreter, the script in the plugin's folder, and its args.
pub fn argv(command: Option<&[Template]>, script: Option<&str>, interpreter: Option<Interpreter>, args: &[Template], plugin_dir: &Path, v: &Values) -> Result<Vec<String>, String> {
    match (command, script) {
        (Some(c), _) => c.iter().map(|t| fill(t, v)).collect(),
        (None, Some(s)) => {
            let mut out = vec![interpreter.unwrap_or(Interpreter::Sh).program().to_string(), plugin_dir.join(s).display().to_string()];
            for t in args {
                out.push(fill(t, v)?);
            }
            Ok(out)
        }
        (None, None) => Err("nothing to run".into()),
    }
}

/// Runs `argv` in `cwd` with exactly `env`, in its own process group.
pub fn run(argv: &[String], cwd: &Path, env: &[(String, String)], timeout: Duration, sink: &mut dyn FnMut(bool, &str), stop: &std::sync::atomic::AtomicBool) -> RunResult {
    let started_at = chrono::Utc::now().timestamp();
    let (mut out, mut err) = (String::new(), String::new());
    let mut on_line = |is_err: bool, line: &str| {
        sink(is_err, line);
        let into = if is_err { &mut err } else { &mut out };
        if into.len() < MAX_OUTPUT_BYTES {
            into.push_str(line);
            into.push('\n');
        }
    };
    let words: Vec<&str> = argv.iter().map(String::as_str).collect();
    let prepare = |c: &mut std::process::Command| {
        c.env_clear().envs(env.iter().map(|(k, v)| (k.as_str(), v.as_str()))).current_dir(cwd);
    };
    let result = crate::host::run_streaming_with(&words, &[], timeout, &prepare, &mut on_line, stop);
    let finished_at = chrono::Utc::now().timestamp();
    let stopped = stop.load(std::sync::atomic::Ordering::Relaxed) || matches!(result, Err(HostError::Timeout));
    let truncated = out.len() >= MAX_OUTPUT_BYTES || err.len() >= MAX_OUTPUT_BYTES;
    match result {
        Ok(_) => RunResult { ok: true, exit_code: Some(0), stdout: out, stderr: err, truncated, started_at, finished_at, ..Default::default() },
        Err(HostError::Failed { status, .. }) => RunResult { ok: false, exit_code: Some(status), stdout: out, stderr: err, truncated, started_at, finished_at, ..Default::default() },
        Err(e) => {
            let error = match e {
                _ if stop.load(std::sync::atomic::Ordering::Relaxed) => "stopped".to_string(),
                HostError::Timeout => format!("timed out after {}s, and was stopped", timeout.as_secs()),
                HostError::Io(m) if m.contains("No such file") => format!("{}: not found", argv.first().cloned().unwrap_or_default()),
                other => other.to_string(),
            };
            RunResult { error: Some(error), stdout: out, stderr: err, truncated, started_at, finished_at, stopped, ..Default::default() }
        }
    }
}

/// The per-server lines a local action wrote to `{results}`: one JSON
/// object each, with at least `"host"`.
pub fn read_results(path: &Path) -> Vec<serde_json::Value> {
    use std::io::Read;
    // At most 4 MB: a plugin can't make Crow read a file of any size.
    let mut text = String::new();
    if let Ok(f) = std::fs::File::open(path) {
        let _ = f.take(4 * 1024 * 1024).read_to_string(&mut text);
    }
    text.lines()
        .take(10_000)
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l.trim()).ok())
        .filter(|v| v.get("host").and_then(|h| h.as_str()).is_some())
        .collect()
}

/// A fresh run folder in the plugin's data folder.
pub fn run_dir(data_dir: &Path, nonce: &str) -> Result<PathBuf, String> {
    let dir = data_dir.join("runs").join(nonce);
    std::fs::create_dir_all(dir.join("tmp")).map_err(|e| format!("{}: {e}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::{alias, argv, environment, missing_here, read_results, run, run_dir, write_targets, LocalPaths};
    use crate::plugin_host::manifest::Template;
    use crate::plugin_host::run::Values;
    use crate::vault::ServerRecord;

    fn temp(case: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("crow-plugin-local-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_environment_is_cleared_and_home_is_the_data_folder() {
        let data = temp("env");
        let rd = run_dir(&data, "n1").unwrap();
        std::env::set_var("CROW_TEST_LEAK", "should-not-pass");
        let declared = BTreeMap::from([("GREETING".to_string(), Template::parse("hi {input.who}").unwrap())]);
        let v = Values { inputs: BTreeMap::from([("who".into(), "there".into())]), ..Default::default() };
        let env = environment(&data, &rd, &declared, &v).unwrap();
        let r = run(&["sh".into(), "-c".into(), "env; pwd".into()], &data, &env, std::time::Duration::from_secs(5), &mut |_, _| {}, &Default::default());
        assert!(r.ok, "{r:?}");
        assert!(!r.stdout.contains("CROW_TEST_LEAK"), "{}", r.stdout);
        assert!(r.stdout.contains(&format!("HOME={}", data.display())));
        assert!(r.stdout.contains("GREETING=hi there"));
        assert!(r.stdout.trim_end().ends_with(&data.display().to_string()), "runs in the data folder");
    }

    #[test]
    fn exit_codes_and_missing_programs() {
        let data = temp("exit");
        let r = run(&["sh".into(), "-c".into(), "echo out; echo err >&2; exit 4".into()], &data, &[("PATH".into(), std::env::var("PATH").unwrap())], std::time::Duration::from_secs(5), &mut |_, _| {}, &Default::default());
        assert_eq!((r.ok, r.exit_code, r.stdout.as_str(), r.stderr.as_str()), (false, Some(4), "out\n", "err\n"));
        let r = run(&["crow-no-such-program".into()], &data, &[], std::time::Duration::from_secs(5), &mut |_, _| {}, &Default::default());
        assert!(r.error.is_some() && !r.ok, "{r:?}");
        assert_eq!(missing_here(&["sh".to_string(), "crow-no-such-program".to_string()]), vec!["crow-no-such-program".to_string()]);
    }

    #[test]
    fn stop_ends_the_whole_group() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let data = temp("stop");
        let stop = AtomicBool::new(false);
        let started = std::time::Instant::now();
        let mut n = 0;
        let r = run(&["sh".into(), "-c".into(), "sleep 298.5 & while true; do echo tick; sleep 0.1; done".into()], &data, &[("PATH".into(), std::env::var("PATH").unwrap())], std::time::Duration::from_secs(30), &mut |_, _| {
            n += 1;
            if n == 3 {
                stop.store(true, Ordering::Relaxed);
            }
        }, &stop);
        assert!(r.stopped && started.elapsed() < std::time::Duration::from_secs(6), "{r:?}");
        std::thread::sleep(std::time::Duration::from_millis(300));
        let left = std::fs::read_dir("/proc").unwrap().filter_map(Result::ok).filter(|e| std::fs::read(e.path().join("cmdline")).is_ok_and(|c| c == b"sleep\0298.5\0")).count();
        assert_eq!(left, 0, "a child outlived STOP");
    }

    #[test]
    fn placeholders_for_local_paths_and_scripts() {
        let paths = LocalPaths { plugin_dir: "/p".into(), data_dir: "/d".into(), inventory: "/r/inventory".into(), ssh_config: "/r/ssh_config".into(), results: "/r/results".into() };
        let v = Values { local: Some(paths), ..Default::default() };
        let cmd = vec![Template::parse("{data_dir}/venv/bin/python").unwrap(), Template::parse("{plugin_dir}/run.py").unwrap(), Template::parse("-i").unwrap(), Template::parse("{inventory}").unwrap()];
        assert_eq!(argv(Some(&cmd), None, None, &[], Path::new("/p"), &v).unwrap(), vec!["/d/venv/bin/python", "/p/run.py", "-i", "/r/inventory"]);
        let args = vec![Template::parse("{results}").unwrap()];
        assert_eq!(argv(None, Some("x.py"), Some(crate::plugin_host::manifest::Interpreter::Python3), &args, Path::new("/p"), &v).unwrap(), vec!["python3", "/p/x.py", "/r/results"]);
    }

    #[test]
    fn targets_are_written_without_the_key_and_lab_containers_are_refused() {
        let dir = temp("targets");
        let key = dir.join("id_test");
        std::fs::write(&key, "-----BEGIN OPENSSH PRIVATE KEY-----\nSECRETKEYBYTES\n-----END OPENSSH PRIVATE KEY-----\n").unwrap();
        crate::host::update_directory(&[], &[]);
        let s = ServerRecord { id: "s1".into(), name: "web-1".into(), host: "10.0.0.5".into(), port: 2222, login_user: "deploy".into(), auth_method: "agent".into(), ..Default::default() };
        let (inv, cfg) = write_targets(&dir, &[s.clone()]).unwrap();
        let config = std::fs::read_to_string(&cfg).unwrap();
        assert!(config.contains("Host web-1\n    HostName 10.0.0.5\n") && config.contains("    Port 2222\n") && config.contains("    User deploy\n") && config.contains("    StrictHostKeyChecking yes\n"), "{config}");
        assert!(!config.contains("SECRETKEYBYTES"));
        let inventory = std::fs::read_to_string(&inv).unwrap();
        assert!(inventory.contains("web-1 ansible_host=web-1 ansible_user=deploy crow_server_id=s1 crow_server_name=web-1 crow_bastion=no"), "{inventory}");
        assert!(inventory.contains(&format!("ansible_ssh_common_args=\"-F {}\"", cfg.display())));
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&cfg).unwrap().permissions().mode() & 0o777, 0o600);
        let lab = ServerRecord { tags: vec!["test-node".into()], name: "lab".into(), ..s.clone() };
        assert!(write_targets(&dir, &[lab]).is_err());
        assert_eq!(alias(&ServerRecord { id: "x9".into(), name: "my server".into(), ..Default::default() }), "crow-x9");
    }

    #[test]
    fn results_lines_need_a_host() {
        let dir = temp("results");
        let p = dir.join("results");
        std::fs::write(&p, "{\"host\":\"web-1\",\"ok\":true,\"changed\":2}\nnot json\n{\"nohost\":1}\n{\"host\":\"web-2\",\"ok\":false}\n").unwrap();
        let r = read_results(&p);
        assert_eq!(r.len(), 2);
        assert_eq!(r[1]["host"], "web-2");
        // A huge file is read only so far.
        let big = format!("{}\n", "{\"host\":\"x\",\"pad\":\"".to_string() + &"a".repeat(100) + "\"}").repeat(60_000);
        std::fs::write(&p, big).unwrap();
        assert!(read_results(&p).len() < 60_000);
    }
}
