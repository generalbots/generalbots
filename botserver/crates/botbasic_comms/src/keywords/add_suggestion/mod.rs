//! ADD_SUGGESTION / CLEAR_SUGGESTIONS keyword registration and read-back.

pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use botbasic_types::UserSession;
pub(crate) use log::{info, trace};
pub(crate) use rhai::{Dynamic, Engine};
pub(crate) use serde_json::json;
pub(crate) use std::sync::Arc;
pub(crate) use std::time::Duration;

/// Bound for each read and write on the blocking connection.
const REPLY_TIMEOUT: Duration = Duration::from_millis(500);

/// The reads run on the chat turn's async worker: a server that accepts the
/// connection and never answers must not hold that worker (and the turn).
fn get_redis_connection(cache_client: &Arc<redis::Client>) -> Option<redis::Connection> {
    let timeout = Duration::from_millis(50);
    let conn = cache_client.get_connection_with_timeout(timeout).ok()?;
    conn.set_read_timeout(Some(REPLY_TIMEOUT)).ok()?;
    conn.set_write_timeout(Some(REPLY_TIMEOUT)).ok()?;
    Some(conn)
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
