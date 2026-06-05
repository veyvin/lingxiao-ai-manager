//! Frontend log bridge with conservative redaction.

use chrono::Timelike;
use log::{Level, Log, Metadata, Record};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

static LOG_QUEUE: OnceLock<Arc<Mutex<Vec<LogEvent>>>> = OnceLock::new();

pub struct FrontendLogger;

impl FrontendLogger {
    fn level_to_string(level: Level) -> String {
        match level {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
            Level::Trace => "trace",
        }
        .to_string()
    }

    fn format_log(record: &Record) -> String {
        let mut message = String::new();
        if let Some(module_path) = record.module_path() {
            if !module_path.is_empty() && module_path != "rs_cursor_mc_lib" {
                message.push_str(&format!("[{}] ", module_path));
            }
        }
        message.push_str(&record.args().to_string());
        message
    }

    fn contains_sensitive_marker(message: &str) -> bool {
        let lower = message.to_ascii_lowercase();
        [
            "access_token",
            "refreshtoken",
            "refresh_token",
            "authorization",
            "bearer ",
            "cookie",
            "password",
            "sessiontoken",
            "workoscursorsessiontoken",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
    }

    fn sanitize_message(message: String) -> String {
        if Self::contains_sensitive_marker(&message) {
            "[redacted sensitive log message]".to_string()
        } else {
            message
        }
    }

    fn should_drop(message: &str) -> bool {
        message.contains("[parse_usage_event]")
            || message.contains("[test_logging]")
            || message.contains("[reqwest::retry]")
            || message.contains("[reqwest::connect]")
            || message.contains("starting new connection")
            || message.contains("shouldn't retry")
    }

    fn send_log_event(log_event: &LogEvent) {
        if let Some(queue) = LOG_QUEUE.get() {
            let mut queue = queue.lock().unwrap();
            queue.push(log_event.clone());
            if queue.len() > 1000 {
                queue.remove(0);
            }
        }
    }
}

impl Log for FrontendLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let raw_message = Self::format_log(record);
        if Self::should_drop(&raw_message) {
            return;
        }

        let message = Self::sanitize_message(raw_message);
        let level = Self::level_to_string(record.level());
        let offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        let now = chrono::Utc::now().with_timezone(&offset);
        let timestamp = format!(
            "{:02}:{:02}:{:02}.{:03}",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        );

        let log_event = LogEvent {
            level: level.clone(),
            message: message.clone(),
            timestamp,
        };

        Self::send_log_event(&log_event);
        eprintln!("[{}] {}", level.to_uppercase(), message);
    }

    fn flush(&self) {}
}

pub fn init_logger(_app_handle: tauri::AppHandle) -> Result<(), log::SetLoggerError> {
    LOG_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    log::set_max_level(log::LevelFilter::Trace);

    static LOGGER_INSTANCE: FrontendLogger = FrontendLogger;
    log::set_logger(&LOGGER_INSTANCE)
}

#[tauri::command]
pub fn get_log_events() -> Vec<LogEvent> {
    if let Some(queue) = LOG_QUEUE.get() {
        let mut queue = queue.lock().unwrap();
        let logs = queue.clone();
        queue.clear();
        logs
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::FrontendLogger;

    #[test]
    fn sanitize_message_redacts_sensitive_markers() {
        let message = "access_token=secret-value should not be logged".to_string();
        let sanitized = FrontendLogger::sanitize_message(message);
        assert_eq!(sanitized, "[redacted sensitive log message]");
    }

    #[test]
    fn sanitize_message_keeps_operational_status() {
        let message = "usage refresh complete".to_string();
        let sanitized = FrontendLogger::sanitize_message(message);
        assert_eq!(sanitized, "usage refresh complete");
    }

    #[test]
    fn should_drop_noisy_internal_messages() {
        assert!(FrontendLogger::should_drop(
            "[test_logging] log bridge test"
        ));
        assert!(FrontendLogger::should_drop(
            "[reqwest::connect] starting new connection"
        ));
        assert!(!FrontendLogger::should_drop("account read complete"));
    }
}
