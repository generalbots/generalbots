/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
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

use crate::basic::keywords::use_account::{
    get_account_credentials, is_account_path, parse_account_path,
};
use crate::core::shared::models::schema::bots::dsl::*;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::trace;
use std::error::Error;

use super::basic_io::execute_delete_file;

pub async fn execute_copy(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let source_is_account = is_account_path(source);
    let dest_is_account = is_account_path(destination);

    if source_is_account || dest_is_account {
        return execute_copy_with_account(state, user, source, destination).await;
    }

    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                log::error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let source_key = format!("{bot_name}.gbdrive/{source}");
    let dest_key = format!("{bot_name}.gbdrive/{destination}");

    let copy_source = format!("{bucket_name}/{source_key}");

    client
        .copy_object()
        .bucket(&bucket_name)
        .key(&dest_key)
        .copy_source(&copy_source)
        .send()
        .await
        .map_err(|e| format!("S3 copy failed: {e}"))?;

    trace!("COPY successful: {source} -> {destination}");
    Ok(())
}

pub async fn execute_copy_with_account(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let source_is_account = is_account_path(source);
    let dest_is_account = is_account_path(destination);

    let content = if source_is_account {
        let (email, path) = parse_account_path(source).ok_or("Invalid account:// path format")?;
        let creds = get_account_credentials(&state.conn, &email, user.bot_id)
            .await
            .map_err(|e| format!("Failed to get credentials: {e}"))?;
        download_from_account(&creds, &path).await?
    } else {
        read_from_local(state, user, source).await?
    };

    if dest_is_account {
        let (email, path) =
            parse_account_path(destination).ok_or("Invalid account:// path format")?;
        let creds = get_account_credentials(&state.conn, &email, user.bot_id)
            .await
            .map_err(|e| format!("Failed to get credentials: {e}"))?;
        upload_to_account(&creds, &path, &content).await?;
    } else {
        write_to_local(state, user, destination, &content).await?;
    }

    trace!("COPY with account successful: {source} -> {destination}");
    Ok(())
}

pub async fn download_from_account(
    creds: &crate::basic::keywords::use_account::AccountCredentials,
    path: &str,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    match creds.provider.as_str() {
        "gmail" | "google" => {
            let url = format!(
                "https://www.googleapis.com/drive/v3/files/{}?alt=media",
                urlencoding::encode(path)
            );
            let resp = client
                .get(&url)
                .bearer_auth(&creds.access_token)
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("Google Drive download failed: {}", resp.status()).into());
            }
            Ok(resp.bytes().await?.to_vec())
        }
        "outlook" | "microsoft" => {
            let url = format!(
                "https://graph.microsoft.com/v1.0/me/drive/root:/{}:/content",
                urlencoding::encode(path)
            );
            let resp = client
                .get(&url)
                .bearer_auth(&creds.access_token)
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("OneDrive download failed: {}", resp.status()).into());
            }
            Ok(resp.bytes().await?.to_vec())
        }
        _ => Err(format!("Unsupported provider: {}", creds.provider).into()),
    }
}

pub async fn upload_to_account(
    creds: &crate::basic::keywords::use_account::AccountCredentials,
    path: &str,
    content: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    match creds.provider.as_str() {
        "gmail" | "google" => {
            let url = format!(
                "https://www.googleapis.com/upload/drive/v3/files?uploadType=media&name={}",
                urlencoding::encode(path)
            );
            let resp = client
                .post(&url)
                .bearer_auth(&creds.access_token)
                .body(content.to_vec())
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("Google Drive upload failed: {}", resp.status()).into());
            }
        }
        "outlook" | "microsoft" => {
            let url = format!(
                "https://graph.microsoft.com/v1.0/me/drive/root:/{}:/content",
                urlencoding::encode(path)
            );
            let resp = client
                .put(&url)
                .bearer_auth(&creds.access_token)
                .body(content.to_vec())
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("OneDrive upload failed: {}", resp.status()).into());
            }
        }
        _ => return Err(format!("Unsupported provider: {}", creds.provider).into()),
    }
    Ok(())
}

pub async fn read_from_local(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bot_name: String = {
        let mut db_conn = state.conn.get()?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };
    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    let result = client
        .get_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await?;
    let bytes = result.body.collect().await?.into_bytes();
    Ok(bytes.to_vec())
}

pub async fn write_to_local(
    state: &AppState,
    user: &UserSession,
    path: &str,
    content: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bot_name: String = {
        let mut db_conn = state.conn.get()?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };
    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(content.to_vec().into())
        .send()
        .await?;
    Ok(())
}

pub async fn execute_move(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    execute_copy(state, user, source, destination).await?;

    execute_delete_file(state, user, source).await?;

    trace!("MOVE successful: {source} -> {destination}");
    Ok(())
}
