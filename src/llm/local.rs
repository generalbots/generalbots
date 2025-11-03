use crate::shared::state::AppState;
use crate::shared::models::schema::bots::dsl::*;
use crate::config::ConfigManager;
use diesel::prelude::*;
use std::sync::Arc;
use log::{info, error};
use tokio;
use reqwest;

use actix_web::{post, web, HttpResponse, Result};

#[post("/api/chat/completions")]
pub async fn chat_completions_local(
    _data: web::Data<AppState>,
    _payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // Placeholder implementation
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "chat_completions_local not implemented" })))
}

#[post("/api/embeddings")]
pub async fn embeddings_local(
    _data: web::Data<AppState>,
    _payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // Placeholder implementation
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "embeddings_local not implemented" })))
}

pub async fn ensure_llama_servers_running(
    app_state: &Arc<AppState>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = app_state.conn.clone();
    let config_manager = ConfigManager::new(conn.clone());

    let default_bot_id = {
        let mut conn = conn.lock().unwrap();
        bots.filter(name.eq("default"))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .unwrap_or_else(|_| uuid::Uuid::nil())
    };

    let llm_url = config_manager.get_config(&default_bot_id, "llm-url", None)?;
    let llm_model = config_manager.get_config(&default_bot_id, "llm-model", None)?;
    let embedding_url = config_manager.get_config(&default_bot_id, "embedding-url", None)?;
    let embedding_model = config_manager.get_config(&default_bot_id, "embedding-model", None)?;
    let llm_server_path = config_manager.get_config(&default_bot_id, "llm-server-path", None)?;

    info!("Starting LLM servers...");
    info!("Configuration:");
    info!("  LLM URL: {}", llm_url);
    info!("  Embedding URL: {}", embedding_url);
    info!("  LLM Model: {}", llm_model);
    info!("  Embedding Model: {}", embedding_model);
    info!("  LLM Server Path: {}", llm_server_path);

    // Restart any existing llama-server processes
    info!("Restarting any existing llama-server processes...");
    if let Err(e) = tokio::process::Command::new("sh")
        .arg("-c")
        .arg("pkill -f llama-server || true")
        .spawn()
    {
        error!("Failed to execute pkill for llama-server: {}", e);
    } else {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        info!("Existing llama-server processes terminated (if any)");
    }

    // Check if servers are already running
    let llm_running = is_server_running(&llm_url).await;
    let embedding_running = is_server_running(&embedding_url).await;

    if llm_running && embedding_running {
        info!("Both LLM and Embedding servers are already running");
        return Ok(());
    }

    // Start servers that aren't running
    let mut tasks = vec![];

    if !llm_running && !llm_model.is_empty() {
        info!("Starting LLM server...");
        tasks.push(tokio::spawn(start_llm_server(
            Arc::clone(app_state),
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

    // Wait for all server startup tasks
    for task in tasks {
        task.await??;
    }

    // Wait for servers to be ready with verbose logging
    info!("Waiting for servers to become ready...");

    let mut llm_ready = llm_running || llm_model.is_empty();
    let mut embedding_ready = embedding_running || embedding_model.is_empty();

    let mut attempts = 0;
    let max_attempts = 60; // 2 minutes total

    while attempts < max_attempts && (!llm_ready || !embedding_ready) {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Checking server health (attempt {}/{})...", attempts + 1, max_attempts);

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
            info!("Still waiting for servers... (attempt {}/{})", attempts, max_attempts);
        }
    }

    if llm_ready && embedding_ready {
        info!("All llama.cpp servers are ready and responding!");
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

    let default_bot_id = {
        let mut conn = conn.lock().unwrap();
        bots.filter(name.eq("default"))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .unwrap_or_else(|_| uuid::Uuid::nil())
    };

    let n_moe = config_manager.get_config(&default_bot_id, "llm-server-n-moe", None).unwrap_or("4".to_string());
    let ctx_size = config_manager.get_config(&default_bot_id, "llm-server-ctx-size", None).unwrap_or("4096".to_string());
    let parallel = config_manager.get_config(&default_bot_id, "llm-server-parallel", None).unwrap_or("1".to_string());
    let cont_batching = config_manager.get_config(&default_bot_id, "llm-server-cont-batching", None).unwrap_or("true".to_string());
    let mlock = config_manager.get_config(&default_bot_id, "llm-server-mlock", None).unwrap_or("true".to_string());
    let no_mmap = config_manager.get_config(&default_bot_id, "llm-server-no-mmap", None).unwrap_or("true".to_string());
    let gpu_layers = config_manager.get_config(&default_bot_id, "llm-server-gpu-layers", None).unwrap_or("20".to_string());

    // Build command arguments dynamically
    let mut args = format!(
        "-m {} --host 0.0.0.0 --port {} --top_p 0.95 --temp 0.6 --ctx-size {} --repeat-penalty 1.2 -ngl {}",
        model_path, port, ctx_size, gpu_layers
    );

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

    if cfg!(windows) {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.arg("/C").arg(format!(
            "cd {} && .\\llama-server.exe {} --verbose>../../../../logs/llm/stdout.log",
            llama_cpp_path, args
        ));
        cmd.spawn()?;
    } else {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(format!(
            "cd {} && ./llama-server {} --verbose >../../../../logs/llm/stdout.log 2>&1 &",
            llama_cpp_path, args
        ));
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
            "cd {} && .\\llama-server.exe -m {} --verbose --host 0.0.0.0 --port {} --embedding --n-gpu-layers 99 >../../../../logs/llm/stdout.log",
            llama_cpp_path, model_path, port
        ));
        cmd.spawn()?;
    } else {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(format!(
            "cd {} && ./llama-server -m {} --verbose --host 0.0.0.0 --port {} --embedding --n-gpu-layers 99 >../../../../logs/llm/stdout.log 2>&1 &",
            llama_cpp_path, model_path, port
        ));
        cmd.spawn()?;
    }

    Ok(())
}
