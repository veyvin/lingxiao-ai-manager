//! Private local account store for user-owned Cursor sessions.

use crate::utils::{AppError, AppResult};
use base64::{
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
    Engine as _,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCursorAccount {
    pub label: Option<String>,
    pub session: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManagedCursorAccount {
    pub index: usize,
    pub label: String,
    pub email: Option<String>,
    pub subject_hint: Option<String>,
    pub status: String,
    pub expires_at: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedCursorSession {
    pub normalized_session: String,
    pub jwt: String,
    pub subject: Option<String>,
    pub email: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_workos_session: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AccountStoreFile {
    #[serde(default)]
    cursor_accounts: Vec<StoredCursorAccount>,
}

pub struct CursorAccountStore {
    path: PathBuf,
}

impl CursorAccountStore {
    pub fn new() -> AppResult<Self> {
        let path = dirs::config_dir()
            .map(|base| base.join("LingxiaoAIManager").join("cursor-accounts.json"))
            .ok_or(AppError::CursorDataNotFound)?;
        Ok(Self { path })
    }

    pub fn load(&self) -> AppResult<Vec<StoredCursorAccount>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)?;
        let parsed: AccountStoreFile = serde_json::from_str(&content)?;
        Ok(parsed.cursor_accounts)
    }

    pub fn save(&self, accounts: &[StoredCursorAccount]) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = AccountStoreFile {
            cursor_accounts: accounts.to_vec(),
        };
        let json = serde_json::to_string_pretty(&payload)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn add(&self, label: Option<String>, session: String) -> AppResult<usize> {
        let parsed = parse_cursor_session(&session)?;
        let mut accounts = self.load()?;
        if accounts
            .iter()
            .filter_map(|account| parse_cursor_session(&account.session).ok())
            .any(|existing| existing.jwt == parsed.jwt)
        {
            return Err(AppError::Unknown(
                "This account already exists in the local account store".to_string(),
            ));
        }

        accounts.push(StoredCursorAccount {
            label: clean_label(label),
            session: parsed.normalized_session,
            created_at: Utc::now(),
        });
        let index = accounts.len() - 1;
        self.save(&accounts)?;
        Ok(index)
    }

    pub fn delete(&self, index: usize) -> AppResult<()> {
        let mut accounts = self.load()?;
        if index >= accounts.len() {
            return Err(AppError::Unknown("Account index was not found".to_string()));
        }
        accounts.remove(index);
        self.save(&accounts)
    }

    pub fn list(&self, current_session: Option<&str>) -> AppResult<Vec<ManagedCursorAccount>> {
        let current_jwt = current_session.and_then(|session| parse_cursor_session(session).ok());
        let current_jwt = current_jwt.map(|session| session.jwt);

        Ok(self
            .load()?
            .iter()
            .enumerate()
            .map(|(index, account)| describe_account(index, account, current_jwt.as_deref()))
            .collect())
    }
}

pub fn parse_cursor_session(input: &str) -> AppResult<ParsedCursorSession> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidToken);
    }

    let decoded = urlencoding::decode(trimmed)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| trimmed.to_string());

    let (jwt, is_workos_session) = if let Some((_, token)) = decoded.split_once("::") {
        (token.trim().to_string(), true)
    } else {
        (decoded.clone(), false)
    };

    let payload = decode_jwt_payload(&jwt).unwrap_or_default();
    let subject = payload
        .get("sub")
        .or_else(|| payload.get("userId"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let email = payload
        .get("email")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let expires_at = payload
        .get("exp")
        .and_then(|value| value.as_i64())
        .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0));

    if jwt.split('.').count() != 3 {
        return Err(AppError::InvalidToken);
    }

    Ok(ParsedCursorSession {
        normalized_session: decoded,
        jwt,
        subject,
        email,
        expires_at,
        is_workos_session,
    })
}

fn decode_jwt_payload(jwt: &str) -> Option<serde_json::Value> {
    let payload = jwt.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn clean_label(label: Option<String>) -> Option<String> {
    label
        .map(|value| value.trim().chars().take(80).collect::<String>())
        .filter(|value| !value.is_empty())
}

fn describe_account(
    index: usize,
    account: &StoredCursorAccount,
    current_jwt: Option<&str>,
) -> ManagedCursorAccount {
    match parse_cursor_session(&account.session) {
        Ok(parsed) => {
            let expired = parsed
                .expires_at
                .map(|expires_at| expires_at < Utc::now())
                .unwrap_or(false);
            let status = if expired { "expired" } else { "valid" }.to_string();
            let label = account
                .label
                .clone()
                .or_else(|| parsed.email.clone())
                .or_else(|| parsed.subject.as_ref().map(|value| redact_hint(value)))
                .unwrap_or_else(|| format!("Account {}", index + 1));

            ManagedCursorAccount {
                index,
                label,
                email: parsed.email,
                subject_hint: parsed.subject.map(|value| redact_hint(&value)),
                status,
                expires_at: parsed.expires_at.map(|value| value.to_rfc3339()),
                is_current: current_jwt == Some(parsed.jwt.as_str()),
            }
        }
        Err(_) => ManagedCursorAccount {
            index,
            label: account
                .label
                .clone()
                .unwrap_or_else(|| format!("Account {}", index + 1)),
            email: None,
            subject_hint: None,
            status: "invalid".to_string(),
            expires_at: None,
            is_current: false,
        },
    }
}

fn redact_hint(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return "***".to_string();
    }
    format!(
        "{}...{}",
        chars.iter().take(4).collect::<String>(),
        chars
            .iter()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::parse_cursor_session;

    const JWT: &str = "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ1c2VyXzEyMzQ1Njc4IiwiZW1haWwiOiJkZXYuZXhhbXBsZUBleGFtcGxlLmNvbSIsImV4cCI6NDEwMjQ0NDgwMH0.";

    #[test]
    fn parses_plain_jwt_without_exposing_session() {
        let parsed = parse_cursor_session(JWT).unwrap();
        assert_eq!(parsed.email.as_deref(), Some("dev.example@example.com"));
        assert!(!parsed.is_workos_session);
    }

    #[test]
    fn parses_workos_session() {
        let parsed = parse_cursor_session(&format!("user_1::{JWT}")).unwrap();
        assert_eq!(parsed.jwt, JWT);
        assert!(parsed.is_workos_session);
    }

    #[test]
    fn rejects_empty_session() {
        assert!(parse_cursor_session(" ").is_err());
    }
}
