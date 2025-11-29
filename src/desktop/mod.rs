#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod drive;
pub mod sync;
pub mod tray;

pub use tray::{RunningMode, ServiceMonitor, TrayManager};
