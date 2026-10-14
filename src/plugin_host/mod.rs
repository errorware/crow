//! User plugins: recipe plugins Crow loads from a folder (ERR-173). Built in
//! stages; this one only reads `plugin.toml` (ERR-175). The built-in
//! integrations in `crate::plugins` are separate and stay as they are.

// Parts are used by later stages (running actions, ERR-178 on).
#![allow(dead_code)]

pub mod manifest;
pub mod check;
pub mod folder;
pub mod run;
pub mod local;
pub mod badges;
pub mod i18n;
