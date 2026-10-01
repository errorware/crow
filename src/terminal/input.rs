//! Key presses to the bytes a terminal sends (ERR-93): xterm conventions,
//! with the cursor keys switching to SS3 form in application cursor mode.

/// Modifier state for one key press.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Mods {
    /// xterm's modifier parameter (1 + shift + 2·alt + 4·ctrl); 1 = none.
    fn param(self) -> u8 {
        1 + self.shift as u8 + 2 * self.alt as u8 + 4 * self.ctrl as u8
    }
}

/// The bytes for `key` (GPUI's key name: "a", "enter", "up", "f5", ...),
/// with `typed` the text the key produced, if any. `None` when the key
/// sends nothing.
pub fn encode(key: &str, typed: Option<&str>, m: Mods, app_cursor: bool) -> Option<Vec<u8>> {
    let csi = |final_: &str| -> Vec<u8> {
        match m.param() {
            1 => format!("\x1b[{final_}").into_bytes(),
            p => format!("\x1b[1;{p}{final_}").into_bytes(),
        }
    };
    let tilde = |n: u8| -> Vec<u8> {
        match m.param() {
            1 => format!("\x1b[{n}~").into_bytes(),
            p => format!("\x1b[{n};{p}~").into_bytes(),
        }
    };
    let cursor = |c: &str| -> Vec<u8> { if app_cursor && m.param() == 1 { format!("\x1bO{c}").into_bytes() } else { csi(c) } };
    let alt = |mut b: Vec<u8>| -> Vec<u8> {
        if m.alt {
            b.insert(0, 0x1b);
        }
        b
    };
    let bytes = match key {
        "enter" => alt(b"\r".to_vec()),
        "backspace" => alt(if m.ctrl { vec![0x08] } else { vec![0x7f] }),
        "tab" if m.shift => b"\x1b[Z".to_vec(),
        "tab" => alt(b"\t".to_vec()),
        "escape" => alt(vec![0x1b]),
        "space" if m.ctrl => vec![0x00],
        "up" => cursor("A"),
        "down" => cursor("B"),
        "right" => cursor("C"),
        "left" => cursor("D"),
        "home" => cursor("H"),
        "end" => cursor("F"),
        "insert" => tilde(2),
        "delete" => tilde(3),
        "pageup" => tilde(5),
        "pagedown" => tilde(6),
        "f1" => csi_or_ss3(m, "P"),
        "f2" => csi_or_ss3(m, "Q"),
        "f3" => csi_or_ss3(m, "R"),
        "f4" => csi_or_ss3(m, "S"),
        "f5" => tilde(15),
        "f6" => tilde(17),
        "f7" => tilde(18),
        "f8" => tilde(19),
        "f9" => tilde(20),
        "f10" => tilde(21),
        "f11" => tilde(23),
        "f12" => tilde(24),
        k if m.ctrl && k.chars().count() == 1 => {
            // Ctrl+letter → 0x01..0x1a; Ctrl+[ \ ] ^ _ → 0x1b..0x1f.
            let c = k.chars().next()?.to_ascii_lowercase();
            let code = match c {
                'a'..='z' => c as u8 - b'a' + 1,
                '[' | '3' => 0x1b,
                '\\' | '4' => 0x1c,
                ']' | '5' => 0x1d,
                '^' | '6' => 0x1e,
                '_' | '-' | '7' => 0x1f,
                '@' | '2' => 0x00,
                '8' | '?' => 0x7f,
                _ => return None,
            };
            alt(vec![code])
        }
        _ => alt(typed.filter(|t| !t.is_empty())?.as_bytes().to_vec()),
    };
    Some(bytes)
}

fn csi_or_ss3(m: Mods, c: &str) -> Vec<u8> {
    match m.param() {
        1 => format!("\x1bO{c}").into_bytes(),
        p => format!("\x1b[1;{p}{c}").into_bytes(),
    }
}

/// Wraps pasted text for a shell that asked for bracketed paste, and keeps a
/// pasted end-marker from closing the bracket early.
pub fn paste(text: &str, bracketed: bool) -> Vec<u8> {
    if bracketed {
        let clean = text.replace("\x1b[201~", "");
        format!("\x1b[200~{clean}\x1b[201~").into_bytes()
    } else {
        text.replace("\r\n", "\r").replace('\n', "\r").into_bytes()
    }
}

/// A mouse report for a program that asked for the mouse: `button` is
/// xterm's code (0 left, 1 middle, 2 right, 64/65 wheel up/down, +32 for
/// motion), at a 0-based cell. SGR form when the program enabled it;
/// otherwise the legacy form (UTF-8 extended if asked), which can't
/// express cells past 223 unless extended.
pub fn mouse(button: u8, col: usize, row: usize, pressed: bool, m: Mods, sgr: bool, utf8: bool) -> Option<Vec<u8>> {
    let code = button + 4 * m.shift as u8 + 8 * m.alt as u8 + 16 * m.ctrl as u8;
    if sgr {
        return Some(format!("\x1b[<{code};{};{}{}", col + 1, row + 1, if pressed { 'M' } else { 'm' }).into_bytes());
    }
    // Legacy releases don't say which button.
    let code = if pressed { code } else { 3 + (code & !3) };
    let mut out = b"\x1b[M".to_vec();
    out.push(32 + code);
    for v in [col, row] {
        let v = 32 + 1 + v as u32;
        if utf8 {
            out.extend(char::from_u32(v)?.to_string().into_bytes());
        } else if v <= 255 {
            out.push(v as u8);
        } else {
            return None;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Mods = Mods { ctrl: false, alt: false, shift: false };

    #[test]
    fn text_and_control_keys() {
        assert_eq!(encode("a", Some("a"), NONE, false), Some(b"a".to_vec()));
        assert_eq!(encode("a", Some("A"), Mods { shift: true, ..NONE }, false), Some(b"A".to_vec()));
        assert_eq!(encode("c", Some("c"), Mods { ctrl: true, ..NONE }, false), Some(vec![0x03]));
        assert_eq!(encode("d", None, Mods { ctrl: true, ..NONE }, false), Some(vec![0x04]));
        assert_eq!(encode("[", None, Mods { ctrl: true, ..NONE }, false), Some(vec![0x1b]));
        assert_eq!(encode("b", Some("b"), Mods { alt: true, ..NONE }, false), Some(b"\x1bb".to_vec()));
        assert_eq!(encode("enter", None, NONE, false), Some(b"\r".to_vec()));
        assert_eq!(encode("backspace", None, NONE, false), Some(vec![0x7f]));
        assert_eq!(encode("tab", None, Mods { shift: true, ..NONE }, false), Some(b"\x1b[Z".to_vec()));
        assert_eq!(encode("shift", None, Mods { shift: true, ..NONE }, false), None, "modifier alone sends nothing");
    }

    #[test]
    fn cursor_keys_follow_application_mode_and_modifiers() {
        assert_eq!(encode("up", None, NONE, false), Some(b"\x1b[A".to_vec()));
        assert_eq!(encode("up", None, NONE, true), Some(b"\x1bOA".to_vec()));
        assert_eq!(encode("right", None, Mods { ctrl: true, ..NONE }, true), Some(b"\x1b[1;5C".to_vec()));
        assert_eq!(encode("delete", None, NONE, false), Some(b"\x1b[3~".to_vec()));
        assert_eq!(encode("pageup", None, Mods { shift: true, ..NONE }, false), Some(b"\x1b[5;2~".to_vec()));
        assert_eq!(encode("f1", None, NONE, false), Some(b"\x1bOP".to_vec()));
        assert_eq!(encode("f5", None, NONE, false), Some(b"\x1b[15~".to_vec()));
    }

    #[test]
    fn paste_brackets_when_asked_and_cant_be_escaped() {
        assert_eq!(paste("ls\n", false), b"ls\r".to_vec());
        assert_eq!(paste("rm -rf /\x1b[201~\n", true), b"\x1b[200~rm -rf /\n\x1b[201~".to_vec());
    }

    #[test]
    fn mouse_reports() {
        assert_eq!(mouse(0, 4, 2, true, NONE, true, false), Some(b"\x1b[<0;5;3M".to_vec()));
        assert_eq!(mouse(0, 4, 2, false, NONE, true, false), Some(b"\x1b[<0;5;3m".to_vec()));
        let ctrl = Mods { ctrl: true, ..NONE };
        assert_eq!(mouse(65, 0, 0, true, ctrl, true, false), Some(b"\x1b[<81;1;1M".to_vec()), "wheel down with ctrl");
        assert_eq!(mouse(2, 0, 0, true, NONE, false, false), Some(b"\x1b[M\x22\x21\x21".to_vec()));
        assert_eq!(mouse(2, 0, 0, false, NONE, false, false), Some(b"\x1b[M\x23\x21\x21".to_vec()), "legacy release is button 3");
        assert_eq!(mouse(0, 300, 0, true, NONE, false, false), None, "too far right for the legacy form");
        assert_eq!(mouse(0, 300, 0, true, NONE, false, true).map(|b| String::from_utf8(b).unwrap()), Some("\x1b[M \u{14d}!".to_string()));
    }
}
