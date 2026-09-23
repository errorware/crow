use std::fs;

/// The distro families Crow actually knows the config/package layout
/// conventions for. Everything else should say so, not guess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistroFamily {
    Debian,
    RedHat,
    Unknown,
}

impl DistroFamily {
    pub fn label(&self) -> &'static str {
        match self {
            DistroFamily::Debian => "DEBIAN FAMILY",
            DistroFamily::RedHat => "RED HAT FAMILY",
            DistroFamily::Unknown => "UNVERIFIED DISTRO",
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, DistroFamily::Unknown)
    }
}

/// Classifies a distro from any human-readable distro string — a real
/// /etc/os-release PRETTY_NAME, or the role-based placeholder text used
/// before a real probe exists. Crow only knows Debian- and Red Hat-family
/// path/package conventions; anything else should say so rather than
/// silently guessing wrong paths.
pub fn classify_distro_family(distro_display: &str) -> DistroFamily {
    let d = distro_display.to_lowercase();
    if d.contains("ubuntu") || d.contains("debian") || d.contains("mint") || d.contains("raspbian") || d.contains("pop!_os") {
        DistroFamily::Debian
    } else if d.contains("fedora") || d.contains("red hat") || d.contains("rhel") || d.contains("centos") || d.contains("rocky") || d.contains("alma") || d.contains("oracle linux") {
        DistroFamily::RedHat
    } else {
        DistroFamily::Unknown
    }
}

/// Reads and parses /etc/os-release on THIS machine.
pub fn detect_local_os_release() -> Option<String> {
    parse_os_release(&fs::read_to_string("/etc/os-release").ok()?)
}

/// Reads and parses /etc/os-release on `host`.
pub fn detect_os_release(host: &dyn crate::host::Host) -> Option<String> {
    parse_os_release(&host.read_file("/etc/os-release").ok()?)
}

/// The distro's display name from os-release content: PRETTY_NAME, else ID.
pub fn parse_os_release(content: &str) -> Option<String> {
    let mut id = String::new();
    let mut pretty_name = String::new();

    for line in content.lines() {
        let line = line.trim();
        let Some((key, raw_val)) = line.split_once('=') else { continue };
        let val = raw_val.trim().trim_matches('"').to_string();
        match key {
            "PRETTY_NAME" => pretty_name = val,
            "ID" => id = val,
            _ => {}
        }
    }

    if pretty_name.is_empty() && id.is_empty() {
        return None;
    }
    Some(if !pretty_name.is_empty() { pretty_name } else { id })
}
