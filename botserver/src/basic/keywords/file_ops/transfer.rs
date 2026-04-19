/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use crate::core::shared::models::schema::bots::dsl::*;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{error, trace};
use std::error::Error;

use super::basic_io::execute_write;

pub struct FileData {
    pub content: Vec<u8>,
    pub filename: String,
}

pub async fn execute_upload(
    state: &AppState,
    user: &UserSession,
    file_data: FileData,
    destination: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{destination}");

    let content_disposition = format!("attachment; filename=\"{}\"", file_data.filename);

    trace!(
        "Uploading file '{}' to {bucket_name}/{key} ({} bytes)",
        file_data.filename,
        file_data.content.len()
    );

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .content_disposition(&content_disposition)
        .body(file_data.content.into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    let url = format!("s3://{bucket_name}/{key}");
    trace!(
        "UPLOAD successful: {url} (original filename: {})",
        file_data.filename
    );
    Ok(url)
}

pub async fn execute_download(
    state: &AppState,
    user: &UserSession,
    url: &str,
    local_path: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    let content = response.bytes().await?;

    execute_write(state, user, local_path, &String::from_utf8_lossy(&content)).await?;

    trace!("DOWNLOAD successful: {url} -> {local_path}");
    Ok(local_path.to_string())
}
