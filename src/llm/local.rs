use crate::core::config::ConfigManager;
use crate::core::kb::embedding_generator::set_embedding_server_ready;
use crate::core::shared::memory_monitor::{log_jemalloc_stats, MemoryStats};
use crate::security::command_guard::SafeCommand;
use crate::core::shared::models::schema::bots::dsl::*;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, trace, warn};
use reqwest;
use std::fmt::Write;
use std::sync::Arc;
use tokio;

pub async fn ensure_llama_servers_running(
    app_state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!("ensure_llama_servers_running ENTER");
    let start_mem = MemoryStats::current();
    trace!(
        "[LLM_LOCAL] ensure_llama_servers_running START, RSS={}",
        MemoryStats::format_bytes(start_mem.rss_bytes)
    );
    log_jemalloc_stats();

    if std::env::var("SKIP_LLM_SERVER").is_ok() {
        trace!("SKIP_LLM_SERVER set, returning early");
        info!("SKIP_LLM_SERVER set - skipping local LLM server startup (using mock/external LLM)");
        return Ok(());
    }

    let config_values = {
        let conn_arc = app_state.conn.clone();
        let (default_bot_id, _default_bot_name) = tokio::task::spawn_blocking(move || -> Result<(uuid::Uuid, String), String> {
            let mut conn = conn_arc
                .get()
                .map_err(|e| format!("failed to get db connection: {e}"))?;
            Ok(crate::core::bot::get_default_bot(&mut *conn))
        })
        .await??;
        let config_manager = ConfigManager::new(app_state.conn.clone());
        info!("Reading config for bot_id: {}", default_bot_id);
        let embedding_model_result = config_manager.get_config(&default_bot_id, "embedding-model", None);
        info!("embedding-model config result: {:?}", embedding_model_result);
        (
            default_bot_id,
            config_manager
                .get_config(&default_bot_id, "llm-server", Some("true"))
                .unwrap_or_else(|_| "true".to_string()),
            config_manager
                .get_config(&default_bot_id, "llm-url", Some("http://localhost:8081"))
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            config_manager
                .get_config(&default_bot_id, "llm-model", None)
                .unwrap_or_default(),
            config_manager
                .get_config(&default_bot_id, "embedding-url", Some("http://localhost:8082"))
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            embedding_model_result.unwrap_or_default(),
            config_manager
                .get_config(&default_bot_id, "llm-server-path", None)
                .unwrap_or_default(),
        )
    };
    let (
        _default_bot_id,
        llm_server_enabled,
        llm_url,
        llm_model,
        embedding_url,
        embedding_model,
        llm_server_path,
    ) = config_values;

    let llm_server_enabled = llm_server_enabled.to_lowercase() == "true";
    if !llm_server_enabled {
        info!("Local LLM server management disabled (llm-server=false). Using external endpoints.");
        info!("  LLM URL: {llm_url}");
        info!("  Embedding URL: {embedding_url}");
        return Ok(());
    }
    info!("Starting LLM servers...");
    info!("Configuration:");
    info!("  LLM URL: {llm_url}");
    info!("  Embedding URL: {embedding_url}");
    info!("  LLM Model: {llm_model}");
    info!("  Embedding Model: {embedding_model}");
    info!("  LLM Server Path: {llm_server_path}");
    info!("Restarting any existing llama-server processes...");
    trace!("About to pkill llama-server...");
    let before_pkill = MemoryStats::current();
    trace!(
        "[LLM_LOCAL] Before pkill, RSS={}",
        MemoryStats::format_bytes(before_pkill.rss_bytes)
    );

    let pkill_result = SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.trusted_shell_script_arg("pkill llama-server -9; true"));

    match pkill_result {
        Ok(cmd) => {
            if let Err(e) = cmd.execute() {
                error!("Failed to execute pkill for llama-server: {e}");
            } else {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                info!("Existing llama-server processes terminated (if any)");
            }
        }
        Err(e) => error!("Failed to build pkill command: {e}"),
    }
    trace!("pkill done");

    let after_pkill = MemoryStats::current();
    trace!(
        "[LLM_LOCAL] After pkill, RSS={} (delta={})",
        MemoryStats::format_bytes(after_pkill.rss_bytes),
        MemoryStats::format_bytes(after_pkill.rss_bytes.saturating_sub(before_pkill.rss_bytes))
    );

    let llm_running = if llm_url.starts_with("https://") {
        info!("Using external HTTPS LLM server, skipping local startup");
        true
    } else {
        is_server_running(&llm_url).await
    };

    let embedding_running = if embedding_url.starts_with("https://") {
        info!("Using external HTTPS embedding server, skipping local startup");
        true
    } else {
        is_server_running(&embedding_url).await
    };
    if llm_running && embedding_running {
        info!("Both LLM and Embedding servers are already running");
        return Ok(());
    }
    let mut tasks = vec![];
    if !llm_running && !llm_model.is_empty() {
        info!("Starting LLM server...");
        let app_state_clone = Arc::clone(&app_state);
        let llm_server_path_clone = llm_server_path.clone();
        let llm_model_clone = llm_model.clone();
        let llm_url_clone = llm_url.clone();
        tasks.push(tokio::spawn(async move {
            start_llm_server(
                app_state_clone,
                llm_server_path_clone,
                llm_model_clone,
                llm_url_clone,
            )
        }));
    } else if llm_model.is_empty() {
        info!("LLM_MODEL not set, skipping LLM server");
    }
    if !embedding_running && !embedding_model.is_empty() {
        info!("Starting Embedding server...");
        tasks.push(tokio::spawn(start_embedding_server(
            llm_server_path.clone(),
            embedding_model.clone(),
            embedding_url.clone(),
        )));
    } else if embedding_model.is_empty() {
        info!("EMBEDDING_MODEL not set, skipping Embedding server");
    }
    for task in tasks {
        task.await??;
    }
    info!("Waiting for servers to become ready...");
    trace!("Starting wait loop for servers...");
    let before_wait = MemoryStats::current();
    trace!(
        "[LLM_LOCAL] Before wait loop, RSS={}",
        MemoryStats::format_bytes(before_wait.rss_bytes)
    );

    let mut llm_ready = llm_running || llm_model.is_empty();
    let mut embedding_ready = embedding_running || embedding_model.is_empty();
    let mut attempts = 0;
    let max_attempts = 120;
    while attempts < max_attempts && (!llm_ready || !embedding_ready) {
        trace!("Wait loop iteration {}", attempts);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        if attempts % 5 == 0 {
            let loop_mem = MemoryStats::current();
            trace!(
                "[LLM_LOCAL] Wait loop attempt {}, RSS={} (delta from start={})",
                attempts,
                MemoryStats::format_bytes(loop_mem.rss_bytes),
                MemoryStats::format_bytes(loop_mem.rss_bytes.saturating_sub(before_wait.rss_bytes))
            );
            log_jemalloc_stats();
        }

        if attempts % 5 == 0 {
            info!(
                "Checking server health (attempt {}/{max_attempts})...",
                attempts + 1
            );
        }
        if !llm_ready && !llm_model.is_empty() {
            if is_server_running(&llm_url).await {
                info!("LLM server ready at {llm_url}");
                llm_ready = true;
            } else {
                info!("LLM server not ready yet");
            }
        }
        if !embedding_ready && !embedding_model.is_empty() {
            if is_server_running(&embedding_url).await {
                info!("Embedding server ready at {embedding_url}");
                embedding_ready = true;
                set_embedding_server_ready(true);
            } else if attempts % 10 == 0 {
                warn!("Embedding server not ready yet at {embedding_url}");

                if let Ok(log_content) =
                    std::fs::read_to_string(format!("{llm_server_path}/llmembd-stdout.log"))
                {
                    let last_lines: Vec<&str> = log_content.lines().rev().take(5).collect();
                    if !last_lines.is_empty() {
                        info!("Embedding server log (last 5 lines):");
                        for line in last_lines.iter().rev() {
                            info!("  {line}");
                        }
                    }
                }
            }
        }
        attempts += 1;
        if attempts % 20 == 0 {
            warn!(
                "Still waiting for servers... (attempt {attempts}/{max_attempts}) - this may take a while for large models"
            );
        }
    }
    if llm_ready && embedding_ready {
        info!("All llama.cpp servers are ready and responding!");
        if !embedding_model.is_empty() {
            set_embedding_server_ready(true);
        }
        trace!("Servers ready!");

        let after_ready = MemoryStats::current();
        trace!(
            "[LLM_LOCAL] Servers ready, RSS={} (delta from start={})",
            MemoryStats::format_bytes(after_ready.rss_bytes),
            MemoryStats::format_bytes(after_ready.rss_bytes.saturating_sub(start_mem.rss_bytes))
        );
        log_jemalloc_stats();

        let _llm_provider1 = Arc::new(crate::llm::OpenAIClient::new(
            llm_model.clone(),
            Some(llm_url.clone()),
            None,
        ));

        let end_mem = MemoryStats::current();
        trace!(
            "[LLM_LOCAL] ensure_llama_servers_running END, RSS={} (total delta={})",
            MemoryStats::format_bytes(end_mem.rss_bytes),
            MemoryStats::format_bytes(end_mem.rss_bytes.saturating_sub(start_mem.rss_bytes))
        );
        log_jemalloc_stats();

        trace!("ensure_llama_servers_running EXIT OK");
        Ok(())
    } else {
        let mut error_msg = "Servers failed to start within timeout:".to_string();
        if !llm_ready && !llm_model.is_empty() {
            let _ = write!(error_msg, "\n   - LLM server at {llm_url}");
        }
        if !embedding_ready && !embedding_model.is_empty() {
            let _ = write!(error_msg, "\n   - Embedding server at {embedding_url}");
        }
        Err(error_msg.into())
    }
}
pub async fn is_server_running(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    match client.get(format!("{url}/health")).send().await {
        Ok(response) => {
            if response.status().is_success() {
                return true;
            }

            info!("Health check returned status: {}", response.status());
            false
        }
        Err(e) => match client.get(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => {
                if !e.is_connect() {
                    warn!("Health check error for {url}: {e}");
                }
                false
            }
        },
    }
}
pub fn start_llm_server(
    app_state: Arc<AppState>,
    llama_cpp_path: String,
    model_path: String,
    url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = extract_port(&url);
    std::env::set_var("OMP_NUM_THREADS", "20");
    std::env::set_var("OMP_PLACES", "cores");
    std::env::set_var("OMP_PROC_BIND", "close");
    let conn = app_state.conn.clone();
    let config_manager = ConfigManager::new(conn.clone());
    let mut conn = conn.get().map_err(|e| {
        Box::new(std::io::Error::other(
            format!("failed to get db connection: {e}"),
        )) as Box<dyn std::error::Error + Send + Sync>
    })?;
    let default_bot_id = bots
        .filter(name.eq("default"))
        .select(id)
        .first::<uuid::Uuid>(&mut *conn)
        .unwrap_or_else(|_| uuid::Uuid::nil());
    let n_moe = config_manager
        .get_config(&default_bot_id, "llm-server-n-moe", None)
        .unwrap_or_else(|_| "4".to_string());
    let parallel = config_manager
        .get_config(&default_bot_id, "llm-server-parallel", None)
        .unwrap_or_else(|_| "1".to_string());
    let cont_batching = config_manager
        .get_config(&default_bot_id, "llm-server-cont-batching", None)
        .unwrap_or_else(|_| "true".to_string());
    let mlock = config_manager
        .get_config(&default_bot_id, "llm-server-mlock", None)
        .unwrap_or_else(|_| "true".to_string());
    let no_mmap = config_manager
        .get_config(&default_bot_id, "llm-server-no-mmap", None)
        .unwrap_or_else(|_| "true".to_string());
    let gpu_layers = config_manager
        .get_config(&default_bot_id, "llm-server-gpu-layers", None)
        .unwrap_or_else(|_| "20".to_string());
    let reasoning_format = config_manager
        .get_config(&default_bot_id, "llm-server-reasoning-format", None)
        .unwrap_or_else(|_| String::new());
    let n_predict = config_manager
        .get_config(&default_bot_id, "llm-server-n-predict", None)
        .unwrap_or_else(|_| "50".to_string());

    let n_ctx_size = config_manager
        .get_config(&default_bot_id, "llm-server-ctx-size", None)
        .unwrap_or_else(|_| "32000".to_string());

    let mut args = format!(
        "-m {model_path} --host 0.0.0.0 --port {port} --top_p 0.95 --temp 0.6 --repeat-penalty 1.2 --n-gpu-layers {gpu_layers} --ubatch-size 2048"
    );
    if !reasoning_format.is_empty() {
        let _ = write!(args, " --reasoning-format {reasoning_format}");
    }

    if n_moe != "0" {
        let _ = write!(args, " --n-cpu-moe {n_moe}");
    }
    if parallel != "1" {
        let _ = write!(args, " --parallel {parallel}");
    }
    if cont_batching == "true" {
        args.push_str(" --cont-batching");
    }
    if mlock == "true" {
        args.push_str(" --mlock");
    }
    if no_mmap == "true" {
        args.push_str(" --no-mmap");
    }
    if n_predict != "0" {
        let _ = write!(args, " --n-predict {n_predict}");
    }
    let _ = write!(args, " --ctx-size {n_ctx_size}");

    if cfg!(windows) {
        let cmd_arg = format!("cd {llama_cpp_path} && .\\llama-server.exe {args}");
        info!(
            "Executing LLM server command: cd {llama_cpp_path} && .\\llama-server.exe {args} --verbose"
        );
        let cmd = SafeCommand::new("cmd")
            .and_then(|c| c.arg("/C"))
            .and_then(|c| c.trusted_shell_script_arg(&cmd_arg))
            .map_err(|e| {
                Box::new(std::io::Error::other(
                    e.to_string(),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        cmd.execute().map_err(|e| {
            Box::new(std::io::Error::other(
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
    } else {
        let cmd_arg = format!(
            "cd {llama_cpp_path} && ./llama-server {args} --verbose >llm-stdout.log 2>&1 &"
        );
        info!(
            "Executing LLM server command: cd {llama_cpp_path} && ./llama-server {args} --verbose"
        );
        let cmd = SafeCommand::new("sh")
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.trusted_shell_script_arg(&cmd_arg))
            .map_err(|e| {
                Box::new(std::io::Error::other(
                    e.to_string(),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        cmd.execute().map_err(|e| {
            Box::new(std::io::Error::other(
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
    }
    Ok(())
}
pub async fn start_embedding_server(
    llama_cpp_path: String,
    model_path: String,
    url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = extract_port(&url);

    let full_model_path = if model_path.starts_with('/') {
        model_path.clone()
    } else {
        format!("{llama_cpp_path}/{model_path}")
    };

    if !std::path::Path::new(&full_model_path).exists() {
        error!("Embedding model file not found: {full_model_path}");
        return Err(format!("Embedding model file not found: {full_model_path}").into());
    }

    info!("Starting embedding server on port {port} with model: {model_path}");

    if cfg!(windows) {
        let cmd_arg = format!(
            "cd {llama_cpp_path} && .\\llama-server.exe -m {model_path} --verbose --host 0.0.0.0 --port {port} --embedding --n-gpu-layers 99 >stdout.log 2>&1"
        );
        let cmd = SafeCommand::new("cmd")
            .and_then(|c| c.arg("/c"))
            .and_then(|c| c.trusted_shell_script_arg(&cmd_arg))
            .map_err(|e| {
                Box::new(std::io::Error::other(
                    e.to_string(),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        cmd.execute().map_err(|e| {
            Box::new(std::io::Error::other(
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
    } else {
        let cmd_arg = format!(
            "cd {llama_cpp_path} && ./llama-server -m {model_path} --verbose --host 0.0.0.0 --port {port} --embedding --n-gpu-layers 99 --ubatch-size 2048 >llmembd-stdout.log 2>&1 &"
        );
        info!(
            "Executing embedding server command: cd {llama_cpp_path} && ./llama-server -m {model_path} --host 0.0.0.0 --port {port} --embedding"
        );
        let cmd = SafeCommand::new("sh")
            .and_then(|c| c.arg("-c"))
            .and_then(|c| c.trusted_shell_script_arg(&cmd_arg))
            .map_err(|e| {
                Box::new(std::io::Error::other(
                    e.to_string(),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        cmd.execute().map_err(|e| {
            Box::new(std::io::Error::other(
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}

fn extract_port(url: &str) -> &str {
    url.rsplit(':').next().unwrap_or("8081")
}
