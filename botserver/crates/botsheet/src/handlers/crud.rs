use crate::auth::{resolve_user_id, SheetUser};
use crate::handlers::admission::admit;
use crate::state::{
    delete_sheet_from_drive, list_sheets_from_drive, load_sheet_from_drive, persist_sheet_to_drive,
    save_sheet_to_drive, SheetState,
};
use crate::storage::import::{
    create_new_spreadsheet, parse_csv_to_worksheets, parse_xlsx_to_worksheets,
};
use crate::storage::retains_snapshot;
use crate::types::{
    LoadFromDriveRequest, LoadQuery, SaveRequest, SaveResponse, SearchQuery, ShareRequest,
    Spreadsheet, SpreadsheetMetadata,
};
use crate::ui_fragments::Lang;
use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use botcore::WorkClass;
use chrono::Utc;
use sha2::Digest;

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// #2530 — cheap estimate of a full-snapshot JSON save: each cell counts as
/// 128 bytes (the same convention as `sheet_payload_bytes` in the admission
/// bridge) plus the literal name. Measuring the request by re-serializing it
/// would allocate the buffer the gate exists to bound, so we never do that.
pub(crate) fn save_payload_bytes(req: &SaveRequest) -> u64 {
    let cells: usize = req.worksheets.iter().map(|sheet| sheet.data.len()).sum();
    let name_bytes = req.name.as_deref().map_or(0, str::len);
    (cells as u64)
        .saturating_mul(128)
        .saturating_add(name_bytes as u64)
}

pub async fn handle_new_sheet(
    State(_state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;
    Ok(Json(create_new_spreadsheet(&user_id)))
}

pub async fn handle_list_sheets(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
) -> Result<Json<Vec<SpreadsheetMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;

    match list_sheets_from_drive(&state, &user_id).await {
        Ok(sheets) => Ok(Json(sheets)),
        Err(e) => {
            log::error!("Failed to list sheets: {e}");
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_sheets(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SpreadsheetMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;

    let sheets = match list_sheets_from_drive(&state, &user_id).await {
        Ok(sheets) => sheets,
        Err(e) => {
            log::warn!("Failed to search sheets in drive: {e}");
            Vec::new()
        }
    };

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        sheets
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        sheets
    };

    Ok(Json(filtered))
}

pub async fn handle_load_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    headers: HeaderMap,
    Query(query): Query<LoadQuery>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;

    let sheet_id = query.id.as_deref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Sheet ID is required" })),
        )
    })?;

    let session = state
        .sessions
        .get_or_load(&state, &user_id, sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let mut sheet = session.sheet.read().await.clone();
    botsheet_core::engine::formats::apply_formats_to_sheet_locale(
        &mut sheet,
        Lang::from_headers(&headers).number_locale(),
    );
    Ok(Json(sheet))
}

pub async fn handle_load_from_drive(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    headers: HeaderMap,
    Json(req): Json<LoadFromDriveRequest>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not available" })),
        )
    })?;

    let bytes = drive
        .get_object(&req.bucket, &req.path)
        .await
        .map_err(|e| {
            // The drive error names the bucket and object path — log it and answer
            // with the lookup result only.
            log::warn!(
                "sheet load: drive read of {}/{} failed: {}",
                req.bucket,
                req.path,
                e
            );
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "File not found" })),
            )
        })?;

    let mut ext = req.path.rsplit('.').next().unwrap_or("").to_lowercase();
    // Detect actual format from magic bytes for misnamed files
    if bytes.len() >= 4
        && &bytes[..4] == b"PK\x03\x04"
        && ext != "ods"
        && ext != "xlsx"
        && ext != "xls"
    {
        ext = "xlsx".to_string();
    }
    let file_name = req.path.rsplit('/').next().unwrap_or("Spreadsheet");
    // `report.xlsx` -> "report"; `rsplit('.').next_back()` returned the
    // extension ("xlsx"), which then became the workbook and CSV tab name.
    let sheet_name = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name)
        .to_string();

    let worksheets = match ext.as_str() {
        "csv" | "tsv" => {
            let delimiter = if ext == "tsv" { b'\t' } else { b',' };
            parse_csv_to_worksheets(&bytes, delimiter, &sheet_name).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
            })?
        }
        "xlsx" | "xls" | "ods" | "xlsb" | "xlsm" => parse_xlsx_to_worksheets(&bytes, &ext)
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
            })?,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: .{ext}") })),
            ));
        }
    };

    let user_id = resolve_user_id(user.as_deref())?;
    // Deterministic working-copy id: SHA-256(bucket|path) truncated to a v4
    // UUID, so reopening the same source reuses its prior edits.
    let digest = sha2::Sha256::digest(format!("{}|{}", req.bucket, req.path).as_bytes());
    let mut id_bytes = [0u8; 16];
    id_bytes.copy_from_slice(&digest[..16]);
    let sheet_id = Uuid::from_bytes(id_bytes).to_string();

    // Reuse an existing working copy (with any prior edits) when present.
    let locale = Lang::from_headers(&headers).number_locale();
    if let Ok(mut existing) = load_sheet_from_drive(&state, &user_id, &Some(sheet_id.clone())).await
    {
        botsheet_core::engine::formats::apply_formats_to_sheet_locale(&mut existing, locale);
        return Ok(Json(existing));
    }

    let sheet = Spreadsheet {
        id: sheet_id.clone(),
        name: sheet_name,
        owner_id: user_id.clone(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
        source_bucket: Some(req.bucket.clone()),
        source_path: Some(req.path.clone()),
        // Retain the original package for the save-back merge (#788).
        // #2043 — keep it for .ods/.xls too: the snapshot is what makes the
        // loss explicit (a non-OOXML package cannot be patched, so the save
        // writes a named sibling instead of overwriting the source), and a
        // future format-aware patcher can use it. `csv`/`tsv` have no package.
        source_bytes: if retains_snapshot(&ext) {
            Some(bytes.clone())
        } else {
            None
        },
        acl: HashMap::new(),
    };

    // Persist (no export hook — opening must not rewrite the source).
    let _ = persist_sheet_to_drive(&state, &user_id, &sheet).await;

    // Return a display-formatted clone; the persisted model keeps raw values.
    let mut response = sheet.clone();
    botsheet_core::engine::formats::apply_formats_to_sheet_locale(&mut response, locale);
    Ok(Json(response))
}

mod handle_save_sheet;
pub use handle_save_sheet::*;

#[cfg(test)]
mod tests {
    use super::*;
    use botcore::{CgroupMemory, CgroupSource, MemoryAdmission, MemoryPolicy};
    use botsheet_core::types::Worksheet;

    const MIB: u64 = 1024 * 1024;

    fn request(cells: usize, name: &str) -> SaveRequest {
        let mut worksheet = Worksheet::default();
        for index in 0..cells {
            worksheet
                .data
                .insert(format!("A{index}"), Default::default());
        }
        SaveRequest {
            id: None,
            name: Some(name.to_string()),
            worksheets: vec![worksheet],
            action: None,
        }
    }

    #[test]
    fn save_payload_counts_cells_and_name() {
        assert_eq!(save_payload_bytes(&request(10, "abc")), 10 * 128 + 3);
        assert_eq!(save_payload_bytes(&request(0, "")), 0);
    }

    #[test]
    fn json_save_over_the_eight_mib_cap_is_a_typed_413() {
        let ceiling = CgroupMemory {
            limit_bytes: 4096 * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        let gate = MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling));
        let payload = save_payload_bytes(&request(70_000, "salaries"));
        let result = gate.try_admit(WorkClass::OfficeJsonSave, payload);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(error.code(), "office_payload_too_large");
            let body = error.json_body().to_string();
            assert!(!body.contains("salaries"));
        }
    }

    #[test]
    fn json_save_under_the_cap_is_admitted() {
        let ceiling = CgroupMemory {
            limit_bytes: 4096 * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        let gate = MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling));
        let payload = save_payload_bytes(&request(10, "Budget"));
        let result = gate.try_admit(WorkClass::OfficeJsonSave, payload);
        assert!(result.is_ok());
        if let Ok(permit) = result {
            assert_eq!(permit.class(), WorkClass::OfficeJsonSave);
        }
    }
}
