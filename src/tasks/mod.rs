// Task API module - split into logical submodules
pub mod task_api;

// Re-export for backward compatibility
pub use task_api::{TaskEngine, configure_task_routes, handle_task_create, handle_task_delete, handle_task_get, handle_task_list, handle_task_update};

// Existing modules
pub mod scheduler;
pub mod types;

// Re-export scheduler
pub use scheduler::TaskScheduler;

// Import types from types module
use crate::tasks::types::*;

