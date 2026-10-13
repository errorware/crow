//! User plugins: recipe plugins Crow loads from a folder (ERR-173). Built in
//! stages; this one only reads `plugin.toml` (ERR-175). The built-in
//! integrations in `crate::plugins` are separate and stay as they are.

// Nothing calls into this until the plugins folder is read (ERR-177).
#![allow(dead_code)]

pub mod manifest;
