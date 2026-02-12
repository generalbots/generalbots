// Canvas module - split into canvas_api subdirectory for better organization
//
// This module has been reorganized into the following submodules:
// - canvas_api/types: All data structures and enums
// - canvas_api/error: Error types and implementations
// - canvas_api/db: Database row types and migrations
// - canvas_api/service: CanvasService business logic
// - canvas_api/handlers: HTTP route handlers
//
// This file re-exports all public items for backward compatibility.

pub mod canvas_api;

// Re-export all public types for backward compatibility
pub use canvas_api::*;

// Re-export the migration function at the module level
pub use canvas_api::create_canvas_tables_migration;

// Re-export canvas routes at the module level
pub use canvas_api::canvas_routes;
