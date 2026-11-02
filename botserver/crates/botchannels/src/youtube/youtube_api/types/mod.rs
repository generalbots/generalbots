//! YouTube Data API Request/Response Types
//!
//! Contains all public types for making requests and handling responses
//! from the YouTube Data API v3.


// ============================================================================
// Request Types
// ============================================================================

pub(crate) use serde::{Deserialize, Serialize};

mod video_upload_request;
mod comment_thread;

pub use video_upload_request::*;
pub use comment_thread::*;
