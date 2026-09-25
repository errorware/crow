//! [`Http`] over the system `curl`, with the whole request (URL, headers,
//! credentials, body) written to its stdin as a config file, so nothing
//! secret appears in argv where `ps` shows it to other local users.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use zeroize::Zeroizing;

use crate::http::{Http, HttpRequest, HttpResponse};
use crate::ProviderError;

pub struct CurlHttp {
    pub timeout: Duration,
}

impl Default for CurlHttp {
    fn default() -> Self {
        Self { timeout: Duration::from_secs(30) }
    }
}

/// A value in curl's config syntax: double-quoted, with `\`, `"` and line
/// breaks escaped.
pub fn quote(s: &str) -> String {
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

/// The curl config for `request`. Response headers are included in the
/// output (`include`) so `Retry-After` can be read. It carries credentials,
/// so it (and every piece it's built from) is wiped when dropped.
pub fn config(request: &HttpRequest, timeout: Duration) -> Zeroizing<String> {
    let mut cfg = Zeroizing::new(format!("url = {}\nrequest = {}\n", quote(&request.url), request.method.as_str()));
    for (name, value) in &request.headers {
        let line = Zeroizing::new(format!("{name}: {}", value.expose()));
        cfg.push_str(&Zeroizing::new(format!("header = {}\n", Zeroizing::new(quote(&line)).as_str())));
    }
    if let Some(body) = &request.body {
        cfg.push_str(&format!("header = {}\n", quote("Content-Type: application/json")));
        cfg.push_str(&format!("data = {}\n", quote(&body.to_string())));
    }
    cfg.push_str(&format!("silent\nshow-error\ninclude\nmax-time = {}\n", timeout.as_secs().max(1)));
    cfg
}

/// Splits curl's `include` output into a response: skips interim (1xx)
/// header blocks, reads the status and `Retry-After`, keeps the body.
pub fn parse_output(out: &str) -> Result<HttpResponse, ProviderError> {
    let mut rest = out;
    loop {
        let (head, body) = rest.split_once("\r\n\r\n").or_else(|| rest.split_once("\n\n")).unwrap_or((rest, ""));
        let mut lines = head.lines();
        let status: u16 = lines
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| ProviderError::Unexpected(format!("no HTTP status in curl's output: {}", out.chars().take(120).collect::<String>())))?;
        if (100..200).contains(&status) {
            rest = body;
            continue;
        }
        let retry_after = lines.find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.trim().eq_ignore_ascii_case("retry-after").then(|| v.trim().parse().ok()).flatten()
        });
        return Ok(HttpResponse { status, body: body.to_string(), retry_after });
    }
}

impl Http for CurlHttp {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, ProviderError> {
        let mut child = Command::new("curl")
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ProviderError::Network(format!("can't run curl: {e}")))?;
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(config(request, self.timeout).as_bytes())
            .map_err(|e| ProviderError::Network(format!("can't talk to curl: {e}")))?;
        let out = child.wait_with_output().map_err(|e| ProviderError::Network(e.to_string()))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(ProviderError::Network(err.trim().trim_start_matches("curl: ").to_string()));
        }
        parse_output(&String::from_utf8_lossy(&out.stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SecretValue;
    use serde_json::json;

    #[test]
    fn config_carries_method_headers_and_body() {
        let req = HttpRequest::post("https://api.example.com/x?a=1", json!({"label": "a \"b\""})).bearer(&SecretValue::new("tok"));
        let cfg = config(&req, Duration::from_secs(30));
        assert!(cfg.starts_with("url = \"https://api.example.com/x?a=1\"\nrequest = POST\n"), "{}", cfg.as_str());
        assert!(cfg.contains("header = \"Authorization: Bearer tok\""));
        assert!(cfg.contains(r#"data = "{\"label\":\"a \\\"b\\\"\"}""#), "{}", cfg.as_str());
    }

    #[test]
    fn parses_status_retry_after_and_body_past_interim_responses() {
        let out = "HTTP/1.1 100 Continue\r\n\r\nHTTP/2 429 \r\ncontent-type: application/json\r\nRetry-After: 12\r\n\r\n{\"errors\":[]}";
        let r = parse_output(out).unwrap();
        assert_eq!((r.status, r.retry_after, r.body.as_str()), (429, Some(12), "{\"errors\":[]}"));
    }

    /// curl really sends what the config says: a tiny local server checks it.
    #[test]
    fn round_trips_through_a_local_server() {
        use std::io::Read;
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
            let reply = r#"{"ok":true}"#;
            write!(s, "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", reply.len(), reply).unwrap();
            String::from_utf8_lossy(&buf).to_string()
        });
        let req = HttpRequest::post(format!("http://127.0.0.1:{port}/v4/things"), json!({"label": "x"})).bearer(&SecretValue::new("tok-123"));
        let response = CurlHttp::default().send(&req).unwrap();
        assert_eq!((response.status, response.body.as_str()), (201, r#"{"ok":true}"#));
        let seen = server.join().unwrap();
        assert!(seen.starts_with("POST /v4/things HTTP/1.1"), "{seen}");
        assert!(seen.contains("Authorization: Bearer tok-123"));
        assert!(seen.ends_with(r#"{"label":"x"}"#));
    }
}
