//! Drive-related utilities

#[cfg(feature = "drive")]
pub async fn ensure_vendor_files_in_minio(drive: &crate::drive::s3_repository::S3Repository) {
    use log::{info, warn};

    if let Err(e) = drive.create_bucket_if_not_exists("default.gborg").await {
        warn!("Failed to ensure bucket default.gborg exists: {}", e);
        return;
    }

    let htmx_paths = [
        "./botui/ui/suite/js/vendor/htmx.min.js",
        "../botui/ui/suite/js/vendor/htmx.min.js",
    ];

    let htmx_content = htmx_paths.iter().find_map(|path| std::fs::read(path).ok());

    if let Some(content) = htmx_content {
        let key = "default.gbai/default.gblib/vendor/htmx.min.js";
        match drive.put_object_direct("default.gborg", key, content, Some("application/javascript")).await {
            Ok(_) => info!("Uploaded vendor file to MinIO: s3://default.gborg/{}", key),
            Err(e) => warn!("Failed to upload vendor file to MinIO: {}", e),
        }
    } else {
        warn!("Could not find htmx.min.js in botui, skipping vendor upload");
    }

    upload_bot_files_to_drive(drive).await;
}

#[cfg(feature = "drive")]
async fn upload_bot_files_to_drive(drive: &crate::drive::s3_repository::S3Repository) {
    use log::{info, warn, debug};
    use std::path::Path;

    // Tenta localizar o diretorio de templates em varios locais relativos
    let candidate_paths = [
        Path::new("../bottemplates/bots/core/default.gbai"),
        Path::new("./bottemplates/bots/core/default.gbai"),
        Path::new("bottemplates/bots/core/default.gbai"),
        Path::new("/opt/gbo/data/default.gbai"),
    ];
    let source_dir = candidate_paths.iter().find(|p| p.exists());

    let Some(source) = source_dir else {
        warn!("No template directory or bot files found, skipping bot file upload");
        return;
    };

    // Tenta criar /opt/gbo/data/ como fallback (best-effort) para que
    // o drive_monitor tambem encontre localmente se o diretorio existir
    let bot_dir = Path::new("/opt/gbo/data/default.gbai");
    if !bot_dir.exists() && source.to_str() != Some("/opt/gbo/data/default.gbai") {
        if let Err(e) = copy_dir_recursive(source, bot_dir).await {
            debug!("Could not copy templates to {} (non-critical): {}", bot_dir.display(), e);
        }
    }

    info!("Uploading bot files to MinIO from {:?}", source);

    let mut count = 0u32;
    let mut stack = vec![source.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let mut read_dir = match tokio::fs::read_dir(&dir).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to read dir {}: {}", dir.display(), e);
                continue;
            }
        };
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let rel = path.strip_prefix(source).unwrap_or(&path);
                let key = format!("default.gbai/{}", rel.to_str().unwrap_or(""));
                let data = match tokio::fs::read(&path).await {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Failed to read {}: {}", path.display(), e);
                        continue;
                    }
                };
                match drive.put_object_direct("default.gborg", &key, data, None).await {
                    Ok(_) => {
                        count += 1;
                        debug!("Uploaded bot file: s3://default.gborg/{}", key);
                    }
                    Err(e) => warn!("Failed to upload {}: {}", key, e),
                }
            }
        }
    }

    if count > 0 {
        info!("Uploaded {} bot files to MinIO: s3://default.gbai/", count);
    }
}

#[cfg(feature = "drive")]
async fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use tokio::fs;

    if !dst.exists() {
        fs::create_dir_all(dst).await?;
    }
    let mut read_dir = fs::read_dir(src).await?;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let entry_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if entry_path.is_dir() {
            Box::pin(copy_dir_recursive(&entry_path, &dest_path)).await?;
        } else if entry_path.is_file() {
            fs::copy(&entry_path, &dest_path).await?;
        }
    }
    Ok(())
}
