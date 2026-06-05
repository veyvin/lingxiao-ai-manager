//! Lingxiao AI Manager desktop entry point.
//!
//! Focuses on local-first Cursor account status and usage visibility.

// Hide the console window on Windows; logs are exposed through the UI.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    rs_cursor_mc_lib::run();
}
