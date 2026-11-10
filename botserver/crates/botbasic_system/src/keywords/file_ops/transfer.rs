/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the MIT License.                                             |
|                                                                             |
| This program is free software: you can redistribute it and/or modify        |
| it under the terms of the MIT License.                                      |
|                                                                             |
| The text of the MIT License can be found in the LICENSE file you have       |
| received along with this program.                                           |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.                        |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the MIT License does not imply a         |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use std::sync::Arc;
use log::trace;
use std::error::Error;

use crate::keywords::file_ops::basic_io::execute_write;

pub struct FileData {
    pub content: Vec<u8>,
    pub filename: String,
}

pub async fn execute_upload(
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    file_data: FileData,
    destination: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = state.drive_repository().ok_or("S3 client not configured")?;

    let loc = {
        let mut db_conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
        botbasic_core::utils::bot_drive_location_for(&mut db_conn, user.bot_id)
    };
    let bucket_name = loc.bucket.clone();
    let key = loc.key_for(destination);

    trace!(
        "Uploading file '{}' to {bucket_name}/{key} ({} bytes)",
        file_data.filename,
        file_data.content.len()
    );

    client
        .put_object(&bucket_name, &key, file_data.content, None).await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    let url = format!("s3://{bucket_name}/{key}");
    trace!(
        "UPLOAD successful: {url} (original filename: {})",
        file_data.filename
    );
    Ok(url)
}

pub async fn execute_download(
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    url: &str,
    local_path: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    // #2131 — `DOWNLOAD <url>` takes the URL from the bot script: shared guard.
    let policy = botsecurity::validation::FetchPolicy::default()
        .with_timeout(std::time::Duration::from_secs(60))
        .with_max_bytes(64 * 1024 * 1024);
    let response = botsecurity::validation::fetch_user_url(url, &policy)
        .await
        .map_err(|e| {
            log::warn!(
                "DOWNLOAD: refused URL ({}): {}",
                e.kind().as_str(),
                e.detail()
            );
            Box::new(e) as Box<dyn Error + Send + Sync>
        })?;
    if !response.is_success() {
        return Err(format!("Download failed: HTTP {}", response.status).into());
    }

    let content = response.body;

    execute_write(state, user, local_path, &String::from_utf8_lossy(&content)).await?;

    trace!("DOWNLOAD successful -> {local_path}");
    Ok(local_path.to_string())
}
