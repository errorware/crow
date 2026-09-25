# Crow's security model

Crow manages servers, so what it stores can hurt you: provider tokens that can power machines off, AI API keys, and SSH access. This page says exactly what Crow protects, against whom, and what it doesn't.

## Two stances, on purpose

Security costs something: typing a password, carrying a second factor, the risk of losing them. Crow doesn't pick a quiet middle ground for you. It has two stances, shows which one you're in (the badge in the titlebar), and makes you choose the first time you store a secret.

### OPEN (the default)

- **How**: no vault password. Crow's encryption key is held by the OS credential store: macOS Keychain, Secret Service on Linux (GNOME Keyring, KWallet), Windows Credential Manager.
- **Protects against**: someone copying `~/.config/crow/crow.db`, or your backups. The secrets in it are encrypted, and the key isn't in the file.
- **Exposed to**: anything running as your user while you're logged in, because your login unlocks the keyring and any program can ask it for Crow's key. Also anyone at your unlocked computer: they can open Crow and use everything in it, including provider tokens that power servers off.
- **Costs**: nothing.

### LOCKED

- **How**: a master password plus TOTP two-factor authentication. The key is stored encrypted by a key derived from your password (Argon2id, 64 MB, 3 iterations).
- **Protects against**: everything Open does, plus anyone or anything using your account while Crow is locked. Nothing can be read without your password and a current 2FA code.
- **Exposed to**: anyone at your computer while Crow is unlocked. Crow locks itself after the auto-lock period (Settings → Vault & Security).
- **Costs**: you unlock with password + code on every launch and after auto-lock. **If you lose the password or the authenticator, the stored secrets can't be recovered**; there's no reset and no cloud copy. Your server list and settings stay readable.

There is deliberately no third stance (for example a password without 2FA): the point is that you always know where you stand.

You can switch either way from Settings → Vault & Security (going back to Open needs your password, a 2FA code and an explicit confirmation), and change the password without re-encrypting anything.

## What's encrypted

- **Encrypted**: provider tokens and passwords, AI API keys. Each is a vault entry sealed with ChaCha20-Poly1305 under one random 256-bit **data key**.
- **The data key**: never written to disk unencrypted. It's held by the OS keyring (Open) or stored wrapped by your password-derived key (Locked). Changing the password or the stance re-wraps this one key; the entries don't change.
- **Not encrypted**: the rest of `crow.db` (server list, settings, change history, metrics), `config.toml`, and your SSH private keys in `~/.ssh`. Unless an SSH key has a passphrase, anything running as you can use it, in either stance.

## How secrets are handled

- **Nowhere else**: never in plain SQLite, config files or logs. Older versions stored AI keys in plain text; Crow moves them into the vault on first launch and rewrites the database so the old text doesn't linger in free pages.
- **Never in argv**: HTTP requests to providers and AI services go through the system `curl` with the whole request, credentials included, on stdin. Nothing appears in `ps`.
- **Never printed**: secrets print and serialize as `[secret]` (`SecretValue`, `SecretString`) and are left out of error messages.
- **Wiped from memory**: secrets are overwritten when dropped, including the copies made to build requests. Locking wipes the AI keys Crow holds in memory, drops open secret forms, and closes its SSH connections, so nothing can reuse them while Crow is locked.
  - Limit: text you type into an input box lives in the UI toolkit's buffer, which Crow can't wipe; the box is dropped as soon as the form closes. Keys copied to the clipboard (the 2FA setup key) stay there until you copy something else.
- **Never shown back**: stored secrets are displayed as "set", or at most their last four characters.

## SSH

- Host keys are checked strictly and recorded when you add a server; a changed key is refused.
- Agent and X11 forwarding are always off, whatever `~/.ssh/config` says.
- Crow reuses one connection per server (ControlMaster) and closes them when it locks.
- Crow is agentless: it never installs anything on your servers.

## Anything that weakens your stance is shown

The stance panel lists what weakens it right now, for example:
- AI keys still in plain text;
- no OS keyring available (Crow then refuses to store secrets rather than store them unencrypted);
- provider tokens that can act on servers while Open;
- auto-lock off, or longer than an hour, while Locked.
