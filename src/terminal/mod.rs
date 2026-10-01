//! A real terminal per server (ERR-92), on Zed's fork of `alacritty_terminal`
//! (Apache-2.0): the crate parses the byte stream and keeps the screen; the
//! PTY runs the server's own transport (ssh -tt, container exec, or this
//! machine's shell). Crow draws the screen itself (views::terminal).
//!
//! Everything here is GPUI-free.

pub mod input;
pub mod launch;
pub mod layout;
pub mod line_edit;
pub mod palette;
pub mod session;
