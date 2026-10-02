//! Country of a server without cloud metadata, from its public IP (ERR-37).
//!
//! The lookup is local: DB-IP's free "IP to Country Lite" database (CC BY
//! 4.0, attribution in Settings and About) is downloaded from db-ip.com
//! about once a month and read on this machine. No server address is ever
//! sent anywhere, and Crow doesn't ask a "what's my IP" service either: it
//! uses the enrolled address, or the public addresses on the server's own
//! interfaces. Off unless `servers.geoip_regions` is on.

use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub const ATTRIBUTION: &str = "IP geolocation by DB-IP (db-ip.com), CC BY 4.0";

/// Re-downloaded when older than this.
const MAX_AGE: Duration = Duration::from_secs(35 * 24 * 3600);

pub fn db_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("crow").join("geoip").join("dbip-country-lite.mmdb"))
}

/// DB-IP publishes `dbip-country-lite-YYYY-MM.mmdb.gz` monthly; early in a
/// month the new one may not be out yet, so last month's is tried too.
pub fn download_urls(today: chrono::NaiveDate) -> [String; 2] {
    use chrono::Datelike;
    let prev = today.with_day(1).and_then(|d| d.pred_opt()).unwrap_or(today);
    [today, prev].map(|d| format!("https://download.db-ip.com/free/dbip-country-lite-{}-{:02}.mmdb.gz", d.year(), d.month()))
}

/// Whether an address is on the public internet (not private, loopback,
/// link-local, CGNAT, documentation, multicast...).
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.is_documentation()
                || (a == 100 && (64..128).contains(&b))
                || a == 0
                || a >= 240)
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public(IpAddr::V4(v4));
            }
            let s = v6.segments()[0];
            !(v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() || (s & 0xfe00) == 0xfc00 || (s & 0xffc0) == 0xfe80 || (s == 0x2001 && v6.segments()[1] == 0x0db8))
        }
    }
}

/// `addr=203.0.113.5/24` lines from the region probe: the server's global
/// interface addresses.
pub fn probe_addrs(stdout: &str) -> Vec<IpAddr> {
    stdout.lines().filter_map(|l| l.trim().strip_prefix("addr=")).filter_map(|a| a.split('/').next()?.parse().ok()).collect()
}

/// The first public address among the enrolled one (resolved through the
/// system resolver if it's a name) and the server's own.
pub fn public_address(enrolled: &str, own: &[IpAddr]) -> Option<IpAddr> {
    let enrolled: Vec<IpAddr> = match enrolled.parse::<IpAddr>() {
        Ok(ip) => vec![ip],
        Err(_) => (enrolled, 22).to_socket_addrs().map(|a| a.map(|s| s.ip()).collect()).unwrap_or_default(),
    };
    enrolled.into_iter().chain(own.iter().copied()).find(|ip| is_public(*ip))
}

pub struct GeoIp {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl GeoIp {
    pub fn open(path: &std::path::Path) -> Result<Self, String> {
        let reader = maxminddb::Reader::open_readfile(path).map_err(|e| format!("the GeoIP database couldn't be read: {e}"))?;
        let db = Self { reader };
        // A country database knows where a well-known address is.
        if db.country(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))).is_none() {
            return Err("the GeoIP database has no country data".into());
        }
        Ok(db)
    }

    /// ISO 3166 alpha-2, uppercase.
    pub fn country(&self, ip: IpAddr) -> Option<String> {
        let found = self.reader.lookup(ip).ok()?;
        let code: Option<String> = found.decode_path(&maxminddb::path!["country", "iso_code"]).ok()?;
        code.filter(|c| c.len() == 2).map(|c| c.to_uppercase())
    }

    /// The database on disk, downloaded first when missing or a month old.
    /// A failed refresh keeps using the old copy.
    pub fn load() -> Result<Self, String> {
        let path = db_path().ok_or("no data directory")?;
        let fresh = std::fs::metadata(&path).and_then(|m| m.modified()).is_ok_and(|t| SystemTime::now().duration_since(t).unwrap_or_default() < MAX_AGE);
        if !fresh {
            if let Err(e) = download(&path) {
                if !path.exists() {
                    return Err(e);
                }
            }
        }
        Self::open(&path)
    }
}

/// Fetches the newest database through the system curl (HTTPS only),
/// unpacks it, checks it opens, then moves it into place.
fn download(path: &std::path::Path) -> Result<(), String> {
    use std::io::Read;
    let dir = path.parent().ok_or("bad database path")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let gz = dir.join("download.mmdb.gz");
    let tmp = dir.join("download.mmdb");
    let mut last = String::new();
    for url in download_urls(chrono::Local::now().date_naive()) {
        let out = std::process::Command::new("curl")
            .args(["-fsSL", "--proto", "=https", "--max-time", "120", "-o"])
            .arg(&gz)
            .arg(&url)
            .output()
            .map_err(|e| format!("curl couldn't run: {e}"))?;
        if !out.status.success() {
            last = format!("{url}: {}", String::from_utf8_lossy(&out.stderr).trim());
            continue;
        }
        let mut bytes = Vec::new();
        let unpacked = std::fs::File::open(&gz).map_err(|e| e.to_string()).and_then(|f| flate2::read::GzDecoder::new(f).read_to_end(&mut bytes).map_err(|e| e.to_string()));
        let _ = std::fs::remove_file(&gz);
        unpacked?;
        std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
        if let Err(e) = GeoIp::open(&tmp) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        return std::fs::rename(&tmp, path).map_err(|e| e.to_string());
    }
    Err(format!("the GeoIP database couldn't be downloaded ({last})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_public_addresses_are_looked_up() {
        for private in ["10.1.2.3", "192.168.1.10", "172.16.0.1", "100.64.3.4", "127.0.0.1", "169.254.1.1", "203.0.113.5", "::1", "fd00::1", "fe80::1", "2001:db8::5", "::ffff:10.0.0.1"] {
            assert!(!is_public(private.parse().unwrap()), "{private}");
        }
        for public in ["8.8.8.8", "100.128.0.1", "2a01:4f8::1", "::ffff:1.1.1.1"] {
            assert!(is_public(public.parse().unwrap()), "{public}");
        }
    }

    #[test]
    fn picks_the_enrolled_address_then_the_servers_own() {
        let own = probe_addrs("vendor=QEMU\naddr=10.0.0.4/24\naddr=2a01:4f8:c0c:1::1/64\naddr=198.51.100.7/32\n");
        assert_eq!(own.len(), 3);
        assert_eq!(public_address("10.0.0.4", &own), Some("2a01:4f8:c0c:1::1".parse().unwrap()));
        assert_eq!(public_address("8.8.4.4", &own), Some("8.8.4.4".parse().unwrap()));
        assert_eq!(public_address("192.168.1.2", &probe_addrs("addr=192.168.1.2/24\n")), None);
    }

    #[test]
    fn falls_back_to_last_months_file() {
        let d = chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        assert_eq!(download_urls(d), [
            "https://download.db-ip.com/free/dbip-country-lite-2026-01.mmdb.gz".to_string(),
            "https://download.db-ip.com/free/dbip-country-lite-2025-12.mmdb.gz".to_string(),
        ]);
    }

    /// Downloads the real database. Opt-in: cargo test live_geoip -- --ignored
    #[test]
    #[ignore]
    fn live_geoip() {
        let db = GeoIp::load().expect("downloads and opens");
        assert_eq!(db.country("1.1.1.1".parse().unwrap()).as_deref(), Some("AU"));
        assert_eq!(db.country("2a01:4f8::1".parse().unwrap()).as_deref(), Some("DE"));
    }
}
