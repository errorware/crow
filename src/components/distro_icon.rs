//! The distribution's logo beside its name (server header, Fleet rows).
//! Icons: Dashboard Icons by homarr-labs (Apache-2.0, see
//! assets/distro/NOTICE.md). Multi-colour, so drawn with `img`, decoded once.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use gpui_kit::*;

/// The icon file for an `os-release` name ("Ubuntu 24.04.1 LTS", "Rocky
/// Linux 9.3"): its own logo when Crow knows it, the generic Linux one for
/// other Linux, none for systems that aren't Linux (or not read yet).
pub fn icon_for(distro: &str) -> Option<&'static str> {
    let d = distro.to_lowercase();
    if d.trim().is_empty() || d.trim() == "—" {
        return None;
    }
    let known: &[(&[&str], &str)] = &[
        (&["ubuntu"], "ubuntu-linux"),
        (&["debian"], "debian-linux"),
        (&["rocky"], "rocky-linux"),
        (&["almalinux", "alma linux"], "almalinux"),
        (&["centos"], "centos"),
        (&["red hat", "redhat", "rhel"], "redhat-linux"),
        (&["alpine"], "alpine-linux"),
        (&["artix"], "artixlinux"),
        (&["arch linux", "archlinux"], "arch-linux"),
        (&["kali"], "kali-linux"),
        (&["azure linux", "cbl-mariner", "mariner"], "azure-linux"),
    ];
    if let Some((_, file)) = known.iter().find(|(names, _)| names.iter().any(|n| d.contains(n))) {
        return Some(file);
    }
    let not_linux = ["solaris", "sunos", "illumos", "openindiana", "freebsd", "openbsd", "netbsd", "dragonfly", "macos", "mac os", "darwin", "windows", "aix", "hp-ux"];
    if not_linux.iter().any(|n| d.contains(n)) {
        return None;
    }
    Some("linux")
}

fn bytes(file: &str) -> Option<&'static [u8]> {
    Some(match file {
        "ubuntu-linux" => include_bytes!("../../assets/distro/ubuntu-linux.svg"),
        "debian-linux" => include_bytes!("../../assets/distro/debian-linux.svg"),
        "rocky-linux" => include_bytes!("../../assets/distro/rocky-linux.svg"),
        "almalinux" => include_bytes!("../../assets/distro/almalinux.svg"),
        "centos" => include_bytes!("../../assets/distro/centos.svg"),
        "redhat-linux" => include_bytes!("../../assets/distro/redhat-linux.svg"),
        "alpine-linux" => include_bytes!("../../assets/distro/alpine-linux.svg"),
        "artixlinux" => include_bytes!("../../assets/distro/artixlinux.svg"),
        "arch-linux" => include_bytes!("../../assets/distro/arch-linux.svg"),
        "kali-linux" => include_bytes!("../../assets/distro/kali-linux.svg"),
        "azure-linux" => include_bytes!("../../assets/distro/azure-linux.svg"),
        "linux" => include_bytes!("../../assets/distro/linux.svg"),
        _ => return None,
    })
}

fn image(file: &'static str) -> Option<Arc<Image>> {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Arc<Image>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().ok()?;
    if let Some(img) = cache.get(file) {
        return Some(img.clone());
    }
    let img = Arc::new(Image::from_bytes(ImageFormat::Svg, bytes(file)?.to_vec()));
    cache.insert(file, img.clone());
    Some(img)
}

/// The logo, `size` square, or nothing.
pub fn distro_icon(distro: &str, size: f32) -> Option<AnyElement> {
    let img_ = image(icon_for(distro)?)?;
    Some(img(img_).size(px(size)).flex_none().object_fit(ObjectFit::Contain).into_any_element())
}

#[cfg(test)]
mod tests {
    use super::icon_for;

    #[test]
    fn os_release_names_find_their_logo() {
        for (name, icon) in [
            ("Ubuntu 26.04.1 LTS", Some("ubuntu-linux")),
            ("Debian GNU/Linux 12 (bookworm)", Some("debian-linux")),
            ("Rocky Linux 9.3 (Blue Onyx)", Some("rocky-linux")),
            ("AlmaLinux 9.4 (Seafoam Ocelot)", Some("almalinux")),
            ("CentOS Stream 9", Some("centos")),
            ("Red Hat Enterprise Linux 9.4 (Plow)", Some("redhat-linux")),
            ("Alpine Linux v3.20", Some("alpine-linux")),
            ("Arch Linux", Some("arch-linux")),
            ("Artix Linux", Some("artixlinux")),
            ("Kali GNU/Linux Rolling", Some("kali-linux")),
            ("CBL-Mariner/Linux", Some("azure-linux")),
            ("Fedora Linux 44 (Bazzite)", Some("linux")),
            ("openSUSE Leap 15.6", Some("linux")),
            ("Oracle Solaris 11.4", None),
            ("FreeBSD 14.1-RELEASE", None),
            ("", None),
            ("—", None),
        ] {
            assert_eq!(icon_for(name), icon, "{name}");
        }
    }

    #[test]
    fn every_known_icon_has_its_file() {
        for f in ["ubuntu-linux", "debian-linux", "rocky-linux", "almalinux", "centos", "redhat-linux", "alpine-linux", "artixlinux", "arch-linux", "kali-linux", "azure-linux", "linux"] {
            assert!(super::bytes(f).is_some_and(|b| b.starts_with(b"<svg")), "{f}");
        }
    }
}
