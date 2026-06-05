//! Usage event API requests.

use crate::cursor::api::client::CursorApiClient;
use crate::cursor::api::responses::UsageEventsResponse;
use crate::cursor::api::utils::parse_usage_event;
use crate::cursor::types::{TokenStats, UsageEvent, UsageStats};
use crate::utils::{AppError, AppResult};
use chrono::{Duration, Utc};
use futures::future;

impl CursorApiClient {
    pub async fn get_usage_events(&self, days: i64) -> AppResult<UsageStats> {
        let now = Utc::now();
        let start = now - Duration::days(days);
        let end_ms = now.timestamp_millis();
        let start_ms = start.timestamp_millis();
        let page_size = 100;
        let url = "https://cursor.com/api/dashboard/get-filtered-usage-events";

        let first_page_payload = serde_json::json!({
            "teamId": 0,
            "startDate": start_ms.to_string(),
            "endDate": end_ms.to_string(),
            "page": 1,
            "pageSize": page_size
        });

        let resp = self
            .client()
            .post(url)
            .headers(self.build_headers())
            .json(&first_page_payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::NetworkError(format!(
                "HTTP request failed with status {}",
                resp.status()
            )));
        }

        let body = resp.text().await?;
        let first_page_data: UsageEventsResponse =
            serde_json::from_str(&body).map_err(|e| AppError::JsonParseError(e.to_string()))?;
        let total = first_page_data.total_usage_events_count.unwrap_or(0);
        let mut all_events = parse_events(first_page_data);

        if total <= page_size {
            return build_usage_stats(all_events, start, now);
        }

        let total_pages = (total + page_size - 1) / page_size;
        let url = url.to_string();
        let start_ms = start_ms.to_string();
        let end_ms = end_ms.to_string();
        let client = self.client().clone();
        let headers = self.build_headers();

        let mut page_futures = Vec::new();
        for page in 2..=total_pages {
            let url = url.clone();
            let start_ms = start_ms.clone();
            let end_ms = end_ms.clone();
            let client = client.clone();
            let headers = headers.clone();

            page_futures.push(async move {
                let payload = serde_json::json!({
                    "teamId": 0,
                    "startDate": start_ms,
                    "endDate": end_ms,
                    "page": page,
                    "pageSize": page_size
                });

                let resp = client
                    .post(&url)
                    .headers(headers)
                    .json(&payload)
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    return Ok::<Vec<_>, AppError>(Vec::new());
                }

                let body = resp.text().await?;
                let data: UsageEventsResponse = serde_json::from_str(&body)?;
                Ok(parse_events(data))
            });
        }

        for result in future::join_all(page_futures).await {
            if let Ok(mut events) = result {
                all_events.append(&mut events);
            }
        }

        build_usage_stats(all_events, start, now)
    }
}

fn parse_events(data: UsageEventsResponse) -> Vec<UsageEvent> {
    data.usage_events_display
        .unwrap_or_default()
        .into_iter()
        .filter_map(parse_usage_event)
        .collect()
}

fn build_usage_stats(
    mut all_events: Vec<UsageEvent>,
    start: chrono::DateTime<chrono::Utc>,
    now: chrono::DateTime<chrono::Utc>,
) -> AppResult<UsageStats> {
    let mut token_stats = TokenStats::default();
    let mut total_cents = 0.0;

    for event in &all_events {
        token_stats.input_tokens += event.input_tokens;
        token_stats.output_tokens += event.output_tokens;
        token_stats.cache_write_tokens += event.cache_write_tokens;
        token_stats.cache_read_tokens += event.cache_read_tokens;
        total_cents += event.total_cents;
    }

    all_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(UsageStats {
        total_events: all_events.len() as i32,
        token_stats,
        total_cents,
        events: all_events,
        query_start: Some(start),
        query_end: Some(now),
    })
}
