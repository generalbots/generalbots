/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
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

use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use flate2::read::GzDecoder;
use log::trace;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tar::Archive;
use uuid::Uuid;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use crate::keywords::file_ops::basic_io::execute_read;

const MAX_EXTRACT_ENTRIES: usize = 10_000;
const MAX_EXTRACT_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_EXTRACT_ENTRY_BYTES: u64 = 256 * 1024 * 1024;

pub async fn execute_compress(
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    files: &[String],
    archive_name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let loc = {
        let mut db_conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
        botbasic_core::utils::bot_drive_location_for(&mut db_conn, user.bot_id)
    };

    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(safe_archive_filename(archive_name)?);
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
    let client = state.drive_repository().ok_or("S3 client not configured")?;
    let bucket_name = loc.bucket.clone();
    let key = loc.key_for(archive_name);

    client
        .put_object(&bucket_name, &key, archive_content, None)
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

fn safe_archive_filename(name: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err("Invalid archive name".into());
    }
    let path = Path::new(trimmed);
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid archive name")?;
    if file_name == "." || file_name == ".." {
        return Err("Invalid archive name".into());
    }
    if file_name.chars().any(|c| matches!(c, '/' | '\\')) {
        return Err("Invalid archive name".into());
    }
    Ok(file_name.to_string())
}

fn validate_relative_path(input: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    if input.is_empty() || input.contains('\0') {
        return Err("Invalid path".into());
    }
    let path = Path::new(input);
    if path.is_absolute() {
        return Err("Absolute paths are not allowed".into());
    }
    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let s = part.to_str().ok_or("Invalid path")?;
                if s.is_empty() || s == "." || s == ".." {
                    return Err("Invalid path".into());
                }
                parts.push(s.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("Path traversal is not allowed".into());
            }
        }
    }
    if parts.is_empty() {
        return Err("Invalid path".into());
    }
    Ok(parts.join("/"))
}

fn join_destination(
    destination: &str,
    entry_name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let safe_entry = validate_relative_path(entry_name)?;
    let dest = destination.trim().trim_matches('/');
    if dest.is_empty() {
        return Ok(safe_entry);
    }
    let safe_dest = validate_relative_path(dest)?;
    Ok(format!("{safe_dest}/{safe_entry}"))
}

fn read_limited<R: Read>(
    reader: &mut R,
    max_bytes: u64,
    remaining_total: u64,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let limit = max_bytes.min(remaining_total);
    let mut buf = Vec::new();
    let mut limited = reader.take(limit.saturating_add(1));
    limited.read_to_end(&mut buf)?;
    if (buf.len() as u64) > limit {
        return Err("Archive entry exceeds allowed size".into());
    }
    Ok(buf)
}

struct TempExtractDir {
    path: PathBuf,
}

impl TempExtractDir {
    fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let path = std::env::temp_dir().join(format!("gb-extract-{}", Uuid::new_v4()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempExtractDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub async fn execute_extract(
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    archive: &str,
    destination: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = state.drive_repository().ok_or("S3 client not configured")?;

    let loc = {
        let mut db_conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
        botbasic_core::utils::bot_drive_location_for(&mut db_conn, user.bot_id)
    };
    let bucket_name = loc.bucket.clone();
    let safe_archive = safe_archive_filename(archive)?;
    let archive_key = loc.key_for(archive);

    let data = client
        .get_object(&bucket_name, &archive_key)
        .await
        .map_err(|e| format!("S3 get failed: {e}"))?;

    let temp_dir = TempExtractDir::new()?;
    let archive_path = temp_dir.path.join(&safe_archive);
    fs::write(&archive_path, &data)?;

    let mut extracted_files = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut entries: usize = 0;

    if has_zip_extension(archive) {
        let file = File::open(&archive_path)?;
        let mut zip = ZipArchive::new(file)?;

        for i in 0..zip.len() {
            if entries >= MAX_EXTRACT_ENTRIES {
                return Err("Archive contains too many entries".into());
            }
            let mut zip_file = zip.by_index(i)?;
            if zip_file.is_dir() {
                continue;
            }
            let name = zip_file
                .name()
                .map_err(|e| format!("Invalid ZIP entry name: {e}"))?
                .to_string();
            let dest_path = join_destination(destination, &name)?;
            let remaining = MAX_EXTRACT_TOTAL_BYTES.saturating_sub(total_bytes);
            if remaining == 0 {
                return Err("Archive exceeds allowed total size".into());
            }
            let content = read_limited(&mut zip_file, MAX_EXTRACT_ENTRY_BYTES, remaining)?;
            total_bytes = total_bytes.saturating_add(content.len() as u64);
            entries += 1;

            let dest_key = loc.key_for(&dest_path);
            client
                .put_object(&bucket_name, &dest_key, content, None)
                .await
                .map_err(|e| format!("S3 put failed: {e}"))?;

            extracted_files.push(dest_path);
        }
    } else if has_tar_gz_extension(archive) {
        let file = File::open(&archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut tar = Archive::new(decoder);

        for entry in tar.entries()? {
            if entries >= MAX_EXTRACT_ENTRIES {
                return Err("Archive contains too many entries".into());
            }
            let mut entry = entry?;
            let header = entry.header();
            let entry_type = header.entry_type();
            if entry_type.is_dir() {
                continue;
            }
            if !(entry_type.is_file() || entry_type.is_contiguous()) {
                continue;
            }
            let file_name = entry.path()?.to_string_lossy().to_string();
            let dest_path = join_destination(destination, &file_name)?;
            let remaining = MAX_EXTRACT_TOTAL_BYTES.saturating_sub(total_bytes);
            if remaining == 0 {
                return Err("Archive exceeds allowed total size".into());
            }
            let content = read_limited(&mut entry, MAX_EXTRACT_ENTRY_BYTES, remaining)?;
            total_bytes = total_bytes.saturating_add(content.len() as u64);
            entries += 1;

            let dest_key = loc.key_for(&dest_path);
            client
                .put_object(&bucket_name, &dest_key, content, None)
                .await
                .map_err(|e| format!("S3 put failed: {e}"))?;

            extracted_files.push(dest_path);
        }
    } else {
        return Err("Unsupported archive format".into());
    }

    trace!("EXTRACT successful: {} files", extracted_files.len());
    Ok(extracted_files)
}
