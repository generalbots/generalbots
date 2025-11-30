#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Desktop Module
//!
//! This module provides desktop-specific functionality including:
//! - Drive synchronization with cloud storage
//! - System tray management
//! - Local file operations
//! - Desktop tools (cleaner, optimizer, etc.)

pub mod drive;
pub mod sync;
pub mod tools;
pub mod tray;

// Re-exports
pub use drive::*;
pub use sync::*;
pub use tools::{
    CleanupCategory, CleanupStats, DesktopToolsConfig, DesktopToolsManager, DiskInfo,
    InstallationStatus, OptimizationStatus, OptimizationTask, TaskStatus,
};
pub use tray::{RunningMode, ServiceMonitor, TrayManager};
