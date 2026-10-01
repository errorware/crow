//! A one-line editor for renaming a tab in place: the tab's own label,
//! with a caret, starting fully selected so typing replaces it. GPUI-free.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineEdit {
    text: Vec<char>,
    /// Caret position, in characters.
    caret: usize,
    /// Everything is selected (the state it opens in).
    all: bool,
}

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditResult {
    Editing,
    /// Keep the name (Enter).
    Commit,
    /// Throw the edit away (Escape).
    Cancel,
}

impl LineEdit {
    pub fn new(text: &str) -> Self {
        let text: Vec<char> = text.chars().collect();
        Self { caret: text.len(), text, all: true }
    }

    pub fn text(&self) -> String {
        self.text.iter().collect()
    }

    /// (before caret, after caret, all selected), for drawing.
    pub fn parts(&self) -> (String, String, bool) {
        (self.text[..self.caret].iter().collect(), self.text[self.caret..].iter().collect(), self.all)
    }

    fn take_selection(&mut self) {
        if self.all {
            self.text.clear();
            self.caret = 0;
            self.all = false;
        }
    }

    /// Applies one key (GPUI key name, the text it typed, Ctrl/⌘ held).
    pub fn key(&mut self, key: &str, typed: Option<&str>, cmd: bool) -> EditResult {
        match key {
            "enter" => return EditResult::Commit,
            "escape" => return EditResult::Cancel,
            "a" if cmd => self.all = true,
            "backspace" | "delete" if self.all => self.take_selection(),
            "backspace" if self.caret > 0 => {
                self.caret -= 1;
                self.text.remove(self.caret);
            }
            "delete" if self.caret < self.text.len() => {
                self.text.remove(self.caret);
            }
            "left" => {
                self.caret = if self.all { 0 } else { self.caret.saturating_sub(1) };
                self.all = false;
            }
            "right" => {
                self.caret = if self.all { self.text.len() } else { (self.caret + 1).min(self.text.len()) };
                self.all = false;
            }
            "home" => {
                self.caret = 0;
                self.all = false;
            }
            "end" => {
                self.caret = self.text.len();
                self.all = false;
            }
            _ if !cmd => {
                if let Some(t) = typed.filter(|t| !t.is_empty() && !t.chars().any(char::is_control)) {
                    self.take_selection();
                    for c in t.chars().take(64usize.saturating_sub(self.text.len())) {
                        self.text.insert(self.caret, c);
                        self.caret += 1;
                    }
                }
            }
            _ => {}
        }
        EditResult::Editing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typing(e: &mut LineEdit, s: &str) {
        for c in s.chars() {
            e.key(&c.to_string(), Some(&c.to_string()), false);
        }
    }

    #[test]
    fn opens_selected_so_typing_replaces() {
        let mut e = LineEdit::new("shell 2");
        typing(&mut e, "db");
        assert_eq!(e.text(), "db");
        assert_eq!(e.key("enter", None, false), EditResult::Commit);
    }

    #[test]
    fn arrows_keep_the_text_and_edit_in_place() {
        let mut e = LineEdit::new("prod");
        e.key("right", None, false);
        typing(&mut e, " logs");
        assert_eq!(e.text(), "prod logs");
        e.key("home", None, false);
        e.key("delete", None, false);
        typing(&mut e, "P");
        assert_eq!(e.text(), "Prod logs");
        e.key("end", None, false);
        e.key("backspace", None, false);
        assert_eq!(e.parts(), ("Prod log".into(), String::new(), false));
    }

    #[test]
    fn select_all_backspace_clears_and_escape_cancels() {
        let mut e = LineEdit::new("x");
        e.key("right", None, false);
        e.key("a", None, true);
        e.key("backspace", None, false);
        assert_eq!(e.text(), "");
        assert_eq!(e.key("escape", None, false), EditResult::Cancel);
    }

    #[test]
    fn names_are_capped_and_control_text_ignored() {
        let mut e = LineEdit::new("");
        typing(&mut e, &"a".repeat(80));
        assert_eq!(e.text().len(), 64);
        e.key("x", Some("\u{1b}"), false);
        assert_eq!(e.text().len(), 64);
    }
}
