//! Export and import handlers (split from [`super::crud`]).

use base64::Engine;
use crate::auth::{resolve_user_id, SheetUser};
use crate::export::{
    export_to_csv, export_to_html, export_to_json, export_to_markdown, export_to_ods,
    export_to_pdf_data, export_to_pdf_data_with, export_to_xlsx,
};
use crate::state::{save_sheet_to_drive, SheetState};
use crate::storage::import::import_spreadsheet_bytes;
use crate::types::{ExportRequest, Spreadsheet};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use std::sync::Arc;

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
            Ok(([(axum::http::header::CONTENT_TYPE, "text/csv")], csv.into_bytes()))
        }
        "xlsx" => {
            // #2043 — export prefers the preserved package, like save-back
            // does: patching the source keeps charts, comments, conditional
            // formats and images that the model cannot re-emit.
            let (xlsx, fidelity) = match sheet.source_bytes.as_deref() {
                Some(original) => match crate::storage::merge_into_original(original, &sheet) {
                    Ok(bytes) => (bytes, "preserved"),
                    Err(error) => {
                        log::warn!(
                            "sheet export: cannot patch the source package, regenerating: {error}"
                        );
                        let bytes = export_to_xlsx(&sheet).map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
                            )
                        })?;
                        (bytes, "regenerated")
                    }
                },
                None => {
                    let bytes = export_to_xlsx(&sheet).map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
                        )
                    })?;
                    (bytes, "regenerated")
                }
            };
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                ),
            );
            headers.insert(
                "x-gb-xlsx-fidelity",
                axum::http::HeaderValue::from_static(fidelity),
            );
            return Ok((headers, xlsx));
        }
        "json" => {
            let json = export_to_json(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json.into_bytes()))
        }
        "html" => {
            let html = export_to_html(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], html.into_bytes()))
        }
        "ods" => {
            let ods = export_to_ods(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
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
            Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], md.into_bytes()))
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
            Ok(([(axum::http::header::CONTENT_TYPE, "application/pdf")], pdf))
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
    let user_id = resolve_user_id(user.as_deref());
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
            Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botsheet::handlers::export_import", None)})),
        ));
    }

    Ok(Json(sheet))
}
