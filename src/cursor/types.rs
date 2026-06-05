//! Shared data types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AccountType {
    #[default]
    Free,
    FreeTrial,
    Pro,
    ProPlus,
    Unknown(String),
}

impl AccountType {
    pub fn from_api(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "free" => Self::Free,
            "free_trial" => Self::FreeTrial,
            "pro" => Self::Pro,
            "pro+" | "pro_plus" => Self::ProPlus,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn display_name(&self) -> &str {
        self.to_display_str()
    }

    pub fn to_display_str(&self) -> &str {
        match self {
            Self::Free => "Free",
            Self::FreeTrial => "Pro Trial",
            Self::Pro => "Pro",
            Self::ProPlus => "Pro+",
            Self::Unknown(s) => s.as_str(),
        }
    }

    pub fn color_css(&self) -> &str {
        match self {
            Self::Free => "#5f6b7a",
            Self::FreeTrial => "#2563eb",
            Self::Pro => "#059669",
            Self::ProPlus => "#7c3aed",
            Self::Unknown(_) => "#5f6b7a",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalAccountInfo {
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub sign_up_type: Option<String>,
}

impl LocalAccountInfo {
    pub fn is_logged_in(&self) -> bool {
        self.email.is_some() && self.access_token.is_some()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub account_type: AccountType,
    pub days_remaining: Option<i32>,
    pub billing_start: Option<DateTime<Utc>>,
    pub billing_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenStats {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_read_tokens: i64,
}

impl TokenStats {
    pub fn total(&self) -> i64 {
        self.input_tokens + self.output_tokens + self.cache_write_tokens + self.cache_read_tokens
    }

    pub fn format_wan(tokens: i64) -> String {
        if tokens == 0 {
            return "0".to_string();
        }
        let wan = tokens as f64 / 10000.0;
        if wan >= 1.0 {
            format!("{wan:.2}w")
        } else if wan >= 0.01 {
            format!("{wan:.4}w")
        } else {
            tokens.to_string()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub used: i32,
    pub limit: i32,
    pub remaining: i32,
}

impl QuotaInfo {
    pub fn percent_used(&self) -> f32 {
        if self.limit == 0 {
            return 0.0;
        }
        (self.used as f32 / self.limit as f32) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub kind: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_cents: f64,
}

impl UsageEvent {
    pub fn total_usd(&self) -> f64 {
        self.total_cents / 100.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsageSummary {
    pub model: String,
    pub event_count: i32,
    pub token_stats: TokenStats,
    pub total_cents: f64,
}

impl ModelUsageSummary {
    pub fn total_usd(&self) -> f64 {
        self.total_cents / 100.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_events: i32,
    pub token_stats: TokenStats,
    pub total_cents: f64,
    pub events: Vec<UsageEvent>,
    pub query_start: Option<DateTime<Utc>>,
    pub query_end: Option<DateTime<Utc>>,
}

impl UsageStats {
    pub fn group_by_model(&self) -> Vec<ModelUsageSummary> {
        use std::collections::HashMap;

        let mut model_map: HashMap<String, ModelUsageSummary> = HashMap::new();

        for event in &self.events {
            let entry = model_map
                .entry(event.model.clone())
                .or_insert_with(|| ModelUsageSummary {
                    model: event.model.clone(),
                    event_count: 0,
                    token_stats: TokenStats::default(),
                    total_cents: 0.0,
                });

            entry.event_count += 1;
            entry.token_stats.input_tokens += event.input_tokens;
            entry.token_stats.output_tokens += event.output_tokens;
            entry.token_stats.cache_write_tokens += event.cache_write_tokens;
            entry.token_stats.cache_read_tokens += event.cache_read_tokens;
            entry.total_cents += event.total_cents;
        }

        let mut summaries: Vec<_> = model_map.into_values().collect();
        summaries.sort_by(|a, b| {
            b.total_cents
                .partial_cmp(&a.total_cents)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        summaries
    }
}

#[derive(Debug, Clone, Default)]
pub struct FullAccountInfo {
    pub local: LocalAccountInfo,
    pub subscription: Option<SubscriptionInfo>,
    pub quota: Option<QuotaInfo>,
    pub usage: Option<UsageStats>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn account_type_maps_known_values() {
        assert_eq!(AccountType::from_api("free"), AccountType::Free);
        assert_eq!(AccountType::from_api("free_trial"), AccountType::FreeTrial);
        assert_eq!(AccountType::from_api("pro_plus"), AccountType::ProPlus);
    }

    #[test]
    fn token_stats_total_sums_all_buckets() {
        let stats = TokenStats {
            input_tokens: 1,
            output_tokens: 2,
            cache_write_tokens: 3,
            cache_read_tokens: 4,
        };
        assert_eq!(stats.total(), 10);
    }

    #[test]
    fn usage_stats_groups_by_model_and_sorts_by_cost() {
        let timestamp = Utc.timestamp_millis_opt(1780660800000).unwrap();
        let stats = UsageStats {
            events: vec![
                UsageEvent {
                    timestamp,
                    model: "b".to_string(),
                    kind: "usage".to_string(),
                    input_tokens: 1,
                    output_tokens: 0,
                    cache_write_tokens: 0,
                    cache_read_tokens: 0,
                    total_cents: 1.0,
                },
                UsageEvent {
                    timestamp,
                    model: "a".to_string(),
                    kind: "usage".to_string(),
                    input_tokens: 2,
                    output_tokens: 0,
                    cache_write_tokens: 0,
                    cache_read_tokens: 0,
                    total_cents: 5.0,
                },
            ],
            ..UsageStats::default()
        };

        let grouped = stats.group_by_model();
        assert_eq!(grouped[0].model, "a");
        assert_eq!(grouped[0].token_stats.input_tokens, 2);
        assert_eq!(grouped[1].model, "b");
    }
}
