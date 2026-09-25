//! The HTTP that providers speak, as plain data.
//!
//! Providers build [`HttpRequest`]s and send them through whichever [`Http`]
//! they were given. Credentials go in [`HeaderValue::Secret`] headers, which
//! print as `[secret]`.

use serde::de::DeserializeOwned;
use serde_json::Value;
use zeroize::Zeroizing;

use crate::{ProviderError, SecretValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeaderValue {
    Plain(String),
    Secret(SecretValue),
}

impl HeaderValue {
    /// The value to put on the wire.
    pub fn expose(&self) -> &str {
        match self {
            Self::Plain(s) => s,
            Self::Secret(s) => s.expose(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, HeaderValue)>,
    /// A JSON body, sent with `Content-Type: application/json`.
    pub body: Option<Value>,
}

impl HttpRequest {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self { method, url: url.into(), headers: vec![("Accept".into(), HeaderValue::Plain("application/json".into()))], body: None }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    pub fn post(url: impl Into<String>, body: Value) -> Self {
        Self::new(Method::Post, url).json(body)
    }

    pub fn json(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    pub fn header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), HeaderValue::Plain(value.into())));
        self
    }

    /// `Authorization: Bearer <token>`, kept secret.
    pub fn bearer(mut self, token: &SecretValue) -> Self {
        self.headers.push(("Authorization".into(), HeaderValue::Secret(SecretValue::new(format!("Bearer {}", token.expose())))));
        self
    }

    /// `Authorization: Basic <base64(user:password)>`, kept secret.
    pub fn basic_auth(mut self, user: &str, password: &SecretValue) -> Self {
        // Every intermediate copy of the password is wiped when dropped.
        let pair = Zeroizing::new(format!("{user}:{}", password.expose()));
        let encoded = Zeroizing::new(base64(pair.as_bytes()));
        self.headers.push(("Authorization".into(), HeaderValue::Secret(SecretValue::new(format!("Basic {}", encoded.as_str())))));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    /// `Retry-After` in seconds, when the server sent one.
    pub retry_after: Option<u64>,
}

impl HttpResponse {
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Self { status, body: body.into(), retry_after: None }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Sends requests. Crow's implementation shells out to `curl`; tests use
/// [`crate::testing::RecordedHttp`].
pub trait Http: Send + Sync {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, ProviderError>;
}

/// Turns a response into `T`, or into the matching [`ProviderError`].
/// `message` pulls the provider's error text out of an error body.
pub fn decode<T: DeserializeOwned>(response: HttpResponse, message: fn(&Value) -> Option<String>) -> Result<T, ProviderError> {
    let response = check(response, message)?;
    serde_json::from_str(&response.body).map_err(|e| ProviderError::Unexpected(format!("{e} in {}", snippet(&response.body))))
}

/// Passes a 2xx response through, or maps it to a [`ProviderError`].
pub fn check(response: HttpResponse, message: fn(&Value) -> Option<String>) -> Result<HttpResponse, ProviderError> {
    if response.is_success() {
        return Ok(response);
    }
    let text = serde_json::from_str::<Value>(&response.body).ok().and_then(|v| message(&v)).unwrap_or_else(|| snippet(&response.body));
    Err(match response.status {
        401 | 403 => ProviderError::Auth(text),
        404 => ProviderError::NotFound(text),
        429 => ProviderError::RateLimited { retry_after: response.retry_after },
        status => ProviderError::Provider { status, message: text },
    })
}

fn snippet(body: &str) -> String {
    let s: String = body.chars().take(200).collect();
    if s.trim().is_empty() { "(empty body)".into() } else { s }
}

/// Standard base64 with padding.
pub fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ABC[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648_vectors() {
        for (input, expected) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="), ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")] {
            assert_eq!(base64(input.as_bytes()), expected);
        }
    }

    #[test]
    fn credentials_never_show_in_debug() {
        let req = HttpRequest::get("https://api.example.com/v1/things").bearer(&SecretValue::new("tok-SECRET")).basic_auth("me", &SecretValue::new("pw-SECRET"));
        let debug = format!("{req:?}");
        assert!(!debug.contains("SECRET"), "{debug}");
        assert_eq!(req.headers[1].1.expose(), "Bearer tok-SECRET");
        assert_eq!(req.headers[2].1.expose(), format!("Basic {}", base64(b"me:pw-SECRET")));
    }

    #[test]
    fn status_codes_map_to_errors() {
        let msg = |v: &Value| v["errors"][0]["reason"].as_str().map(str::to_string);
        let err = |status, body: &str| check(HttpResponse::new(status, body), msg).unwrap_err();
        assert_eq!(err(401, r#"{"errors":[{"reason":"Invalid Token"}]}"#), ProviderError::Auth("Invalid Token".into()));
        assert_eq!(err(404, "not json"), ProviderError::NotFound("not json".into()));
        let limited = check(HttpResponse { status: 429, body: String::new(), retry_after: Some(30) }, msg).unwrap_err();
        assert_eq!(limited.to_string(), "rate limited by the provider; try again in 30s");
        assert_eq!(err(500, ""), ProviderError::Provider { status: 500, message: "(empty body)".into() });
        let ok: Value = decode(HttpResponse::new(200, r#"{"a":1}"#), msg).unwrap();
        assert_eq!(ok["a"], 1);
    }
}
