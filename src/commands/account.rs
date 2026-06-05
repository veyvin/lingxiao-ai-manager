//! Account and usage commands exposed to the desktop UI.

use crate::cursor::{self, AccountType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub email: Option<String>,
    pub usage: Option<UsageInfo>,
    pub account_type: Option<String>,
    pub days_remaining: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageInfo {
    pub used: i32,
    pub limit: i32,
    pub remaining: i32,
}

#[tauri::command]
pub fn test_logging() -> String {
    log::info!("[test_logging] log bridge test");
    "Log bridge is working".to_string()
}

#[tauri::command]
pub async fn get_local_accounts() -> Result<Vec<AccountInfo>, String> {
    log::info!("[get_local_accounts] reading local account status");
    let accounts = cursor::get_all_accounts().await.map_err(|e| {
        log::error!("[get_local_accounts] account read failed: {}", e);
        format!("Failed to read local account: {}", e)
    })?;

    let mut result = Vec::new();

    for account in accounts {
        let (usage, account_type, days_remaining) = if let Some(ref token) = account.access_token {
            match cursor::CursorApiClient::new(token) {
                Ok(client) => {
                    let (usage_result, subscription_result) =
                        tokio::join!(client.get_usage_summary(), client.get_subscription_status());

                    match usage_result {
                        Ok((mut sub, quota)) => {
                            if let Ok(subscription) = subscription_result {
                                if subscription.days_remaining.is_some() {
                                    sub.days_remaining = subscription.days_remaining;
                                }
                                if matches!(sub.account_type, AccountType::Unknown(_)) {
                                    sub.account_type = subscription.account_type;
                                }
                            }

                            let usage = Some(UsageInfo {
                                used: quota.used,
                                limit: quota.limit,
                                remaining: quota.remaining,
                            });
                            let account_type = Some(sub.account_type.to_display_str().to_string());
                            let days_remaining = sub.days_remaining;
                            (usage, account_type, days_remaining)
                        }
                        Err(_) => {
                            if let Ok(subscription) = subscription_result {
                                let account_type =
                                    Some(subscription.account_type.to_display_str().to_string());
                                let days_remaining = subscription.days_remaining;
                                (None, account_type, days_remaining)
                            } else {
                                (None, None, None)
                            }
                        }
                    }
                }
                Err(_) => (None, None, None),
            }
        } else {
            (None, None, None)
        };

        result.push(AccountInfo {
            email: account.email.clone(),
            usage,
            account_type,
            days_remaining,
        });
    }

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageEventStats {
    pub total_cost: f64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_read_tokens: i64,
    pub models: Vec<ModelStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<UsageEventDetail>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEventDetail {
    pub timestamp: String,
    pub model: String,
    pub kind: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_cents: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub name: String,
    pub request_count: i32,
    pub total_tokens: i64,
    pub cost: f64,
}

#[tauri::command]
pub async fn get_usage_events() -> Result<UsageEventStats, String> {
    let accounts = cursor::get_all_accounts().await.map_err(|e| {
        log::error!("[get_usage_events] account read failed: {}", e);
        format!("Failed to read local account: {}", e)
    })?;

    let access_token = accounts
        .first()
        .and_then(|account| account.access_token.as_deref())
        .ok_or_else(|| "No logged-in Cursor account was found".to_string())?;

    let client = cursor::CursorApiClient::new(access_token).map_err(|e| {
        log::error!("[get_usage_events] API client creation failed: {}", e);
        format!("Failed to create API client: {}", e)
    })?;

    let events = client.get_usage_events(30).await.map_err(|e| {
        log::error!("[get_usage_events] usage fetch failed: {}", e);
        format!("Failed to fetch usage events: {}", e)
    })?;

    log::info!(
        "[get_usage_events] fetched {} usage events",
        events.events.len()
    );

    let mut total_cost = 0.0;
    let mut total_tokens = 0i64;
    let mut input_tokens = 0i64;
    let mut output_tokens = 0i64;
    let mut cache_write_tokens = 0i64;
    let mut cache_read_tokens = 0i64;
    let mut model_map: std::collections::HashMap<String, (i32, i64, f64)> =
        std::collections::HashMap::new();

    for event in &events.events {
        total_cost += event.total_cents;
        let event_total_tokens = event.input_tokens
            + event.output_tokens
            + event.cache_write_tokens
            + event.cache_read_tokens;
        total_tokens += event_total_tokens;
        input_tokens += event.input_tokens;
        output_tokens += event.output_tokens;
        cache_write_tokens += event.cache_write_tokens;
        cache_read_tokens += event.cache_read_tokens;

        let entry = model_map.entry(event.model.clone()).or_insert((0, 0, 0.0));
        entry.0 += 1;
        entry.1 += event_total_tokens;
        entry.2 += event.total_cents;
    }

    let mut models: Vec<ModelStats> = model_map
        .into_iter()
        .map(|(name, (count, tokens, cost))| ModelStats {
            name,
            request_count: count,
            total_tokens: tokens,
            cost: cost / 100.0,
        })
        .collect();

    models.sort_by(|a, b| {
        b.cost
            .partial_cmp(&a.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let event_details: Vec<UsageEventDetail> = events
        .events
        .into_iter()
        .map(|e| UsageEventDetail {
            timestamp: e.timestamp.to_rfc3339(),
            model: e.model,
            kind: e.kind,
            input_tokens: e.input_tokens,
            output_tokens: e.output_tokens,
            cache_write_tokens: e.cache_write_tokens,
            cache_read_tokens: e.cache_read_tokens,
            total_cents: e.total_cents,
        })
        .collect();

    Ok(UsageEventStats {
        total_cost: total_cost / 100.0,
        total_tokens,
        input_tokens,
        output_tokens,
        cache_write_tokens,
        cache_read_tokens,
        models,
        events: Some(event_details),
    })
}
