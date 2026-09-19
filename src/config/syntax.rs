use gpui_kit::Rgba;
use crate::theme::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SyntaxToken {
    pub text: String,
    pub color: Rgba,
    pub is_bold: bool,
}

impl SyntaxToken {
    pub fn new(text: impl Into<String>, color: Rgba, is_bold: bool) -> Self {
        Self {
            text: text.into(),
            color,
            is_bold,
        }
    }
}

pub const SYNTAX_KEY: Rgba = hex_rgb(0x67e8f9);       // Bright Cyan
pub const SYNTAX_HEADER: Rgba = hex_rgb(0xc084fc);    // Light Purple
pub const SYNTAX_STRING: Rgba = hex_rgb(0x86efac);    // Mint Green
pub const SYNTAX_NUMBER: Rgba = hex_rgb(0xfba060);    // Warm Amber / Orange
pub const SYNTAX_BOOLEAN: Rgba = hex_rgb(0xf472b6);   // Pink
pub const SYNTAX_KEYWORD: Rgba = hex_rgb(0x38bdf8);   // Sky Blue
pub const SYNTAX_COMMENT: Rgba = hex_rgb(0x71717a);   // Dim Slate Gray
pub const SYNTAX_OPERATOR: Rgba = hex_rgb(0xa1a1aa);  // Muted Operator
pub const SYNTAX_IP: Rgba = hex_rgb(0x2dd4bf);        // Teal
pub const SYNTAX_SPECIAL: Rgba = hex_rgb(0xfacc15);   // Gold / Yellow

/// Tokenize a single configuration line into colored syntax segments.
pub fn highlight_config_line(line: &str, filename: &str) -> Vec<SyntaxToken> {
    if line.is_empty() {
        return vec![SyntaxToken::new(" ", TEXT_PRIMARY, false)];
    }

    let trimmed = line.trim_start();
    let leading_spaces = &line[..line.len() - trimmed.len()];
    let mut tokens = Vec::new();

    if !leading_spaces.is_empty() {
        tokens.push(SyntaxToken::new(leading_spaces, TEXT_PRIMARY, false));
    }

    // 1. Comments: lines starting with #, ;, or //
    if trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with("//") {
        tokens.push(SyntaxToken::new(trimmed, SYNTAX_COMMENT, false));
        return tokens;
    }

    // 2. Section Headers: [section] or [[table]]
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        tokens.push(SyntaxToken::new("[", SYNTAX_OPERATOR, false));
        tokens.push(SyntaxToken::new(inner, SYNTAX_HEADER, true));
        tokens.push(SyntaxToken::new("]", SYNTAX_OPERATOR, false));
        return tokens;
    }

    // Check for inline trailing comments: e.g. "key = val # comment"
    let (content_part, comment_part) = if let Some(idx) = find_comment_start(trimmed) {
        (&trimmed[..idx], Some(&trimmed[idx..]))
    } else {
        (trimmed, None)
    };

    let fn_lower = filename.to_lowercase();

    // 3. PostgreSQL pg_hba.conf line format: TYPE DATABASE USER ADDRESS METHOD
    if fn_lower.contains("pg_hba") {
        tokenize_pg_hba_line(content_part, &mut tokens);
    }
    // 4. Hosts file format: IP HOSTNAME [ALIASES...]
    else if fn_lower == "hosts" {
        tokenize_hosts_line(content_part, &mut tokens);
    }
    // 5. Standard Key-Value or Key: Value or Directive Value
    else if let Some(eq_idx) = content_part.find('=') {
        tokenize_key_value_line(content_part, eq_idx, '=', &mut tokens);
    } else if let Some(colon_idx) = content_part.find(':').filter(|&i| i < content_part.len() - 1 && !content_part.starts_with("::")) {
        tokenize_key_value_line(content_part, colon_idx, ':', &mut tokens);
    } else {
        // Space-separated directives: e.g. "Port 22", "worker_processes auto;", "nameserver 1.1.1.1"
        tokenize_directive_line(content_part, &mut tokens);
    }

    if let Some(comm) = comment_part {
        tokens.push(SyntaxToken::new(comm, SYNTAX_COMMENT, false));
    }

    if tokens.is_empty() {
        tokens.push(SyntaxToken::new(line, TEXT_PRIMARY, false));
    }

    tokens
}

fn find_comment_start(s: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let chars: Vec<(usize, char)> = s.char_indices().collect();

    for (i, &(byte_pos, ch)) in chars.iter().enumerate() {
        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '#' | ';' if !in_single_quote && !in_double_quote => {
                // Must be preceded by whitespace or at start to be a comment
                if byte_pos == 0 || s[..byte_pos].ends_with(char::is_whitespace) {
                    return Some(byte_pos);
                }
            }
            '/' if !in_single_quote && !in_double_quote => {
                if i + 1 < chars.len() && chars[i + 1].1 == '/' {
                    if byte_pos == 0 || s[..byte_pos].ends_with(char::is_whitespace) {
                        return Some(byte_pos);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn tokenize_key_value_line(content: &str, sep_idx: usize, sep_char: char, tokens: &mut Vec<SyntaxToken>) {
    let key_part = &content[..sep_idx];
    let val_part = &content[sep_idx + 1..];

    // Key
    tokens.push(SyntaxToken::new(key_part, SYNTAX_KEY, false));
    // Separator
    tokens.push(SyntaxToken::new(sep_char.to_string(), SYNTAX_OPERATOR, false));

    // Value
    tokenize_value_tokens(val_part, tokens);
}

fn tokenize_value_tokens(val: &str, tokens: &mut Vec<SyntaxToken>) {
    let mut rest = val;
    while !rest.is_empty() {
        let trimmed = rest.trim_start();
        let leading = &rest[..rest.len() - trimmed.len()];
        if !leading.is_empty() {
            tokens.push(SyntaxToken::new(leading, TEXT_PRIMARY, false));
        }
        if trimmed.is_empty() {
            break;
        }

        // Quoted string
        if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            let quote = trimmed.chars().next().unwrap();
            if let Some(end_idx) = trimmed[1..].find(quote) {
                let s = &trimmed[..end_idx + 2];
                tokens.push(SyntaxToken::new(s, SYNTAX_STRING, false));
                rest = &trimmed[end_idx + 2..];
                continue;
            } else {
                tokens.push(SyntaxToken::new(trimmed, SYNTAX_STRING, false));
                break;
            }
        }

        // Word
        let word_end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let word = &trimmed[..word_end];
        let tok = classify_word(word);
        tokens.push(tok);
        rest = &trimmed[word_end..];
    }
}

fn tokenize_directive_line(content: &str, tokens: &mut Vec<SyntaxToken>) {
    let parts: Vec<&str> = content.split_inclusive(char::is_whitespace).collect();
    for (i, p) in parts.iter().enumerate() {
        let trimmed = p.trim();
        let whitespace = &p[trimmed.len()..];

        if i == 0 && !trimmed.is_empty() {
            // Directive keyword
            tokens.push(SyntaxToken::new(trimmed, SYNTAX_KEYWORD, true));
        } else if !trimmed.is_empty() {
            tokens.push(classify_word(trimmed));
        }

        if !whitespace.is_empty() {
            tokens.push(SyntaxToken::new(whitespace, TEXT_PRIMARY, false));
        }
    }
}

fn tokenize_pg_hba_line(content: &str, tokens: &mut Vec<SyntaxToken>) {
    let parts: Vec<&str> = content.split_inclusive(char::is_whitespace).collect();
    let non_empty_count = parts.iter().filter(|p| !p.trim().is_empty()).count();

    let mut col = 0;
    for p in parts {
        let trimmed = p.trim();
        let whitespace = &p[trimmed.len()..];

        if !trimmed.is_empty() {
            let color = match col {
                0 => SYNTAX_KEYWORD, // TYPE (local, host, hostssl)
                1 => SYNTAX_HEADER,  // DATABASE (all, postgres, acme_prod)
                2 => SYNTAX_KEY,     // USER (all, postgres, acme_app)
                3 if non_empty_count >= 5 => {
                    if is_ip_or_cidr(trimmed) {
                        SYNTAX_IP
                    } else {
                        SYNTAX_NUMBER
                    }
                }
                _ => {
                    // Auth method (scram-sha-256, md5, trust, reject)
                    match trimmed {
                        "scram-sha-256" | "cert" => OK,
                        "trust" | "reject" => CRIT,
                        "md5" | "ldap" => WARN,
                        _ => SYNTAX_SPECIAL,
                    }
                }
            };
            tokens.push(SyntaxToken::new(trimmed, color, col == 0 || trimmed == "trust" || trimmed == "reject"));
            col += 1;
        }

        if !whitespace.is_empty() {
            tokens.push(SyntaxToken::new(whitespace, TEXT_PRIMARY, false));
        }
    }
}

fn tokenize_hosts_line(content: &str, tokens: &mut Vec<SyntaxToken>) {
    let parts: Vec<&str> = content.split_inclusive(char::is_whitespace).collect();
    for (i, p) in parts.iter().enumerate() {
        let trimmed = p.trim();
        let whitespace = &p[trimmed.len()..];

        if !trimmed.is_empty() {
            if i == 0 || is_ip_or_cidr(trimmed) {
                tokens.push(SyntaxToken::new(trimmed, SYNTAX_IP, true));
            } else {
                tokens.push(SyntaxToken::new(trimmed, SYNTAX_KEY, false));
            }
        }

        if !whitespace.is_empty() {
            tokens.push(SyntaxToken::new(whitespace, TEXT_PRIMARY, false));
        }
    }
}

fn classify_word(w: &str) -> SyntaxToken {
    let clean = w.trim_end_matches(';').trim_end_matches(',');

    // Booleans
    match clean.to_lowercase().as_str() {
        "true" | "false" | "yes" | "no" | "on" | "off" => {
            return SyntaxToken::new(w, SYNTAX_BOOLEAN, true);
        }
        "null" | "none" | "nil" => {
            return SyntaxToken::new(w, SYNTAX_OPERATOR, false);
        }
        _ => {}
    }

    // IP or CIDR
    if is_ip_or_cidr(clean) {
        return SyntaxToken::new(w, SYNTAX_IP, false);
    }

    // Numbers & unit sizes (e.g. 1024, 4096M, 30day, 65s, 0.0.0.0)
    if is_number_or_unit(clean) {
        return SyntaxToken::new(w, SYNTAX_NUMBER, false);
    }

    // Quoted strings
    if (w.starts_with('"') && w.ends_with('"')) || (w.starts_with('\'') && w.ends_with('\'')) {
        return SyntaxToken::new(w, SYNTAX_STRING, false);
    }

    // Path like /var/log/nginx
    if w.starts_with('/') {
        return SyntaxToken::new(w, SYNTAX_STRING, false);
    }

    // Variables: $uri, $host
    if w.starts_with('$') {
        return SyntaxToken::new(w, SYNTAX_SPECIAL, false);
    }

    // Operators / symbols
    if w == "{" || w == "}" || w == ";" || w == "=" {
        return SyntaxToken::new(w, SYNTAX_OPERATOR, false);
    }

    // Default text
    SyntaxToken::new(w, TEXT_PRIMARY, false)
}

fn is_ip_or_cidr(s: &str) -> bool {
    let without_cidr = if let Some((ip, cidr)) = s.split_once('/') {
        if cidr.parse::<u8>().is_err() {
            return false;
        }
        ip
    } else {
        s
    };

    without_cidr.parse::<std::net::IpAddr>().is_ok()
        || without_cidr == "::1"
        || without_cidr == "0.0.0.0"
        || without_cidr == "127.0.0.1"
}

fn is_number_or_unit(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Strip trailing unit characters (M, G, K, s, m, h, d, day, days, min, ms)
    let digits = s.trim_end_matches(|c: char| c.is_alphabetic());
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_comment() {
        let tokens = highlight_config_line("# This is a comment", "nginx.conf");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].color, SYNTAX_COMMENT);
    }

    #[test]
    fn test_highlight_section() {
        let tokens = highlight_config_line("[systemd]", "journald.conf");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].text, "systemd");
        assert_eq!(tokens[1].color, SYNTAX_HEADER);
        assert!(tokens[1].is_bold);
    }

    #[test]
    fn test_highlight_key_value() {
        let tokens = highlight_config_line("SystemMaxUse=4096M", "journald.conf");
        assert!(tokens.iter().any(|t| t.text == "SystemMaxUse" && t.color == SYNTAX_KEY));
        assert!(tokens.iter().any(|t| t.text == "=" && t.color == SYNTAX_OPERATOR));
        assert!(tokens.iter().any(|t| t.text == "4096M" && t.color == SYNTAX_NUMBER));
    }

    #[test]
    fn test_highlight_booleans() {
        let tokens = highlight_config_line("enabled = true", "config.toml");
        assert!(tokens.iter().any(|t| t.text == "true" && t.color == SYNTAX_BOOLEAN));
    }

    #[test]
    fn test_highlight_pg_hba() {
        let tokens = highlight_config_line("host  all  all  10.0.4.0/24  scram-sha-256", "pg_hba.conf");
        assert!(tokens.iter().any(|t| t.text == "host" && t.color == SYNTAX_KEYWORD));
        assert!(tokens.iter().any(|t| t.text == "10.0.4.0/24" && t.color == SYNTAX_IP));
        assert!(tokens.iter().any(|t| t.text == "scram-sha-256" && t.color == OK));
    }

    #[test]
    fn test_highlight_hosts() {
        let tokens = highlight_config_line("127.0.0.1  localhost  web-01", "hosts");
        assert!(tokens.iter().any(|t| t.text == "127.0.0.1" && t.color == SYNTAX_IP));
    }
}
