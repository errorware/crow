//! A recorded-response [`Http`] for provider tests: answers each request from
//! a table of canned responses and keeps every request it saw.

use std::sync::Mutex;

use crate::http::{Http, HttpRequest, HttpResponse, Method};
use crate::ProviderError;

#[derive(Default)]
pub struct RecordedHttp {
    routes: Vec<(Method, String, HttpResponse)>,
    form_routes: Vec<(String, String, HttpResponse)>,
    seen: Mutex<Vec<HttpRequest>>,
}

impl RecordedHttp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Answers `method url` (exact URL, query string included) with `status`
    /// and `body`.
    pub fn on(mut self, method: Method, url: &str, status: u16, body: &str) -> Self {
        self.routes.push((method, url.into(), HttpResponse::new(status, body)));
        self
    }

    /// Answers a form `POST` to `url` whose `form_body` contains `needle`.
    pub fn on_form(mut self, url: &str, needle: &str, status: u16, body: &str) -> Self {
        self.form_routes.push((url.into(), needle.into(), HttpResponse::new(status, body)));
        self
    }

    /// The requests sent so far.
    pub fn requests(&self) -> Vec<HttpRequest> {
        self.seen.lock().unwrap().clone()
    }
}

impl Http for RecordedHttp {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, ProviderError> {
        self.seen.lock().unwrap().push(request.clone());
        if let Some(form) = &request.form_body {
            if let Some((_, _, r)) = self
                .form_routes
                .iter()
                .find(|(u, needle, _)| *u == request.url && form.contains(needle.as_str()))
            {
                return Ok(r.clone());
            }
        }
        self.routes
            .iter()
            .find(|(m, u, _)| *m == request.method && *u == request.url)
            .map(|(_, _, r)| r.clone())
            .ok_or_else(|| ProviderError::Network(format!("no recorded response for {} {}", request.method.as_str(), request.url)))
    }
}
