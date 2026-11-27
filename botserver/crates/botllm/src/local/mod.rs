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

/// When set, the legacy launcher leaves the chat engine (port 8081) alone:
/// another component (runtime control) owns its lifecycle.  Set once before
/// the first `ensure_llama_servers_running` call; read inside the launcher.
static CHAT_ENGINE_MANAGED: AtomicBool = AtomicBool::new(false);

/// Mark the chat engine as managed by another component (runtime control).
/// Must be called before the first `ensure_llama_servers_running` invocation.
pub fn set_chat_engine_managed_elsewhere(managed: bool) {
    CHAT_ENGINE_MANAGED.store(managed, Ordering::SeqCst);
}

/// Whether `set_chat_engine_managed_elsewhere(true)` was called.
pub fn chat_engine_managed_elsewhere() -> bool {
    CHAT_ENGINE_MANAGED.load(Ordering::SeqCst)
}

mod model_file_name;
mod ensure_llama_servers_running;
mod extract_base_url;
mod launcher_decision;
pub mod turn_order;

use model_file_name::*;
pub use ensure_llama_servers_running::*;
pub use extract_base_url::*;
pub use launcher_decision::{LauncherDecision, LauncherInput, decide};
