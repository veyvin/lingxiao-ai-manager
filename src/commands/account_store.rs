//! Local account-store commands.

use crate::cursor::{
    parse_cursor_session, AccountReader, CursorAccountStore, ManagedCursorAccount,
};
use crate::utils::{AppError, AppResult};
use rusqlite::{params, Connection, OpenFlags};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddManagedAccountRequest {
    pub label: Option<String>,
    pub session: String,
}

#[tauri::command]
pub async fn list_managed_accounts() -> Result<Vec<ManagedCursorAccount>, String> {
    run_blocking(|| {
        let store = CursorAccountStore::new()?;
        let current = AccountReader::new()?.read_local_account()?.access_token;
        store.list(current.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn add_managed_account(request: AddManagedAccountRequest) -> Result<String, String> {
    run_blocking(move || {
        let store = CursorAccountStore::new()?;
        let index = store.add(request.label, request.session)?;
        Ok(format!("Account {} was added locally", index + 1))
    })
    .await
}

#[tauri::command]
pub async fn delete_managed_account(index: usize) -> Result<String, String> {
    run_blocking(move || {
        CursorAccountStore::new()?.delete(index)?;
        Ok("Account was removed from the local store".to_string())
    })
    .await
}

#[tauri::command]
pub async fn switch_managed_account(index: usize) -> Result<String, String> {
    run_blocking(move || {
        let store = CursorAccountStore::new()?;
        let accounts = store.load()?;
        let account = accounts
            .get(index)
            .ok_or_else(|| AppError::Unknown("Account index was not found".to_string()))?;
        let parsed = parse_cursor_session(&account.session)?;
        write_cursor_auth(&account.session, &parsed.jwt, parsed.email.as_deref())?;
        Ok("Cursor account was switched locally. Restart Cursor if it is open.".to_string())
    })
    .await
}

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> AppResult<T> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|e| format!("Background task failed: {e}"))?
        .map_err(|e| e.to_string())
}

fn write_cursor_auth(session: &str, jwt: &str, email: Option<&str>) -> AppResult<()> {
    let db_path = AccountReader::cursor_state_db_path()?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    conn.busy_timeout(std::time::Duration::from_secs(2))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB)",
        [],
    )?;

    upsert(&conn, "cursorAuth/accessToken", jwt)?;
    upsert(&conn, "cursorAuth/refreshToken", jwt)?;
    upsert(&conn, "cursorAuth/cachedSignUpType", "Auth_0")?;
    if let Some(email) = email {
        upsert(&conn, "cursorAuth/cachedEmail", email)?;
    }
    if session.contains("::") {
        upsert(&conn, "cursorAuth/WorkosCursorSessionToken", session)?;
    }
    Ok(())
}

fn upsert(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}
