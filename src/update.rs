//! Knowing about, and optionally installing, new versions of Crow (ERR-88).
//!
//! **The check** (Settings → General, on by default): one plain request for
//! the release list on GitHub, with nothing but a User-Agent naming the
//! version. No telemetry. Offline, rate-limited or not found: nothing is
//! said, Crow tries again later. Stable releases only, unless this build is
//! itself a pre-release: then newer pre-releases are offered too.
//!
//! **Installing** (opt-in): the release's archive and its SLSA provenance
//! (`.intoto.jsonl`) are downloaded and checked with `slsa-verifier`, which
//! verifies the provenance's signature, that it was built from this repo at
//! that tag, and the archive's SHA-256. Without `slsa-verifier`, or if it
//! says no, nothing is installed and the user is told why: a hash checked
//! against unsigned provenance proves nothing. Only an installed copy is
//! replaced (`~/.local/bin/crow`, `Crow.app`), never a build run from a
//! source tree. Crow offers a restart; it never restarts on its own.

use std::path::{Path, PathBuf};
use std::process::Command;

pub const REPO: &str = "errorware/crow";
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub version: semver::Version,
    pub prerelease: bool,
    /// The release page, with its notes.
    pub url: String,
    /// (file name, download URL)
    pub assets: Vec<(String, String)>,
}

/// GitHub's `GET /repos/{repo}/releases` JSON. Drafts and tags that aren't
/// versions are left out.
pub fn parse_releases(json: &str) -> Result<Vec<Release>, String> {
    let list: Vec<serde_json::Value> = serde_json::from_str(json).map_err(|e| format!("unexpected answer from GitHub: {e}"))?;
    Ok(list
        .iter()
        .filter(|r| !r["draft"].as_bool().unwrap_or(false))
        .filter_map(|r| {
            let tag = r["tag_name"].as_str()?.to_string();
            let version = semver::Version::parse(tag.trim_start_matches('v')).ok()?;
            Some(Release {
                prerelease: r["prerelease"].as_bool().unwrap_or(false) || !version.pre.is_empty(),
                version,
                url: r["html_url"].as_str().unwrap_or_default().to_string(),
                assets: r["assets"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|x| Some((x["name"].as_str()?.to_string(), x["browser_download_url"].as_str()?.to_string()))).collect())
                    .unwrap_or_default(),
                tag,
            })
        })
        .collect())
}

/// The newest release worth offering: newer than `current`, stable unless
/// pre-releases are wanted (or `current` is one), not skipped.
pub fn newest(current: &str, releases: &[Release], include_pre: bool, skipped: Option<&str>) -> Option<Release> {
    let current = semver::Version::parse(current).ok()?;
    let pre_ok = include_pre || !current.pre.is_empty();
    releases
        .iter()
        .filter(|r| r.version > current && (pre_ok || !r.prerelease) && skipped != Some(r.tag.as_str()))
        .max_by(|a, b| a.version.cmp(&b.version))
        .cloned()
}

fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--proto", "=https", "--max-time", "300", "-A", concat!("crow/", env!("CARGO_PKG_VERSION"))])
        .args(args)
        .output()
        .map_err(|e| format!("curl couldn't run: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// The release list; `Ok(empty)` when there's nothing to learn (offline,
/// rate-limited, repository not public).
pub fn fetch_releases() -> Vec<Release> {
    curl(&["-H", "Accept: application/vnd.github+json", &format!("https://api.github.com/repos/{REPO}/releases?per_page=20")])
        .ok()
        .and_then(|body| parse_releases(&String::from_utf8_lossy(&body)).ok())
        .unwrap_or_default()
}

/// This platform's release archive.
pub fn asset_name() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("crow-linux-x86_64.tar.gz"),
        ("macos", "aarch64") => Some("crow-macos-arm64.tar.gz"),
        _ => None,
    }
}

/// Where this copy of Crow is installed, if it's an installed copy: the
/// binary in `~/.local/bin` (or `$XDG_BIN_HOME`), or the `Crow.app` bundle.
pub fn installed_target(exe: &Path) -> Result<PathBuf, String> {
    if std::env::consts::OS == "macos" {
        return exe
            .ancestors()
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("this Crow runs from {} rather than an installed Crow.app, so it isn't replaced", exe.display()));
    }
    let bin = std::env::var_os("XDG_BIN_HOME").map(PathBuf::from).or_else(|| dirs::home_dir().map(|h| h.join(".local/bin"))).ok_or("no home directory")?;
    let exe = exe.canonicalize().unwrap_or_else(|_| exe.to_path_buf());
    let installed = bin.join("crow");
    if installed.canonicalize().ok().as_deref() == Some(exe.as_path()) {
        Ok(installed)
    } else {
        Err(format!("this Crow runs from {} rather than the installed {}, so it isn't replaced (update your build from source instead)", exe.display(), installed.display()))
    }
}

/// Downloads, verifies and installs `release`. Returns what to tell the
/// user; the new version runs after a restart.
pub fn install(release: &Release) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let target = installed_target(&exe)?;
    let name = asset_name().ok_or("there's no release build for this platform")?;
    let find = |pred: &dyn Fn(&str) -> bool| release.assets.iter().find(|(n, _)| pred(n)).map(|(_, u)| u.clone());
    let archive_url = find(&|n| n == name).ok_or_else(|| format!("{} has no {name}", release.tag))?;
    let provenance_url = find(&|n| n.ends_with(".intoto.jsonl")).ok_or_else(|| format!("{} has no provenance (.intoto.jsonl), so it can't be verified", release.tag))?;
    let verifier = which("slsa-verifier").ok_or("installing updates needs slsa-verifier (github.com/slsa-framework/slsa-verifier) to check the release's signed provenance; it isn't on PATH. Nothing was installed")?;

    let dir = std::env::temp_dir().join(format!("crow-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let result = (|| {
        let archive = dir.join(name);
        let provenance = dir.join("provenance.intoto.jsonl");
        curl(&["-o", &archive.to_string_lossy(), &archive_url])?;
        curl(&["-o", &provenance.to_string_lossy(), &provenance_url])?;
        let out = Command::new(&verifier)
            .args(["verify-artifact", &archive.to_string_lossy(), "--provenance-path", &provenance.to_string_lossy()])
            .args(["--source-uri", &format!("github.com/{REPO}"), "--source-tag", &release.tag])
            .output()
            .map_err(|e| format!("slsa-verifier couldn't run: {e}"))?;
        if !out.status.success() {
            return Err(format!("the release didn't verify, so nothing was installed: {}", String::from_utf8_lossy(&out.stderr).trim().lines().last().unwrap_or("verification failed")));
        }
        let unpacked = dir.join("unpacked");
        std::fs::create_dir_all(&unpacked).map_err(|e| e.to_string())?;
        run(Command::new("tar").arg("-xzf").arg(&archive).arg("-C").arg(&unpacked))?;
        if std::env::consts::OS == "macos" {
            swap_app(&unpacked.join("Crow.app"), &target)
        } else {
            // The release's own installer: binary, desktop entry, icons.
            let installer = unpacked.join(name.trim_end_matches(".tar.gz")).join("install.sh");
            run(Command::new("sh").arg(&installer))
        }
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result.map(|()| format!("Crow {} is installed (verified). Restart Crow to use it.", release.version))
}

/// Replaces `target` (an .app) with `new`: the old bundle is moved aside
/// first and put back if the move fails.
fn swap_app(new: &Path, target: &Path) -> Result<(), String> {
    let aside = target.with_extension("app.old");
    let _ = std::fs::remove_dir_all(&aside);
    std::fs::rename(target, &aside).map_err(|e| format!("couldn't move the old Crow.app aside: {e}"))?;
    if let Err(e) = run(Command::new("mv").arg(new).arg(target)) {
        let _ = std::fs::rename(&aside, target);
        return Err(e);
    }
    let _ = std::fs::remove_dir_all(&aside);
    Ok(())
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let out = cmd.output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn which(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| std::env::split_paths(&paths).map(|p| p.join(program)).find(|p| p.is_file()))
}

/// Starts the installed Crow and leaves; the caller quits right after.
pub fn restart_into_installed() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let target = installed_target(&exe)?;
    let program = if std::env::consts::OS == "macos" { target.join("Contents/MacOS/crow") } else { target };
    Command::new(program).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASES: &str = r#"[
      {"tag_name": "v0.3.0-beta.1", "draft": false, "prerelease": true, "html_url": "https://github.com/errorware/crow/releases/tag/v0.3.0-beta.1", "assets": []},
      {"tag_name": "v0.2.1", "draft": false, "prerelease": false, "html_url": "https://github.com/errorware/crow/releases/tag/v0.2.1",
       "assets": [{"name": "crow-linux-x86_64.tar.gz", "browser_download_url": "https://github.com/errorware/crow/releases/download/v0.2.1/crow-linux-x86_64.tar.gz"},
                  {"name": "crow.intoto.jsonl", "browser_download_url": "https://github.com/errorware/crow/releases/download/v0.2.1/crow.intoto.jsonl"}]},
      {"tag_name": "v0.4.0", "draft": true, "prerelease": false, "html_url": "", "assets": []},
      {"tag_name": "nightly", "draft": false, "prerelease": true, "html_url": "", "assets": []},
      {"tag_name": "v0.2.0", "draft": false, "prerelease": false, "html_url": "", "assets": []}
    ]"#;

    #[test]
    fn offers_the_newest_stable_unless_running_a_pre_release() {
        let all = parse_releases(RELEASES).unwrap();
        assert_eq!(all.len(), 3, "draft and non-version tags left out");
        assert_eq!(newest("0.2.0", &all, false, None).unwrap().tag, "v0.2.1");
        assert_eq!(newest("0.2.0", &all, true, None).unwrap().tag, "v0.3.0-beta.1");
        assert_eq!(newest("0.2.0-beta.1", &all, false, None).unwrap().tag, "v0.3.0-beta.1", "beta builds see newer betas");
        assert_eq!(newest("0.2.1", &all, false, None), None);
        assert_eq!(newest("0.2.0", &all, false, Some("v0.2.1")), None, "skipped");
        assert_eq!(all[1].assets.len(), 2);
    }

    #[test]
    fn only_an_installed_copy_is_replaced() {
        let dev = Path::new("/home/someone/src/crow/target/debug/crow");
        assert!(installed_target(dev).unwrap_err().contains("isn't replaced"));
    }

    #[test]
    fn garbage_from_github_is_an_error_not_a_crash() {
        assert!(parse_releases("{\"message\": \"Not Found\"}").is_err());
    }
}
