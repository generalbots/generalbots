use anyhow::{Context, Result};
use diesel::{Connection, PgConnection};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::trace;
use reqwest::Client;
use rhai::{Array, Dynamic};
use serde_json::Value;
use smartstring::SmartString;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

pub fn extract_zip_recursive(
    zip_path: &Path,
    destination_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let buf_reader = BufReader::new(file);
    let mut archive = ZipArchive::new(buf_reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination_path.join(file.mangled_name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(&parent)?;
                }

use crate::llm::LLMProvider;
use std::sync::Arc;
use serde_json::Value;

/// Unified chat utility to interact with any LLM provider
pub async fn chat_with_llm(
    provider: Arc<dyn LLMProvider>,
    prompt: &str,
    config: &Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    provider.generate(prompt, config).await
}
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

pub fn json_value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => Dynamic::from(
            arr.iter()
                .map(json_value_to_dynamic)
                .collect::<rhai::Array>(),
        ),
        Value::Object(obj) => Dynamic::from(
            obj.iter()
                .map(|(k, v)| (SmartString::from(k), json_value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}

pub fn to_array(value: Dynamic) -> Array {
    if value.is_array() {
        value.cast::<Array>()
    } else if value.is_unit() || value.is::<()>() {
        Array::new()
    } else {
        Array::from([value])
    }
}

pub async fn download_file(
    url: &str,
    output_path: &str,
) -> Result<(), anyhow::Error> {
    let url = url.to_string();
    let output_path = output_path.to_string();

    let download_handle = tokio::spawn(async move {
        let client = Client::builder()
    .user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
    .build()?;
        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let total_size = response.content_length().unwrap_or(0);
            let pb = ProgressBar::new(total_size);
            pb.set_style(ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"));
            pb.set_message(format!("Downloading {}", url));

            let mut file = TokioFile::create(&output_path).await?;
            let mut downloaded: u64 = 0;
            let mut stream = response.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }

            pb.finish_with_message(format!("Downloaded {}", output_path));
            trace!("Download completed: {} -> {}", url, output_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("HTTP {}: {}", response.status(), url))
        }
    });

    download_handle.await?
}

pub fn parse_filter(filter_str: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let parts: Vec<&str> = filter_str.split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid filter format. Expected 'KEY=VALUE'".into());
    }

    let column = parts[0].trim();
    let value = parts[1].trim();

    if !column
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Invalid column name in filter".into());
    }

    Ok((format!("{} = $1", column), vec![value.to_string()]))
}

pub fn parse_filter_with_offset(
    filter_str: &str,
    offset: usize,
) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let mut clauses = Vec::new();
    let mut params = Vec::new();

    for (i, condition) in filter_str.split('&').enumerate() {
        let parts: Vec<&str> = condition.split('=').collect();
        if parts.len() != 2 {
            return Err("Invalid filter format".into());
        }

        let column = parts[0].trim();
        let value = parts[1].trim();

        if !column
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err("Invalid column name".into());
        }

        clauses.push(format!("{} = ${}", column, i + 1 + offset));
        params.push(value.to_string());
    }

    Ok((clauses.join(" AND "), params))
}

pub async fn call_llm(
    prompt: &str,
    _llm_config: &crate::config::LLMConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(format!("Generated response for: {}", prompt))
}

/// Estimates token count for text using simple heuristic (1 token ≈ 4 chars)
pub fn estimate_token_count(text: &str) -> usize {
    // Basic token estimation - count whitespace-separated words
    // Add 1 token for every 4 characters as a simple approximation
    let char_count = text.chars().count();
    (char_count / 4).max(1) // Ensure at least 1 token
}

/// Establishes a PostgreSQL connection using DATABASE_URL environment variable
pub fn establish_pg_connection() -> Result<PgConnection> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://gbuser:@localhost:5432/botserver".to_string());
    
    PgConnection::establish(&database_url)
        .with_context(|| format!("Failed to connect to database at {}", database_url))
}
