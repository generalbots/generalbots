// Canvas API modules
pub mod types;
pub mod error;
pub mod db;
pub mod service;
pub mod handlers;

// Re-export public types for backward compatibility
pub use types::*;
pub use error::CanvasError;
pub use db::{CanvasRow, TemplateRow, row_to_canvas, create_canvas_tables_migration};
pub use service::CanvasService;
pub use handlers::canvas_routes;
