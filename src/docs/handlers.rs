// Re-export all handlers from the handlers_api submodule
// This maintains backward compatibility while organizing code into logical modules
pub mod handlers_api;

// Re-export all handlers for backward compatibility
pub use handlers_api::*;
