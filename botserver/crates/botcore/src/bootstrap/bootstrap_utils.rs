// Bootstrap utility functions
//
// Split by responsibility (#1730): process control and shell helpers
// (`procs`), the cache protocol probe (`cache`) and the remaining component
// health probes (`health`). Every public item keeps its `bootstrap_utils::`
// path through the re-exports below.

mod cache;
mod health;
mod procs;

pub use cache::cache_health_check;
pub use health::{
    alm_ci_health_check, alm_health_check, drive_health_check, llm_health_check,
    tables_health_check, vault_health_check, vector_db_health_check, zitadel_health_check,
};
pub use procs::{
    component_process_patterns, dump_all_component_logs, get_processes_to_kill,
    kill_component_processes, safe_pgrep, safe_pkill, safe_sh_command,
};
#[cfg(windows)]
pub use procs::{kill_stack_component_windows, kill_stack_processes_windows};

/// Result type for bot existence check
#[derive(Debug)]
pub enum BotExistsResult {
    BotExists,
    BotNotFound,
}
