use base64::Engine;
use crate::export::{
    export_to_csv, export_to_html, export_to_json, export_to_markdown, export_to_ods,
    export_to_pdf_data, export_to_xlsx,
};
use crate::auth::{resolve_user_id, SheetUser};
use crate::state::{
    delete_sheet_from_drive, list_sheets_from_drive,
    save_sheet_to_drive, SheetState,
};
use crate::storage::import::{import_spreadsheet_bytes, parse_csv_to_worksheets, parse_xlsx_to_worksheets};
use crate::storage::import::create_new_spreadsheet;
use crate::types::{
    ExportRequest, LoadFromDriveRequest, LoadQuery, SaveRequest,
    SaveResponse, SearchQuery, ShareRequest, Spreadsheet, SpreadsheetMetadata,
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_new_sheet(
    State(_state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    Ok(Json(create_new_spreadsheet(&user_id)))
}

pub async fn handle_list_sheets(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
) -> Result<Json<Vec<SpreadsheetMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

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
    let user_id = resolve_user_id(user.as_deref());

    let sheets = list_sheets_from_drive(&state, &user_id).await.unwrap_or_default();

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
    Query(query): Query<LoadQuery>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

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
    botsheet_core::engine::formats::apply_formats_to_sheet(&mut sheet);
    Ok(Json(sheet))
}

pub async fn handle_load_from_drive(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<LoadFromDriveRequest>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not available" })),
        )
    })?;

    let bytes = drive.get_object(&req.bucket, &req.path).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("File not found: {e}") })),
        )
    })?;

    let mut ext = req.path.rsplit('.').next().unwrap_or("").to_lowercase();
    // Detect actual format from magic bytes for misnamed files
    if bytes.len() >= 4 && &bytes[..4] == b"PK\x03\x04" && ext != "ods" && ext != "xlsx" && ext != "xls" {
        ext = "xlsx".to_string();
    }
    let file_name = req.path.rsplit('/').next().unwrap_or("Spreadsheet");
    let sheet_name = file_name
        .rsplit('.')
        .next_back()
        .unwrap_or("Spreadsheet")
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
        "xlsx" | "xls" | "ods" | "xlsb" | "xlsm" => {
            parse_xlsx_to_worksheets(&bytes, &ext).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
            })?
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: .{ext}") })),
            ));
        }
    };

    let user_id = resolve_user_id(user.as_deref());
    let sheet_id = Uuid::new_v4().to_string();
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
        // Retain the original package so the save-back hook can merge the
        // edited cells into the untouched original instead of regenerating
        // the workbook from the lossy model (#788).
        source_bytes: if ext == "xlsx" || ext == "xlsm" {
            Some(bytes.clone())
        } else {
            None
        },
        acl: HashMap::new(),
    };

    // Persist to Drive so subsequent /api/sheet/range calls find the data
    let _ = save_sheet_to_drive(&state, &user_id, &sheet).await;

    Ok(Json(sheet))
}

pub async fn handle_save_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    let action = req.action.as_deref().unwrap_or("save");

    if action == "rename" {
        let sheet_id = req.id.as_deref().ok_or_else(|| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "rename requires a sheet id" })),
            )
        })?;
        let name = req.name.as_deref().ok_or_else(|| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "rename requires a name" })),
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
        {
            let mut sheet = session.sheet.write().await;
            sheet.name = name.to_string();
            sheet.updated_at = Utc::now();
        }
        state.sessions.record(
            &session,
            &state,
            &user_id,
            "rename",
            serde_json::json!({ "sheet_id": sheet_id, "name": name }),
        );
        return Ok(Json(SaveResponse {
            id: sheet_id.to_string(),
            success: true,
            message: Some("Sheet renamed".to_string()),
        }));
    }

    if req.worksheets.is_empty() {
        // No full snapshot: flush the live session (which already holds the
        // authoritative cell state), or create a blank document. This keeps
        // the client from sending a lossy viewport-only dump over the wire.
        if let Some(sheet_id) = req.id.as_deref() {
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
            {
                let mut sheet = session.sheet.write().await;
                if let Some(name) = req.name.as_deref() {
                    sheet.name = name.to_string();
                }
                sheet.updated_at = Utc::now();
            }
            if let Err(e) = session.persist_now(&state).await {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                ));
            }
            return Ok(Json(SaveResponse {
                id: sheet_id.to_string(),
                success: true,
                message: Some("Sheet saved".to_string()),
            }));
        }

        let mut sheet = create_new_spreadsheet(&user_id);
        sheet.name = req.name.unwrap_or_else(|| "Untitled Spreadsheet".to_string());
        sheet.updated_at = Utc::now();
        let sheet_id = sheet.id.clone();
        if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ));
        }
        return Ok(Json(SaveResponse {
            id: sheet_id,
            success: true,
            message: Some("Sheet saved".to_string()),
        }));
    }

    let sheet_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let sheet = Spreadsheet {
        id: sheet_id.clone(),
        name: req.name.unwrap_or_else(|| "Untitled Spreadsheet".to_string()),
        owner_id: user_id.clone(),
        worksheets: req.worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
    source_bucket: None,
    source_path: None,
        source_bytes: None,
    acl: HashMap::new(),
    };

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: sheet_id,
        success: true,
        message: Some("Sheet saved successfully".to_string()),
    }))
}

pub async fn handle_delete_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<LoadQuery>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    if let Err(e) = delete_sheet_from_drive(&state, &user_id, &req.id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }
    if let Some(id) = req.id.as_ref() {
        state.sessions.close(&state, id).await;
    }

    Ok(Json(SaveResponse {
        id: req.id.unwrap_or_default(),
        success: true,
        message: Some("Sheet deleted".to_string()),
    }))
}

pub async fn handle_get_sheet_by_id(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Path(sheet_id): Path<String>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;
    let mut sheet = session.sheet.read().await.clone();
    botsheet_core::engine::formats::apply_formats_to_sheet(&mut sheet);
    Ok(Json(sheet))
}

/// Returns the ops recorded after `since` (0 = all), plus the current version
/// so a reconnecting client can converge with the session state (#789, #791).
pub async fn handle_sheet_ops(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Path(sheet_id): Path<String>,
    Query(q): Query<OpsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let version = session.version.load(std::sync::atomic::Ordering::SeqCst);
    Ok(Json(serde_json::json!({
        "version": version,
        "ops": session.ops_since(q.since),
    })))
}

#[derive(serde::Deserialize, Default)]
pub struct OpsQuery {
    #[serde(default)]
    pub since: u64,
}

pub async fn handle_share_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ShareRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let mut sheet = session.sheet.write().await;
    if sheet.owner_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only the sheet owner can share it" })),
        ));
    }

    // Share by explicit user id when provided, otherwise key the grant by the
    // email so a future email-to-id resolution can migrate it (#789).
    let grant_key = req.user_id.unwrap_or_else(|| req.email.clone());
    sheet.acl.insert(grant_key, req.permission.clone());

    sheet.updated_at = Utc::now();
    state.sessions.record(&session, &state, &user_id, "share", serde_json::json!({ "sheet_id": req.sheet_id, "email": req.email, "permission": req.permission }));

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some(format!("Shared with {} as {}", req.email, req.permission)),
    }))
}

pub async fn handle_export_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;
    let sheet = session.sheet.read().await.clone();

    match req.format.as_str() {
        "csv" => {
            let csv = export_to_csv(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/csv")], csv))
        }
        "xlsx" => {
            let xlsx = export_to_xlsx(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                )],
                xlsx,
            ))
        }
        "json" => {
            let json = export_to_json(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json))
        }
        "html" => {
            let html = export_to_html(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], html))
        }
        "ods" => {
            let ods = export_to_ods(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.oasis.opendocument.spreadsheet",
                )],
                ods,
            ))
        }
        "md" | "markdown" => {
            let md = export_to_markdown(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], md))
        }
        "pdf" => {
            let pdf = export_to_pdf_data(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&pdf);
            Ok((
                [(axum::http::header::CONTENT_TYPE, "application/pdf")],
                encoded,
            ))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported format" })),
        )),
    }
}

pub async fn handle_import_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.xlsx".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("import.xlsx").to_string();
            if let Ok(bytes) = field.bytes().await {
                file_bytes = Some(bytes.to_vec());
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    let mut sheet = import_spreadsheet_bytes(&bytes, &filename, &user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    let user_id = resolve_user_id(user.as_deref());
    sheet.owner_id = user_id.clone();

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(sheet))
}
