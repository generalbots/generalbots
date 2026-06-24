//! YouTube Data API v3 Integration
//!
//! This module provides a complete interface to the YouTube Data API v3,
//! organized into submodules for better maintainability.

mod client;
mod models;
mod provider;
mod types;

// Re-export all public types and the provider
pub use provider::YouTubeProvider;
pub use types::*;
