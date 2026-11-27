//! Stop leftover processes of services that the product file switches off.
//!
//! When a new version starts while Redis, Qdrant, Zitadel or the embedding
//! llama-server of an older version are still running from this stack, nothing
//! stopped them until now. This module kills those processes at the start of
//! the bootstrap so they release their ports and memory.

use log::{info, warn};

#[cfg(not(windows))]
use super::procs::safe_pkill;
#[cfg(windows)]
use super::procs::kill_stack_component_windows;
#[cfg(not(windows))]
use crate::shared::utils::get_stack_path;

/// Service directories under `<stack>/bin/` whose processes must be stopped
/// when the product file switches them off.
const SWITCHABLE_SERVICE_DIRS: &[&str] = &["cache", "vector_db", "directory"];

/// Stop processes of services that the product file switches off and that
/// are still running from this stack's `bin/` folders. Also stops a
/// llama-server of this stack that serves embeddings when the vector store
/// is off (command line has `--embeddings` or `--port 8082`), but never the
/// chat engine on 8081 (runtime control owns it).
///
/// Returns the total number of processes stopped.
pub fn stop_switched_off_leftovers(is_service_off: impl Fn(&str) -> bool) -> usize {
    let mut total = 0usize;

    for dir in SWITCHABLE_SERVICE_DIRS {
        if !is_service_off(dir) {
            continue;
        }
        let killed = stop_service_dir_processes(dir);
        total += killed;
    }

    if is_service_off("vector_db") {
        total += stop_stack_embedding_engine();
    }

    total
}

/// Kill processes running from `<stack>/bin/<dir>/`.
#[cfg(windows)]
fn stop_service_dir_processes(dir: &str) -> usize {
    kill_stack_component_windows(dir)
}

#[cfg(not(windows))]
fn stop_service_dir_processes(dir: &str) -> usize {
    let pattern = format!("{}/bin/{}", get_stack_path(), dir);
    safe_pkill(&[pattern.as_str()], &["-9", "-f"]);
    0
}

/// Kill llama-server processes of this stack that serve embeddings (port
/// 8082 or `--embeddings`). The chat engine on 8081 is never touched.
#[cfg(windows)]
fn stop_stack_embedding_engine() -> usize {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    let stack_bin = super::procs::resolve_stack_bin_dir();
    let llm_fragment = format!("{}llm{}", std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);
    let llm_fragment = llm_fragment.to_ascii_lowercase();

    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    );

    let mut killed = 0usize;
    for (pid, proc) in sys.processes() {
        let exe = match proc.exe() {
            Some(p) => p.to_string_lossy().to_ascii_lowercase().replace('/', "\\"),
            None => continue,
        };
        if !exe.starts_with(&stack_bin) || !exe.contains(&llm_fragment) {
            continue;
        }
        let cmd: String = proc
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if !is_embedding_engine(&cmd) {
            continue;
        }
        let name = proc.name().to_string_lossy();
        if proc.kill() {
            info!("Stopped leftover embedding engine {name} (PID {pid})");
            killed += 1;
        } else {
            warn!("Could not stop leftover embedding engine {name} (PID {pid})");
        }
    }
    killed
}

#[cfg(not(windows))]
fn stop_stack_embedding_engine() -> usize {
    0
}

/// Whether `cmd_line` indicates an embedding engine (not the chat engine).
fn is_embedding_engine(cmd_line: &str) -> bool {
    let args: Vec<&str> = cmd_line.split_whitespace().collect();
    if args.contains(&"--embeddings") {
        return true;
    }
    args.windows(2)
        .any(|pair| pair[0] == "--port" && pair[1] == "8082")
        || args.iter().any(|arg| *arg == "--port=8082")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_engine_detection() {
        assert!(is_embedding_engine(
            "llama-server --port 8082 --host 127.0.0.1 --embeddings"
        ));
        assert!(is_embedding_engine(
            "llama-server --port=8082 --host 127.0.0.1"
        ));
        assert!(is_embedding_engine(
            "llama-server --host 127.0.0.1 --embeddings --port 8082"
        ));
        assert!(!is_embedding_engine(
            "llama-server --port 8081 --host 127.0.0.1"
        ));
        assert!(!is_embedding_engine(
            "llama-server --port 80820 --host 127.0.0.1"
        ));
        assert!(!is_embedding_engine(""));
    }

    #[test]
    fn nothing_stopped_when_no_services_are_off() {
        let count = stop_switched_off_leftovers(|_| false);
        assert_eq!(count, 0);
    }

    #[test]
    fn service_dirs_are_all_switchable() {
        for dir in SWITCHABLE_SERVICE_DIRS {
            assert!(
                matches!(*dir, "cache" | "vector_db" | "directory"),
                "unexpected service dir: {dir}"
            );
        }
    }
}
