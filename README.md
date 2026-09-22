# Crow

<p align="center">
  <strong>Native Linux Server Manager over SSH</strong><br>
  <em>A high-density operator terminal built with Rust and GPUI.</em>
</p>

<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#key-features">Features</a> •
  <a href="#design-system">Design System</a> •
  <a href="#getting-started">Getting Started</a> •
  <a href="#keyboard-shortcuts">Keyboard Shortcuts</a> •
  <a href="#license">License</a>
</p>

---

## Overview

**Crow** is a high-performance native desktop application for operators, sysadmins, and site reliability engineers managing remote Linux infrastructure over SSH.

Inspired by the information density and keyboard efficiency of high-octane TUIs (`k9s`, `lazygit`) and financial trading terminals, Crow translates operator workflows into a hardware-accelerated GUI. Density is a core feature: no excessive whitespace, no decorative charts without telemetry signals, and zero compromises on speed.

Built natively in **Rust** on top of the **GPUI** vector rendering engine, Crow delivers sub-millisecond input latency, smooth 120fps scrolling, and minimal memory footprint.

---

## Key Features

### 1. Live Telemetry & Stat Strip
- **Real-Time Metrics**: Instant visibility into 1m/5m/15m CPU load, memory breakdown (active usage vs cache/buffers), disk usage per mount, network throughput with bandwidth sparklines, kernel uptime, and SELinux/AppArmor/UFW security posture.
- **Signal-First Color Palette**: Color is reserved strictly for operational status (`OK` green, `WARN` amber, `CRIT` red). All non-status elements remain crisp monochrome.

### 2. Services, Processes & Sockets Inspector
- **Dense Service Units Table**: Real-time listing of active, degraded, and failed `systemd` units with instant sort by CPU, memory, RSS, or uptime.
- **Destructive Safety Guards**: Inline confirmation step before restarting or stopping production services, displaying active connection counts and potential blast radius.
- **Processes & Sockets Sub-views**: Quick switching between systemd service units, raw process trees, and open listening sockets.

### 3. Streaming Journalctl Log Tail
- **Live Stream**: Direct tailing of `journalctl -f` across all systemd units.
- **Log Filters & Autoscroll**: Filter dynamically by log level (`ERR`, `WARN`, `INFO`), pause streaming to inspect stack traces, or lock auto-scroll to the latest entries.

### 4. Grammar-Aware Graphical Config Editor
- **Structured Table Editing**: Edit complex system configuration files (such as PostgreSQL's `pg_hba.conf`) as structured, validated table rows instead of brittle raw text files.
- **Pre-Flight Grammar Validation**: Rules are continuously parsed and validated against target grammar schemas (e.g. PostgreSQL 16 syntax) before applying.
- **Staged Diffs & Blast Radius**: Inspect color-coded unified diffs and evaluate live session impacts prior to execution.
- **Atomic Application & Auto-Rollback**: Files are written via atomic renames with file backups and an armed 60-second auto-rollback timer if services fail authentication after reload.

### 5. Native Wayland & Desktop Integration
- **Client-Side Decorations (CSD)**: Seamless, frameless window chrome that integrates natively with modern Wayland compositors (KDE Plasma, GNOME) without duplicate OS titlebar borders.
- **Multi-Server Tab Strip**: Manage concurrent SSH sessions across staging, production, and edge servers with integrated tab dragging and window management.

---

## Design System: Obsidian Edge

Crow is built around the **Obsidian Edge** design language:

- **Typography**: 100% pure **JetBrains Mono** monospace typography across the entire interface. Visual hierarchy is established strictly through font weight (`BOLD`, `SEMIBOLD`, `NORMAL`), micro-scale font sizing, and contrast tokens.
- **Vector Icons**: Standardized catalog of 24×24 **Tabler SVG icons** (`assets/icons/`) embedded into the binary at compile time and tinted dynamically via GPUI's vector engine.
- **High Information Density**: Precise hairlines (1px), subdued cool-neutral surfaces (`#050507` ground, `#0b0b0e` panels), and responsive vertical virtualized scrolling.

---

## Getting Started

### Download

Prebuilt binaries for **Linux (x86_64)** and **macOS (Apple Silicon)** are attached to each [GitHub release](https://github.com/errorware/crow/releases), alongside SLSA provenance (`.intoto.jsonl`) you can check with [slsa-verifier](https://github.com/slsa-framework/slsa-verifier).

```bash
tar -xzf crow-linux-x86_64.tar.gz   # or crow-macos-arm64.tar.gz
./crow
```

The macOS build is not yet signed or notarized, so Gatekeeper blocks it on first launch. Clear the quarantine flag once:

```bash
xattr -d com.apple.quarantine ./crow
```

### Prerequisites

#### Linux (Fedora / Bazzite / Debian / Ubuntu / Arch)
Crow requires a standard Rust toolchain (1.80+) and native windowing/font libraries.

```bash
# Fedora / Bazzite
sudo dnf install fontconfig-devel libxkbcommon-devel libxcb-devel wayland-devel

# Ubuntu / Debian
sudo apt install libfontconfig1-dev libxkbcommon-dev libxcb1-dev libwayland-dev
```

Ensure **JetBrains Mono** is installed on your system:
```bash
# macOS
brew install --cask font-jetbrains-mono

# Linux (Homebrew)
brew install font-jetbrains-mono
```

### Build & Run

```bash
# Clone the repository
git clone git@github.com:errorware/crow.git
cd crow

# Check and build debug binary
cargo build

# Run Crow
cargo run --release
```

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `⌘K` / `Ctrl+K` | Open Command Palette / Global Search |
| `⌘\` / `Ctrl+\` | Toggle Sidebar Collapse / Expand |
| `⌘T` / `Ctrl+T` | Open Terminal |
| `⌘N` / `Ctrl+N` | Add New Rule (Config View) |
| `⌘⏎` / `Ctrl+Enter` | Apply Staged Config Plan & Reload |
| `f` | Toggle Table Sort by CPU |
| `p` | Toggle Log Autoscroll Pause |
| `c` | Clear Log Stream Buffer |
| `Esc` | Cancel Destructive Action / Close Modal |

---

## Acknowledgements

- **[Tabler Icons](https://tabler.io/icons)** by Paweł Kuna and contributors (licensed under MIT). See [assets/README.md](assets/README.md) for full license and attribution.

---

## License

Crow is licensed under the **AEUPL-1.2** (*The Ancient European Union Public License*).

Free for anyone to use, copy, modify, and share for any non-commercial purpose without restriction. Commercial use requires a physical letter sent to the author written in Latin on genuine sheepskin parchment.

See the [LICENSE](LICENSE) file or visit [aeupl.org](https://aeupl.org/) for full terms and conditions.
