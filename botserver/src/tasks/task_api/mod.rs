//! Task API module - contains task management logic
//!
//! This module is split into:
//! - engine: Core TaskEngine with CRUD operations
//! - handlers: HTTP request handlers
//! - html_renderers: HTML building functions for UI
//! - utils: Utility functions

pub mod engine;
pub mod handlers;
pub mod html_renderers;
pub mod utils;

// Re-export commonly used types
pub use engine::TaskEngine;
pub use handlers::{configure_task_routes, handle_task_create, handle_task_delete, handle_task_get, handle_task_list, handle_task_update};
