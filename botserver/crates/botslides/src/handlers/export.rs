use crate::storage::{get_current_user_id, load_presentation_by_id, DriveOps};
use crate::types::ExportRequest;
use crate::ui::{export_to_html, export_to_json, export_to_markdown, export_to_odp_content, export_to_svg};
use crate::SlidesState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn handle_export_presentation<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not configured" })),
        )
    })?;

    let presentation = load_presentation_by_id(drive, &user_id, &req.id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    match req.format.as_str() {
        "html" => {
            let html = export_to_html(&presentation);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], html))
        }
        "json" => {
            let json = export_to_json(&presentation);
            Ok((
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                json,
            ))
        }
        "svg" => {
            if !presentation.slides.is_empty() {
                let svg = export_to_svg(&presentation.slides[0], 960, 540);
                Ok((
                    [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
                    svg,
                ))
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "No slides to export" })),
                ))
            }
        }
        "md" | "markdown" => {
            let md = export_to_markdown(&presentation);
            Ok((
                [(axum::http::header::CONTENT_TYPE, "text/markdown")],
                md,
            ))
        }
        "odp" => {
            let odp = export_to_odp_content(&presentation);
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.oasis.opendocument.presentation",
                )],
                odp,
            ))
        }
        "pptx" => Ok((
            [(
                axum::http::header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            )],
            "PPTX export not yet implemented".to_string(),
        )),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported format" })),
        )),
    }
}
