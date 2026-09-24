//! Asking the configured AI provider ("clanker") to explain log lines in
//! plain English. Anthropic's Messages API and OpenAI-compatible chat APIs
//! (OpenAI, Mistral, DeepSeek, Qwen, ...) are supported.
//!
//! Requests go through the system `curl` with its whole configuration,
//! API key included, on stdin (`curl --config -`), so the key never appears
//! in argv (visible to other local processes via `ps`).

use std::time::Duration;

use serde_json::{json, Value};

use crate::host::{Host, LocalHost};
use crate::vault::ClankerProviderConfig;

const SYSTEM_PROMPT: &str = "You are a senior Linux site reliability engineer. Explain the systemd journal lines you are given in plain English for an operator who manages this server. Be concise and concrete:\n1. What is happening, in one or two sentences.\n2. Anything that looks wrong or risky, most important first, quoting the relevant line.\n3. The likely cause.\n4. What to check or run next (exact commands where useful).\nIf the lines look healthy, say so briefly. Do not invent details that are not in the lines.";

/// Most characters of log text sent in one request.
pub const MAX_LOG_CHARS: usize = 24_000;

fn is_anthropic(p: &ClankerProviderConfig) -> bool {
    p.id == "anthropic" || p.base_url.contains("anthropic.com")
}

/// A value for curl's config file syntax: double-quoted, with `\` and `"`
/// escaped and line breaks as `\n`.
pub fn curl_config_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The curl config (URL, headers with the key, JSON body) for a request.
pub fn request_config(p: &ClankerProviderConfig, context: &str, logs: &str) -> String {
    let base = p.base_url.trim_end_matches('/');
    let user = format!("{context}\n\nJournal lines:\n{logs}");
    let (url, headers, body) = if is_anthropic(p) {
        (
            format!("{base}/messages"),
            vec![format!("x-api-key: {}", p.api_key), "anthropic-version: 2023-06-01".to_string()],
            json!({"model": p.model, "max_tokens": 1200, "system": SYSTEM_PROMPT, "messages": [{"role": "user", "content": user}]}),
        )
    } else {
        (
            format!("{base}/chat/completions"),
            vec![format!("Authorization: Bearer {}", p.api_key)],
            json!({"model": p.model, "max_tokens": 1200, "messages": [{"role": "system", "content": SYSTEM_PROMPT}, {"role": "user", "content": user}]}),
        )
    };
    let mut cfg = format!("url = {}\n", curl_config_quote(&url));
    for h in headers.iter().chain(std::iter::once(&"Content-Type: application/json".to_string())) {
        cfg.push_str(&format!("header = {}\n", curl_config_quote(h)));
    }
    cfg.push_str(&format!("data = {}\n", curl_config_quote(&body.to_string())));
    cfg.push_str("silent\nshow-error\nmax-time = 90\n");
    cfg
}

/// The model's text from an Anthropic or OpenAI-style response, or the
/// provider's error message.
pub fn parse_response(v: &Value) -> Result<String, String> {
    if let Some(err) = v.get("error") {
        let msg = err["message"].as_str().or_else(|| err.as_str()).unwrap_or("unknown error");
        return Err(format!("the provider said: {msg}"));
    }
    let text = v["content"]
        .as_array()
        .map(|parts| parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join(""))
        .filter(|t| !t.is_empty())
        .or_else(|| v["choices"][0]["message"]["content"].as_str().map(str::to_string));
    text.ok_or_else(|| "the provider's answer had no text".into())
}

/// Asks `provider` to explain `logs` (already capped by the caller).
/// `context` says what the lines are (filters, time range), never which host.
pub fn explain_logs(provider: &ClankerProviderConfig, context: &str, logs: &str) -> Result<String, String> {
    if provider.api_key.trim().is_empty() {
        return Err(format!("{} has no API key; add one in Settings → Clankers", provider.display_name));
    }
    let cfg = request_config(provider, context, logs);
    let out = LocalHost
        .exec_stdin(&["curl", "--config", "-"], cfg.as_bytes(), Duration::from_secs(100))
        .map_err(|e| format!("request to {} failed: {e}", provider.display_name))?;
    let v: Value = serde_json::from_str(&out.stdout).map_err(|_| format!("{} answered with something unexpected: {}", provider.display_name, out.stdout.chars().take(200).collect::<String>()))?;
    parse_response(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(id: &str, base: &str) -> ClankerProviderConfig {
        ClankerProviderConfig {
            id: id.into(),
            display_name: id.into(),
            api_key: "sk-test\"key".into(),
            model: "m".into(),
            base_url: base.into(),
            is_default: true,
            total_calls: 0,
            calls_30d: 0,
            last_used_at: None,
            daily_history: Vec::new(),
        }
    }

    #[test]
    fn config_quoting_survives_quotes_backslashes_and_newlines() {
        assert_eq!(curl_config_quote("a\"b\\c\nd"), "\"a\\\"b\\\\c\\nd\"");
    }

    #[test]
    fn builds_anthropic_and_openai_requests() {
        let a = request_config(&provider("anthropic", "https://api.anthropic.com/v1/"), "ctx", "line");
        assert!(a.starts_with("url = \"https://api.anthropic.com/v1/messages\""));
        assert!(a.contains("header = \"x-api-key: sk-test\\\"key\"") && a.contains("anthropic-version"));
        let o = request_config(&provider("deepseek", "https://api.deepseek.com/v1"), "ctx", "line");
        assert!(o.contains("/v1/chat/completions") && o.contains("Authorization: Bearer"));
    }

    #[test]
    fn parses_both_response_shapes_and_errors() {
        let a: Value = serde_json::from_str(r#"{"content":[{"type":"text","text":"All fine."}]}"#).unwrap();
        assert_eq!(parse_response(&a).unwrap(), "All fine.");
        let o: Value = serde_json::from_str(r#"{"choices":[{"message":{"content":"Disk full."}}]}"#).unwrap();
        assert_eq!(parse_response(&o).unwrap(), "Disk full.");
        let e: Value = serde_json::from_str(r#"{"error":{"message":"invalid x-api-key"}}"#).unwrap();
        assert!(parse_response(&e).unwrap_err().contains("invalid x-api-key"));
    }

    /// curl really sends what the config says: run a tiny local HTTP server,
    /// point the request at it, and compare the body it received.
    #[test]
    fn curl_config_round_trips_through_a_local_server() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let n = s.read(&mut chunk).unwrap();
                buf.extend_from_slice(&chunk[..n]);
                let text = String::from_utf8_lossy(&buf);
                if let Some(i) = text.find("\r\n\r\n") {
                    let len: usize = text.lines().find_map(|l| l.to_lowercase().strip_prefix("content-length: ").map(|v| v.trim().parse().unwrap())).unwrap_or(0);
                    if buf.len() >= i + 4 + len {
                        break;
                    }
                }
            }
            let reply = r#"{"content":[{"type":"text","text":"ok"}]}"#;
            write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", reply.len(), reply).unwrap();
            String::from_utf8_lossy(&buf).to_string()
        });
        let p = provider("anthropic", &format!("http://127.0.0.1:{port}/v1"));
        let logs = "Sep 24 kernel: \"quoted\" \\backslash\\ and\ttab";
        let answer = explain_logs(&p, "ctx", logs).unwrap();
        assert_eq!(answer, "ok");
        let request = server.join().unwrap();
        assert!(request.contains("x-api-key: sk-test\"key"), "header with a quote arrives intact");
        let body: Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert!(body["messages"][0]["content"].as_str().unwrap().ends_with(logs), "log text arrives byte for byte");
    }
}
