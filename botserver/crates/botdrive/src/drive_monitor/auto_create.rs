pub async fn ensure_bot_exists(
    _bot_name: &str,
) -> Result<bool, String> {
    // TODO(#506): Query bot table, INSERT if not exists
    Ok(false)
}

pub async fn sync_bots_from_buckets() -> Result<u32, String> {
    // TODO(#506): List .gbai buckets in MinIO, call ensure_bot_exists for each
    Ok(0)
}
