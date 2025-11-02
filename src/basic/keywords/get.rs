use crate::shared::models::schema::bots::dsl::*;
use diesel::prelude::*;
use crate::kb::minio_handler;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, error, info, trace};
use reqwest::{self, Client};
use rhai::{Dynamic, Engine};
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub fn get_keyword(state: Arc<AppState>, user_session: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(&["GET", "$expr$"], false, move |context, inputs| {
            let url = context.eval_expression_tree(&inputs[0])?;
            let url_str = url.to_string();

            info!("GET command executed: {}", url_str);

            if !is_safe_path(&url_str) {
                return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "URL contains invalid or unsafe path sequences".into(),
                    rhai::Position::NONE,
                )));
            }

            let state_for_blocking = Arc::clone(&state_clone);
            let url_for_blocking = url_str.clone();

            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        if url_for_blocking.starts_with("https://")
                            || url_for_blocking.starts_with("http://")
                        {
                            info!("HTTP(S) GET request: {}", url_for_blocking);
                            execute_get(&url_for_blocking).await
                        } else {
                            info!("Local file GET request from bucket: {}", url_for_blocking);
                            get_from_bucket(&state_for_blocking, &url_for_blocking, 
                                user_session.bot_id)
                                .await
                        }
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(40)) {
                Ok(Ok(content)) => Ok(Dynamic::from(content)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(Box::new(
                    rhai::EvalAltResult::ErrorRuntime("GET timed out".into(), rhai::Position::NONE),
                )),
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("GET failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .unwrap();
}

fn is_safe_path(path: &str) -> bool {
    if path.starts_with("https://") || path.starts_with("http://") {
        return true;
    }
    if path.contains("..") || path.starts_with('/') {
        return false;
    }
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return false;
    }
    if path.contains("//") || path.contains("~") || path.contains("*") || path.contains("?") {
        return false;
    }
    if !path.starts_with("http") {
        let path_obj = Path::new(path);
        if path_obj.components().count()
            != path_obj
                .components()
                .filter(|c| matches!(c, std::path::Component::Normal(_)))
                .count()
        {
            return false;
        }
    }
    true
}

pub async fn execute_get(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    debug!("Starting execute_get with URL: {}", url);

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            error!("Failed to build HTTP client: {}", e);
            e
        })?;

    let response = client.get(url).send().await.map_err(|e| {
        error!("HTTP request failed for URL {}: {}", url, e);
        e
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        error!(
            "HTTP request returned non-success status for URL {}: {} - {}",
            url, status, error_body
        );
        return Err(format!(
            "HTTP request failed with status: {} - {}",
            status, error_body
        )
        .into());
    }

    let content = response.text().await.map_err(|e| {
        error!("Failed to read response text for URL {}: {}", url, e);
        e
    })?;

    debug!(
        "Successfully executed GET request for URL: {}, content length: {}",
        url,
        content.len()
    );
    Ok(content)
}

pub async fn get_from_bucket(
    state: &AppState,
    file_path: &str,
    bot_id: uuid::Uuid,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    debug!("Getting file from bucket: {}", file_path);

    if !is_safe_path(file_path) {
        error!("Unsafe file path detected: {}", file_path);
        return Err("Invalid file path".into());
    }

    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bot_name: String = {
        let mut db_conn = state.conn.lock().unwrap();
        bots.filter(id.eq(&bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name for {}: {}", bot_id, e);
                e
            })?
    };

    let bucket_name = {
        let bucket = format!("{}.gbai", bot_name);
        trace!("Resolved GET bucket name: {}", bucket);
        bucket
    };

    let bytes = match tokio::time::timeout(
        Duration::from_secs(30),
        minio_handler::get_file_content(client, &bucket_name, file_path),
    )
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            error!("drive read failed: {}", e);
            return Err(format!("S3 operation failed: {}", e).into());
        }
        Err(_) => {
            error!("drive read timed out");
            return Err("drive operation timed out".into());
        }
    };

    let content = if file_path.to_ascii_lowercase().ends_with(".pdf") {
        match pdf_extract::extract_text_from_mem(&bytes) {
            Ok(text) => text,
            Err(e) => {
                error!("PDF extraction failed: {}", e);
                return Err(format!("PDF extraction failed: {}", e).into());
            }
        }
    } else {
        match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                error!("File content is not valid UTF-8 text");
                return Err("File content is not valid UTF-8 text".into());
            }
        }
    };

    info!(
        "Successfully retrieved file from bucket: {}, content length: {}",
        file_path,
        content.len()
    );
    Ok(content)
}
