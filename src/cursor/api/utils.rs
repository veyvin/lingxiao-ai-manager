//! Helpers for parsing Cursor API payloads.

use crate::cursor::api::responses::*;
use crate::cursor::types::*;
use chrono::{DateTime, Utc};

pub(crate) fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| s.replace('Z', "+00:00").parse::<DateTime<Utc>>().ok())
}

pub(crate) fn parse_usage_event(raw: RawUsageEvent) -> Option<UsageEvent> {
    let timestamp_ms: i64 = raw.timestamp?.parse().ok()?;
    let timestamp = DateTime::from_timestamp_millis(timestamp_ms)?;
    let usage = raw.token_usage.unwrap_or_default();

    Some(UsageEvent {
        timestamp,
        model: raw
            .model
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        kind: raw.kind.unwrap_or_default(),
        input_tokens: usage.input_tokens.unwrap_or(0),
        output_tokens: usage.output_tokens.unwrap_or(0),
        cache_write_tokens: usage.cache_write_tokens.unwrap_or(0),
        cache_read_tokens: usage.cache_read_tokens.unwrap_or(0),
        total_cents: usage.total_cents.unwrap_or(0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_datetime_accepts_rfc3339_values() {
        let parsed = parse_datetime("2026-06-05T12:00:00Z").unwrap();
        assert_eq!(parsed.timestamp(), 1780660800);
    }

    #[test]
    fn parse_usage_event_defaults_missing_usage_values() {
        let raw = RawUsageEvent {
            timestamp: Some("1780660800000".to_string()),
            model: Some("".to_string()),
            kind: None,
            token_usage: None,
        };

        let event = parse_usage_event(raw).unwrap();
        assert_eq!(event.model, "unknown");
        assert_eq!(event.kind, "");
        assert_eq!(event.input_tokens, 0);
        assert_eq!(event.total_cents, 0.0);
    }

    #[test]
    fn parse_usage_event_rejects_invalid_timestamp() {
        let raw = RawUsageEvent {
            timestamp: Some("not-a-timestamp".to_string()),
            model: Some("model".to_string()),
            kind: Some("usage".to_string()),
            token_usage: None,
        };

        assert!(parse_usage_event(raw).is_none());
    }
}
