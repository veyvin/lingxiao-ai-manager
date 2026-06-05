//! HTTP client used by the local usage reader.

use crate::utils::AppResult;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, COOKIE, ORIGIN, REFERER, USER_AGENT,
};
use std::sync::{Arc, OnceLock};

fn get_shared_client() -> Arc<reqwest::Client> {
    static CLIENT: OnceLock<Arc<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            Arc::new(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .connect_timeout(std::time::Duration::from_secs(10))
                    .pool_max_idle_per_host(10)
                    .tcp_keepalive(std::time::Duration::from_secs(60))
                    .build()
                    .expect("failed to create HTTP client"),
            )
        })
        .clone()
}

pub struct CursorApiClient {
    client: Arc<reqwest::Client>,
    local_session: String,
}

impl CursorApiClient {
    pub fn new(access_token: &str) -> AppResult<Self> {
        Ok(Self {
            client: get_shared_client(),
            local_session: Self::normalize_local_session(access_token),
        })
    }

    fn normalize_local_session(value: &str) -> String {
        let decoded = urlencoding::decode(value)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| value.to_string());

        if decoded.contains("::") {
            return decoded;
        }

        if let Some(subject) = Self::extract_subject_from_jwt(&decoded) {
            return format!("{subject}::{decoded}");
        }

        format!("user_01OOOOOOOOOOOOOOOOOOOOOOOO::{decoded}")
    }

    fn extract_subject_from_jwt(value: &str) -> Option<String> {
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
        let payload_str = String::from_utf8(payload_bytes).ok()?;
        let payload: serde_json::Value = serde_json::from_str(&payload_str).ok()?;
        let sub = payload.get("sub").and_then(|v| v.as_str())?;
        sub.find("user_").map(|pos| sub[pos..].to_string())
    }

    pub(crate) fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );
        headers.insert(ORIGIN, HeaderValue::from_static("https://cursor.com"));
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://cursor.com/cn/dashboard"),
        );

        let cookie = if let Some(pos) = self.local_session.find("::") {
            let subject = &self.local_session[..pos];
            let session = &self.local_session[pos + 2..];
            format!(
                "WorkosCursorSessionToken={}%3A%3A{}",
                urlencoding::encode(subject),
                session
            )
        } else {
            format!(
                "WorkosCursorSessionToken=user_01OOOOOOOOOOOOOOOOOOOOOOOO%3A%3A{}",
                &self.local_session
            )
        };

        if let Ok(value) = HeaderValue::from_str(&cookie) {
            headers.insert(COOKIE, value);
        }

        headers
    }

    pub(crate) fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::COOKIE;

    #[test]
    fn normalize_local_session_keeps_encoded_subject() {
        let encoded = "user_test%3A%3Asession_value";
        let normalized = CursorApiClient::normalize_local_session(encoded);
        assert_eq!(normalized, "user_test::session_value");
    }

    #[test]
    fn normalize_local_session_extracts_subject_from_jwt_payload() {
        let payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"auth0|user_from_payload"}"#);
        let jwt = format!("header.{payload}.signature");
        let normalized = CursorApiClient::normalize_local_session(&jwt);
        assert_eq!(normalized, format!("user_from_payload::{jwt}"));
    }

    #[test]
    fn build_headers_uses_cookie_header_without_exposing_to_public_api() {
        let client = CursorApiClient::new("user_test::session_value").unwrap();
        let headers = client.build_headers();
        let cookie = headers.get(COOKIE).unwrap().to_str().unwrap();
        assert!(cookie.starts_with("WorkosCursorSessionToken=user_test%3A%3A"));
        assert!(cookie.ends_with("session_value"));
    }
}
