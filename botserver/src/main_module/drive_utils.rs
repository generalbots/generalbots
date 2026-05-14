//! Drive-related utilities

#[cfg(feature = "drive")]
pub async fn ensure_vendor_files_in_minio(drive: &crate::drive::s3_repository::S3Repository) {
    use log::{info, warn};

    if let Err(e) = drive.create_bucket_if_not_exists("default.gbai").await {
        warn!("Failed to ensure bucket default.gbai exists: {}", e);
        return;
    }

    let htmx_paths = [
        "./botui/ui/suite/js/vendor/htmx.min.js",
        "../botui/ui/suite/js/vendor/htmx.min.js",
    ];

    let htmx_content = htmx_paths.iter().find_map(|path| std::fs::read(path).ok());

    if let Some(content) = htmx_content {
        let key = "default.gblib/vendor/htmx.min.js";
        match drive.put_object_direct("default.gbai", key, content, Some("application/javascript")).await {
            Ok(_) => info!("Uploaded vendor file to MinIO: s3://default.gbai/{}", key),
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

    let bot_dir = Path::new("/opt/gbo/data/default.gbai");
    if !bot_dir.exists() {
        warn!("Bot directory not found: /opt/gbo/data/default.gbai, skipping bot file upload");
        return;
    }

    let mut count = 0u32;
    let mut stack = vec![bot_dir.to_path_buf()];

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
                let rel = path.strip_prefix(bot_dir).unwrap_or(&path);
                let key = rel.to_str().unwrap_or("");
                let data = match tokio::fs::read(&path).await {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Failed to read {}: {}", path.display(), e);
                        continue;
                    }
                };
                match drive.put_object_direct("default.gbai", key, data, None).await {
                    Ok(_) => {
                        count += 1;
                        debug!("Uploaded bot file: s3://default.gbai/{}", key);
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
