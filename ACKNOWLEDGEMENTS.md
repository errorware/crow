# Acknowledgements

Crow is built on the work of many projects. Thank you to all of them.

Licenses for the bundled graphic assets and fonts are kept beside the files, in [`assets/`](assets/README.md).

## Software

### Interface and rendering

| Project | Used for | License |
|---|---|---|
| [GPUI](https://github.com/zed-industries/zed) (Zed Industries), as `gpui-pre` | The GPU-accelerated UI framework | Apache-2.0 |
| [gpui-kit](https://github.com/longbridge/gpui-kit) (Longbridge), with gpui-component | Inputs, scroll areas, tooltips and other components | Apache-2.0 |
| [wgpu](https://github.com/gfx-rs/wgpu) | GPU rendering under GPUI | MIT or Apache-2.0 |
| [image](https://github.com/image-rs/image) | The Fleet background picture | MIT or Apache-2.0 |

### Terminal

| Project | Used for | License |
|---|---|---|
| [alacritty_terminal](https://github.com/alacritty/alacritty) (Zed's fork) | The terminal emulator and PTY behind the Terminal screen | Apache-2.0 |

### Security and the vault

| Project | Used for | License |
|---|---|---|
| [RustCrypto](https://github.com/RustCrypto): `argon2`, `chacha20poly1305`, `sha2`, `ssh-key` | Key derivation, secret encryption, config hashes, SSH keys | MIT or Apache-2.0 |
| [zeroize](https://github.com/RustCrypto/utils) | Wiping secrets from memory | MIT or Apache-2.0 |
| [totp-rs](https://github.com/constantoine/totp-rs) | Two-factor codes for the Locked stance | MIT |
| [rand](https://github.com/rust-random/rand) | Key and secret generation | MIT or Apache-2.0 |

### Storage, formats and networking

| Project | Used for | License |
|---|---|---|
| [SQLite](https://sqlite.org), via [rusqlite](https://github.com/rusqlite/rusqlite) | `crow.db` | Public domain; MIT |
| [serde](https://github.com/serde-rs/serde), [serde_json](https://github.com/serde-rs/json) | Data formats | MIT or Apache-2.0 |
| [toml](https://github.com/toml-rs/toml), `toml_edit` | `config.toml`, keeping its layout | MIT or Apache-2.0 |
| [chrono](https://github.com/chronotope/chrono) | Dates and times | MIT or Apache-2.0 |
| [maxminddb](https://github.com/oschwald/maxminddb-rust) | Reading the GeoIP database | ISC |
| [flate2](https://github.com/rust-lang/flate2-rs) | Unpacking the GeoIP download | MIT or Apache-2.0 |
| [lettre](https://github.com/lettre/lettre), [rustls](https://github.com/rustls/rustls) | Email notifications over TLS | MIT; Apache-2.0, ISC or MIT |
| [semver](https://github.com/dtolnay/semver) | Comparing release versions | MIT or Apache-2.0 |
| [dirs](https://codeberg.org/dirs/dirs-rs), [libc](https://github.com/rust-lang/libc) | Platform paths and calls | MIT or Apache-2.0 |

### Crow's own libraries

| Project | Used for | License |
|---|---|---|
| [crow-config](https://github.com/errorware/crow-config) | Structured, lossless config editing and its schemas | See its repository |
| `crates/crow-provider-*` (this repository) | Cloud provider contracts, Linode and UpCloud | Crow's license |

### Programs Crow runs

Crow doesn't link these; it calls the copies already on your machine.

- [OpenSSH](https://www.openssh.com): every connection to a server.
- [curl](https://curl.se): provider, AI and update requests (secrets go to it on stdin).
- [slsa-verifier](https://github.com/slsa-framework/slsa-verifier): checking a release before Crow installs it (optional).

## Graphic assets and fonts

| Asset | From | License |
|---|---|---|
| UI icons | [Tabler Icons](https://tabler.io/icons) by Paweł Kuna and contributors | MIT |
| Country flags | [flag-icons](https://github.com/lipis/flag-icons) by Panayiotis Lipiridis and contributors | MIT |
| Linux distribution logos | [Dashboard Icons](https://github.com/homarr-labs/dashboard-icons) by homarr-labs | Apache-2.0; the logos are their owners' trademarks |
| JetBrains Mono | [JetBrains](https://github.com/JetBrains/JetBrainsMono) | SIL Open Font License 1.1 |

## Data and services

| Source | Used for | Terms |
|---|---|---|
| [DB-IP](https://db-ip.com) IP to Country Lite | Server regions from public IPs, looked up on your machine (off by default) | CC BY 4.0 |
| [OSV](https://osv.dev) | Known vulnerabilities in installed packages | [OSV's terms](https://osv.dev) |
| [Linode](https://www.linode.com) and [UpCloud](https://upcloud.com) APIs | Importing servers, power and snapshots | Your account's terms |
