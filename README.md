# Crow
 
<p align="center">
  <img width="150" height="150" alt="crow-logo" src="https://github.com/user-attachments/assets/3d9027d4-67b1-4548-a105-de92db8577a9" />
  <br />
  <strong>Native Linux Server Manager over SSH</strong><br>
  <em>A high-density operator terminal built with Rust and GPUI.</em>
</p>
<p align="center">
  <a href="https://github.com/errorware/crow/actions/workflows/rust.yml"><img src="https://github.com/errorware/crow/actions/workflows/rust.yml/badge.svg" alt="Rust"></a>
</p>
<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#what-it-does">What it does</a> •
  <a href="#security">Security</a> •
  <a href="#providers-and-plugins">Providers & Plugins</a> •
  <a href="#getting-started">Getting Started</a> •
  <a href="#keyboard-shortcuts">Keyboard Shortcuts</a> •
  <a href="#acknowledgements">Acknowledgements</a> •
  <a href="#license">License</a>
</p>
---
 
> [!WARNING]
> **Crow is in active development.** Features change or move between releases, things break, and the vault format can change. Try it on servers you can afford to experiment with, not production. Bug reports are welcome.
 
## Overview
 
<img width="1494" height="911" alt="image" src="https://github.com/user-attachments/assets/24f12684-14eb-4a80-8943-8dc8f730ff36" />
Crow is a native desktop control plane for Linux server management over plain SSH. Broadly, it gives you what you'd get from a traditional shell, but with the bells and whistles of a dashboard-style UI. It bridges the gap between the grey-bearded SSH artisan and the neophyte, without taking anything away from either.
 
I originally envisioned it as a one-stop shop for all my server management needs. Over the years I've used every combination of shells, shell wrappers, IDEs, external services, web apps and more esoteric things to get done what I can now do in Crow alone. I have way more freedom now than with any single product before.
 
I can deploy a VM, provision and harden it, see patch states, and follow logs and services, without losing the ability to just jump into a shell and do things by hand when that gets me there faster.
 
None of Crow's niceties are mandatory. You can keep working however you did before, and nothing gets in the way. That's by design. Of course, design and usability are subjective, so you're entitled to think it sucks too.
 
It's agentless: Crow talks to your servers over plain SSH, reuses one connection per server, and never installs anything on them. The feel is borrowed from `k9s` and `lazygit`, dense and keyboard-driven, just drawn as a GPU-accelerated GUI.
 
I've tested it on Ubuntu, Fedora and macOS with no issues so far. It could probably run on Windows, but I didn't want to complicate my life with PATHs and WSL, so that's backlogged for now.
 
Other quality of life improvements include:
 
- **crow-config**, a small library that parses different config file formats into one common structure Crow understands, so it can show them as structured, validated rows instead of raw text. This is easier said than done, so of course you can always just go and edit your configs the way you always did.
- A built-in plugin system for extensibility.
- A fleet map: your managed infrastructure in a more convenient representation. I'm working on editing things from there and adding more nodes and categories (users, networks, etc.) to the visualisation.
Website: [crow.rs](https://crow.rs) · Community: [forum.errorware.net](https://forum.errorware.net)
 
---
 
## What it does
 
### Fleet
- All enrolled servers with health, environment (PROD / STAGE / DEV / LAB) and region (from the cloud provider, metadata service, or set by you). Optional offline IP-to-country lookup via DB-IP; no addresses leave your machine.
- Add Server verifies the host key, can bootstrap a password-only server onto Crow's SSH key, and reads distro, kernel, memory and disk.
- Import instances from Linode and UpCloud.
- Archive servers you no longer manage; their data is purged after a set window.
### Fleet Setup
Multi-server changes preview what they'll do, run one server at a time, and stop at the first failure.
- Groups, policies and the fleet map, with posture-check risks
- Patching (apt, dnf, apk) with orderly reboots
- Rollouts of commands or config files, with a per-server diff first
- Config drift detection and search across every file Crow has read
- SSH and firewall hardening, with fixes that verify a fresh login still works or roll back
- Certificates, user accounts and keys, cross-server log search, external checks (HTTP/TCP/DNS), incidents, DR plans, team roles
- Audit log of every change Crow made
### Per server
- **Overview**: live CPU, memory, disk, network, pending updates, and CVEs from [OSV](https://osv.dev)
- **History**: 24h / 7d metrics, sampled while Crow runs
- **Services, processes, sockets**: search, filter, act with confirmation
- **Logs**: `journalctl` with filters, plus optional AI explanations
- **Configs**: `sshd_config`, `/etc/hosts`, `pg_hba.conf` and ufw as structured rows via [crow-config](https://github.com/errorware/crow-config). Every change is a revision you can roll back; writes are atomic.
- **Firewall**: edit ufw and firewalld; read-only nftables/iptables. A lock-out guard asks for confirmation before anything that would block Crow's SSH port.
- **Terminal** with tabs and split panes
- **Files**: browse, view, chmod, edit in nano (via sudoedit when needed)
- **Users and cron**
- **Danger Zone**: destructive actions behind a typed keyword. Power and snapshots go through the provider when linked, so they work with SSH down. Fleet-wide rolling reboot, session revocation and host key rotation (verified, or reverted).
### Settings
Timeouts, auto-lock, retention, appearance, cloud providers, AI providers ("Clankers": Anthropic or OpenAI-compatible), background image, font size and language.
 
### Languages
English by default. A language pack is a single `.ftl` file in `~/.config/crow/languages/`. Spanish and Japanese ship in [`languages/`](languages/); see [`languages/README.md`](languages/README.md) to write one.
 
---
 
## Security
 
Crow holds provider tokens, AI keys and SSH access to your fleet, so it makes you pick a security stance. The titlebar badge always shows which one you're in.
 
| | **OPEN** (default) | **LOCKED** |
|---|---|---|
| Key held by | OS keyring (Keychain, Secret Service, Credential Manager) | Master password + TOTP |
| Protects against | Someone copying `crow.db` or backups | That, plus anyone using your machine while Crow is locked |
| Exposed to | Anything running as you; anyone at your unlocked desk | Anyone at your machine while Crow is unlocked (auto-locks when idle) |
| Cost | Nothing | Lose the password or 2FA and stored secrets are gone |
 
Either way:
- Secrets are encrypted with ChaCha20-Poly1305; the data key never touches disk unencrypted.
- Secrets never appear in argv, logs or errors, and are wiped from memory when no longer needed.
- Host keys are always checked strictly; agent and X11 forwarding are always off.
- Not encrypted: the rest of `crow.db` and your SSH keys in `~/.ssh`.
Full details in [docs/security.md](docs/security.md).
 
---
 
## Providers and plugins
 
Plugins are described by crow-config manifests. A plugin implements a **category** (`provider.hosts` today, `provider.dns` next) and declares the **capabilities** it supports; Crow only offers what's declared.
 
| Provider | Category | Capabilities |
|---|---|---|
| Linode | `provider.hosts` | list instances, power, backup snapshots |
| UpCloud | `provider.hosts` | list instances, power |
 
Providers are compiled in as cargo features (`provider-linode`, `provider-upcloud`, both on by default). Tokens live only in the encrypted vault.
 
**Your own plugins** go in `~/.config/crow/plugins/` as a folder with a `plugin.toml` and scripts. Crow shows every command a plugin can run before you accept it, disables it if its files change, asks before making changes, and logs every run. Plugins can act on a single server, across the fleet with a canary, or locally against a generated inventory (e.g. Ansible). Validate one with `crow --check-plugin <folder>`. Examples in [`plugins/`](plugins), docs in [docs/plugins.md](docs/plugins.md).
 
### Repository layout
 
```
crow/
├── src/                         # the app
├── crates/
│   ├── crow-provider-core/      # provider contracts: Provider, ListInstances, PowerControl, Snapshots; curl-based HTTP
│   ├── crow-provider-linode/    # Linode API v4
│   └── crow-provider-upcloud/   # UpCloud API 1.3
├── assets/                      # icons, flags, distro logos, fonts (each with its license), i18n/en.ftl
├── languages/                   # language packs (Spanish, Japanese) and how to write one
└── docs/
```
 
[crow-config](https://github.com/errorware/crow-config) has its own repository.
 
---
 
## Getting Started
 
### Download
 
Prebuilt binaries for **Linux (x86_64)** and **macOS (Apple Silicon)** are attached to each [GitHub release](https://github.com/errorware/crow/releases), alongside SLSA provenance (`.intoto.jsonl`) you can check with [slsa-verifier](https://github.com/slsa-framework/slsa-verifier).
 
**Linux**: unpack and run the installer. It puts `crow` in `~/.local/bin` and adds Crow (with its icon) to your app launcher; `./install.sh --uninstall` removes it.
 
```bash
tar -xzf crow-linux-x86_64.tar.gz
cd crow-linux-x86_64 && ./install.sh
```
 
**macOS**: unpack `crow-macos-arm64.tar.gz` and drag `Crow.app` to Applications. The app is ad-hoc signed but not notarized, so Gatekeeper blocks the first launch: right-click it and choose **Open**, or clear the quarantine flag once:
 
```bash
xattr -dr com.apple.quarantine /Applications/Crow.app
```
 
**Updates**: Crow checks GitHub releases once a day and shows `vX available` next to its version (disable in Settings → General). Auto-install is opt-in and only runs after [slsa-verifier](https://github.com/slsa-framework/slsa-verifier) (must be on your PATH) confirms the release's provenance. Only an installed copy (`~/.local/bin/crow`, `Crow.app`) is replaced, and Crow offers a restart rather than restarting itself.
 
### Prerequisites
 
A Rust toolchain (1.85+), native windowing and font libraries, and `curl` (Crow uses it for provider and AI requests).
 
```bash
# Fedora / Bazzite
sudo dnf install fontconfig-devel libxkbcommon-devel libxcb-devel wayland-devel
 
# Ubuntu / Debian
sudo apt install libfontconfig1-dev libxkbcommon-dev libxcb1-dev libwayland-dev
```
 
For the Open stance on Linux you need a Secret Service provider (GNOME Keyring or KWallet); most desktops have one running.
 
Crow uses **JetBrains Mono**:
```bash
brew install --cask font-jetbrains-mono   # macOS
brew install font-jetbrains-mono          # Linux (Homebrew)
```
 
### Build & Run
 
Crow builds against crow-config from a sibling checkout:
 
```bash
git clone git@github.com:errorware/crow-config.git crow-config-core
git clone git@github.com:errorware/crow.git
cd crow
 
cargo test --workspace
cargo run --release
```
 
---
 
## Keyboard Shortcuts
 
| Shortcut | Action |
| :--- | :--- |
| `⌘K` / `Ctrl+K` | Command palette |
| `⌘1` / `⌘2` / `⌘3` / `⌘4` | Fleet / server overview / configs / logs |
| `⌘N` | Add server |
| `⇧⌘F` | Fleet Setup |
| `⌘,` | Settings |
| `⌘S` | Save settings (on Settings) |
| `⌘/` | Open `config.toml` in your editor (on Settings) |
| `⌘\` | Collapse / expand the sidebar |
| `⇧⌘L` | Lock Crow (Locked stance) |
| `/` | Search the table (services, processes, users, logs) |
| `Tab` / `⇧Tab` | Next / previous field in a form |
| `Esc` | Close the dialog or form, or cancel |
 
`Ctrl` works wherever `⌘` is shown.
 
---
 
## Acknowledgements
 
Crow builds on GPUI, alacritty_terminal, the RustCrypto crates, SQLite and many more, and ships Tabler Icons, flag-icons, Dashboard Icons and JetBrains Mono. [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md) lists them all with their licenses; the bundled assets carry their own in [`assets/`](assets/README.md).
 
---
 
## License
 
Crow is licensed under the **AEUPL-1.2** (*The Ancient European Union Public License*).
 
Free for anyone to use, copy, modify, and share for any non-commercial purpose without restriction. Commercial use requires a physical letter sent to the author written in Latin on genuine sheepskin parchment.
 
See the [LICENSE](LICENSE) file or visit [aeupl.org](https://aeupl.org/) for full terms and conditions.

Yes, it's a meme license. You're welcome. 
