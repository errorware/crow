//! Country flags (ERR-36). The SVGs are multi-colour, so they're drawn with
//! `img` (full colour) rather than `svg` (a single tint), and decoded once.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use gpui_kit::*;

use crate::theme::*;

fn flag_image(code: &str) -> Option<Arc<Image>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Image>>>> = OnceLock::new();
    let key = code.to_ascii_uppercase();
    let mut cache = CACHE.get_or_init(Default::default).lock().ok()?;
    if let Some(img) = cache.get(&key) {
        return Some(img.clone());
    }
    let bytes = crate::region::flag_svg(&key)?;
    let img = Arc::new(Image::from_bytes(ImageFormat::Svg, bytes.to_vec()));
    cache.insert(key, img.clone());
    Some(img)
}

/// A flag `height` tall (4:3). Countries without a flag show their code in a
/// small badge; an empty code shows nothing.
pub fn flag(code: &str, height: f32) -> AnyElement {
    if code.is_empty() {
        return div().into_any_element();
    }
    match flag_image(code) {
        Some(image) => img(image).h(px(height)).w(px(height * 4.0 / 3.0)).flex_none().rounded(px(1.5)).into_any_element(),
        None => div()
            .flex_none()
            .px(px(3.0))
            .rounded_sm()
            .bg(BG_CHIP)
            .text_color(TEXT_DIM)
            .text_size(px(height * 0.75))
            .font_weight(FontWeight::BOLD)
            .child(code.to_ascii_uppercase())
            .into_any_element(),
    }
}
