//! Process spawn for the stack's `llama-server` (chat and embedding).

use botsecurity::command_guard::SafeCommand;

/// Embedding port bound by the stack's second llama-server.
pub const EMBEDDING_PORT: u16 = 8082;

/// Spawn the embedding runtime the way the app start does. Used after an
/// engine update, which has to stop it to replace the engine files.
pub fn start_embedding_runtime() -> Result<u32, String> {
    let stack = botcore::shared::utils::get_stack_path();
    let model = format!("{stack}/data/llm/bge-small-en-v1.5-f32.gguf");
    if !std::path::Path::new(&model).exists() {
        return Err(format!("Embedding model not found at {model}"));
    }
    let args: Vec<String> = [
        "--port", &EMBEDDING_PORT.to_string(), "--host", "127.0.0.1", "-m", &model,
        "--ubatch-size", "512", "--embeddings", "--pooling", "mean",
        "--n-gpu-layers", "0", "--ctx-size", "512",
    ]
    .iter()
    .map(|arg| (*arg).to_string())
    .collect();
    log::info!("[runtime_ctl] starting embedding runtime");
    spawn_server(&format!("{stack}/bin/llm"), &args, "embedding.log")
}

/// Spawn `llama-server` from `server_dir`, its output appended to `log_name`
/// in the stack's engine log folder. Returns the child PID.
pub(super) fn spawn_server(server_dir: &str, args: &[String], log_name: &str) -> Result<u32, String> {
    let binary = if cfg!(windows) {
        format!("{server_dir}\\llama-server.exe")
    } else {
        format!("{server_dir}/llama-server")
    };
    if !std::path::Path::new(&binary).exists() {
        return Err(format!("llama-server binary not found at {binary}"));
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut command = SafeCommand::new(&binary).map_err(|e| e.to_string())?;
    command = command.args(&arg_refs).map_err(|e| e.to_string())?;
    command = command
        .working_dir(std::path::Path::new(server_dir))
        .map_err(|e| e.to_string())?;
    if !cfg!(windows) {
        command = command
            .env("LD_LIBRARY_PATH", server_dir)
            .map_err(|e| e.to_string())?;
    }
    let stack = botcore::shared::utils::get_stack_path();
    let log_dir = format!("{stack}/logs/llm");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        log::warn!("[runtime_spawn] cannot create LLM log dir {log_dir}: {e}");
    }
    let log_path = format!("{log_dir}/{log_name}");
    match std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(log_file) => {
            if let Ok(clone) = log_file.try_clone() {
                command = command.stdout(std::process::Stdio::from(clone));
            } else {
                command = command.stdout(std::process::Stdio::null());
            }
            command = command.stderr(std::process::Stdio::from(log_file));
        }
        Err(_) => {
            command = command.stdout(std::process::Stdio::null());
            command = command.stderr(std::process::Stdio::null());
        }
    }
    let child = command.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    Ok(child.id())
}
