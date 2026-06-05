//! Local Cursor account discovery.

mod reader;

use crate::utils::AppResult;
pub use reader::AccountReader;

#[derive(Debug, Clone)]
pub struct CursorAccount {
    pub email: Option<String>,
    pub access_token: Option<String>,
}

pub async fn get_all_accounts() -> AppResult<Vec<CursorAccount>> {
    log::info!("[get_all_accounts] reading local Cursor account");

    let local_account = tokio::task::spawn_blocking(|| {
        let reader = AccountReader::new()?;
        reader.read_local_account()
    })
    .await
    .map_err(|e| crate::utils::AppError::Unknown(format!("account reader task failed: {e}")))??;

    if local_account.is_logged_in() {
        Ok(vec![CursorAccount {
            email: local_account.email,
            access_token: local_account.access_token,
        }])
    } else {
        log::warn!("[get_all_accounts] no logged-in Cursor account found");
        Ok(vec![])
    }
}
