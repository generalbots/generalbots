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
use flate2::read::GzDecoder;
use log::{error, trace};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tar::Archive;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use super::basic_io::execute_read;

pub async fn execute_compress(
    state: &AppState,
    user: &UserSession,
    files: &[String],
    archive_name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
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

    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(archive_name);
    let file = File::create(&archive_path)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    for file_path in files {
        let content = execute_read(state, user, file_path).await?;
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);

        zip.start_file(file_name, options)?;
        zip.write_all(content.as_bytes())?;
    }

    zip.finish()?;

    let archive_content = fs::read(&archive_path)?;
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{archive_name}");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(archive_content.into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    fs::remove_file(&archive_path).ok();

    trace!("COMPRESS successful: {archive_name}");
    Ok(archive_name.to_string())
}

pub fn has_zip_extension(archive: &str) -> bool {
    Path::new(archive)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

pub fn has_tar_gz_extension(archive: &str) -> bool {
    let path = Path::new(archive);
    if let Some(ext) = path.extension() {
        if ext.eq_ignore_ascii_case("tgz") {
            return true;
        }
        if ext.eq_ignore_ascii_case("gz") {
            if let Some(stem) = path.file_stem() {
                return Path::new(stem)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("tar"));
            }
        }
    }
    false
}

pub async fn execute_extract(
    state: &AppState,
    user: &UserSession,
    archive: &str,
    destination: &str,
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
    let archive_key = format!("{bot_name}.gbdrive/{archive}");

    let response = client
        .get_object()
        .bucket(&bucket_name)
        .key(&archive_key)
        .send()
        .await
        .map_err(|e| format!("S3 get failed: {e}"))?;

    let data = response.body.collect().await?.into_bytes();

    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(archive);
    fs::write(&archive_path, &data)?;

    let mut extracted_files = Vec::new();

    if has_zip_extension(archive) {
        let file = File::open(&archive_path)?;
        let mut zip = ZipArchive::new(file)?;

        for i in 0..zip.len() {
            let mut zip_file = zip.by_index(i)?;
            let file_name = zip_file.name().to_string();

            let mut content = Vec::new();
            zip_file.read_to_end(&mut content)?;

            let dest_path = format!("{}/{file_name}", destination.trim_end_matches('/'));

            let dest_key = format!("{bot_name}.gbdrive/{dest_path}");
            client
                .put_object()
                .bucket(&bucket_name)
                .key(&dest_key)
                .body(content.into())
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {e}"))?;

            extracted_files.push(dest_path);
        }
    } else if has_tar_gz_extension(archive) {
        let file = File::open(&archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut tar = Archive::new(decoder);

        for entry in tar.entries()? {
            let mut entry = entry?;
            let file_name = entry.path()?.to_string_lossy().to_string();

            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;

            let dest_path = format!("{}/{file_name}", destination.trim_end_matches('/'));

            let dest_key = format!("{bot_name}.gbdrive/{dest_path}");
            client
                .put_object()
                .bucket(&bucket_name)
                .key(&dest_key)
                .body(content.into())
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {e}"))?;

            extracted_files.push(dest_path);
        }
    }

    fs::remove_file(&archive_path).ok();

    trace!("EXTRACT successful: {} files", extracted_files.len());
    Ok(extracted_files)
}
