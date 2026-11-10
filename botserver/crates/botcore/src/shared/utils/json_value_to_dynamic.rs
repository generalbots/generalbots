use super::*;

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
#[cfg(feature = "progress-bars")]
pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
    use botsecurity_core::validation::{fetch_user_url, FetchPolicy};
    use std::time::Duration;
    let url = url.to_string();
    let output_path = output_path.to_string();
    let download_handle = tokio::spawn(async move {
        // #2131 — the URL is caller-supplied: guarded fetch with a size cap.
        // #2152 — model data is big: the DeepSeek GGUF is ~880 MB, so the cap
        // and the wall-clock budget must cover real model downloads, not just
        // HTML pages.
        let policy = FetchPolicy::default()
            .with_timeout(Duration::from_secs(1800))
            .with_user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
            .with_max_bytes(2 * 1024 * 1024 * 1024);
        let response = fetch_user_url(&url, &policy).await?;
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP {}: {}", response.status, url));
        }
        if response.truncated {
            return Err(anyhow::anyhow!("download exceeds the size cap"));
        }
        let total_size = response.body.len() as u64;
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", url));
        let mut file = TokioFile::create(&output_path).await?;
        file.write_all(&response.body).await?;
        pb.set_position(total_size);
        pb.finish_with_message(format!("Downloaded {}", output_path));
        Ok(())
    });
    download_handle.await?
}
#[cfg(not(feature = "progress-bars"))]
pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
    use botsecurity_core::validation::{fetch_user_url, FetchPolicy};
    use std::time::Duration;
    let url = url.to_string();
    let output_path = output_path.to_string();
    let download_handle = tokio::spawn(async move {
        // #2131 — the URL is caller-supplied: guarded fetch with a size cap.
        // #2152 — model data is big: the DeepSeek GGUF is ~880 MB, so the cap
        // and the wall-clock budget must cover real model downloads, not just
        // HTML pages.
        let policy = FetchPolicy::default()
            .with_timeout(Duration::from_secs(1800))
            .with_user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
            .with_max_bytes(2 * 1024 * 1024 * 1024);
        let response = fetch_user_url(&url, &policy).await?;
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP {}: {}", response.status, url));
        }
        if response.truncated {
            return Err(anyhow::anyhow!("download exceeds the size cap"));
        }
        let mut file = TokioFile::create(&output_path).await?;
        file.write_all(&response.body).await?;
        Ok(())
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
    let database_url = get_database_url_sync()?;
    PgConnection::establish(&database_url)
        .with_context(|| format!("Failed to connect to database at {}", database_url))
}
