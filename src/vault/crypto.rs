use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2, Params,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngExt;
use std::fmt;
use totp_rs::{Algorithm, Builder, Secret, Totp};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

/// Secure memory wrapper for 32-byte derived encryption keys.
/// Automatically zeroed out on drop from RAM.
#[derive(Zeroize, ZeroizeOnDrop, Clone)]
pub struct MasterKey(pub [u8; KEY_LEN]);

impl MasterKey {
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MasterKey(***REDACTED***)")
    }
}

#[derive(Debug)]
pub enum CryptoError {
    KeyDerivationFailed(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    InvalidTotp(String),
    PasswordVerificationFailed,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyDerivationFailed(e) => write!(f, "Key derivation failed: {}", e),
            CryptoError::EncryptionFailed(e) => write!(f, "Encryption failed: {}", e),
            CryptoError::DecryptionFailed(e) => write!(f, "Decryption failed: {}", e),
            CryptoError::InvalidTotp(e) => write!(f, "TOTP error: {}", e),
            CryptoError::PasswordVerificationFailed => write!(f, "Password verification failed"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Generates a cryptographically secure random 16-byte salt.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rng().fill(&mut salt);
    salt
}

/// Generates a cryptographically secure random 96-bit (12-byte) nonce for ChaCha20-Poly1305.
pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    rand::rng().fill(&mut nonce);
    nonce
}

/// Creates a standard Argon2id instance with production parameters:
/// 64MB memory (65536 KiB), 3 iterations, 4 lanes/parallelism.
pub fn standard_argon2() -> Result<Argon2<'static>, CryptoError> {
    let params = Params::new(65536, 3, 4, Some(KEY_LEN))
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    Ok(Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params))
}

/// Derives the 32-byte MasterKey from password and salt using Argon2id.
pub fn derive_master_key(password: &str, salt: &[u8; SALT_LEN]) -> Result<MasterKey, CryptoError> {
    let argon2 = standard_argon2()?;
    let mut key_bytes = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    Ok(MasterKey(key_bytes))
}

/// Computes an Argon2id PHC password hash string for verification.
pub fn hash_password_verifier(password: &str) -> Result<String, CryptoError> {
    let argon2 = standard_argon2()?;
    let password_hash = argon2
        .hash_password(password.as_bytes())
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    Ok(password_hash.to_string())
}

/// Verifies a password against the stored PHC verifier hash.
pub fn verify_password(password: &str, verifier_hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(verifier_hash)
        .map_err(|_| CryptoError::PasswordVerificationFailed)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Encrypts plaintext using ChaCha20-Poly1305 with the MasterKey and a fresh random 96-bit nonce.
pub fn encrypt_data(key: &MasterKey, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; NONCE_LEN]), CryptoError> {
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok((ciphertext, nonce_bytes))
}

/// A fresh random data key: the key every vault secret is encrypted with.
/// It's never derived from a password; the password (or the OS keyring)
/// only protects it, so either can change without re-encrypting anything.
pub fn generate_data_key() -> MasterKey {
    let mut key = [0u8; KEY_LEN];
    rand::rng().fill(&mut key);
    MasterKey(key)
}

/// Encrypts `data_key` with `wrapping_key` (e.g. the password-derived key).
pub fn wrap_key(wrapping_key: &MasterKey, data_key: &MasterKey) -> Result<(Vec<u8>, [u8; NONCE_LEN]), CryptoError> {
    encrypt_data(wrapping_key, data_key.as_bytes())
}

/// Decrypts a data key wrapped by [`wrap_key`].
pub fn unwrap_key(wrapping_key: &MasterKey, wrapped: &[u8], nonce: &[u8; NONCE_LEN]) -> Result<MasterKey, CryptoError> {
    let mut bytes = decrypt_data(wrapping_key, wrapped, nonce)?;
    let key = key_from_bytes(&bytes);
    bytes.zeroize();
    key.ok_or_else(|| CryptoError::DecryptionFailed("wrapped key has the wrong length".into()))
}

/// A key from exactly [`KEY_LEN`] bytes (e.g. read back from the OS keyring).
pub fn key_from_bytes(bytes: &[u8]) -> Option<MasterKey> {
    let arr: [u8; KEY_LEN] = bytes.try_into().ok()?;
    Some(MasterKey(arr))
}

/// Decrypts ciphertext using ChaCha20-Poly1305 with the MasterKey and the specified nonce.
pub fn decrypt_data(key: &MasterKey, ciphertext: &[u8], nonce_bytes: &[u8; NONCE_LEN]) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    let nonce = Nonce::from(*nonce_bytes);

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}

// ---------------------------------------------------------------------------
// TOTP 2FA Helpers
// ---------------------------------------------------------------------------

/// Creates a standard RFC 6238 TOTP instance for Crow (SHA1, 6 digits, 30s step).
pub fn build_totp(secret: Secret, account_name: &str) -> Result<Totp, CryptoError> {
    Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_digits(6)
        .with_skew(1)
        .with_step_duration(30)
        .with_secret(secret)
        .with_issuer(Some("Crow"))
        .with_account_name(account_name)
        .build()
        .map_err(|e| CryptoError::InvalidTotp(e.to_string()))
}

/// Generates a new random Base32 secret for TOTP.
pub fn generate_totp_secret() -> String {
    let secret = Secret::generate();
    secret.to_base32()
}

/// Verifies a 6-digit TOTP code against a Base32 encoded secret.
pub fn verify_totp_code(secret_base32: &str, code: &str) -> bool {
    let Ok(secret) = Secret::try_from_base32(secret_base32.trim()) else {
        return false;
    };
    let Ok(totp) = build_totp(secret, "crow-user") else {
        return false;
    };
    totp.check_current(code.trim()).is_some()
}

/// Generates the standard otpauth URL for QR code import into Authenticator apps.
pub fn totp_auth_url(secret_base32: &str, account: &str) -> Result<String, CryptoError> {
    let secret = Secret::try_from_base32(secret_base32.trim())
        .map_err(|e| CryptoError::InvalidTotp(e.to_string()))?;
    let totp = build_totp(secret, account)?;
    totp.to_url().map_err(|e| CryptoError::InvalidTotp(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_and_chacha_roundtrip() {
        let password = "SuperSecretPassword123!#";
        let salt = generate_salt();

        let key = derive_master_key(password, &salt).expect("failed to derive key");
        let verifier = hash_password_verifier(password).expect("failed to hash verifier");

        assert!(verify_password(password, &verifier).unwrap());
        assert!(!verify_password("wrong_password", &verifier).unwrap());

        let payload = b"Hello Obsidian Edge Vault -- top secret SSH private key";
        let (ciphertext, nonce) = encrypt_data(&key, payload).expect("encryption failed");
        assert_ne!(&ciphertext[..], payload);

        let decrypted = decrypt_data(&key, &ciphertext, &nonce).expect("decryption failed");
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_totp_generation_and_verification() {
        let secret_b32 = generate_totp_secret();
        assert!(!secret_b32.is_empty());

        let secret = Secret::try_from_base32(&secret_b32).unwrap();
        let totp = build_totp(secret, "test-user").unwrap();
        let current_code = totp.generate_current().to_string();

        assert!(verify_totp_code(&secret_b32, &current_code));
        assert!(!verify_totp_code(&secret_b32, "000000"));
    }
}
