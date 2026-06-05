//! Local account reader.

use crate::cursor::types::LocalAccountInfo;
use crate::utils::{AppError, AppResult};
use rusqlite::Connection;
use std::path::PathBuf;

pub struct AccountReader {
    base_path: PathBuf,
}

impl AccountReader {
    pub fn new() -> AppResult<Self> {
        let base_path = Self::get_cursor_base_path()?;
        Ok(Self { base_path })
    }

    pub fn cursor_state_db_path() -> AppResult<PathBuf> {
        Ok(Self::get_cursor_base_path()?.join("state.vscdb"))
    }

    fn get_cursor_base_path() -> AppResult<PathBuf> {
        let path = if cfg!(windows) {
            dirs::config_dir().map(|p| p.join("Cursor").join("User").join("globalStorage"))
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|p| p.join("Library/Application Support/Cursor/User/globalStorage"))
        } else {
            dirs::config_dir().map(|p| p.join("Cursor").join("User").join("globalStorage"))
        };

        path.ok_or(AppError::CursorDataNotFound)
    }

    pub fn read_local_account(&self) -> AppResult<LocalAccountInfo> {
        let db_path = self.base_path.join("state.vscdb");
        if !db_path.exists() {
            log::warn!("[read_local_account] local Cursor state database was not found");
            return Ok(LocalAccountInfo::default());
        }

        let conn = Connection::open(&db_path)?;
        let mut info = LocalAccountInfo::default();
        let mut stmt =
            conn.prepare("SELECT key, value FROM ItemTable WHERE key LIKE 'cursorAuth/%'")?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value = match row.get::<_, String>(1) {
                Ok(s) => s,
                Err(_) => {
                    let bytes: Vec<u8> = row.get(1)?;
                    String::from_utf8_lossy(&bytes).to_string()
                }
            };
            Ok((key, value))
        })?;

        for row in rows.flatten() {
            let (key, value) = row;
            let parsed_value: String = serde_json::from_str(&value).unwrap_or(value);

            match key.as_str() {
                "cursorAuth/accessToken" => {
                    info.access_token = Some(parsed_value.trim().trim_matches('"').to_string());
                }
                "cursorAuth/cachedEmail" => {
                    info.email = Some(parsed_value);
                }
                "cursorAuth/cachedSignUpType" => {
                    info.sign_up_type = Some(parsed_value);
                }
                _ => {}
            }
        }

        log::info!(
            "[read_local_account] local account read complete: has_email={}, has_session={}",
            info.email.is_some(),
            info.access_token.is_some()
        );

        Ok(info)
    }
}

impl Default for AccountReader {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            base_path: PathBuf::new(),
        })
    }
}
