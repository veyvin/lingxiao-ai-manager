//! Response payloads for Cursor API calls.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionStatusResponse {
    pub(crate) membership_type: Option<String>,
    pub(crate) days_remaining_on_trial: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UsageSummaryResponse {
    pub(crate) membership_type: Option<String>,
    pub(crate) billing_cycle_start: Option<String>,
    pub(crate) billing_cycle_end: Option<String>,
    pub(crate) individual_usage: Option<IndividualUsage>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IndividualUsage {
    pub(crate) plan: Option<PlanUsage>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PlanUsage {
    pub(crate) used: Option<i32>,
    pub(crate) limit: Option<i32>,
    pub(crate) remaining: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UsageEventsResponse {
    pub(crate) total_usage_events_count: Option<i32>,
    pub(crate) usage_events_display: Option<Vec<RawUsageEvent>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawUsageEvent {
    pub(crate) timestamp: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) token_usage: Option<RawTokenUsage>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawTokenUsage {
    pub(crate) input_tokens: Option<i64>,
    pub(crate) output_tokens: Option<i64>,
    pub(crate) cache_write_tokens: Option<i64>,
    pub(crate) cache_read_tokens: Option<i64>,
    pub(crate) total_cents: Option<f64>,
}
