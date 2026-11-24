use super::super::*;

/// Drive location for the bot a catalog command runs as.
///
/// The bucket cannot be derived from the bot name: an org-hosted bot lives in
/// `{slug}.gborg` with the bot directory as a key prefix, while a standalone
/// bot owns `{bot}.gbai`. Every `drive.*` command goes through this so the
/// chat side addresses exactly the Drive the UI and the file keywords use.
pub(in crate::core::bot::api_catalog) fn catalog_drive_location(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
) -> Result<botbasic_core::utils::BotDriveLocation, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    Ok(botbasic_core::utils::bot_drive_location_for(
        &mut conn, *bot_uuid,
    ))
}
/// Lists drive files under a folder (read-only, via the configured S3 repository).
pub(in crate::core::bot::api_catalog) async fn list_drive_files(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    path: Option<&str>,
    query: Option<&str>,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let loc = catalog_drive_location(state, bot_uuid)?;
    let bucket = loc.bucket.clone();
    let rel = normalize_drive_path(path.unwrap_or(""))?;
    let prefix = if rel.is_empty() {
        loc.key_prefix.clone()
    } else {
        loc.key_for(&rel)
    };
    let mut keys = drive
        .list_objects_with_metadata(&bucket, Some(&prefix))
        .await
        .map_err(|e| format!("drive list error: {e}"))?;
    if let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) {
        let needle = q.to_ascii_lowercase();
        keys.retain(|o| {
            let key = o.key.to_ascii_lowercase();
            let name = key.rsplit('/').next().unwrap_or(&key);
            needle
                .split_whitespace()
                .filter(|t| !t.is_empty())
                .all(|token| key.contains(token) || name.contains(token))
        });
    }
    keys.sort_by(|a, b| b.key.cmp(&a.key));
    let items: Vec<Value> = keys
        .into_iter()
        .take(100)
        .map(|o| {
            let ext = o.key.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
            let is_doc = matches!(
                ext.as_str(),
                "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "markdown" | "pdf" | "html" | "htm"
            );
            let file = urlencoding::encode(&o.key);
            let bucket_enc = urlencoding::encode(&bucket);
            json!({
                "path": o.key,
                "size": o.size,
                "etag": o.etag,
                "drive_link": format!("app://drive?path={file}&scope=bot&bucket={bucket_enc}"),
                "docs_link": if is_doc {
                    Value::String(format!("app://docs?file={file}&bucket={bucket_enc}"))
                } else {
                    Value::Null
                }
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "bucket": bucket, "files": items }))
}
#[cfg(all(feature = "banking", feature = "products", feature = "tax"))]
pub(in crate::core::bot::api_catalog) async fn execute_tax(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    service: Option<String>,
    value: Option<String>,
) -> Result<Value, String> {
    let scope = botbanking::cashflow::resolve_bot_scope(&state.conn, bot_uuid, None)
        .ok_or_else(|| "bot not found".to_string())?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let result = match service {
        Some(service) => {
            botproducts::service_tax::calculate_service_tax_inner(
                &mut conn,
                scope.branch_id,
                &service,
                None,
                value.as_deref(),
            )
            .await
        }
        None => calculate_anonymous_service_tax(&mut conn, &scope.branch_id, value.as_deref()),
    };
    match result {
        Ok(v) => Ok(v),
        // The inner calculation returns a stable catalogue key (#2094); the
        // chat path has no request locale, so the default one is used.
        Err((_code, key)) => Err(botcore::i18n::t(&botcore::i18n::Locale::default(), key)),
    }
}
#[cfg(not(all(feature = "banking", feature = "products", feature = "tax")))]
pub(in crate::core::bot::api_catalog) async fn execute_tax(
    _state: &Arc<AppState>,
    _bot_uuid: &Uuid,
    _service: Option<String>,
    _value: Option<String>,
) -> Result<Value, String> {
    Err("tax calculation requires banking, products and tax features".to_string())
}
#[cfg(all(feature = "banking", feature = "products", feature = "tax"))]
pub(in crate::core::bot::api_catalog) fn calculate_anonymous_service_tax(
    conn: &mut diesel::PgConnection,
    branch_id: &Uuid,
    value: Option<&str>,
) -> Result<serde_json::Value, (axum::http::StatusCode, &'static str)> {
    use rust_decimal::Decimal;
    use std::str::FromStr as _;
    let service_value = value
        .map(Decimal::from_str)
        .transpose()
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "error-invalid-value"))?
        .ok_or_else(|| (axum::http::StatusCode::BAD_REQUEST, "error-value-required"))?;
    let loaded = bottax::storage::load_rates_from_billing(conn, branch_id);
    let rates = loaded.unwrap_or(Default::default());
    let source = if loaded.is_some() {
        "billing_tax_rates"
    } else {
        "default"
    };
    let breakdown = bottax::calculator::calculate_service_tax(service_value, &rates);
    if let Err((_code, msg)) =
        bottax::handlers::persist_calculation(conn, branch_id, None, None, &breakdown, source)
    {
        log::warn!("service.tax audit insert skipped: {msg}");
    }
    Ok(serde_json::json!({
        "service": null,
        "service_value": breakdown.service_value.to_string(),
        "breakdown": breakdown,
        "rates": {
            "irpj_pct": rates.irpj_pct.to_string(),
            "csll_pct": rates.csll_pct.to_string(),
            "pis_cofins_pct": rates.pis_cofins_pct.to_string(),
            "iss_pct": rates.iss_pct.to_string(),
        },
        "rate_source": source,
    }))
}
pub(in crate::core::bot::api_catalog) async fn write_drive_file(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    path: &str,
    content_b64: &str,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let data = base64::engine::general_purpose::STANDARD
        .decode(content_b64)
        .map_err(|e| format!("invalid base64 content: {e}"))?;
    let loc = catalog_drive_location(state, bot_uuid)?;
    let bucket = loc.bucket.clone();
    let key = loc.key_for(&normalize_drive_path(path)?);
    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("api_call drive.write -> {bucket}/{key}");
    Ok(json!({ "success": true, "bucket": bucket, "path": key }))
}
/// Copies a file from one drive path to another (e.g. an attached invoice
/// from `inbox/` into its `faturas/<month>/` folder).
pub(in crate::core::bot::api_catalog) async fn move_drive_file(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    from: &str,
    to: &str,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let loc = catalog_drive_location(state, bot_uuid)?;
    let bucket = loc.bucket.clone();
    let from_key = loc.key_for(&normalize_drive_path(from)?);
    let to_key = loc.key_for(&normalize_drive_path(to)?);
    let bytes = drive
        .get_object(&bucket, &from_key)
        .await
        .map_err(|e| format!("not found at {from_key}: {e}"))?;
    drive
        .put_object(&bucket, &to_key, bytes, None)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("api_call drive.file -> {bucket}/{to_key}");
    Ok(json!({ "success": true, "bucket": bucket, "from": from_key, "to": to_key }))
}
