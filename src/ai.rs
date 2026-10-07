//! Asking the configured AI provider ("clanker") to explain log lines in
//! plain English. Anthropic's Messages API and OpenAI-compatible chat APIs
//! (OpenAI, Mistral, DeepSeek, Qwen, ...) are supported.
//!
//! Requests go through the system `curl` with its whole configuration,
//! API key included, on stdin (`curl --config -`), so the key never appears
//! in argv (visible to other local processes via `ps`).

use std::time::Duration;

use serde_json::{json, Value};
use zeroize::Zeroizing;

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
/// It holds the API key, so it's wiped when dropped.
pub fn request_config(p: &ClankerProviderConfig, context: &str, logs: &str) -> Zeroizing<String> {
    chat_config(p, SYSTEM_PROMPT, &format!("{context}\n\nJournal lines:\n{logs}"), 1200)
}

fn chat_config(p: &ClankerProviderConfig, system: &str, user: &str, max_tokens: u32) -> Zeroizing<String> {
    let base = p.base_url.trim_end_matches('/');
    let (url, headers, body) = if is_anthropic(p) {
        (
            format!("{base}/messages"),
            auth_headers(p),
            json!({"model": p.model, "max_tokens": max_tokens, "system": system, "messages": [{"role": "user", "content": user}]}),
        )
    } else {
        (
            format!("{base}/chat/completions"),
            auth_headers(p),
            json!({"model": p.model, "max_tokens": max_tokens, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}]}),
        )
    };
    let mut cfg = Zeroizing::new(format!("url = {}\n", curl_config_quote(&url)));
    for h in headers.iter().map(|h| h.as_str()).chain(std::iter::once("Content-Type: application/json")) {
        cfg.push_str(&Zeroizing::new(format!("header = {}\n", curl_config_quote(h))));
    }
    cfg.push_str(&format!("data = {}\n", curl_config_quote(&body.to_string())));
    cfg.push_str("silent\nshow-error\nmax-time = 90\n");
    cfg
}

/// A provider's error, in the shapes they use: `{"error": {"message"}}`
/// (OpenAI, DeepSeek, Anthropic), `{"error": "…"}`, Mistral's
/// `{"detail": "…"}` and a bare `{"message": "…"}`.
fn provider_error(v: &Value) -> Option<String> {
    let msg = match v.get("error") {
        Some(err) => err["message"].as_str().or_else(|| err.as_str()).unwrap_or("unknown error").to_string(),
        None => v["detail"].as_str().or_else(|| v["detail"][0]["msg"].as_str()).or_else(|| v["message"].as_str().filter(|_| v.get("data").is_none() && v.get("choices").is_none()))?.to_string(),
    };
    Some(format!("the provider said: {msg}"))
}

/// The model's text from an Anthropic or OpenAI-style response, or the
/// provider's error message.
pub fn parse_response(v: &Value) -> Result<String, String> {
    if let Some(e) = provider_error(v) {
        return Err(e);
    }
    let text = v["content"]
        .as_array()
        .map(|parts| parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join(""))
        .filter(|t| !t.is_empty())
        .or_else(|| v["choices"][0]["message"]["content"].as_str().map(str::to_string));
    text.ok_or_else(|| "the provider's answer had no text".into())
}

/// The headers that carry `p`'s key (Anthropic's own, or Bearer).
fn auth_headers(p: &ClankerProviderConfig) -> Vec<Zeroizing<String>> {
    if is_anthropic(p) {
        vec![Zeroizing::new(format!("x-api-key: {}", p.api_key)), Zeroizing::new("anthropic-version: 2023-06-01".to_string())]
    } else {
        vec![Zeroizing::new(format!("Authorization: Bearer {}", p.api_key))]
    }
}

/// The curl config for `GET {base}/models`: free, no tokens spent.
fn models_config(p: &ClankerProviderConfig) -> Zeroizing<String> {
    let mut cfg = Zeroizing::new(format!("url = {}\n", curl_config_quote(&format!("{}/models", p.base_url.trim_end_matches('/')))));
    for h in auth_headers(p) {
        cfg.push_str(&Zeroizing::new(format!("header = {}\n", curl_config_quote(&h))));
    }
    cfg.push_str("silent\nshow-error\nmax-time = 30\n");
    cfg
}

/// The model ids in a `/models` answer (`{"data": [{"id": …}, …]}`, the
/// shape OpenAI, Mistral, DeepSeek, Qwen and Anthropic all use).
pub fn parse_models(v: &Value) -> Result<Vec<String>, String> {
    if let Some(e) = provider_error(v) {
        return Err(e);
    }
    let mut ids: Vec<String> = v["data"].as_array().ok_or("the provider's model list had no data")?.iter().filter_map(|m| m["id"].as_str().map(str::to_string)).collect();
    ids.sort();
    Ok(ids)
}

/// The models `provider`'s key can use, as the provider lists them.
pub fn list_models(provider: &ClankerProviderConfig) -> Result<Vec<String>, String> {
    let v = fetch(provider, &models_config(provider))?;
    parse_models(&v)
}

/// Checks `provider`'s key and model without spending tokens: lists its
/// models and looks for the configured one.
pub fn check_key(provider: &ClankerProviderConfig) -> Result<String, String> {
    let models = list_models(provider)?;
    if models.iter().any(|m| m == &provider.model) {
        Ok(format!("key works · {} is available", provider.model))
    } else {
        Err(format!("the key works, but {} isn't one of its {} models: pick one with LOAD MODELS", provider.model, models.len()))
    }
}

const SERVICE_PROMPT: &str = "You explain systemd services to an operator who is not a Linux expert, like a patient senior SRE would. You are given one service's unit name, description, current state and the server's distribution. Answer in this exact shape:\nSEVERITY: <LOW|MEDIUM|HIGH|CRITICAL> - <one short reason>\n\nWhat it is: <one or two sentences, plain words>\nWhat it does here: <one or two sentences>\nIf you stop it: <what breaks or keeps working>\nIf you restart it: <what users would notice>\nIf you disable it: <what changes at the next boot>\n\nSEVERITY rates stopping or restarting it: LOW when nothing users rely on breaks, MEDIUM when a feature degrades, HIGH when a service users rely on goes down, CRITICAL when it can cut off access to the server (SSH, networking, firewall, storage, the init system). If you don't recognize the service, say so and rate from the description. Don't invent what isn't given.";

/// What Crow sends about a service: no host name, address or anything
/// else that identifies the server.
pub struct ServiceFacts<'a> {
    pub unit: &'a str,
    pub description: &'a str,
    pub state: &'a str,
    pub distro: &'a str,
}

/// How risky stopping or restarting a service is, as the model rated it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// The rating line (`SEVERITY: HIGH - reason`) split from the rest; no
/// rating when the model didn't give one in that shape.
pub fn split_severity(answer: &str) -> (Option<(Severity, String)>, String) {
    let trimmed = answer.trim_start();
    let (first, rest) = trimmed.split_once('\n').unwrap_or((trimmed, ""));
    let Some(after) = first.trim().trim_start_matches(['*', '#', ' ']).strip_prefix("SEVERITY:").or_else(|| first.trim().trim_start_matches(['*', '#', ' ']).strip_prefix("Severity:")) else {
        return (None, answer.trim().to_string());
    };
    let after = after.trim().trim_matches('*').trim();
    let (word, reason) = after.split_once(['-', '—', ':']).map(|(w, r)| (w.trim(), r.trim())).unwrap_or((after, ""));
    let level = match word.to_uppercase().as_str() {
        "LOW" => Severity::Low,
        "MEDIUM" => Severity::Medium,
        "HIGH" => Severity::High,
        "CRITICAL" => Severity::Critical,
        _ => return (None, answer.trim().to_string()),
    };
    (Some((level, reason.trim_matches('*').trim().to_string())), rest.trim().to_string())
}

/// Asks for a plain-words explanation of a service and how risky
/// stopping or restarting it is (primary, then the backup).
pub fn explain_service(primary: &ClankerProviderConfig, backup: Option<&ClankerProviderConfig>, f: &ServiceFacts) -> Result<Answer, String> {
    let user = format!("Unit: {}\nDescription: {}\nState: {}\nDistribution: {}", f.unit, f.description, f.state, if f.distro.is_empty() { "unknown" } else { f.distro });
    with_fallback(primary, backup, |p| send(p, &chat_config(p, SERVICE_PROMPT, &user, 700)))
}

/// An answer, with who gave it and, when the backup did, why the primary didn't.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub text: String,
    pub provider_id: String,
    /// "Mistral AI · mistral-small-2603".
    pub provider_label: String,
    pub primary_failed: Option<String>,
}

fn label(p: &ClankerProviderConfig) -> String {
    format!("{} · {}", p.display_name, p.model)
}

/// Asks the primary; if it fails, the backup. `ask` makes the request.
pub fn with_fallback(primary: &ClankerProviderConfig, backup: Option<&ClankerProviderConfig>, ask: impl Fn(&ClankerProviderConfig) -> Result<String, String>) -> Result<Answer, String> {
    match ask(primary) {
        Ok(text) => Ok(Answer { text, provider_id: primary.id.clone(), provider_label: label(primary), primary_failed: None }),
        Err(first) => match backup {
            Some(b) => ask(b)
                .map(|text| Answer { text, provider_id: b.id.clone(), provider_label: label(b), primary_failed: Some(format!("{}: {first}", primary.display_name)) })
                .map_err(|second| format!("{first}; the backup failed too: {second}")),
            None => Err(first),
        },
    }
}

/// `explain_logs` on the primary, falling back to the backup.
pub fn explain_logs_with_fallback(primary: &ClankerProviderConfig, backup: Option<&ClankerProviderConfig>, context: &str, logs: &str) -> Result<Answer, String> {
    with_fallback(primary, backup, |p| explain_logs(p, context, logs))
}

/// Asks `provider` to explain `logs` (already capped by the caller).
/// `context` says what the lines are (filters, time range), never which host.
pub fn explain_logs(provider: &ClankerProviderConfig, context: &str, logs: &str) -> Result<String, String> {
    send(provider, &request_config(provider, context, logs))
}

fn send(provider: &ClankerProviderConfig, cfg: &str) -> Result<String, String> {
    parse_response(&fetch(provider, cfg)?)
}

/// Runs curl with `cfg` (on stdin: it holds the key) and reads the JSON.
fn fetch(provider: &ClankerProviderConfig, cfg: &str) -> Result<Value, String> {
    if provider.api_key.trim().is_empty() {
        return Err(format!("{} has no API key; add one in Settings → Clankers", provider.display_name));
    }
    let out = LocalHost
        .exec_stdin(&["curl", "--config", "-"], cfg.as_bytes(), Duration::from_secs(100))
        .map_err(|e| format!("request to {} failed: {e}", provider.display_name))?;
    serde_json::from_str(&out.stdout).map_err(|_| format!("{} answered with something unexpected: {}", provider.display_name, out.stdout.chars().take(200).collect::<String>()))
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
            is_backup: false,
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

    #[test]
    fn model_lists_parse_as_the_providers_document_them() {
        // DeepSeek's documented /models example (api-docs.deepseek.com, 2026-10).
        let v: Value = serde_json::from_str(r#"{"object":"list","data":[{"id":"deepseek-v4-pro","object":"model","owned_by":"deepseek"},{"id":"deepseek-flash","object":"model","owned_by":"deepseek","name":"DeepSeek-V4.1-Flash"}]}"#).unwrap();
        assert_eq!(parse_models(&v).unwrap(), ["deepseek-flash", "deepseek-v4-pro"]);
        // Real 401 bodies, 2026-10-07: DeepSeek's and Mistral's (a different shape).
        let deepseek: Value = serde_json::from_str(r#"{"error":{"message":"Authentication Fails, Your api key: ****heck is invalid","type":"authentication_error","param":null,"code":"invalid_request_error"}}"#).unwrap();
        assert!(parse_models(&deepseek).unwrap_err().contains("Authentication Fails"));
        let mistral: Value = serde_json::from_str(r#"{"detail":"Invalid API Key"}"#).unwrap();
        assert_eq!(parse_models(&mistral).unwrap_err(), "the provider said: Invalid API Key");
        assert_eq!(parse_response(&mistral).unwrap_err(), "the provider said: Invalid API Key");
    }

    #[test]
    fn the_backup_answers_only_when_the_primary_fails() {
        let (mistral, deepseek) = (provider("mistral", "x"), provider("deepseek", "y"));
        let ok = with_fallback(&mistral, Some(&deepseek), |p| Ok(format!("from {}", p.id))).unwrap();
        assert_eq!((ok.provider_id.as_str(), ok.primary_failed), ("mistral", None));
        let fell = with_fallback(&mistral, Some(&deepseek), |p| if p.id == "mistral" { Err("429 rate limited".into()) } else { Ok("fine".into()) }).unwrap();
        assert_eq!(fell.provider_id, "deepseek");
        assert_eq!(fell.primary_failed.as_deref(), Some("mistral: 429 rate limited"));
        let both = with_fallback(&mistral, Some(&deepseek), |p| Err(format!("{} down", p.id))).unwrap_err();
        assert!(both.contains("mistral down") && both.contains("backup failed too: deepseek down"));
        assert_eq!(with_fallback(&mistral, None, |_| Err("down".into())).unwrap_err(), "down");
    }

    /// The model list is a plain GET to {base}/models with the key, and no body.
    #[test]
    fn listing_models_is_a_get_with_the_key_and_no_body() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 8192];
            let n = s.read(&mut buf).unwrap();
            let reply = r#"{"object":"list","data":[{"id":"mistral-small-2603"},{"id":"mistral-medium-2604"}]}"#;
            write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", reply.len(), reply).unwrap();
            String::from_utf8_lossy(&buf[..n]).to_string()
        });
        let mut p = provider("mistral", &format!("http://127.0.0.1:{port}/v1/"));
        p.model = "mistral-small-2603".into();
        assert_eq!(check_key(&p).unwrap(), "key works · mistral-small-2603 is available");
        let request = server.join().unwrap();
        assert!(request.starts_with("GET /v1/models HTTP/1.1"), "{request}");
        assert!(request.contains("Authorization: Bearer sk-test\"key"));
        assert!(!request.to_lowercase().contains("content-length"), "no body, nothing billed");
    }

    #[test]
    fn the_severity_line_is_split_from_the_explanation() {
        let (sev, body) = split_severity("SEVERITY: CRITICAL - it's how you log in\n\nWhat it is: the SSH server.");
        assert_eq!(sev, Some((Severity::Critical, "it's how you log in".into())));
        assert_eq!(body, "What it is: the SSH server.");
        let (sev, _) = split_severity("**SEVERITY: low — nothing depends on it**\nWhat it is: …");
        assert_eq!(sev.map(|s| s.0), Some(Severity::Low), "markdown bold and an em dash are fine");
        let (sev, body) = split_severity("I don't know this service.");
        assert_eq!((sev, body.as_str()), (None, "I don't know this service."));
    }
}
