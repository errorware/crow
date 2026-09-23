//! The plain-text config editor: gpui-kit's code editor with Crow's own
//! config-aware syntax colors (`highlight_config_line`) plugged in as its
//! highlighter, for files no crow-config plugin structures.

use std::ops::Range;
use std::rc::Rc;

use gpui_kit::component::input::{
    EditorState, FoldRange, HighlightStyleResolver, InputEdit, InputHighlighter, InputHighlighterFactory, Rope,
};
use gpui_kit::{Context, FontWeight, HighlightStyle, SharedString, Window};

use crate::config::highlight_config_line;

/// Language name the editor is given; it only needs to be non-empty so the
/// editor asks the factory for a highlighter.
pub const CONFIG_LANGUAGE: &str = "crow-config-text";

/// A factory producing a highlighter that colors lines the way `filename`'s
/// format wants (hosts, pg_hba, sshd, ini-style, ...).
pub fn highlighter_factory(filename: &str) -> InputHighlighterFactory {
    let filename = filename.to_string();
    Rc::new(move |_language: &str| {
        Some(Box::new(ConfigHighlighter { filename: filename.clone(), runs: Vec::new() }) as Box<dyn InputHighlighter>)
    })
}

/// Colors a config file line by line with Crow's tokenizer. Config files are
/// small, so each edit simply re-tokenizes the whole text.
pub struct ConfigHighlighter {
    filename: String,
    /// Styled byte ranges in document order, non-overlapping.
    runs: Vec<(Range<usize>, HighlightStyle)>,
}

impl InputHighlighter for ConfigHighlighter {
    fn language(&self) -> SharedString {
        CONFIG_LANGUAGE.into()
    }

    fn update(
        &mut self,
        _edit: Option<InputEdit>,
        text: &Rope,
        _folding: bool,
        _window: &mut Window,
        _cx: &mut Context<EditorState>,
    ) {
        self.runs = highlight_runs(&text.to_string(), &self.filename);
    }

    fn styles(&self, range: &Range<usize>, _resolver: &dyn HighlightStyleResolver) -> Vec<(Range<usize>, HighlightStyle)> {
        cover_range(&self.runs, range)
    }

    fn fold_ranges(&self, _text: &Rope) -> Vec<FoldRange> {
        Vec::new()
    }
}

/// Tokenizes every line of `text` into styled byte ranges.
pub fn highlight_runs(text: &str, filename: &str) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut runs = Vec::new();
    let mut line_start = 0;
    for line in text.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let mut offset = line_start;
        for tok in highlight_config_line(content, filename) {
            // Tokens are consecutive slices of the line; stop if one isn't.
            if !content[offset - line_start..].starts_with(&tok.text) {
                break;
            }
            let end = offset + tok.text.len();
            let style = HighlightStyle {
                color: Some(tok.color.into()),
                font_weight: tok.is_bold.then_some(FontWeight::BOLD),
                ..Default::default()
            };
            runs.push((offset..end, style));
            offset = end;
        }
        line_start += line.len();
    }
    runs
}

/// Returns ordered, non-overlapping runs that fully cover `range`, clipping
/// the styled runs to it and filling gaps with the default style.
pub fn cover_range(runs: &[(Range<usize>, HighlightStyle)], range: &Range<usize>) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut out = Vec::new();
    let mut pos = range.start;
    for (r, style) in runs {
        if r.end <= range.start || r.start >= range.end {
            continue;
        }
        let start = r.start.max(range.start);
        let end = r.end.min(range.end);
        if start > pos {
            out.push((pos..start, HighlightStyle::default()));
        }
        out.push((start..end, *style));
        pos = end;
    }
    if pos < range.end {
        out.push((pos..range.end, HighlightStyle::default()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_cover_requested_range_without_gaps() {
        let text = "127.0.0.1  localhost\n# comment\n10.0.4.12 db-01\n";
        let runs = highlight_runs(text, "hosts");
        assert!(!runs.is_empty());
        let covered = cover_range(&runs, &(0..text.len()));
        assert_eq!(covered.first().unwrap().0.start, 0);
        assert_eq!(covered.last().unwrap().0.end, text.len());
        for pair in covered.windows(2) {
            assert_eq!(pair[0].0.end, pair[1].0.start, "runs must be contiguous");
        }
        // A sub-range in the middle of a token is clipped, not dropped.
        let mid = cover_range(&runs, &(3..14));
        assert_eq!(mid.first().unwrap().0.start, 3);
        assert_eq!(mid.last().unwrap().0.end, 14);
    }

    #[test]
    fn runs_stay_on_char_boundaries_with_multibyte_text() {
        let text = "# café — naïve\nkey = välue\n";
        for (r, _) in highlight_runs(text, "app.conf") {
            assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
        }
    }
}
