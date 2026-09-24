//! Pending package updates and kernel state, read from the server's own
//! package manager in one read-only round trip (nothing is installed or
//! refreshed; `apt update` is never run).

use std::collections::HashMap;

/// Marks the start of each section in [`UPDATES_PROBE`]'s output.
const SECTION: &str = "@@crow-sec@@";

/// Read-only probe: OS, kernel, package manager, pending upgrades, reboot
/// flag, and (Debian/Ubuntu) the source packages behind installed and running
/// binaries, for CVE lookups.
pub const UPDATES_PROBE: &str = r#"PATH="$PATH:/usr/sbin:/sbin"
m='@@crow-sec@@'
echo "$m os"; . /etc/os-release 2>/dev/null; echo "id=$ID"; echo "version=$VERSION_ID"
echo "$m kernel"; uname -r
if command -v dpkg-query >/dev/null 2>&1 && command -v apt >/dev/null 2>&1; then
  echo "$m manager"; echo apt
  echo "$m cache"; stat -c %Y /var/lib/apt/lists 2>/dev/null
  echo "$m upgradable"; apt list --upgradable 2>/dev/null | grep -v '^Listing'
  echo "$m reboot"; [ -f /var/run/reboot-required ] && echo yes
  echo "$m packages"; dpkg-query -W -f='${Package}\t${source:Package}\t${Version}\t${source:Version}\n' 2>/dev/null
  echo "$m running"; for e in /proc/[0-9]*/exe; do readlink "$e" 2>/dev/null; done | sed 's/ (deleted)$//' | sort -u | xargs -r dpkg -S 2>/dev/null
  echo "$m kernel-running"; dpkg-query -W -f='${source:Package}\t${source:Version}\n' "linux-image-$(uname -r)" 2>/dev/null
  k=$(ls /boot/vmlinuz-* 2>/dev/null | sort -V | tail -1); k=${k#/boot/vmlinuz-}
  echo "$m kernel-newest"; [ -n "$k" ] && dpkg-query -W -f='${source:Package}\t${source:Version}\n' "linux-image-$k" 2>/dev/null
elif command -v dnf >/dev/null 2>&1; then
  echo "$m manager"; echo dnf
  echo "$m upgradable"; dnf -q check-update 2>/dev/null
  echo "$m security"; dnf -q updateinfo list --security 2>/dev/null
  echo "$m reboot"; needs-restarting -r >/dev/null 2>&1; [ $? -eq 1 ] && echo yes
elif command -v apk >/dev/null 2>&1; then
  echo "$m manager"; echo apk
  echo "$m upgradable"; apk version -l '<' 2>/dev/null | grep -v '^Installed'
fi
true"#;

#[derive(Clone, Debug, PartialEq)]
pub struct PackageUpdate {
    pub name: String,
    pub installed: String,
    pub candidate: String,
    /// From a security pocket/advisory.
    pub security: bool,
}

/// A source package at a version (what OSV indexes Debian/Ubuntu by).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourcePkg {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UpdatesReport {
    pub os_id: String,
    pub os_version: String,
    pub kernel: String,
    /// "apt", "dnf", "apk", or empty when none was found.
    pub manager: String,
    /// When the package lists were last refreshed (unix seconds), if known.
    pub cache_time: Option<i64>,
    pub updates: Vec<PackageUpdate>,
    pub reboot_required: bool,
    /// Debian/Ubuntu: binary package → its source package and version.
    pub sources: HashMap<String, SourcePkg>,
    /// Debian/Ubuntu: source packages behind the running programs.
    pub running: Vec<SourcePkg>,
    /// The running and newest installed kernel, as OSV source packages.
    pub kernel_running: Option<SourcePkg>,
    pub kernel_newest: Option<SourcePkg>,
}

impl UpdatesReport {
    pub fn security_count(&self) -> usize {
        self.updates.iter().filter(|u| u.security).count()
    }

    /// A pending kernel upgrade ("7.0.0-31.31 → 7.0.0-34.34"), and whether it
    /// comes from the security pocket.
    pub fn kernel_update(&self) -> Option<(String, bool)> {
        let u = self.updates.iter().find(|u| u.name.starts_with("linux-image-") || u.name == "kernel" || u.name == "kernel-core" || u.name.starts_with("linux-lts") || u.name == "linux-virt")?;
        Some((format!("{} → {}", if u.installed.is_empty() { "installed" } else { &u.installed }, u.candidate), u.security))
    }

    /// OSV ecosystem for this OS, when CVE lookups are supported.
    pub fn osv_ecosystem(&self) -> Option<String> {
        match self.os_id.as_str() {
            "ubuntu" if !self.os_version.is_empty() => Some(format!("Ubuntu:{}", self.os_version)),
            "debian" if !self.os_version.is_empty() => Some(format!("Debian:{}", self.os_version)),
            _ => None,
        }
    }
}

/// OSV files kernel CVEs under `linux` / `linux-hwe-6.8`, not the signed
/// wrapper packages that `linux-image-*` comes from.
fn kernel_source(line: &str) -> Option<SourcePkg> {
    let (name, version) = line.trim().split_once('\t')?;
    let name = name.strip_prefix("linux-signed").map(|rest| format!("linux{rest}")).unwrap_or_else(|| name.to_string());
    (!name.is_empty() && !version.is_empty()).then(|| SourcePkg { name, version: version.to_string() })
}

fn parse_apt_upgradable(text: &str) -> Vec<PackageUpdate> {
    // "openssl/noble-updates,noble-security 3.0.13-0ubuntu3.5 amd64 [upgradable from: 3.0.13-0ubuntu3.4]"
    text.lines()
        .filter_map(|l| {
            let (name_suites, rest) = l.trim().split_once(' ')?;
            let (name, suites) = name_suites.split_once('/')?;
            let candidate = rest.split_whitespace().next()?.to_string();
            let installed = rest.split("upgradable from: ").nth(1)?.trim_end_matches(']').trim().to_string();
            Some(PackageUpdate { name: name.to_string(), installed, candidate, security: suites.split(',').any(|s| s.ends_with("-security")) })
        })
        .collect()
}

fn parse_dnf_upgradable(text: &str, security: &str) -> Vec<PackageUpdate> {
    // check-update: "openssl-libs.x86_64   1:3.0.7-27.el9   baseos"
    // updateinfo:   "RHSA-2024:1234 Important/Sec. openssl-libs-1:3.0.7-27.el9.x86_64"
    text.lines()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let (name_arch, candidate, _repo) = (parts.next()?, parts.next()?, parts.next()?);
            if parts.next().is_some() || !name_arch.contains('.') {
                return None;
            }
            let name = name_arch.rsplit_once('.').map(|(n, _)| n).unwrap_or(name_arch).to_string();
            let is_sec = security.lines().any(|s| s.split_whitespace().nth(2).is_some_and(|p| p.starts_with(&format!("{name}-"))));
            Some(PackageUpdate { name, installed: String::new(), candidate: candidate.to_string(), security: is_sec })
        })
        .collect()
}

fn parse_apk_upgradable(text: &str) -> Vec<PackageUpdate> {
    // "openssl-3.1.4-r5  < 3.1.4-r6"
    text.lines()
        .filter_map(|l| {
            let (left, candidate) = l.split_once('<')?;
            let left = left.trim();
            // name-version-rN: the version starts at the second-to-last '-'.
            let mut dashes = left.rmatch_indices('-').map(|(i, _)| i);
            let (_, split) = (dashes.next()?, dashes.next()?);
            Some(PackageUpdate { name: left[..split].to_string(), installed: left[split + 1..].to_string(), candidate: candidate.trim().to_string(), security: false })
        })
        .collect()
}

/// Parses [`UPDATES_PROBE`] output.
pub fn parse_updates(stdout: &str) -> UpdatesReport {
    let mut sections: HashMap<&str, &str> = HashMap::new();
    for chunk in stdout.split(SECTION).skip(1) {
        let (name, body) = chunk.split_once('\n').unwrap_or((chunk, ""));
        sections.insert(name.trim(), body);
    }
    let get = |k: &str| sections.get(k).copied().unwrap_or("");
    let mut r = UpdatesReport::default();
    for line in get("os").lines() {
        if let Some(v) = line.strip_prefix("id=") {
            r.os_id = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("version=") {
            r.os_version = v.trim().to_string();
        }
    }
    r.kernel = get("kernel").trim().to_string();
    r.manager = get("manager").trim().to_string();
    r.cache_time = get("cache").trim().parse().ok();
    r.reboot_required = get("reboot").trim() == "yes";
    r.updates = match r.manager.as_str() {
        "apt" => parse_apt_upgradable(get("upgradable")),
        "dnf" => parse_dnf_upgradable(get("upgradable"), get("security")),
        "apk" => parse_apk_upgradable(get("upgradable")),
        _ => Vec::new(),
    };
    for line in get("packages").lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if let [bin, src, _ver, src_ver] = f[..] {
            let src = if src.is_empty() { bin } else { src };
            r.sources.insert(bin.to_string(), SourcePkg { name: src.to_string(), version: src_ver.to_string() });
        }
    }
    // dpkg -S: "openssh-server: /usr/sbin/sshd", "libc-bin, libc6:amd64: /path"
    let mut running: Vec<SourcePkg> = get("running")
        .lines()
        .filter_map(|l| l.rsplit_once(": ").map(|(pkgs, _)| pkgs))
        .flat_map(|pkgs| pkgs.split(", "))
        .filter_map(|p| r.sources.get(p.split(':').next().unwrap_or(p)).cloned())
        .collect();
    running.sort();
    running.dedup();
    r.running = running;
    r.kernel_running = get("kernel-running").lines().next().and_then(kernel_source);
    r.kernel_newest = get("kernel-newest").lines().next().and_then(kernel_source);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const APT: &str = "@@crow-sec@@ os\nid=ubuntu\nversion=24.04\n@@crow-sec@@ kernel\n6.8.0-40-generic\n@@crow-sec@@ manager\napt\n@@crow-sec@@ cache\n1727000000\n@@crow-sec@@ upgradable\nopenssl/noble-updates,noble-security 3.0.13-0ubuntu3.5 amd64 [upgradable from: 3.0.13-0ubuntu3.4]\nbase-files/noble-updates 13ubuntu10.2 amd64 [upgradable from: 13ubuntu10.1]\n@@crow-sec@@ reboot\nyes\n@@crow-sec@@ packages\nopenssl\topenssl\t3.0.13-0ubuntu3.4\t3.0.13-0ubuntu3.4\nlibssl3t64\topenssl\t3.0.13-0ubuntu3.4\t3.0.13-0ubuntu3.4\nopenssh-server\topenssh\t1:9.6p1-3ubuntu13\t1:9.6p1-3ubuntu13\nbase-files\t\t13ubuntu10.1\t13ubuntu10.1\n@@crow-sec@@ running\nopenssh-server: /usr/sbin/sshd\nlibssl3t64:amd64, openssl: /usr/lib/x\n@@crow-sec@@ kernel-running\nlinux-signed\t6.8.0-40.40\n@@crow-sec@@ kernel-newest\nlinux-signed\t6.8.0-45.45\n";

    #[test]
    fn parses_apt_probe() {
        let r = parse_updates(APT);
        assert_eq!((r.os_id.as_str(), r.osv_ecosystem().as_deref()), ("ubuntu", Some("Ubuntu:24.04")));
        assert_eq!(r.manager, "apt");
        assert_eq!(r.cache_time, Some(1727000000));
        assert!(r.reboot_required);
        assert_eq!(r.updates.len(), 2);
        assert_eq!(r.updates[0], PackageUpdate { name: "openssl".into(), installed: "3.0.13-0ubuntu3.4".into(), candidate: "3.0.13-0ubuntu3.5".into(), security: true });
        assert!(!r.updates[1].security);
        assert_eq!(r.security_count(), 1);
        assert_eq!(r.sources["base-files"].name, "base-files", "source defaults to the binary name");
        let running: Vec<&str> = r.running.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(running, ["openssh", "openssl"], "deduped source packages of running binaries");
        assert_eq!(r.kernel_running, Some(SourcePkg { name: "linux".into(), version: "6.8.0-40.40".into() }), "signed wrapper mapped to linux");
        assert_eq!(r.kernel_newest.as_ref().unwrap().version, "6.8.0-45.45");
        assert_eq!(r.kernel_update(), None, "no linux-image upgrade in this fixture");
    }

    #[test]
    fn parses_dnf_and_apk_lists() {
        let dnf = parse_dnf_upgradable(
            "openssl-libs.x86_64   1:3.0.7-27.el9   baseos\nkernel.x86_64 5.14.0-427.el9 baseos\nObsoleting Packages\n",
            "RHSA-2024:1234 Important/Sec. openssl-libs-1:3.0.7-27.el9.x86_64\n",
        );
        assert_eq!(dnf.len(), 2);
        assert!(dnf[0].security && dnf[0].name == "openssl-libs");
        assert!(!dnf[1].security);
        let apk = parse_apk_upgradable("openssl-3.1.4-r5   < 3.1.4-r6\nmusl-1.2.4-r2 < 1.2.4-r3\n");
        assert_eq!((apk[0].name.as_str(), apk[0].installed.as_str(), apk[0].candidate.as_str()), ("openssl", "3.1.4-r5", "3.1.4-r6"));
    }

    /// Runs the real probe on this machine when it has apt.
    #[test]
    fn real_probe_parses_here() {
        use crate::host::{Host, LocalHost, DEFAULT_TIMEOUT};
        let Ok(out) = LocalHost.exec(&["sh", "-c", UPDATES_PROBE], DEFAULT_TIMEOUT) else { return };
        let r = parse_updates(&out.stdout);
        assert!(!r.kernel.is_empty());
        if r.manager == "apt" {
            assert!(!r.sources.is_empty());
            assert!(r.kernel_running.is_some(), "running kernel's source package found");
        }
    }
}
