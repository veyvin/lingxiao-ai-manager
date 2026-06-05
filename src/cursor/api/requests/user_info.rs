//! User profile API request.

use crate::cursor::api::client::CursorApiClient;
use crate::cursor::api::responses::UserInfoResponse;
use crate::utils::{AppError, AppResult};

impl CursorApiClient {
    pub async fn get_user_info(&self) -> AppResult<UserInfoResponse> {
        let url = "https://cursor.com/api/dashboard/get-me";

        let resp = self
            .client()
            .post(url)
            .headers(self.build_headers())
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(AppError::InvalidToken);
        }

        let body_text = resp.text().await?;
        serde_json::from_str(&body_text).map_err(|e| AppError::JsonParseError(e.to_string()))
    }
}
