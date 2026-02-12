//! Drive-related utilities

use log::{info, warn};

#[cfg(feature = "drive")]
pub async fn ensure_vendor_files_in_minio(drive: &aws_sdk_s3::Client) {
    use aws_sdk_s3::primitives::ByteStream;

    let htmx_paths = [
        "./botui/ui/suite/js/vendor/htmx.min.js",
        "../botui/ui/suite/js/vendor/htmx.min.js",
    ];

    let htmx_content = htmx_paths.iter().find_map(|path| std::fs::read(path).ok());

    let Some(content) = htmx_content else {
        warn!("Could not find htmx.min.js in botui, skipping MinIO upload");
        return;
    };

    let bucket = "default.gbai";
    let key = "default.gblib/vendor/htmx.min.js";

    match drive
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(content))
        .content_type("application/javascript")
        .send()
        .await
    {
        Ok(_) => info!("Uploaded vendor file to MinIO: s3://{}/{}", bucket, key),
        Err(e) => warn!("Failed to upload vendor file to MinIO: {}", e),
    }
}
