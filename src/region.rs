//! Where a server lives (ERR-36): asked of the server itself over SSH, from
//! its cloud provider's metadata service, then mapped to a country and city.
//! Read-only; nothing leaves the server except its own region code.

/// Read-only probe run on the server. Prints `vendor=` (from DMI, names the
/// provider even when its metadata service is off) and, when a provider's
/// metadata service answers, `provider=` and `region=`; and `addr=` for each
/// global interface address (for the GeoIP fallback). Each request has a
/// 2-second timeout; providers that can't be the host are skipped.
pub const REGION_PROBE: &str = r#"v=$(cat /sys/class/dmi/id/sys_vendor 2>/dev/null); echo "vendor=$v"
ip -o addr show scope global 2>/dev/null | awk '{print "addr="$4}'
if command -v curl >/dev/null 2>&1; then
  get() { curl -fsS -m 2 "$@" 2>/dev/null; }
  put() { curl -fsS -m 2 -X PUT "$@" 2>/dev/null; }
  hdr() { echo "-H"; echo "$1"; }
else
  get() { a=""; while [ "$1" = "-H" ]; do a="$a --header=$2"; shift 2; done; wget -qO- -T 2 $a "$1" 2>/dev/null; }
  put() { a=""; while [ "$1" = "-H" ]; do a="$a --header=$2"; shift 2; done; wget -qO- -T 2 --method=PUT $a "$1" 2>/dev/null; }
fi
m=http://169.254.169.254
case "$v" in
  Linode*|Akamai*)
    t=$(put -H "Metadata-Token-Expiry-Seconds: 60" $m/v1/token)
    [ -n "$t" ] && r=$(get -H "Metadata-Token: $t" $m/v1/instance | sed -n 's/^region: *//p')
    echo "provider=Linode"; [ -n "$r" ] && echo "region=$r" ;;
  DigitalOcean*)
    echo "provider=DigitalOcean"; r=$(get $m/metadata/v1/region); [ -n "$r" ] && echo "region=$r" ;;
  Hetzner*)
    echo "provider=Hetzner"; r=$(get $m/hetzner/v1/metadata/availability-zone); [ -n "$r" ] && echo "region=$r" ;;
  Amazon*)
    t=$(put -H "X-aws-ec2-metadata-token-ttl-seconds: 60" $m/latest/api/token)
    echo "provider=AWS"; r=$(get -H "X-aws-ec2-metadata-token: $t" $m/latest/meta-data/placement/region); [ -n "$r" ] && echo "region=$r" ;;
  Google*)
    echo "provider=GCP"; r=$(get -H "Metadata-Flavor: Google" http://metadata.google.internal/computeMetadata/v1/instance/zone); [ -n "$r" ] && echo "region=${r##*/}" ;;
  Microsoft*)
    echo "provider=Azure"; r=$(get -H "Metadata: true" "$m/metadata/instance/compute/location?api-version=2021-02-01&format=text"); [ -n "$r" ] && echo "region=$r" ;;
esac
true"#;

/// A detected location. `country` is an ISO 3166 alpha-2 code, uppercase.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DetectedRegion {
    pub provider: String,
    pub code: String,
    pub country: String,
    pub city: String,
}

/// Parses [`REGION_PROBE`] output. `None` when neither a provider nor a
/// region was found (bare metal, a laptop, a provider Crow doesn't know).
pub fn parse_region_probe(stdout: &str) -> Option<DetectedRegion> {
    let field = |k: &str| stdout.lines().find_map(|l| l.strip_prefix(k)).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    let provider = field("provider=")?;
    let code = field("region=").unwrap_or_default();
    let (country, city) = locate(&provider, &code).map(|(c, city)| (c.to_string(), city.to_string())).unwrap_or_default();
    Some(DetectedRegion { provider, code, country, city })
}

/// (country, city) for a provider's region code.
pub fn locate(provider: &str, code: &str) -> Option<(&'static str, &'static str)> {
    let code = code.trim().to_lowercase();
    if code.is_empty() {
        return None;
    }
    let exact = |table: &[(&'static str, &'static str, &'static str)]| table.iter().find(|(c, _, _)| *c == code).map(|(_, cc, city)| (*cc, *city));
    let prefix = |table: &[(&'static str, &'static str, &'static str)]| table.iter().find(|(c, _, _)| code.starts_with(c)).map(|(_, cc, city)| (*cc, *city));
    match provider {
        // Linode's newer codes start with the country ("de-fra-2", "fr-par");
        // the older ones are named per area.
        "Linode" => exact(LINODE_LEGACY).or_else(|| prefix(LINODE_CITIES)).or_else(|| country_from_prefix(&code)),
        "DigitalOcean" => prefix(DIGITALOCEAN),
        // Hetzner reports the zone ("fsn1-dc14").
        "Hetzner" => prefix(HETZNER),
        "AWS" => prefix(AWS),
        // GCP reports the zone ("europe-west3-b").
        "GCP" => prefix(GCP),
        "Azure" => exact(AZURE),
        // UpCloud zones ("de-fra1") start with the country, like Linode's.
        "UpCloud" => prefix(UPCLOUD).or_else(|| country_from_prefix(&code)),
        _ => None,
    }
}

/// "de-fra-2" → Germany, when the code starts with a country we have a flag for.
fn country_from_prefix(code: &str) -> Option<(&'static str, &'static str)> {
    let cc = code.get(..2)?.to_uppercase();
    COUNTRIES.iter().find(|(c, _)| *c == cc).map(|(c, _)| (*c, ""))
}

const LINODE_LEGACY: &[(&str, &str, &str)] = &[
    ("us-east", "US", "Newark"),
    ("us-central", "US", "Dallas"),
    ("us-west", "US", "Fremont"),
    ("us-southeast", "US", "Atlanta"),
    ("ca-central", "CA", "Toronto"),
    ("eu-west", "GB", "London"),
    ("eu-central", "DE", "Frankfurt"),
    ("ap-south", "SG", "Singapore"),
    ("ap-northeast", "JP", "Tokyo"),
    ("ap-southeast", "AU", "Sydney"),
    ("ap-west", "IN", "Mumbai"),
];

const LINODE_CITIES: &[(&str, &str, &str)] = &[
    ("de-fra", "DE", "Frankfurt"),
    ("fr-par", "FR", "Paris"),
    ("gb-lon", "GB", "London"),
    ("nl-ams", "NL", "Amsterdam"),
    ("se-sto", "SE", "Stockholm"),
    ("it-mil", "IT", "Milan"),
    ("es-mad", "ES", "Madrid"),
    ("us-iad", "US", "Washington"),
    ("us-ord", "US", "Chicago"),
    ("us-sea", "US", "Seattle"),
    ("us-mia", "US", "Miami"),
    ("us-lax", "US", "Los Angeles"),
    ("br-gru", "BR", "São Paulo"),
    ("jp-osa", "JP", "Osaka"),
    ("jp-tyo", "JP", "Tokyo"),
    ("in-maa", "IN", "Chennai"),
    ("in-bom", "IN", "Mumbai"),
    ("id-cgk", "ID", "Jakarta"),
    ("sg-sin", "SG", "Singapore"),
    ("au-mel", "AU", "Melbourne"),
];

const DIGITALOCEAN: &[(&str, &str, &str)] = &[
    ("nyc", "US", "New York"),
    ("sfo", "US", "San Francisco"),
    ("atl", "US", "Atlanta"),
    ("tor", "CA", "Toronto"),
    ("ams", "NL", "Amsterdam"),
    ("lon", "GB", "London"),
    ("fra", "DE", "Frankfurt"),
    ("sgp", "SG", "Singapore"),
    ("blr", "IN", "Bangalore"),
    ("syd", "AU", "Sydney"),
];

const HETZNER: &[(&str, &str, &str)] = &[
    ("fsn", "DE", "Falkenstein"),
    ("nbg", "DE", "Nuremberg"),
    ("hel", "FI", "Helsinki"),
    ("ash", "US", "Ashburn"),
    ("hil", "US", "Hillsboro"),
    ("sin", "SG", "Singapore"),
];

const UPCLOUD: &[(&str, &str, &str)] = &[
    ("au-syd", "AU", "Sydney"),
    ("de-fra", "DE", "Frankfurt"),
    ("dk-cph", "DK", "Copenhagen"),
    ("es-mad", "ES", "Madrid"),
    ("fi-hel", "FI", "Helsinki"),
    ("nl-ams", "NL", "Amsterdam"),
    ("no-svg", "NO", "Stavanger"),
    ("pl-waw", "PL", "Warsaw"),
    ("se-sto", "SE", "Stockholm"),
    ("sg-sin", "SG", "Singapore"),
    ("uk-lon", "GB", "London"),
    ("us-chi", "US", "Chicago"),
    ("us-nyc", "US", "New York"),
    ("us-sjo", "US", "San Jose"),
];

const AWS: &[(&str, &str, &str)] = &[
    ("us-east-1", "US", "N. Virginia"),
    ("us-east-2", "US", "Ohio"),
    ("us-west-1", "US", "N. California"),
    ("us-west-2", "US", "Oregon"),
    ("ca-central-1", "CA", "Montreal"),
    ("ca-west-1", "CA", "Calgary"),
    ("sa-east-1", "BR", "São Paulo"),
    ("mx-central-1", "MX", "Querétaro"),
    ("eu-west-1", "IE", "Ireland"),
    ("eu-west-2", "GB", "London"),
    ("eu-west-3", "FR", "Paris"),
    ("eu-central-1", "DE", "Frankfurt"),
    ("eu-central-2", "CH", "Zurich"),
    ("eu-north-1", "SE", "Stockholm"),
    ("eu-south-1", "IT", "Milan"),
    ("eu-south-2", "ES", "Spain"),
    ("me-south-1", "BH", "Bahrain"),
    ("me-central-1", "AE", "UAE"),
    ("il-central-1", "IL", "Tel Aviv"),
    ("af-south-1", "ZA", "Cape Town"),
    ("ap-south-1", "IN", "Mumbai"),
    ("ap-south-2", "IN", "Hyderabad"),
    ("ap-east-1", "HK", "Hong Kong"),
    ("ap-northeast-1", "JP", "Tokyo"),
    ("ap-northeast-2", "KR", "Seoul"),
    ("ap-northeast-3", "JP", "Osaka"),
    ("ap-southeast-1", "SG", "Singapore"),
    ("ap-southeast-2", "AU", "Sydney"),
    ("ap-southeast-3", "ID", "Jakarta"),
    ("ap-southeast-4", "AU", "Melbourne"),
    ("ap-southeast-5", "MY", "Malaysia"),
    ("ap-southeast-7", "TH", "Thailand"),
];

const GCP: &[(&str, &str, &str)] = &[
    ("us-central1", "US", "Iowa"),
    ("us-east1", "US", "South Carolina"),
    ("us-east4", "US", "N. Virginia"),
    ("us-east5", "US", "Columbus"),
    ("us-west1", "US", "Oregon"),
    ("us-west2", "US", "Los Angeles"),
    ("us-west3", "US", "Salt Lake City"),
    ("us-west4", "US", "Las Vegas"),
    ("us-south1", "US", "Dallas"),
    ("northamerica-northeast1", "CA", "Montréal"),
    ("northamerica-northeast2", "CA", "Toronto"),
    ("southamerica-east1", "BR", "São Paulo"),
    ("southamerica-west1", "CL", "Santiago"),
    ("europe-west1", "BE", "Belgium"),
    ("europe-west2", "GB", "London"),
    ("europe-west3", "DE", "Frankfurt"),
    ("europe-west4", "NL", "Netherlands"),
    ("europe-west6", "CH", "Zurich"),
    ("europe-west8", "IT", "Milan"),
    ("europe-west9", "FR", "Paris"),
    ("europe-west10", "DE", "Berlin"),
    ("europe-west12", "IT", "Turin"),
    ("europe-north1", "FI", "Finland"),
    ("europe-central2", "PL", "Warsaw"),
    ("europe-southwest1", "ES", "Madrid"),
    ("me-west1", "IL", "Tel Aviv"),
    ("me-central1", "QA", "Doha"),
    ("me-central2", "SA", "Dammam"),
    ("africa-south1", "ZA", "Johannesburg"),
    ("asia-south1", "IN", "Mumbai"),
    ("asia-south2", "IN", "Delhi"),
    ("asia-east1", "TW", "Taiwan"),
    ("asia-east2", "HK", "Hong Kong"),
    ("asia-northeast1", "JP", "Tokyo"),
    ("asia-northeast2", "JP", "Osaka"),
    ("asia-northeast3", "KR", "Seoul"),
    ("asia-southeast1", "SG", "Singapore"),
    ("asia-southeast2", "ID", "Jakarta"),
    ("australia-southeast1", "AU", "Sydney"),
    ("australia-southeast2", "AU", "Melbourne"),
];

const AZURE: &[(&str, &str, &str)] = &[
    ("eastus", "US", "Virginia"),
    ("eastus2", "US", "Virginia"),
    ("centralus", "US", "Iowa"),
    ("northcentralus", "US", "Illinois"),
    ("southcentralus", "US", "Texas"),
    ("westus", "US", "California"),
    ("westus2", "US", "Washington"),
    ("westus3", "US", "Arizona"),
    ("canadacentral", "CA", "Toronto"),
    ("canadaeast", "CA", "Quebec"),
    ("brazilsouth", "BR", "São Paulo"),
    ("mexicocentral", "MX", "Querétaro"),
    ("northeurope", "IE", "Ireland"),
    ("westeurope", "NL", "Netherlands"),
    ("uksouth", "GB", "London"),
    ("ukwest", "GB", "Cardiff"),
    ("francecentral", "FR", "Paris"),
    ("germanywestcentral", "DE", "Frankfurt"),
    ("switzerlandnorth", "CH", "Zurich"),
    ("norwayeast", "NO", "Oslo"),
    ("swedencentral", "SE", "Gävle"),
    ("polandcentral", "PL", "Warsaw"),
    ("italynorth", "IT", "Milan"),
    ("spaincentral", "ES", "Madrid"),
    ("uaenorth", "AE", "Dubai"),
    ("qatarcentral", "QA", "Doha"),
    ("israelcentral", "IL", "Israel"),
    ("southafricanorth", "ZA", "Johannesburg"),
    ("centralindia", "IN", "Pune"),
    ("southindia", "IN", "Chennai"),
    ("eastasia", "HK", "Hong Kong"),
    ("southeastasia", "SG", "Singapore"),
    ("japaneast", "JP", "Tokyo"),
    ("japanwest", "JP", "Osaka"),
    ("koreacentral", "KR", "Seoul"),
    ("australiaeast", "AU", "New South Wales"),
    ("newzealandnorth", "NZ", "Auckland"),
];

/// Countries Crow ships a flag for (the big clouds' locations), by name.
pub const COUNTRIES: &[(&str, &str)] = &[
    ("AE", "United Arab Emirates"), ("AR", "Argentina"), ("AT", "Austria"), ("AU", "Australia"), ("BE", "Belgium"),
    ("BH", "Bahrain"), ("BR", "Brazil"), ("CA", "Canada"), ("CH", "Switzerland"), ("CL", "Chile"), ("CN", "China"),
    ("DE", "Germany"), ("DK", "Denmark"), ("ES", "Spain"), ("FI", "Finland"), ("FR", "France"), ("GB", "United Kingdom"),
    ("HK", "Hong Kong"), ("ID", "Indonesia"), ("IE", "Ireland"), ("IL", "Israel"), ("IN", "India"), ("IT", "Italy"),
    ("JP", "Japan"), ("KR", "South Korea"), ("MX", "Mexico"), ("MY", "Malaysia"), ("NL", "Netherlands"), ("NO", "Norway"),
    ("NZ", "New Zealand"), ("PL", "Poland"), ("PT", "Portugal"), ("QA", "Qatar"), ("SA", "Saudi Arabia"), ("SE", "Sweden"),
    ("SG", "Singapore"), ("TH", "Thailand"), ("TW", "Taiwan"), ("US", "United States"), ("ZA", "South Africa"),
];

pub fn country_name(code: &str) -> Option<&'static str> {
    COUNTRIES.iter().find(|(c, _)| c.eq_ignore_ascii_case(code)).map(|(_, n)| *n)
}

/// The SVG flag for a country (flag-icons, MIT; assets/flags/LICENSE).
pub fn flag_svg(code: &str) -> Option<&'static [u8]> {
    macro_rules! flags {
        ($($cc:literal),*) => {
            match code.to_ascii_lowercase().as_str() {
                $($cc => Some(include_bytes!(concat!("../assets/flags/", $cc, ".svg")).as_slice()),)*
                _ => None,
            }
        };
    }
    flags!("ae", "ar", "at", "au", "be", "bh", "br", "ca", "ch", "cl", "cn", "de", "dk", "es", "fi", "fr", "gb", "hk", "id", "ie", "il", "in", "it", "jp", "kr", "mx", "my", "nl", "no", "nz", "pl", "pt", "qa", "sa", "se", "sg", "th", "tw", "us", "za")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_linode_probe() {
        // As read from a real Linode (Frankfurt) on 2026-09-25.
        let r = parse_region_probe("vendor=Linode\nprovider=Linode\nregion=de-fra-2\n").unwrap();
        assert_eq!((r.provider.as_str(), r.code.as_str(), r.country.as_str(), r.city.as_str()), ("Linode", "de-fra-2", "DE", "Frankfurt"));
    }

    #[test]
    fn maps_every_provider_style() {
        assert_eq!(locate("Linode", "eu-central"), Some(("DE", "Frankfurt")));
        assert_eq!(locate("Linode", "nl-ams"), Some(("NL", "Amsterdam")));
        assert_eq!(locate("Linode", "pt-lis-1"), Some(("PT", "")), "new-style code, country from its prefix");
        assert_eq!(locate("DigitalOcean", "fra1"), Some(("DE", "Frankfurt")));
        assert_eq!(locate("Hetzner", "hel1-dc2"), Some(("FI", "Helsinki")));
        assert_eq!(locate("AWS", "ap-southeast-2"), Some(("AU", "Sydney")));
        assert_eq!(locate("AWS", "ap-southeast-1"), Some(("SG", "Singapore")), "prefix match doesn't confuse -1 with -12");
        assert_eq!(locate("GCP", "europe-west3-b"), Some(("DE", "Frankfurt")));
        assert_eq!(locate("GCP", "europe-west1-c"), Some(("BE", "Belgium")), "europe-west1 isn't europe-west10/12");
        assert_eq!(locate("Azure", "westeurope"), Some(("NL", "Netherlands")));
        assert_eq!(locate("UpCloud", "de-fra1"), Some(("DE", "Frankfurt")));
        assert_eq!(locate("UpCloud", "uk-lon1"), Some(("GB", "London")), "UpCloud says uk, the flag is GB");
        assert_eq!(locate("Linode", ""), None);
    }

    #[test]
    fn bare_metal_has_no_region_and_provider_without_region_is_kept() {
        assert_eq!(parse_region_probe("vendor=LENOVO\n"), None);
        let r = parse_region_probe("vendor=Linode\nprovider=Linode\n").unwrap();
        assert_eq!((r.provider.as_str(), r.country.as_str()), ("Linode", ""), "metadata off: provider only");
    }

    #[test]
    fn every_country_has_a_flag() {
        for (cc, _) in COUNTRIES {
            assert!(flag_svg(cc).is_some_and(|b| b.starts_with(b"<svg")), "{cc}");
        }
        assert!(flag_svg("xx").is_none());
    }
}

#[cfg(test)]
mod live {
    /// `CROW_LIVE_REGION=user@host[:port] CROW_LIVE_KEY=~/.ssh/key cargo test live_region -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_region() {
        use crate::host::{Host, SshHost, DEFAULT_TIMEOUT};
        let target = std::env::var("CROW_LIVE_REGION").expect("CROW_LIVE_REGION=user@host[:port]");
        let key = std::env::var("CROW_LIVE_KEY").expect("CROW_LIVE_KEY");
        let (user, rest) = target.split_once('@').unwrap();
        let (host, port) = rest.split_once(':').map(|(h, p)| (h, p.parse().unwrap())).unwrap_or((rest, 22));
        let srv = crate::vault::ServerRecord { id: "live-region".into(), host: host.into(), port, login_user: user.into(), auth_method: "publickey".into(), key_id: Some("k".into()), ..Default::default() };
        let dir = std::env::temp_dir().join(format!("crow-live-region-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let h = SshHost::new(&srv, Some(key), Vec::new(), dir);
        let out = h.exec(&["sh", "-c", super::REGION_PROBE], DEFAULT_TIMEOUT).expect("probe ran");
        println!("{}", out.stdout);
        let r = super::parse_region_probe(&out.stdout).expect("a cloud server");
        println!("{r:?}");
        assert!(!r.country.is_empty());
    }
}
