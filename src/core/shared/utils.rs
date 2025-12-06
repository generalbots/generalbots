use crate::config::DriveConfig;
use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Builder as S3ConfigBuilder, Client as S3Client};
use diesel::Connection;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use futures_util::StreamExt;
#[cfg(feature = "progress-bars")]
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use rhai::{Array, Dynamic};
use serde_json::Value;
use smartstring::SmartString;
use std::error::Error;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;

pub async fn create_s3_operator(
    config: &DriveConfig,
) -> Result<S3Client, Box<dyn std::error::Error>> {
    let endpoint = if !config.server.ends_with('/') {
        format!("{}/", config.server)
    } else {
        config.server.clone()
    };
    let base_config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region("auto")
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            config.access_key.clone(),
            config.secret_key.clone(),
            None,
            None,
            "static",
        ))
        .load()
        .await;
    let s3_config = S3ConfigBuilder::from(&base_config)
        .force_path_style(true)
        .build();
    Ok(S3Client::from_conf(s3_config))
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

/// Download a file from a URL with progress bar (when progress-bars feature is enabled)
#[cfg(feature = "progress-bars")]
pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
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
            Ok(())
        } else {
            Err(anyhow::anyhow!("HTTP {}: {}", response.status(), url))
        }
    });
    download_handle.await?
}

/// Download a file from a URL (without progress bar when progress-bars feature is disabled)
#[cfg(not(feature = "progress-bars"))]
pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
    let url = url.to_string();
    let output_path = output_path.to_string();
    let download_handle = tokio::spawn(async move {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
            .build()?;
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            let mut file = TokioFile::create(&output_path).await?;
            let mut stream = response.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                file.write_all(&chunk).await?;
            }
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

pub fn estimate_token_count(text: &str) -> usize {
    let char_count = text.chars().count();
    (char_count / 4).max(1)
}

pub fn establish_pg_connection() -> Result<PgConnection> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    PgConnection::establish(&database_url)
        .with_context(|| format!("Failed to connect to database at {}", database_url))
}

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_conn() -> Result<DbPool, diesel::r2d2::PoolError> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn parse_database_url(url: &str) -> (String, String, String, u32, String) {
    if let Some(stripped) = url.strip_prefix("postgres://") {
        let parts: Vec<&str> = stripped.split('@').collect();
        if parts.len() == 2 {
            let user_pass: Vec<&str> = parts[0].split(':').collect();
            let host_db: Vec<&str> = parts[1].split('/').collect();
            if user_pass.len() >= 2 && host_db.len() >= 2 {
                let username = user_pass[0].to_string();
                let password = user_pass[1].to_string();
                let host_port: Vec<&str> = host_db[0].split(':').collect();
                let server = host_port[0].to_string();
                let port = host_port
                    .get(1)
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5432);
                let database = host_db[1].to_string();
                return (username, password, server, port, database);
            }
        }
    }
    (
        "".to_string(),
        "".to_string(),
        "".to_string(),
        5432,
        "".to_string(),
    )
}

/// Run database migrations
pub fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS).map_err(
        |e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Migration error: {}", e),
            ))
        },
    )?;
    Ok(())
}
