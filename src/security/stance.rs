//! Crow's security stance (ERR-60): Open or Locked, and what each one
//! protects, exposes and costs. There is no middle stance on purpose: the
//! user should always know exactly where they stand.

use crate::vault::KeyringState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    /// No vault password: secrets are encrypted, the OS keyring holds the key.
    Open,
    /// Password + TOTP: the key is locked by the password.
    Locked,
}

impl Stance {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Locked => "LOCKED",
        }
    }

    pub fn protects(self) -> &'static [&'static str] {
        match self {
            Self::Open => &[
                "Secrets (provider tokens, AI keys) are encrypted in crow.db, so a copy of the file or a backup doesn't reveal them.",
                "Tokens never appear on a command line or in logs.",
            ],
            Self::Locked => &[
                "Everything Open protects.",
                "Nobody can open Crow or use its secrets without your password and a 2FA code, not even programs running as you.",
                "The key is wiped from memory when Crow locks.",
            ],
        }
    }

    pub fn exposed(self) -> &'static [&'static str] {
        match self {
            Self::Open => &[
                "Anyone at your unlocked computer can open Crow and use everything in it.",
                "Any program running as you can ask the OS keyring for Crow's key while you're logged in (your login unlocks the keyring), then read every secret.",
                "Provider tokens can power servers off and take snapshots; whoever has your session has that power.",
                "Crow's SSH keys in ~/.ssh are files Crow doesn't encrypt: unless they have a passphrase, anything running as you can use them to reach your servers.",
            ],
            Self::Locked => &[
                "While Crow is unlocked, anyone at your computer can use it, until it auto-locks.",
                "Crow's SSH keys in ~/.ssh are files Crow doesn't encrypt: unless they have a passphrase, anything running as you can use them, locked or not.",
                "Your server list, settings and change history are not encrypted in either stance.",
            ],
        }
    }

    pub fn costs(self) -> &'static [&'static str] {
        match self {
            Self::Open => &["Nothing to type, nothing to lose. That convenience is the exposure above."],
            Self::Locked => &[
                "You unlock Crow with your password and a 2FA code on every launch and after auto-lock.",
                "Lose the password or the authenticator and the stored secrets are gone for good. There is no reset and no recovery.",
            ],
        }
    }
}

/// What the assessment looks at.
#[derive(Clone, Debug)]
pub struct StanceInputs {
    pub password_on: bool,
    pub keyring: KeyringState,
    /// AI keys still stored in plain text by an older version.
    pub plaintext_ai_keys: bool,
    /// Configured auto-lock, in minutes (0 = never).
    pub auto_lock_minutes: i64,
    /// Provider accounts whose provider can act on servers (power, snapshots).
    pub action_capable_accounts: usize,
}

#[derive(Clone, Debug)]
pub struct StanceReport {
    pub stance: Stance,
    /// What weakens the stance right now.
    pub findings: Vec<String>,
}

/// Auto-lock longer than this weakens Locked.
pub const LONG_AUTO_LOCK_MINUTES: i64 = 60;

pub fn assess(i: &StanceInputs) -> StanceReport {
    let stance = if i.password_on { Stance::Locked } else { Stance::Open };
    let mut findings = Vec::new();
    let mut weak = |text: String| findings.push(text);
    if i.plaintext_ai_keys {
        weak("AI API keys are still stored in plain text (from an older version of Crow). They move into the vault as soon as an encryption key is available.".into());
    }
    match stance {
        Stance::Open => {
            if let KeyringState::Unavailable(why) = &i.keyring {
                weak(format!("No OS keyring is available ({why}), so Crow can't store secrets at all. Switch to Locked to store them."));
            }
            if i.action_capable_accounts > 0 {
                weak(format!(
                    "{} provider account{} can power servers or take snapshots, usable by anyone at this session.",
                    i.action_capable_accounts,
                    if i.action_capable_accounts == 1 { "" } else { "s" }
                ));
            }
        }
        Stance::Locked => {
            if i.auto_lock_minutes == 0 {
                weak("Auto-lock is off: once unlocked, Crow stays unlocked until you lock it or quit.".into());
            } else if i.auto_lock_minutes > LONG_AUTO_LOCK_MINUTES {
                weak(format!("Auto-lock is {} minutes: an unattended, unlocked Crow stays usable that long.", i.auto_lock_minutes));
            }
        }
    }
    StanceReport { stance, findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> StanceInputs {
        StanceInputs { password_on: false, keyring: KeyringState::Ready, plaintext_ai_keys: false, auto_lock_minutes: 15, action_capable_accounts: 0 }
    }

    #[test]
    fn no_password_is_open_and_says_what_that_exposes() {
        let r = assess(&inputs());
        assert_eq!(r.stance, Stance::Open);
        assert!(r.findings.is_empty());
        assert!(Stance::Open.exposed().iter().any(|e| e.contains("keyring")), "Open must name the keyring exposure");
    }

    #[test]
    fn open_with_power_capable_tokens_is_flagged() {
        let r = assess(&StanceInputs { action_capable_accounts: 2, ..inputs() });
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].starts_with("2 provider accounts can power servers"));
    }

    #[test]
    fn open_without_a_keyring_is_flagged() {
        let r = assess(&StanceInputs { keyring: KeyringState::Unavailable("no Secret Service".into()), ..inputs() });
        assert!(r.findings[0].contains("no Secret Service"));
    }

    #[test]
    fn locked_with_long_or_no_auto_lock_is_flagged() {
        let locked = StanceInputs { password_on: true, ..inputs() };
        assert_eq!(assess(&locked).stance, Stance::Locked);
        assert_eq!(assess(&locked).findings.len(), 0);
        assert_eq!(assess(&StanceInputs { auto_lock_minutes: 240, ..locked.clone() }).findings.len(), 1);
        assert!(assess(&StanceInputs { auto_lock_minutes: 0, ..locked }).findings[0].contains("Auto-lock is off"));
    }

    #[test]
    fn plaintext_keys_weaken_either_stance() {
        for password_on in [false, true] {
            let r = assess(&StanceInputs { password_on, plaintext_ai_keys: true, ..inputs() });
            assert!(r.findings.iter().any(|f| f.contains("plain text")));
        }
    }
}
