use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botpaper::{
    configure_paper_routes as crate_configure_paper_routes,
    handle_ai_custom, handle_ai_expand, handle_ai_improve, handle_ai_simplify,
    handle_ai_summarize, handle_ai_translate, handle_autosave, handle_delete_document,
    handle_export_docx, handle_export_html, handle_export_md, handle_export_pdf,
    handle_export_txt, handle_get_document, handle_list_documents, handle_new_document,
    handle_save_document, handle_search_documents, handle_template_blank, handle_template_letter,
    handle_template_meeting, handle_template_report, handle_template_research,
    handle_template_todo, format_ai_response, format_document_content,
    format_document_list_item, format_error, format_relative_time, html_escape,
    markdown_to_html, strip_markdown, delete_document_from_drive,
    list_documents_from_drive, load_document_from_drive, save_document_to_drive,
    call_llm, get_current_user, PaperState,
};

async fn paper_call_llm(
    state: &Arc<AppState>,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    #[cfg(feature = "llm")]
    {
        let llm = &state.llm_provider;
        let messages = crate::llm::OpenAIClient::build_messages(
            system_prompt,
            "",
            &[("user".to_string(), user_content.to_string())],
        );
        let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
        let model = config_manager
            .get_config(&uuid::Uuid::nil(), "llm-model", None)
            .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
        let key = config_manager
            .get_config(&uuid::Uuid::nil(), "llm-key", None)
            .unwrap_or_else(|_| String::new());
        llm.generate(user_content, &messages, &model, &key)
            .await
            .map_err(|e| format!("LLM error: {}", e))
    }
    #[cfg(not(feature = "llm"))]
    {
        let _ = (state, system_prompt);
        Ok(format!(
            "[LLM not available] Processing: {}...",
            &user_content[..50.min(user_content.len())]
        ))
    }
}

fn make_paper_state(app_state: &Arc<AppState>) -> Arc<PaperState> {
    let drive = app_state.drive.clone();
    let bucket_name = app_state
        .drive_bucket
        .clone()
        .unwrap_or_else(|| "papers".to_string());

    let drive_for_put = drive.clone();
    let s3_put: botpaper::state::S3PutFn = Arc::new(
        move |bucket: &str, key: &str, data: Vec<u8>, content_type: Option<&str>| {
            let drive = drive_for_put.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            let ct = content_type.map(String::from);
            Box::pin(async move {
                let s3 = drive.as_ref().ok_or_else(|| "S3 not available".to_string())?;
                s3.put_object_direct(&bucket, &key, data, ct.as_deref())
                    .await
                    .map_err(|e| e.to_string())
            })
        },
    );

    let drive_for_get = drive.clone();
    let s3_get: botpaper::state::S3GetFn = Arc::new(move |bucket: &str, key: &str| {
        let drive = drive_for_get.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        Box::pin(async move {
            let s3 = drive.as_ref().ok_or_else(|| "S3 not available".to_string())?;
            s3.get_object_direct(&bucket, &key)
                .await
                .map_err(|e| e.to_string())
        })
    });

    let drive_for_del = drive.clone();
    let s3_delete: botpaper::state::S3DeleteFn = Arc::new(move |bucket: &str, key: &str| {
        let drive = drive_for_del.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        Box::pin(async move {
            let s3 = drive.as_ref().ok_or_else(|| "S3 not available".to_string())?;
            s3.delete_object_direct(&bucket, &key)
                .await
                .map_err(|e| e.to_string())
        })
    });

    let drive_for_list = drive;
    let s3_list: botpaper::state::S3ListFn = Arc::new(
        move |bucket: &str, prefix: &str| {
            let drive = drive_for_list.clone();
            let bucket = bucket.to_string();
            let prefix = prefix.to_string();
            Box::pin(async move {
                let s3 = drive.as_ref().ok_or_else(|| "S3 not available".to_string())?;
                s3.list_objects(&bucket, Some(prefix.as_str()))
                    .await
                    .map_err(|e| e.to_string())
            })
        },
    );

    let llm_state = app_state.clone();
    let call_llm: botpaper::state::CallLlmFn = Arc::new(
        move |system_prompt: &str, user_content: &str| {
            let state = llm_state.clone();
            let sp = system_prompt.to_string();
            let uc = user_content.to_string();
            Box::pin(async move { paper_call_llm(&state, &sp, &uc).await })
        },
    );

    Arc::new(PaperState {
        conn: app_state.conn.clone(),
        bucket_name,
        s3_put,
        s3_get,
        s3_delete,
        s3_list,
        call_llm,
    })
}

pub fn configure_paper_routes(app_state: Arc<AppState>) -> Router {
    crate_configure_paper_routes().with_state(make_paper_state(&app_state))
}
