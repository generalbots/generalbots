//! Credential extraction and token validation helpers.
//!
//! Split by responsibility: `extraction` (request fields and cookies),
//! `tokens` (API keys, JWTs and sessions) and `extracted` (the structured
//! credential view used by the provider chain).

mod extracted;
mod extraction;
mod tokens;

pub use extracted::ExtractedAuthData;
pub use extraction::{
    extract_bot_id_from_request, extract_session_from_cookies, extract_user_from_request,
};
pub use tokens::{basic_password_as_token, is_jwt_format, validate_session_sync};

pub(crate) use tokens::validate_bearer_token_sync;
