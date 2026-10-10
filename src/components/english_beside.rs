//! The English under a translated danger prompt (ERR-171): a language pack
//! is someone else's words, so what a destructive action does is also shown
//! in Crow's own.

use gpui_kit::*;

use crate::theme::*;

/// `EN · …` in small dim text, or nothing when Crow is in English.
pub fn english_beside(english: Option<String>) -> Option<Div> {
    english.map(|en| div().text_size(px(10.0)).text_color(TEXT_FAINT).child(format!("EN · {en}")))
}
