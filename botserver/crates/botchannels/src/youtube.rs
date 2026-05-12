//! YouTube Data API v3 Integration
//!
//! Provides video upload, community posts, and channel management capabilities.
//! Supports OAuth 2.0 authentication flow.
//!
//! This module re-exports from the youtube_api submodule for better organization.

// Re-export everything from the youtube_api module for backward compatibility
pub use youtube_api::*;

// The actual implementation is in the youtube_api submodule
mod youtube_api;
