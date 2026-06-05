//! Lingxiao AI Manager desktop application.
//!
//! The open-source build focuses on local-first AI IDE usage visibility.
//! Supported platforms are Windows, macOS, and Linux where the underlying
//! Cursor data directory is available.

pub mod commands;
pub mod cursor;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            if let Err(e) = utils::init_logger(app.handle().clone()) {
                eprintln!("failed to initialize frontend logger: {}", e);
            } else {
                eprintln!("[FrontendLogger] logger initialized");
                log::info!(
                    "Lingxiao AI Manager started, version: {}",
                    env!("CARGO_PKG_VERSION")
                );
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::test_logging,
            utils::get_log_events,
            commands::get_local_accounts,
            commands::get_usage_events,
            commands::list_managed_accounts,
            commands::add_managed_account,
            commands::delete_managed_account,
            commands::switch_managed_account,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Tauri application");
}
