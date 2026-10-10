//! A pseudo-language for testing: English with accents and a third more
//! length, so untranslated strings (plain English) and layouts that can't
//! take longer text stand out. Offered in Settings when CROW_I18N_PSEUDO=1.

use std::borrow::Cow;

use super::LanguageInfo;

pub const CODE: &str = "qps";

pub fn info() -> LanguageInfo {
    LanguageInfo { code: CODE.into(), name: "Pseudo".into(), native: "Ƥšéûđö".into(), author: "Crow".into(), ..Default::default() }
}

pub fn enabled() -> bool {
    std::env::var("CROW_I18N_PSEUDO").is_ok_and(|v| v == "1")
}

fn accent(c: char) -> char {
    match c {
        'a' => 'á', 'e' => 'é', 'i' => 'í', 'o' => 'ö', 'u' => 'ü', 'n' => 'ñ', 'c' => 'ç', 's' => 'š', 'z' => 'ž', 'y' => 'ý', 'l' => 'ł', 'd' => 'đ',
        'A' => 'Á', 'E' => 'É', 'I' => 'Í', 'O' => 'Ö', 'U' => 'Ü', 'N' => 'Ñ', 'C' => 'Ç', 'S' => 'Š', 'Z' => 'Ž', 'Y' => 'Ý', 'L' => 'Ł', 'D' => 'Đ',
        c => c,
    }
}

/// Fluent's transform: applied to the text of messages, not their
/// arguments (server names and paths stay readable).
pub fn transform(s: &str) -> Cow<'_, str> {
    if s.trim().is_empty() {
        return Cow::Borrowed(s);
    }
    let letters = s.chars().filter(|c| c.is_alphabetic()).count();
    let mut out: String = s.chars().map(accent).collect();
    let pad = letters.div_ceil(3);
    if pad > 0 {
        // Before trailing spaces: "Name: " becomes "Ñámé:~~ ".
        let end = out.trim_end().len();
        out.insert_str(end, &"~".repeat(pad));
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::transform;

    #[test]
    fn accents_and_pads() {
        assert_eq!(transform("Delete"), "Đéłété~~");
        assert_eq!(transform("Name: "), "Ñámé:~~ ");
        assert_eq!(transform("  "), "  ");
    }
}
