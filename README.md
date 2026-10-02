# Crow

<p align="center">
  <img width="300" height="300" alt="crow-logo" src="https://github.com/user-attachments/assets/3d9027d4-67b1-4548-a105-de92db8577a9" />
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
  <a href="#license">License</a>
</p>

---

## Overview

**Crow** is a native desktop app for operators and SREs who manage Linux servers over SSH. It's inspired by the density and keyboard speed of `k9s` and `lazygit`, drawn as a hardware-accelerated GUI with **GPUI**.

Crow is **agentless**: it talks to your servers over plain SSH, reusing one connection per server, and never installs anything on them.

---

## What it does

### Fleet
- Every enrolled server with its health, environment (PROD / STAGE / DEV / LAB) and **region**, shown as a country flag. The region comes from the cloud provider, the machine's metadata service, or your own choice. Optionally (Settings → Servers, off by default), servers without cloud metadata are located by their public IP in a local copy of DB-IP's free country database: no server address is ever sent anywhere.
- Filter by environment or region. Archive servers you no longer manage; their data is purged after a window you set.
- **Add Server** checks the host key before trusting it, can bootstrap a password-only server onto Crow's own SSH key, and reads the machine's facts (distro, kernel, memory, disk, region).
- **Import from providers**: list your Linode / UpCloud instances, link the servers already in the fleet by IP, and add the rest through Add Server.

### A server
- **Overview**: live CPU, memory, disk and network; pending package updates; CVEs affecting installed packages (from [OSV](https://osv.dev)).
- **Services, processes and sockets**: live search, filters, pagination, and actions with confirmation.
- **Logs**: `journalctl` with unit, priority, time and PID filters, highlighted search matches, and an optional plain-English explanation from an AI provider you configure.
- **Configs**: edit `sshd_config`, `/etc/hosts`, `pg_hba.conf` and ufw rules as structured, validated rows through [crow-config](https://github.com/errorware/crow-config). Edits are surgical, every change is a revision you can roll back, and values rated *never on prod* need a typed confirmation. Files are written atomically (temp file, then rename).
- **Firewall**: ufw and firewalld (read and edit: rules, ports, services, rich rules; firewalld changes go to the running and saved config together), bare nftables/iptables rulesets (read-only, the raw ruleset one click away). A lock-out guard asks for a typed CONFIRM before any change that would stop the SSH port Crow is connected through from being allowed. **Users** (create, sudo, passwords over stdin), **files**.
- **Danger Zone**: destructive actions behind a typed keyword. For servers linked to a provider, **power off / reboot / boot and snapshots go through the provider**, so they work even when SSH is down. Fleet-wide (one host at a time, stopping at the first failure): rolling reboot, push baselines, **revoke sessions** (every SSH login but Crow's), and **rotate SSH host keys**: new keys are read over the connection the old key authenticated, re-pinned in known_hosts and proven with a fresh strict login, or everything is put back.

### Settings
- Connection and SSH timeouts, auto-lock, archive retention, appearance. Only settings that change something are shown.
- **Providers**: connect cloud accounts (Linode, UpCloud).
- **Clankers**: AI providers for the Logs explanation (Anthropic, OpenAI-compatible).
- **Personalisation**: a picture behind the Fleet list, with opacity and blur.

---

## Security

Crow holds things that matter: provider tokens that can power servers off, AI keys, and SSH access to your fleet. Security has a cost, so Crow doesn't hide the trade-off. It shows you which **stance** you're in, and what that protects and exposes. The badge in the titlebar is always visible; click it for the details.

| | **OPEN** (default) | **LOCKED** |
|---|---|---|
| How | No password; the OS keyring (Keychain, Secret Service, Credential Manager) holds Crow's encryption key | Master password + TOTP 2FA; the key is locked by your password |
| Protects against | Someone copying `crow.db` or your backups | That, and anything or anyone using your computer while Crow is locked |
| Exposed to | Anything running as you while you're logged in can read the key from the keyring; anyone at your unlocked desk can use Crow | Anyone at your computer while Crow is unlocked (it auto-locks when idle) |
| Costs | Nothing to type | Unlock with password + 2FA; lose either and the stored secrets are gone for good |

In both stances:
- **Secrets** (provider tokens, AI keys) are encrypted with ChaCha20-Poly1305 under one random data key. That key is never written to disk unencrypted.
- **Never on the command line or in logs**: secrets never appear in argv, logs or error messages. HTTP requests carry them to `curl` on stdin.
- **Wiped from memory**: secrets are wiped when no longer needed, and locking wipes the ones Crow holds in memory and closes its SSH connections.
- **Not encrypted**: the rest of `crow.db` (server list, settings, history) and your SSH keys in `~/.ssh`.
- **Strict SSH**: host keys are always checked strictly, and agent and X11 forwarding are always off, whatever `~/.ssh/config` says.

The first secret you save asks you to choose a stance on purpose. See [docs/security.md](docs/security.md) for the full model.

---

## Providers and plugins

Crow's extensions are **plugins** described by crow-config manifests:

- A **category** is a contract: `provider.hosts` (where servers come from) today, `provider.dns` next.
- **Capabilities** say which parts of that contract a plugin supports (`instances.list`, `instances.power`, `snapshots`). Crow only offers what a provider declares.
- **Modules** (planned) declare the categories they need rather than naming a provider.

| Provider | Category | Capabilities |
|---|---|---|
| Linode | `provider.hosts` | list instances, power, backup snapshots |
| UpCloud | `provider.hosts` | list instances, power |

Providers are compiled in, one cargo feature each (`provider-linode`, `provider-upcloud`, both on by default). Their settings forms are generated from the manifest, and tokens are stored only in the encrypted vault.

### Repository layout

```
crow/
├── src/                         # the app
├── crates/
│   ├── crow-provider-core/      # provider contracts: Provider, ListInstances, PowerControl, Snapshots; curl-based HTTP
│   ├── crow-provider-linode/    # Linode API v4
│   └── crow-provider-upcloud/   # UpCloud API 1.3
├── assets/                      # icons, flags
└── docs/
```

The provider crates live here, next to the app they serve. [crow-config](https://github.com/errorware/crow-config), the general-purpose config engine, has its own repository.

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

**Updates**: Crow checks the release list on GitHub once a day and shows `vX available` next to its version (Settings → General turns this off; nothing else is sent). Installing updates by themselves is opt-in, and only happens after [slsa-verifier](https://github.com/slsa-framework/slsa-verifier) (which must be on your PATH) confirms the release's signed provenance. Only an installed copy (`~/.local/bin/crow`, `Crow.app`) is replaced, and Crow offers a restart rather than restarting itself.

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

- **[Tabler Icons](https://tabler.io/icons)** by Paweł Kuna and contributors (MIT). See [assets/README.md](assets/README.md).
- **[flag-icons](https://flagicons.lipis.dev)** (MIT) for country flags.

---

## License

Crow is licensed under the **AEUPL-1.2** (*The Ancient European Union Public License*).

Free for anyone to use, copy, modify, and share for any non-commercial purpose without restriction. Commercial use requires a physical letter sent to the author written in Latin on genuine sheepskin parchment.

See the [LICENSE](LICENSE) file or visit [aeupl.org](https://aeupl.org/) for full terms and conditions.
