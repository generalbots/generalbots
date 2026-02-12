// Document handlers split into logical modules

pub mod ai;
pub mod comments;
pub mod crud;
pub mod export;
pub mod import;
pub mod notes;
pub mod styles;
pub mod structure;
pub mod templates;
pub mod track_changes;
pub mod toc;

// Re-export all handlers for backward compatibility
pub use ai::*;
pub use comments::*;
pub use crud::*;
pub use export::*;
pub use import::*;
pub use notes::*;
pub use styles::*;
pub use structure::*;
pub use templates::*;
pub use track_changes::*;
pub use toc::*;
