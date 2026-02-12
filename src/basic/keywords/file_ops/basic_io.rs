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

use crate::core::shared::models::schema::bots::dsl::*;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{error, trace};
use std::error::Error;

pub async fn execute_read(
    state: &AppState,
    user: &UserSession,
    path: &str,
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
    let key = format!("{bot_name}.gbdrive/{path}");

    let response = client
        .get_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("S3 get failed: {e}"))?;

    let data = response.body.collect().await?.into_bytes();
    let content =
        String::from_utf8(data.to_vec()).map_err(|_| "File content is not valid UTF-8")?;

    trace!("READ successful: {} bytes", content.len());
    Ok(content)
}

pub async fn execute_write(
    state: &AppState,
    user: &UserSession,
    path: &str,
    content: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    let key = format!("{bot_name}.gbdrive/{path}");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(content.as_bytes().to_vec().into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    trace!("WRITE successful: {} bytes to {path}", content.len());
    Ok(())
}

pub async fn execute_delete_file(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    let key = format!("{bot_name}.gbdrive/{path}");

    client
        .delete_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("S3 delete failed: {e}"))?;

    trace!("DELETE_FILE successful: {path}");
    Ok(())
}

pub async fn execute_list(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
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
    let prefix = format!("{bot_name}.gbdrive/{path}");

    let response = client
        .list_objects_v2()
        .bucket(&bucket_name)
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("S3 list failed: {e}"))?;

    let files: Vec<String> = response
        .contents()
        .iter()
        .filter_map(|obj| {
            obj.key().map(|k| {
                k.strip_prefix(&format!("{bot_name}.gbdrive/"))
                    .unwrap_or(k)
                    .to_string()
            })
        })
        .collect();

    trace!("LIST successful: {} files", files.len());
    Ok(files)
}
