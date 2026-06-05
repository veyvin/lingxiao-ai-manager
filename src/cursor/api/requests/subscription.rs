//! Subscription and quota API requests.

use crate::cursor::api::client::CursorApiClient;
use crate::cursor::api::responses::{SubscriptionStatusResponse, UsageSummaryResponse};
use crate::cursor::api::utils::parse_datetime;
use crate::cursor::types::{AccountType, QuotaInfo, SubscriptionInfo};
use crate::utils::{AppError, AppResult};

impl CursorApiClient {
    /// Reads subscription status with the local authenticated session.
    pub async fn get_subscription_status(&self) -> AppResult<SubscriptionInfo> {
        let url = "https://cursor.com/api/auth/stripe";

        let resp = self
            .client()
            .get(url)
            .headers(self.build_headers())
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(AppError::InvalidToken);
        }

        let body_text = resp.text().await?;
        let data: SubscriptionStatusResponse = serde_json::from_str(&body_text)?;

        let membership_type = data.membership_type.as_deref().unwrap_or_default();
        Ok(SubscriptionInfo {
            account_type: AccountType::from_api(membership_type),
            days_remaining: data.days_remaining_on_trial,
            billing_start: None,
            billing_end: None,
        })
    }

    /// Reads subscription status and user profile metadata.
    pub async fn get_subscription(&self) -> AppResult<(SubscriptionInfo, Option<String>)> {
        let (subscription_result, user_info_result) =
            tokio::join!(self.get_subscription_status(), self.get_user_info());

        let subscription = subscription_result?;
        let email = user_info_result.ok().and_then(|info| info.email);

        Ok((subscription, email))
    }

    /// Reads quota summary for the current billing cycle.
    pub async fn get_usage_summary(&self) -> AppResult<(SubscriptionInfo, QuotaInfo)> {
        let url = "https://cursor.com/api/usage-summary";

        let resp = self
            .client()
            .get(url)
            .headers(self.build_headers())
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(AppError::InvalidToken);
        }

        let body_text = resp.text().await?;
        let data: UsageSummaryResponse = serde_json::from_str(&body_text)
            .map_err(|e| AppError::JsonParseError(e.to_string()))?;

        let membership_type = data.membership_type.as_deref().unwrap_or_default();
        let subscription = SubscriptionInfo {
            account_type: AccountType::from_api(membership_type),
            days_remaining: None,
            billing_start: data.billing_cycle_start.and_then(|s| parse_datetime(&s)),
            billing_end: data.billing_cycle_end.and_then(|s| parse_datetime(&s)),
        };

        let plan = data
            .individual_usage
            .and_then(|u| u.plan)
            .unwrap_or_default();
        let quota = QuotaInfo {
            used: plan.used.unwrap_or(0),
            limit: plan.limit.unwrap_or(0),
            remaining: plan.remaining.unwrap_or(0),
        };

        Ok((subscription, quota))
    }
}
