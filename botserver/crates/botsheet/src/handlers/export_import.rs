//! Export and import handlers (split from [`super::crud`]).

use base64::Engine;
use crate::auth::{resolve_user_id, SheetUser};
use crate::export::{
    export_to_csv, export_to_html, export_to_json, export_to_markdown, export_to_ods,
    export_to_pdf_data, export_to_pdf_data_with,
};
use crate::state::{save_sheet_to_drive, SheetState};
use crate::storage::import::import_spreadsheet_bytes;
use crate::types::{ExportRequest, Spreadsheet};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Html,
    Json,
};
use botsecurity_core::upload_policy::{self, UploadClass};
use std::sync::Arc;

/// One content-type header for the download arms; the xlsx arm adds its
/// fidelity marker on top. Returning a uniform `HeaderMap` keeps every arm of
/// the match on the same type (an array literal and a `HeaderMap` cannot be
/// unified by inference).
fn content_headers(content_type: &'static str) -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(content_type),
    );
    headers
}

pub async fn handle_export_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ExportRequest>,
) -> Result<(axum::http::HeaderMap, Vec<u8>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;

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
            Ok((content_headers("text/csv"), csv.into_bytes()))
        }
        "xlsx" => {
            // #2043 — export takes the same route as save-back: patch the
            // retained package when one exists so charts, comments,
            // conditional formats and images survive; rebuild only when there
            // is nothing to patch, and label the result so the loss is never
            // silent.
            let produced = crate::storage::produce_xlsx(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
                )
            })?;
            if let Some(reason) = produced.plan.regenerate_reason() {
                log::warn!(
                    "sheet export regenerated from the model: {}",
                    reason.description()
                );
            }
            let mut headers = content_headers(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            );
            headers.insert(
                "x-gb-xlsx-fidelity",
                axum::http::HeaderValue::from_static(produced.plan.as_str()),
            );
            headers.insert(
                "x-gb-xlsx-fidelity-reason",
                axum::http::HeaderValue::from_static(
                    produced
                        .plan
                        .regenerate_reason()
                        .map(crate::storage::RegenerateReason::as_str)
                        .unwrap_or("none"),
                ),
            );
            return Ok((headers, produced.bytes));
        }
        "json" => {
            let json = export_to_json(&sheet);
            Ok((content_headers("application/json"), json.into_bytes()))
        }
        "html" => {
            let html = export_to_html(&sheet);
            Ok((content_headers("text/html"), html.into_bytes()))
        }
        "ods" => {
            let ods = export_to_ods(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
                )
            })?;
            Ok((
                content_headers("application/vnd.oasis.opendocument.spreadsheet"),
                ods,
            ))
        }
        "md" | "markdown" => {
            let md = export_to_markdown(&sheet);
            Ok((content_headers("text/markdown"), md.into_bytes()))
        }
        "pdf" => {
            let pdf = export_to_pdf_data(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
                )
            })?;
            // Stream the bytes: the download UI saves the response body as a
            // file, so returning base64 here produced a corrupt PDF. The print
            // preview builds its own data URL from `export_to_pdf_data_with`.
            Ok((content_headers("application/pdf"), pdf))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported format" })),
        )),
    }
}

/// Print preview as an HTML modal with the PDF embedded as a data URL: the
/// export endpoint streams bytes, which htmx cannot render into a modal.
pub async fn handle_print_preview(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ExportRequest>,
) -> Result<Html<String>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.id)
        .await
        .map_err(|error| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": error })),
            )
        })?;
    let sheet = session.sheet.read().await.clone();
    let landscape = req
        .orientation
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("landscape") || value == "horizontal")
        .unwrap_or(false);
    let scale = req
        .scale
        .as_deref()
        .and_then(|value| value.trim().trim_end_matches('%').parse::<f32>().ok())
        .map(|value| if value > 10.0 { value / 100.0 } else { value })
        .unwrap_or(1.0);
    let pdf = export_to_pdf_data_with(&sheet, landscape, scale).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&error, "botsheet::handlers::export_import", None)})),
        )
    })?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&pdf);
    Ok(Html(format!(
        r##"<div class="ss-modal" id="print-preview-modal" style="position:fixed;inset:0;background:rgba(0,0,0,.7);z-index:1000;display:flex;align-items:center;justify-content:center;"><div style="background:#1e293b;border:1px solid #334155;border-radius:8px;width:92%;max-width:1100px;height:88vh;display:flex;flex-direction:column;"><div style="display:flex;justify-content:space-between;align-items:center;padding:12px 16px;border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;font-size:15px;">Print preview</h3><button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;font-size:20px;cursor:pointer;">×</button></div><iframe title="Print preview" src="data:application/pdf;base64,{encoded}" style="flex:1;width:100%;border:0;border-radius:0 0 8px 8px;"></iframe></div></div>"##
    )))
}

#[derive(serde::Deserialize, Default)]
struct ImportPathRequest {
    path: Option<String>,
    file: Option<String>,
    file_path: Option<String>,
    bucket: Option<String>,
}

fn content_type_for_sheet_path(path: &str) -> &'static str {
    match std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => "text/csv",
        "json" => "application/json",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "tsv" => "text/tab-separated-values",
        _ => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    }
}

pub async fn handle_import_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    headers: axum::http::HeaderMap,
    req: axum::extract::Request,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;

    // #2132 — refuse an oversized body before it is buffered.
    if let Err(error) =
        upload_policy::check_declared_length(UploadClass::SpreadsheetImport, &headers)
    {
        return Err(upload_policy::rejection_json(&error, &headers));
    }

    let is_json = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().starts_with("application/json"))
        .unwrap_or(false);

    let (bytes, filename, content_type) = if is_json {
        let body = axum::body::to_bytes(req.into_body(), upload_policy::max_bytes(UploadClass::SpreadsheetImport))
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Invalid JSON body" })),
                )
            })?;
        let payload: ImportPathRequest = serde_json::from_slice(&body).unwrap_or_default();
        let path = payload
            .path
            .or(payload.file)
            .or(payload.file_path)
            .unwrap_or_default();
        let path = path.trim().trim_start_matches('/').to_string();
        if path.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing file path for import" })),
            ));
        }
        let bucket = payload.bucket.unwrap_or_else(|| "gbo".to_string());
        let drive = state.drive.as_ref().ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Drive not available" })),
            )
        })?;
        let file_bytes = drive.get_object(&bucket, &path).await.map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Failed to read import file: {e}") })),
            )
        })?;
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("import.xlsx")
            .to_string();
        let ct = content_type_for_sheet_path(&path).to_string();
        (file_bytes, name, ct)
    } else {
        use axum::extract::FromRequest;
        let mut multipart = axum::extract::Multipart::from_request(req, &state)
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Invalid multipart body" })),
                )
            })?;

        let mut file_bytes: Option<Vec<u8>> = None;
        let mut filename = "import.xlsx".to_string();
        let mut content_type = String::new();

        while let Ok(Some(field)) = multipart.next_field().await {
            if field.name() == Some("file") {
                filename = field.file_name().unwrap_or("import.xlsx").to_string();
                content_type = field.content_type().map(str::to_string).unwrap_or_default();
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
        (bytes, filename, content_type)
    };

    // #2132 — the extension, the declared type and the magic bytes must agree.
    let upload = match upload_policy::validate_upload(
        UploadClass::SpreadsheetImport,
        &filename,
        &content_type,
        &bytes,
    ) {
        Ok(upload) => upload,
        Err(error) => return Err(upload_policy::rejection_json(&error, &headers)),
    };

    let mut sheet = import_spreadsheet_bytes(&bytes, &upload.filename, &user_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    sheet.owner_id = user_id.clone();

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
        ));
    }

    Ok(Json(sheet))
}
