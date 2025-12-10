use crate::config::ConfigManager;
use crate::shared::models::schema::bots::dsl::*;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, warn};
use reqwest;
use std::path::Path;
use std::sync::Arc;
use tokio;

pub async fn ensure_llama_servers_running(
    app_state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_values = {
        let conn_arc = app_state.conn.clone();
        let default_bot_id = tokio::task::spawn_blocking(move || {
            let mut conn = conn_arc.get().unwrap();
            bots.filter(name.eq("default"))
                .select(id)
                .first::<uuid::Uuid>(&mut *conn)
                .unwrap_or_else(|_| uuid::Uuid::nil())
        })
        .await?;
        let config_manager = ConfigManager::new(app_state.conn.clone());
        (
            default_bot_id,
            config_manager
                .get_config(&default_bot_id, "llm-server", None)
                .unwrap_or_else(|_| "false".to_string()),
            config_manager
                .get_config(&default_bot_id, "llm-url", None)
                .unwrap_or_default(),
            config_manager
                .get_config(&default_bot_id, "llm-model", None)
                .unwrap_or_default(),
            config_manager
                .get_config(&default_bot_id, "embedding-url", None)
                .unwrap_or_default(),
            config_manager
                .get_config(&default_bot_id, "embedding-model", None)
                .unwrap_or_default(),
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

    // Check if local LLM server management is enabled
    let llm_server_enabled = llm_server_enabled.to_lowercase() == "true";
    if !llm_server_enabled {
        info!("Local LLM server management disabled (llm-server=false). Using external endpoints.");
        info!("  LLM URL: {}", llm_url);
        info!("  Embedding URL: {}", embedding_url);
        return Ok(());
    }
    info!("Starting LLM servers...");
    info!("Configuration:");
    info!("  LLM URL: {}", llm_url);
    info!("  Embedding URL: {}", embedding_url);
    info!("  LLM Model: {}", llm_model);
    info!("  Embedding Model: {}", embedding_model);
    info!("  LLM Server Path: {}", llm_server_path);

    // Check if llama-server binary exists
    let llama_server_path = if llm_server_path.is_empty() {
        "./botserver-stack/bin/llm/build/bin/llama-server".to_string()
    } else {
        format!("{}/llama-server", llm_server_path)
    };

    if !Path::new(&llama_server_path).exists() {
        warn!("llama-server binary not found at: {}", llama_server_path);
        warn!("Local LLM server will not be available.");
        warn!("This may be because:");
        warn!("  1. The LLM component was not installed (check if CPU supports AVX2)");
        warn!("  2. The binary path is incorrect");
        warn!("Continuing without local LLM - use external LLM API instead.");
        return Ok(());
    }

    // Test if the binary can actually run (check for illegal instruction)
    info!("Testing llama-server binary compatibility...");
    let test_result = std::process::Command::new(&llama_server_path)
        .arg("--version")
        .output();

    match test_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("llama-server test failed: {}", stderr);
                if stderr.contains("Illegal instruction") {
                    error!("CPU does not support required instructions (AVX2) for llama-server");
                    error!("Your CPU: Check /proc/cpuinfo for 'avx2' flag");
                    error!("Options:");
                    error!("  1. Compile llama.cpp from source with your CPU's instruction set");
                    error!("  2. Use an external LLM API (OpenAI, Anthropic, etc.)");
                    return Ok(());
                }
            }
        }
        Err(e) => {
            warn!("Failed to test llama-server: {}", e);
            // Continue anyway - might work at runtime
        }
    }

    info!("Restarting any existing llama-server processes...");

    if let Err(e) = tokio::process::Command::new("sh")
        .arg("-c")
        .arg("pkill llama-server -9 || true")
        .spawn()
    {
        error!("Failed to execute pkill for llama-server: {}", e);
    } else {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        info!("Existing llama-server processes terminated (if any)");
    }

    // Skip local server startup if using HTTPS endpoints
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
        tasks.push(tokio::spawn(start_llm_server(
            Arc::clone(&app_state),
            llm_server_path.clone(),
            llm_model.clone(),
            llm_url.clone(),
        )));
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
    let mut llm_ready = llm_running || llm_model.is_empty();
    let mut embedding_ready = embedding_running || embedding_model.is_empty();
    let mut attempts = 0;
    let max_attempts = 60;
    while attempts < max_attempts && (!llm_ready || !embedding_ready) {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        info!(
            "Checking server health (attempt {}/{})...",
            attempts + 1,
            max_attempts
        );
        if !llm_ready && !llm_model.is_empty() {
            if is_server_running(&llm_url).await {
                info!("LLM server ready at {}", llm_url);
                llm_ready = true;
            } else {
                info!("LLM server not ready yet");
            }
        }
        if !embedding_ready && !embedding_model.is_empty() {
            if is_server_running(&embedding_url).await {
                info!("Embedding server ready at {}", embedding_url);
                embedding_ready = true;
            } else {
                info!("Embedding server not ready yet");
            }
        }
        attempts += 1;
        if attempts % 10 == 0 {
            info!(
                "Still waiting for servers... (attempt {}/{})",
                attempts, max_attempts
            );
        }
    }
    if llm_ready && embedding_ready {
        info!("All llama.cpp servers are ready and responding!");

        // Update LLM provider with new endpoints
        let _llm_provider1 = Arc::new(crate::llm::OpenAIClient::new(
            llm_model.clone(),
            Some(llm_url.clone()),
        ));
        Ok(())
    } else {
        let mut error_msg = "Servers failed to start within timeout:".to_string();
        if !llm_ready && !llm_model.is_empty() {
            error_msg.push_str(&format!("\n   - LLM server at {}", llm_url));
        }
        if !embedding_ready && !embedding_model.is_empty() {
            error_msg.push_str(&format!("\n   - Embedding server at {}", embedding_url));
        }
        Err(error_msg.into())
    }
}
pub async fn is_server_running(url: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get(&format!("{}/health", url)).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
pub async fn start_llm_server(
    app_state: Arc<AppState>,
    llama_cpp_path: String,
    model_path: String,
    url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = url.split(':').last().unwrap_or("8081");
    std::env::set_var("OMP_NUM_THREADS", "20");
    std::env::set_var("OMP_PLACES", "cores");
    std::env::set_var("OMP_PROC_BIND", "close");
    let conn = app_state.conn.clone();
    let config_manager = ConfigManager::new(conn.clone());
    let mut conn = conn.get().unwrap();
    let default_bot_id = bots
        .filter(name.eq("default"))
        .select(id)
        .first::<uuid::Uuid>(&mut *conn)
        .unwrap_or_else(|_| uuid::Uuid::nil());
    let n_moe = config_manager
        .get_config(&default_bot_id, "llm-server-n-moe", None)
        .unwrap_or("4".to_string());
    let parallel = config_manager
        .get_config(&default_bot_id, "llm-server-parallel", None)
        .unwrap_or("1".to_string());
    let cont_batching = config_manager
        .get_config(&default_bot_id, "llm-server-cont-batching", None)
        .unwrap_or("true".to_string());
    let mlock = config_manager
        .get_config(&default_bot_id, "llm-server-mlock", None)
        .unwrap_or("true".to_string());
    let no_mmap = config_manager
        .get_config(&default_bot_id, "llm-server-no-mmap", None)
        .unwrap_or("true".to_string());
    let gpu_layers = config_manager
        .get_config(&default_bot_id, "llm-server-gpu-layers", None)
        .unwrap_or("20".to_string());
    let reasoning_format = config_manager
        .get_config(&default_bot_id, "llm-server-reasoning-format", None)
        .unwrap_or("".to_string());
    let n_predict = config_manager
        .get_config(&default_bot_id, "llm-server-n-predict", None)
        .unwrap_or("50".to_string());

    let n_ctx_size = config_manager
        .get_config(&default_bot_id, "llm-server-ctx-size", None)
        .unwrap_or("4096".to_string());

    // Configuration for flash-attn, temp, top_p, repeat-penalty is handled via config.csv
    // Jinja templating is enabled by default when available

    let mut args = format!(
        "-m {} --host 0.0.0.0 --port {} --top_p 0.95 --temp 0.6 --repeat-penalty 1.2 --n-gpu-layers {}",
        model_path, port,  gpu_layers
    );
    if !reasoning_format.is_empty() {
        args.push_str(&format!(" --reasoning-format {}", reasoning_format));
    }

    if n_moe != "0" {
        args.push_str(&format!(" --n-cpu-moe {}", n_moe));
    }
    if parallel != "1" {
        args.push_str(&format!(" --parallel {}", parallel));
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
        args.push_str(&format!(" --n-predict {}", n_predict));
    }
    args.push_str(&format!(" --ctx-size {}", n_ctx_size));

    if cfg!(windows) {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.arg("/C").arg(format!(
            "cd {} && .\\llama-server.exe {}",
            llama_cpp_path, args
        ));
        info!(
            "Executing LLM server command: cd {} && .\\llama-server.exe {} --verbose",
            llama_cpp_path, args
        );
        cmd.spawn()?;
    } else {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(format!(
            "cd {} && ./llama-server {} --verbose >llm-stdout.log 2>&1 &",
            llama_cpp_path, args
        ));
        info!(
            "Executing LLM server command: cd {} && ./llama-server {} --verbose",
            llama_cpp_path, args
        );
        cmd.spawn()?;
    }
    Ok(())
}
pub async fn start_embedding_server(
    llama_cpp_path: String,
    model_path: String,
    url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = url.split(':').last().unwrap_or("8082");
    if cfg!(windows) {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.arg("/c").arg(format!(
            "cd {} && .\\llama-server.exe -m {} --verbose --host 0.0.0.0 --port {} --embedding --n-gpu-layers 99 >stdout.log",
            llama_cpp_path, model_path, port
        ));
        cmd.spawn()?;
    } else {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(format!(
            "cd {} && ./llama-server -m {} --verbose --host 0.0.0.0 --port {} --embedding --n-gpu-layers 99 >llmembd-stdout.log 2>&1 &",
            llama_cpp_path, model_path, port
        ));
        cmd.spawn()?;
    }
    Ok(())
}
