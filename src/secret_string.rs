//! A `String` for secrets being typed or held briefly (passwords in forms):
//! derefs to `String`, but prints `[secret]` and is wiped when dropped.

use std::ops::{Deref, DerefMut};

use zeroize::Zeroize;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Deref for SecretString {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl DerefMut for SecretString {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[secret]")
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works_like_a_string_but_never_prints() {
        let mut s = SecretString::default();
        s.push_str("hunter2");
        assert_eq!(s.as_str(), "hunter2");
        assert_eq!(format!("{s:?}"), "[secret]");
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Form {
            password: SecretString,
        }
        assert!(!format!("{:?}", Form { password: s }).contains("hunter2"));
    }
}
