use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botdesigner::{
    bas_analyzer::{BasFileAnalyzer, BasFileType, WorkflowMetadata},
    canvas_api::{
        CanvasError, CanvasRow, CanvasService, TemplateRow, canvas_routes,
        create_canvas_tables_migration, row_to_canvas,
    },
    designer_api::{
        configure_designer_routes,
        types::*,
    },
    workflow_canvas::{
        AnalyzeFileRequest, AnalyzeFileResponse, GenerateCodeRequest, NodeConfig, NodeType,
        Position, WorkflowCanvas, WorkflowConnection, WorkflowNode, analyze_bas_file,
        generate_workflow_code, workflow_designer_page,
    },
    DesignerState,
};

fn make_designer_state(app_state: &Arc<AppState>) -> Arc<DesignerState> {
    let conn = Arc::new(app_state.conn.clone());
    let get_default_bot: botdesigner::GetDefaultBotFn =
        Arc::new(|conn: &mut diesel::PgConnection| crate::core::bot::get_default_bot(conn));
    let get_error_ctx: botdesigner::GetDesignerErrorContextFn =
        Arc::new(|_msg: &str| None);
    let get_content_type: botdesigner::GetContentTypeFn =
        Arc::new(|path: &str| -> &'static str {
            match path.rsplit('.').next() {
                Some("bas") => "text/plain",
                Some("json") => "application/json",
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("html") => "text/html",
                _ => "application/octet-stream",
            }
        });
    let get_stack_path: botdesigner::GetStackPathFn =
        Arc::new(|| crate::core::shared::utils::get_stack_path());
    let drive_for_load = app_state.drive.clone();
    let bucket_for_load = app_state.bucket_name.clone();
    let load_from_drive: botdesigner::LoadFromDriveFn =
        Arc::new(move |bucket: &str, key: &str| -> Result<String, String> {
            let s3 = drive_for_load.as_ref().ok_or_else(|| "S3 not available".to_string())?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            rt.block_on(async {
                let data = s3.get_object_direct(bucket, key).await.map_err(|e| e.to_string())?;
                String::from_utf8(data).map_err(|e| e.to_string())
            })
        });
    let drive_for_write = app_state.drive.clone();
    let write_to_drive: botdesigner::WriteToDriveFn =
        Arc::new(move |bucket: &str, key: &str, data: &[u8], content_type: &str| -> Result<(), String> {
            let s3 = drive_for_write.as_ref().ok_or_else(|| "S3 not available".to_string())?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let data = data.to_vec();
            rt.block_on(async {
                s3.put_object_direct(bucket, key, data, Some(content_type))
                    .await
                    .map_err(|e| e.to_string())
            })
        });
    let call_llm: botdesigner::CallLlmFn =
        Arc::new(|_prompt: &str, _params: &serde_json::Value| -> Result<String, String> {
            Err("LLM not available in designer shim".to_string())
        });
    let conn_for_config = app_state.conn.clone();
    let get_config: botdesigner::GetConfigFn =
        Arc::new(move |bot_id: &str, key: &str, default: Option<&str>| -> Result<String, String> {
            let config = crate::core::shared::config::ConfigManager::new(conn_for_config.clone());
            let bot_uuid = uuid::Uuid::parse_str(bot_id).unwrap_or(uuid::Uuid::nil());
            Ok(config.get_config(&bot_uuid, key, default).unwrap_or_default())
        });
    let bucket_name = bucket_for_load;
    let site_path = app_state.config.as_ref().map(|c| c.site_path.clone());

    Arc::new(DesignerState {
        conn,
        get_default_bot,
        get_designer_error_context: get_error_ctx,
        get_content_type,
        get_stack_path,
        load_from_drive,
        write_to_drive,
        call_llm,
        get_config,
        bucket_name,
        site_path,
    })
}

pub fn configure_designer_routes(state: &Arc<AppState>) -> Router {
    botdesigner::designer_api::configure_designer_routes()
        .with_state(make_designer_state(state))
}

pub fn configure_designer_ui_routes(state: &Arc<AppState>) -> Router {
    botdesigner::ui::configure_designer_ui_routes()
        .with_state(make_designer_state(state))
}
