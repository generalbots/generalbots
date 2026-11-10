//! ADD_SUGGESTION / CLEAR_SUGGESTIONS keyword registration and read-back.

pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use botbasic_types::UserSession;
pub(crate) use log::{info, trace};
pub(crate) use rhai::{Dynamic, Engine};
pub(crate) use serde_json::json;
pub(crate) use std::sync::Arc;
pub(crate) use std::time::Duration;

fn get_redis_connection(cache_client: &Arc<redis::Client>) -> Option<redis::Connection> {
    let timeout = Duration::from_millis(50);
    cache_client.get_connection_with_timeout(timeout).ok()
}

#[derive(Debug, Clone)]
pub enum SuggestionType {
    Context(String),

    Tool {
        name: String,
        params: Option<Vec<String>>,
    },
}

mod labels;
mod read;
mod registration;
mod suggestions;
#[cfg(test)]
mod tests;

pub use labels::*;
pub use read::*;
pub use registration::*;
use suggestions::*;
