use super::*;

pub(super) fn execute_get_from(
    pool: &botlib::db_pool::DbPool,
    table: &str,
    filter: &str,
    _bot_id: uuid::Uuid,
) -> Result<Value, String> {
    use botbasic_core::utils::parse_filter;

    let safe_table = sanitize_identifier(table);
    let (where_clause, params) = parse_filter(filter).map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT row_to_json(t)::text as row_data FROM (SELECT * FROM {safe_table} WHERE {where_clause}) t"
    );

    let mut conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<JsonRow> = if params.is_empty() {
        diesel::sql_query(&query)
            .load(&mut conn)
            .map_err(|e| format!("SQL error: {e}"))?
    } else {
        diesel::sql_query(&query)
            .bind::<Text, _>(&params[0])
            .load(&mut conn)
            .map_err(|e| format!("SQL error: {e}"))?
    };

    let results: Vec<Value> = rows
        .into_iter()
        .filter_map(|row| serde_json::from_str(&row.row_data).ok())
        .collect();

    Ok(serde_json::json!({
        "command": "get_from",
        "table": table,
        "filter": filter,
        "results": results,
        "count": results.len()
    }))
}
pub(super) fn is_safe_path(path: &str) -> bool {
    if path.starts_with("https://") || path.starts_with("http://") {
        // #2131 — replaces the ad-hoc host substring checks: the shared
        // validator covers encoded IPv4, IPv6, metadata names and ports.
        return validate_url_ssrf_strict(path).is_ok();
    }
    if path.contains("..") || path.starts_with('/') {
        return false;
    }
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return false;
    }
    if path.contains("//") || path.contains('~') || path.contains('*') || path.contains('?') {
        return false;
    }
    if !path.starts_with("http") {
        let path_obj = Path::new(path);
        if path_obj.components().count()
            != path_obj
                .components()
                .filter(|c| matches!(c, std::path::Component::Normal(_)))
                .count()
        {
            return false;
        }
    }
    true
}
pub async fn execute_get(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    // #2131 — `GET <url>` takes the URL straight from the bot script. The
    // shared guard is the only thing between that script and the internal
    // network: public http(s) only, per-hop redirect checks, capped body.
    let policy = FetchPolicy::default()
        .with_timeout(Duration::from_secs(30))
        .with_max_bytes(16 * 1024 * 1024);
    let response = fetch_user_url(url, &policy).await.map_err(|e| {
        error!(
            "GET: refused URL ({}): {}",
            e.kind().as_str(),
            e.detail()
        );
        Box::new(e) as Box<dyn Error + Send + Sync>
    })?;
    if !response.is_success() {
        error!(
            "HTTP request returned non-success status for GET: {}",
            response.status
        );
        return Err(format!("HTTP request failed with status: {}", response.status).into());
    }
    trace!(
        "Successfully executed GET request, content length: {}",
        response.body.len()
    );
    Ok(response.text())
}
#[cfg(feature = "drive")]
pub async fn get_from_bucket(
    state: Arc<dyn BasicRuntime>,
    file_path: &str,
    bot_id: uuid::Uuid,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    if !is_safe_path(file_path) {
        log::error!("Unsafe file path detected: {}", file_path);
        return Err("Invalid file path".into());
    }
    let drive_repo = state.drive_repository().ok_or("S3 client not configured")?;
    let client = drive_repo.as_ref();
    // `GET path` must address the bot's `.gbdrive` exactly like every other file
    // keyword (MOVE / CREATE FILE / the channel inbound stagers). The bucket is
    // resolved per bot: an org-hosted bot lives under `{slug}.gborg`, so the
    // object key carries the `{bot}.gbai/` prefix inside that shared bucket.
    let loc = {
        let mut db_conn = state.db_pool().get().map_err(|e| format!("DB error: {e}"))?;
        botbasic_core::utils::bot_drive_location_for(&mut db_conn, bot_id)
    };
    let bucket_name = loc.bucket.clone();
    // Callers that already pass a qualified key are normalized rather than
    // double-prefixed.
    let object_key = if file_path.starts_with(&loc.key_prefix) {
        file_path.to_string()
    } else if let Some(stripped) = file_path.strip_prefix("gbdrive/") {
        loc.key_for(stripped)
    } else {
        loc.key_for(file_path)
    };
    let bytes: Vec<u8> = match tokio::time::timeout(Duration::from_secs(30), async {
        client
            .get_object(&bucket_name, &object_key)
            .await
            .map_err(|e| format!("S3 operation failed: {}", e))
    })
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            log::error!("drive read failed for {bucket_name}/{object_key}: {e}");
            return Err(format!("S3 operation failed: {}", e).into());
        }
        Err(_) => {
            log::error!("drive read timed out");
            return Err("drive operation timed out".into());
        }
    };
    // Every supported document format is decoded here; PDFs and plain text in
    // memory, office formats through the shared document processor.
    let content = match crate::keywords::document_text::extract_document_text(file_path, bytes).await
    {
        Ok(content) => content,
        Err(e) => {
            log::error!("document extraction failed for {file_path}: {e}");
            return Err(e);
        }
    };
    trace!(
        "Successfully retrieved file from bucket: {}, content length: {}",
        file_path,
        content.len()
    );
    Ok(content)
}
#[cfg(not(feature = "drive"))]
pub async fn get_from_bucket(
    _state: Arc<dyn BasicRuntime>,
    _file_path: &str,
    _bot_id: uuid::Uuid,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    Err("S3 drive is not enabled. Configure MinIO to use this feature.".into())
}
