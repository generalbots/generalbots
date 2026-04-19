//! Authentication and authorization module
//!
//! This module has been split into the auth_api subdirectory for better organization.
//! All items are re-exported here for backward compatibility.

// Re-export everything from auth_api for backward compatibility
pub use crate::security::auth_api::*;
