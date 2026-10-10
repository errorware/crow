//! Isolation and helpers for workflow tests.

use std::collections::HashMap;
use std::path::PathBuf;

use gpui_kit::test::TestWindowExt;
use gpui_kit::{px, size, AnyWindowHandle, AppContext, Context, Entity, TestAppContext, Window};

use crate::app::CrowApp;

/// The scratch directory, or None when workflows aren't being run.
pub fn root() -> Option<PathBuf> {
    std::env::var("CROW_WORKFLOWS").ok().map(PathBuf::from)
}

/// Points Crow at the scratch profile: its own HOME (config.toml, ~/.ssh),
/// XDG dirs (crow.db), no keyring, no agent, and an `ssh` that reads the
/// profile's known_hosts (ssh itself reads the passwd home, not $HOME).
/// Must run before Crow reads any of them.
pub fn isolate() -> PathBuf {
    let root = root().expect("CROW_WORKFLOWS");
    for d in ["home/.ssh", "home/.config/crow", "xdg/config", "xdg/data", "xdg/cache", "bin"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::env::set_var("HOME", root.join("home"));
    std::env::set_var("XDG_CONFIG_HOME", root.join("xdg/config"));
    std::env::set_var("XDG_DATA_HOME", root.join("xdg/data"));
    std::env::set_var("XDG_CACHE_HOME", root.join("xdg/cache"));
    std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/nonexistent");
    std::env::remove_var("SSH_AUTH_SOCK");
    // Password logins: ssh runs Crow as its askpass helper; the test runner
    // can't be that, the debug build of Crow can.
    let crow = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/crow");
    assert!(crow.exists(), "build Crow first (cargo build): password logins need it as askpass");
    std::env::set_var("CROW_TEST_ASKPASS", crow);
    let path = std::env::var("PATH").unwrap_or_default();
    let bin = root.join("bin").display().to_string();
    if !path.starts_with(&bin) {
        std::env::set_var("PATH", format!("{bin}:{path}"));
    }
    root
}

/// `targets.env` in the scratch dir: NAME=address lines.
pub fn targets() -> HashMap<String, String> {
    let text = std::fs::read_to_string(root().unwrap().join("targets.env")).expect("targets.env");
    text.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.trim().to_string(), v.trim().to_string())).collect()
}

/// Runs a command on a target as root with the test key, outside Crow: to
/// set up a scenario or check what Crow did.
pub fn ssh(addr: &str, cmd: &str) -> (bool, String) {
    let root = root().unwrap();
    let out = std::process::Command::new("/usr/bin/ssh")
        .args(["-F", "/dev/null", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", "-o", "IdentitiesOnly=yes", "-o", "ConnectTimeout=10"])
        .arg("-o")
        .arg(format!("UserKnownHostsFile={}", root.join("outside_known_hosts").display()))
        .arg("-i")
        .arg(root.join("home/.ssh/id_ed25519"))
        .arg(format!("root@{addr}"))
        .arg(cmd)
        .output()
        .expect("ssh");
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// Like `ssh`, but through the bastion (for servers that only let it in).
pub fn ssh_via(bastion: &str, addr: &str, cmd: &str) -> (bool, String) {
    let root = root().unwrap();
    let key = root.join("home/.ssh/id_ed25519");
    let kh = root.join("outside_known_hosts");
    let common = format!("-F /dev/null -o BatchMode=yes -o StrictHostKeyChecking=no -o IdentitiesOnly=yes -o UserKnownHostsFile={} -i {}", kh.display(), key.display());
    let out = std::process::Command::new("/usr/bin/ssh")
        .args(common.split(' '))
        .arg("-o")
        .arg(format!("ProxyCommand=/usr/bin/ssh {common} -W %h:%p root@{bastion}"))
        .arg(format!("root@{addr}"))
        .arg(cmd)
        .output()
        .expect("ssh");
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// The server Crow has under `name`.
pub fn server(cx: &mut TestAppContext, app: &Entity<CrowApp>, name: &str) -> crate::vault::ServerRecord {
    app.read_with(cx, |this, _| this.fleet.servers.iter().find(|s| s.name == name).cloned()).unwrap_or_else(|| panic!("{name} isn't enrolled"))
}

/// Runs a command on a server the way Crow does (its transport, bastions
/// included).
pub fn crow_exec(srv: &crate::vault::ServerRecord, cmd: &str) -> Result<String, String> {
    crate::host::host_for(srv).exec(&["sh", "-c", cmd], std::time::Duration::from_secs(30)).map(|o| o.stdout).map_err(|e| e.to_string())
}

/// Opens Crow in a headless window.
pub fn open_crow(cx: &mut TestAppContext) -> (Entity<CrowApp>, AnyWindowHandle) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::init_obsidian_theme(cx);
    });
    let handle = cx.open_window(size(px(1400.), px(900.)), |window, cx| CrowApp::new(window, cx));
    let app = handle.update(cx, |_, _, cx| cx.entity()).unwrap();
    (app, handle.into())
}

/// Lets Crow's background work finish, then draws a frame (which also
/// proves the screen renders without panicking).
pub fn settle(cx: &mut TestAppContext, w: AnyWindowHandle) {
    cx.run_until_parked();
    cx.update_window(w, |_, window, cx| window.render_frame(cx)).unwrap();
    cx.run_until_parked();
}

/// Calls into Crow with its window, like a click handler.
pub fn act<R>(cx: &mut TestAppContext, app: &Entity<CrowApp>, w: AnyWindowHandle, f: impl FnOnce(&mut CrowApp, &mut Window, &mut Context<CrowApp>) -> R) -> R {
    let app = app.clone();
    let r = cx.update_window(w, |_, window, cx| app.update(cx, |this, cx| f(this, window, cx))).unwrap();
    settle(cx, w);
    r
}

/// Prints a step to the test log.
pub fn step(text: &str) {
    eprintln!("  · {text}");
}
