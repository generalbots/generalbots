use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::engine::VideoEngine;
use crate::requests::*;
use crate::responses::*;
use crate::routes::AppState;
use crate::safe_error::SafeErrorResponse;

pub async fn list_templates(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let templates = vec![
        TemplateInfo {
            id: "social-promo".to_string(),
            name: "Social Promo".to_string(),
            description: "Quick social media promotional video".to_string(),
            thumbnail_url: "/video/templates/social-promo.jpg".to_string(),
            duration_ms: 15000,
            category: "social".to_string(),
        },
        TemplateInfo {
            id: "youtube-intro".to_string(),
            name: "YouTube Intro".to_string(),
            description: "Professional YouTube channel intro".to_string(),
            thumbnail_url: "/video/templates/youtube-intro.jpg".to_string(),
            duration_ms: 5000,
            category: "intro".to_string(),
        },
        TemplateInfo {
            id: "talking-head".to_string(),
            name: "Talking Head".to_string(),
            description: "Interview or presentation style".to_string(),
            thumbnail_url: "/video/templates/talking-head.jpg".to_string(),
            duration_ms: 30000,
            category: "presentation".to_string(),
        },
        TemplateInfo {
            id: "product-showcase".to_string(),
            name: "Product Showcase".to_string(),
            description: "E-commerce product highlight".to_string(),
            thumbnail_url: "/video/templates/product-showcase.jpg".to_string(),
            duration_ms: 20000,
            category: "commercial".to_string(),
        },
    ];

    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "templates": templates })),
    )
}

pub async fn apply_template_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<ApplyTemplateRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    let customizations = req.customizations.map(|h| serde_json::json!(h));

    match engine
        .apply_template(project_id, &req.template_id, customizations)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        ),
        Err(e) => {
            error!("Apply template failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn chat_edit(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<ChatEditRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine
        .process_chat_command(project_id, &req.message, req.playhead_ms, req.selection)
        .await
    {
        Ok(response) => (StatusCode::OK, axum::Json(serde_json::json!(response))),
        Err(e) => {
            error!("Chat edit failed: {e}");
            (
                StatusCode::OK,
                axum::Json(serde_json::json!(ChatEditResponse {
                    success: false,
                    message: "Could not process that request".to_string(),
                    commands_executed: vec![],
                    project: None,
                })),
            )
        }
    }
}

pub async fn video_ui() -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Video Editor</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#1a1a1a;color:#fff;overflow:hidden;height:100vh}
.toolbar{display:flex;align-items:center;padding:8px 16px;background:#252525;border-bottom:1px solid #333;gap:8px}
.toolbar h1{font-size:16px;margin-right:16px}
.toolbar button{padding:6px 12px;border:1px solid #444;background:#333;color:#fff;border-radius:4px;cursor:pointer;font-size:12px}
.toolbar button:hover{background:#444}
.main{display:flex;height:calc(100vh - 44px)}
.sidebar{width:240px;background:#252525;border-right:1px solid #333;overflow-y:auto;padding:12px}
.sidebar h3{font-size:12px;color:#888;text-transform:uppercase;margin-bottom:8px}
.asset-item{padding:8px;border-radius:4px;cursor:pointer;margin-bottom:4px;font-size:13px}
.asset-item:hover{background:#333}
.canvas-area{flex:1;display:flex;flex-direction:column}
.preview{flex:1;display:flex;align-items:center;justify-content:center;background:#000;position:relative}
.preview video{max-width:100%;max-height:100%}
.timeline{height:200px;background:#252525;border-top:1px solid #333;display:flex;flex-direction:column}
.timeline-header{display:flex;align-items:center;padding:4px 12px;border-bottom:1px solid #333;gap:8px;font-size:12px;color:#888}
.timeline-tracks{flex:1;overflow-x:auto;padding:4px 12px}
.track{display:flex;align-items:center;height:36px;margin-bottom:2px}
.track-label{width:80px;font-size:11px;color:#888;flex-shrink:0}
.track-content{flex:1;height:100%;position:relative;background:#333;border-radius:4px}
.clip{position:absolute;height:100%;background:#0066cc;border-radius:4px;cursor:pointer;font-size:10px;display:flex;align-items:center;padding:0 8px;color:#fff;overflow:hidden;white-space:nowrap}
.clip:hover{background:#0052a3}
.properties{width:240px;background:#252525;border-left:1px solid #333;padding:12px;overflow-y:auto}
.properties h3{font-size:12px;color:#888;text-transform:uppercase;margin-bottom:8px}
.prop-group{margin-bottom:12px}
.prop-label{font-size:11px;color:#888;margin-bottom:4px}
.prop-input{width:100%;padding:4px 8px;background:#333;border:1px solid #444;border-radius:4px;color:#fff;font-size:12px}
</style>
</head>
<body>
<div class="toolbar">
<h1>Video Editor</h1>
<button id="btnUndo">Undo</button>
<button id="btnRedo">Redo</button>
<button id="btnCut">Cut</button>
<button id="btnSplit">Split</button>
<button id="btnDelete">Delete</button>
<button id="btnExport">Export</button>
</div>
<div class="main">
<div class="sidebar">
<h3>Media</h3>
<div id="mediaList"></div>
<h3>Templates</h3>
<div id="templateList"></div>
</div>
<div class="canvas-area">
<div class="preview" id="preview"><video id="videoPlayer" controls style="display:none"></video><p style="color:#666">No clip selected</p></div>
<div class="timeline">
<div class="timeline-header"><span id="timeDisplay">00:00.000</span><span id="durationDisplay">/ 00:00.000</span></div>
<div class="timeline-tracks" id="timelineTracks"><div class="track"><div class="track-label">Video 1</div><div class="track-content" id="track1"></div></div></div>
</div>
</div>
<div class="properties">
<h3>Properties</h3>
<div id="propsContent"><p style="color:#666;font-size:12px">Select a clip or layer</p></div>
</div>
</div>
</body>
</html>"#;
    Html(html.to_string())
}
