//! Secret redaction for outbound wire payloads (`ERR-211`).
//!
//! The Wire Monitor shows the operator the exact SSH scripts, HTTP requests,
//! AI prompts, OSV queries, and responses that leave or enter the machine,
//! while scrubbing credentials (`Authorization`, `x-api-key`, passwords, and
//! secret values) so the audit log and SQLite `wire_log` table never hold raw
//! secrets.

/// Scrubs credentials from an arbitrary outgoing or incoming payload while
/// keeping commands, URLs, headers, prompts, and JSON bodies intact.
pub fn scrub_payload(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(text.len());
    for (idx, line) in text.lines().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(&scrub_url_passwords(&scrub_line(line)));
    }
    out
}

/// Commands whose output is an app's environment (`dokku config:export`,
/// `config:show`, `config:get`, `config:set` echoing what it set): their
/// replies are all secrets, in formats a line scrubber can't know.
pub fn dumps_environment(request: &str) -> bool {
    request.contains("dokku")
        && ["config:export", "config:show", "config:get", "config:set", "config:unset", "config:bundle"]
            .iter()
            .any(|c| request.contains(c))
}

/// An app's environment with every value blanked and its keys kept, so the
/// ledger still says which variables left: a JSON object, `KEY: value`
/// and `KEY=value` lines and Dokku's banners as they are; anything else
/// (a bare value, a truncated dump) is blanked whole.
pub fn scrub_environment(text: &str) -> String {
    if text.trim().is_empty() {
        return text.to_string();
    }
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(text.trim()) {
        let blank: serde_json::Map<String, serde_json::Value> = map.into_iter().map(|(k, _)| (k, REDACTED.into())).collect();
        return serde_json::Value::Object(blank).to_string();
    }
    text.lines().map(scrub_env_line).collect::<Vec<_>>().join("\n")
}

const REDACTED: &str = "[REDACTED]";

fn scrub_env_line(line: &str) -> String {
    let t = line.trim();
    if t.is_empty() || t.starts_with("=====>") || t.starts_with("----->") || t.starts_with('!') || t == "… [truncated]" {
        return line.to_string();
    }
    let body = t.strip_prefix("export ").unwrap_or(t);
    if let Some(i) = body.find([':', '=']) {
        let key = body[..i].trim();
        let is_key = key.starts_with(|c: char| c.is_ascii_uppercase() || c == '_') && key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if is_key {
            let indent = &line[..line.len() - line.trim_start().len()];
            let export = if t.starts_with("export ") { "export " } else { "" };
            let sep = &body[i..=i];
            return format!("{indent}{export}{key}{sep}{}{REDACTED}", if sep == ":" { " " } else { "" });
        }
    }
    REDACTED.to_string()
}

/// `scheme://user:password@host` → `scheme://user:[REDACTED]@host`, in any
/// payload: database URLs travel in env dumps, service info and links.
fn scrub_url_passwords(line: &str) -> String {
    if !line.contains("://") {
        return line.to_string();
    }
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(i) = rest.find("://") {
        let (head, tail) = rest.split_at(i + 3);
        out.push_str(head);
        let end = tail.find(|c: char| c.is_whitespace() || matches!(c, '/' | '"' | '\'' | '?' | '#' | '`' | '\\')).unwrap_or(tail.len());
        let authority = &tail[..end];
        match (authority.find(':'), authority.rfind('@')) {
            (Some(colon), Some(at)) if colon < at => {
                out.push_str(&authority[..=colon]);
                out.push_str(REDACTED);
                out.push_str(&authority[at..]);
            }
            _ => out.push_str(authority),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn scrub_line(line: &str) -> String {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();

    // 1. curl --config header lines: header = "Authorization: Bearer ..." or header = "x-api-key: ..."
    if lower.starts_with("header") && (lower.contains("authorization:") || lower.contains("x-api-key:") || lower.contains("api-key:")) {
        if let Some(colon) = line.find(':') {
            let prefix = &line[..=colon];
            let rest = line[colon + 1..].trim_start();
            if rest.to_ascii_lowercase().starts_with("bearer ") {
                return format!("{prefix} Bearer [REDACTED]\"");
            }
            if rest.to_ascii_lowercase().starts_with("basic ") {
                return format!("{prefix} Basic [REDACTED]\"");
            }
            return format!("{prefix} [REDACTED]\"");
        }
    }

    // 2. Raw HTTP Authorization / API key headers
    if lower.starts_with("authorization:") {
        let colon = line.find(':').unwrap();
        let prefix = &line[..=colon];
        let rest = line[colon + 1..].trim();
        let scheme = rest.split_whitespace().next().unwrap_or("");
        if scheme.eq_ignore_ascii_case("bearer") || scheme.eq_ignore_ascii_case("basic") {
            return format!("{prefix} {scheme} [REDACTED]");
        }
        return format!("{prefix} [REDACTED]");
    }
    if lower.starts_with("x-api-key:") || lower.starts_with("api-key:") {
        let colon = line.find(':').unwrap();
        return format!("{}: [REDACTED]", &line[..colon]);
    }

    // 3. curl -u user:pass
    if lower.starts_with("-u ") || lower.contains(" --user ") {
        return redact_basic_user_arg(line);
    }

    // 4. JSON secret fields in plugin/provider payloads ("api_key": "...", "api_token": "...", "password": "...")
    if (lower.contains("\"api_key\"")
        || lower.contains("\"api_token\"")
        || lower.contains("\"password\"")
        || lower.contains("\"secret\"")
        || lower.contains("\"token\""))
        && line.contains(':')
    {
        return scrub_json_secrets(line);
    }

    // 5. Dokku encoded config:set commands (`config:set --encoded app KEY=b64...`)
    if line.contains("config:set") {
        return scrub_dokku_encoded_line(line);
    }

    line.to_string()
}

fn redact_basic_user_arg(line: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut redact_next = false;
    for w in line.split_whitespace() {
        if redact_next {
            if let Some((user, _)) = w.split_once(':') {
                words.push(format!("{user}:[REDACTED]"));
            } else {
                words.push("[REDACTED]".into());
            }
            redact_next = false;
        } else if w == "-u" || w == "--user" {
            words.push(w.to_string());
            redact_next = true;
        } else {
            words.push(w.to_string());
        }
    }
    words.join(" ")
}

fn scrub_json_secrets(line: &str) -> String {
    let mut out = line.to_string();
    for key in ["api_key", "api_token", "password", "secret", "private_key", "passphrase"] {
        let needle = format!("\"{key}\"");
        let mut search_from = 0;
        while let Some(pos) = out[search_from..].find(&needle) {
            let abs = search_from + pos + needle.len();
            let after_key = &out[abs..];
            if let Some(colon) = after_key.find(':') {
                let after_colon = &after_key[colon + 1..];
                let ws_len = after_colon.len() - after_colon.trim_start().len();
                let val_start = abs + colon + 1 + ws_len;
                if out[val_start..].starts_with('"') {
                    // Find closing quote (handling escaped quotes)
                    let mut i = val_start + 1;
                    let bytes = out.as_bytes();
                    while i < bytes.len() {
                        if bytes[i] == b'\\' {
                            i += 2;
                            continue;
                        }
                        if bytes[i] == b'"' {
                            break;
                        }
                        i += 1;
                    }
                    if i < bytes.len() {
                        out.replace_range(val_start..=i, "\"[REDACTED]\"");
                        search_from = val_start + "\"[REDACTED]\"".len();
                        continue;
                    }
                }
            }
            search_from = abs;
        }
    }
    out
}

fn scrub_dokku_encoded_line(line: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut after_encoded = false;
    for w in line.split_whitespace() {
        if w == "--encoded" || w.ends_with("config:set") {
            after_encoded = true;
            words.push(w.to_string());
            continue;
        }
        if after_encoded {
            if let Some((k, _v)) = w.split_once('=') {
                if !k.starts_with('-') {
                    words.push(format!("{k}=[REDACTED]"));
                    continue;
                }
            }
        }
        words.push(w.to_string());
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrubs_bearer_and_api_key_headers_and_keeps_prompts() {
        let raw = "url = \"https://api.anthropic.com/v1/messages\"\nheader = \"x-api-key: sk-ant-secret123\"\nheader = \"Authorization: Bearer sk-live-999\"\ndata-raw = \"{\\\"model\\\":\\\"claude-3-5-sonnet\\\",\\\"messages\\\":[{\\\"role\\\":\\\"user\\\",\\\"content\\\":\\\"explain nginx\\\"}]}\"";
        let scrubbed = scrub_payload(raw);
        assert!(!scrubbed.contains("sk-ant-secret123"), "{scrubbed}");
        assert!(!scrubbed.contains("sk-live-999"), "{scrubbed}");
        assert!(scrubbed.contains("x-api-key: [REDACTED]"), "{scrubbed}");
        assert!(scrubbed.contains("Bearer [REDACTED]"), "{scrubbed}");
        assert!(scrubbed.contains("explain nginx"), "{scrubbed}");
    }

    #[test]
    fn scrubs_json_secret_fields_and_dokku_encoded_pairs() {
        let json_line = r#"{"account":"prod","api_token":"linode-secret-token-xyz","region":"us-east"}"#;
        let s1 = scrub_payload(json_line);
        assert!(!s1.contains("linode-secret-token-xyz"), "{s1}");
        assert!(s1.contains("\"api_token\":\"[REDACTED]\""), "{s1}");
        assert!(s1.contains("\"region\":\"us-east\""), "{s1}");

        let dokku_cmd = "dokku config:set --encoded billing-api RAILS_MASTER_KEY=c2VjcmV0 DATABASE_URL=cG9zdGdyZXM=";
        let s2 = scrub_payload(dokku_cmd);
        assert_eq!(s2, "dokku config:set --encoded billing-api RAILS_MASTER_KEY=[REDACTED] DATABASE_URL=[REDACTED]");
        assert_eq!(scrub_payload("dokku config:set shop SECRET_KEY_BASE=abc"), "dokku config:set shop SECRET_KEY_BASE=[REDACTED]");
    }

    #[test]
    fn an_app_environment_keeps_its_keys_and_loses_its_values() {
        // What Crow's SSH layer records for the Env tab's read.
        let req = "sh -c 'if [ \"$(id -u)\" = 0 ]; then exec \"$@\"; else exec sudo -n -- \"$@\"; fi' crow-priv dokku config:export --format json quorum";
        assert!(dumps_environment(req));
        assert!(!dumps_environment("dokku ps:report quorum") && !dumps_environment("cat /etc/app/config:set"));
        let json = r#"{"DATABASE_URL":"postgres://postgres:hunter2@dokku-postgres-db:5432/db","RAILS_MASTER_KEY":"0123abcd"}"#;
        assert_eq!(scrub_environment(json), r#"{"DATABASE_URL":"[REDACTED]","RAILS_MASTER_KEY":"[REDACTED]"}"#);
        // config:set echoes what it set.
        let set = "-----> Setting config vars\n       RAILS_MASTER_KEY:   0123abcd\n       SOLID_QUEUE_IN_PUMA: true";
        assert_eq!(scrub_environment(set), "-----> Setting config vars\n       RAILS_MASTER_KEY: [REDACTED]\n       SOLID_QUEUE_IN_PUMA: [REDACTED]");
        assert_eq!(scrub_environment("export TOKEN='x'\nPORT=5000"), "export TOKEN=[REDACTED]\nPORT=[REDACTED]");
        // A bare value (config:get) or a dump cut short: nothing to keep.
        assert_eq!(scrub_environment("0123abcd"), "[REDACTED]");
        assert_eq!(scrub_environment("{\"RAILS_MASTER_KEY\":\"0123\n… [truncated]"), "[REDACTED]\n… [truncated]");
        assert_eq!(scrub_environment(" !     App quorum does not exist"), " !     App quorum does not exist");
    }

    #[test]
    fn passwords_in_urls_are_blanked_anywhere() {
        let info = "       Dsn:                 postgres://postgres:9f8e7d@dokku-postgres-quorum-db:5432/quorum_db";
        assert_eq!(scrub_payload(info), "       Dsn:                 postgres://postgres:[REDACTED]@dokku-postgres-quorum-db:5432/quorum_db");
        assert_eq!(scrub_payload("redis://:s3cret@cache:6379 and https://api.osv.dev/v1/query"), "redis://:[REDACTED]@cache:6379 and https://api.osv.dev/v1/query");
        assert_eq!(scrub_payload("ssh://root@host:22"), "ssh://root@host:22", "no password, nothing to blank");
    }
}
