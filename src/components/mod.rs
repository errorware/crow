#![allow(unused_imports)]

pub mod about;
pub mod danger_zone;
pub mod danger_zone_state;
pub mod text_caret;
pub mod icons;
pub mod identity_bar;
pub mod palette;
pub mod sidebar;
pub mod sparkline;
pub mod stat_strip;
pub mod text_input;
pub mod titlebar;

pub use about::about_modal;
pub use icons::{TablerIcon, tabler_icon};
pub use text_input::{handle_text_key_event, terminal_text_input, terminal_text_input_styled};
pub mod window_frame;
