//! REST API Module
//!
//! Provides HTTP endpoints for cloud-based functionality.
//! Supports web, desktop, and mobile clients.
//!
//! Note: Local operations require native access and are handled separately:
//! - Screen capture: Tauri commands (desktop) or WebRTC (web/mobile)
//! - File sync: Tauri commands with local rclone process (desktop only)

pub mod drive;
pub mod keyword_services;
pub mod queue;
