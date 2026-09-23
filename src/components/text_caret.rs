/// Caret and selection for the app's hand-rolled text fields. Only one such
/// field has focus at a time, so a single caret is shared across screens.
#[derive(Default)]
pub struct TextCaret {
    /// Blink phase; forced on after any keystroke so typing never hides the caret.
    pub blink: bool,
    /// Char index of the caret in the focused field.
    pub cursor: usize,
    /// Selected char range, if any.
    pub selection: Option<(usize, usize)>,
    /// Where a mouse drag-select started.
    pub drag_anchor: Option<usize>,
}

impl TextCaret {
    /// Moves the caret to `pos` (e.g. end of a newly focused field) and clears selection.
    pub fn place(&mut self, pos: usize) {
        self.cursor = pos;
        self.selection = None;
    }
}
