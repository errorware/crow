//! Click to copy (ERR-191): the text goes to the clipboard and the element
//! says "copied" for a moment in its place.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use gpui_kit::*;

use crate::theme::*;

const SHOWN: Duration = Duration::from_millis(1500);

/// Which element copied last, and when.
static LAST: Mutex<Option<(String, Instant)>> = Mutex::new(None);

/// Whether the element `key` copied a moment ago.
pub fn flashing(key: &str) -> bool {
    LAST.lock().ok().and_then(|l| l.as_ref().map(|(k, at)| k == key && at.elapsed() < SHOWN)).unwrap_or(false)
}

fn mark(key: &str) {
    if let Ok(mut l) = LAST.lock() {
        *l = Some((key.to_string(), Instant::now()));
    }
}

/// Copies `text`, and redraws once the "copied" mark is due to go.
pub fn copy(key: &str, text: &str, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
    mark(key);
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        cx.background_executor().timer(SHOWN + Duration::from_millis(50)).await;
        let _ = cx.update(|cx| cx.refresh_windows());
    })
    .detach();
}

/// `label`, copying `text` when clicked; "copied" in its place a moment after.
pub fn copyable(key: String, label: impl Into<SharedString>, text: String, ink: Rgba) -> Stateful<Div> {
    let shown = flashing(&key);
    let k = key.clone();
    div()
        .id(ElementId::Name(format!("copy-{key}").into()))
        .cursor_pointer()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(if shown { OK } else { ink })
        .hover(|s| s.text_color(TEXT_PRIMARY))
        .tooltip(|window, cx| gpui_kit::component::tooltip::Tooltip::new(t!("copy-flash-click-to-copy")).build(window, cx))
        .on_click(move |_ev, _window, cx| {
            cx.stop_propagation();
            copy(&k, &text, cx);
        })
        .child(if shown { SharedString::from(t!("copy-flash-copied")) } else { label.into() })
}

#[cfg(test)]
mod tests {
    use super::{flashing, mark};

    #[test]
    fn only_the_element_copied_says_so() {
        assert!(!flashing("fleet-ip-x"));
        mark("fleet-ip-a");
        assert!(flashing("fleet-ip-a"));
        assert!(!flashing("fleet-ip-b"), "another row stays as it was");
        mark("fleet-ip-b");
        assert!(!flashing("fleet-ip-a"), "one at a time");
    }
}
