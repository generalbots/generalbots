pub(crate) use botcore::config::ConfigManager;
pub(crate) use botcore::kb::embedding_generator::set_embedding_server_ready;
pub(crate) use botcore::shared::memory_monitor::{log_jemalloc_stats, MemoryStats};
pub(crate) use botsecurity::command_guard::SafeCommand;
pub(crate) use botcore::shared::models::schema::bots::dsl::*;
pub(crate) use botcore::shared::state::AppState;
pub(crate) use diesel::prelude::*;
pub(crate) use log::{info, trace, warn};
pub(crate) use reqwest;
pub(crate) use std::sync::atomic::{AtomicBool, Ordering};
pub(crate) use std::sync::Arc;
pub(crate) use tokio;

static LLAMA_SERVERS_STARTED: AtomicBool = AtomicBool::new(false);

mod model_file_name;
mod ensure_llama_servers_running;
mod extract_base_url;

use model_file_name::*;
pub use ensure_llama_servers_running::*;
pub use extract_base_url::*;
