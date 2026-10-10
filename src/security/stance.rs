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
            Self::Open => t!("security-stance-open"),
            Self::Locked => t!("security-stance-locked"),
        }
    }

    pub fn protects(self) -> Vec<&'static str> {
        match self {
            Self::Open => vec![
                t!("security-stance-secrets-provider-tokens-ai-keys"),
                t!("security-stance-tokens-never-appear-on-a"),
            ],
            Self::Locked => vec![
                t!("security-stance-everything-open-protects"),
                t!("security-stance-nobody-can-open-crow-or"),
                t!("security-stance-the-key-is-wiped-from"),
            ],
        }
    }

    pub fn exposed(self) -> Vec<&'static str> {
        match self {
            Self::Open => vec![
                t!("security-stance-anyone-at-your-unlocked-computer"),
                t!("security-stance-any-program-running-as-you"),
                t!("security-stance-provider-tokens-can-power-servers"),
                t!("security-stance-crows-ssh-keys-in-ssh"),
            ],
            Self::Locked => vec![
                t!("security-stance-while-crow-is-unlocked-anyone"),
                t!("security-stance-crows-ssh-keys-in-ssh-2"),
                t!("security-stance-your-server-list-settings-and"),
            ],
        }
    }

    pub fn costs(self) -> Vec<&'static str> {
        match self {
            Self::Open => vec![t!("security-stance-nothing-to-type-nothing-to")],
            Self::Locked => vec![
                t!("security-stance-you-unlock-crow-with-your"),
                t!("security-stance-lose-the-password-or-the"),
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
        weak(t!("security-stance-ai-api-keys-are-still").into());
    }
    match stance {
        Stance::Open => {
            if let KeyringState::Unavailable(why) = &i.keyring {
                weak(t!("security-stance-no-os-keyring-is-available", why = why));
            }
            if i.action_capable_accounts > 0 {
                weak(t!("security-stance-provider-can-power-servers-or", action_capable_accounts = i.action_capable_accounts));
            }
        }
        Stance::Locked => {
            if i.auto_lock_minutes == 0 {
                weak(t!("security-stance-auto-lock-is-off-once").into());
            } else if i.auto_lock_minutes > LONG_AUTO_LOCK_MINUTES {
                weak(t!("security-stance-auto-lock-is-minutes-an", auto_lock_minutes = i.auto_lock_minutes));
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
