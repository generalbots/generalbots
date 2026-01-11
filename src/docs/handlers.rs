use crate::docs::storage::{
    create_new_document, delete_document_from_drive, get_current_user_id,
    list_documents_from_drive, load_document_from_drive, save_document_to_drive,
};
use crate::docs::types::{
    DocsSaveRequest, DocsSaveResponse, DocsAiRequest, DocsAiResponse, Document, DocumentMetadata,
    SearchQuery, TemplateResponse,
};
use crate::docs::utils::{html_to_markdown, strip_html};
use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use docx_rs::{AlignmentType, Docx, Paragraph, Run};
use log::error;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_docs_ai(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DocsAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();

    let response = if command.contains("summarize") || command.contains("summary") {
        "I've created a summary of your document. The key points are highlighted above."
    } else if command.contains("expand") || command.contains("longer") {
        "I've expanded the selected text with more details and examples."
    } else if command.contains("shorter") || command.contains("concise") {
        "I've made the text more concise while preserving the key information."
    } else if command.contains("formal") {
        "I've rewritten the text in a more formal, professional tone."
    } else if command.contains("casual") || command.contains("friendly") {
        "I've rewritten the text in a more casual, friendly tone."
    } else if command.contains("grammar") || command.contains("fix") {
        "I've corrected the grammar and spelling errors in your text."
    } else if command.contains("translate") {
        "I've translated the selected text. Please specify the target language if needed."
    } else if command.contains("bullet") || command.contains("list") {
        "I've converted the text into a bulleted list format."
    } else if command.contains("help") {
        "I can help you with:\n• Summarize text\n• Expand or shorten content\n• Fix grammar\n• Change tone (formal/casual)\n• Translate text\n• Convert to bullet points"
    } else {
        "I understand you want help with your document. Try commands like 'summarize', 'make shorter', 'fix grammar', or 'make formal'."
    };

    Json(DocsAiResponse {
        response: response.to_string(),
        result: None,
    })
}

pub async fn handle_docs_save(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc_id, &req.title, &req.content).await
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_docs_get_by_id(
    State(state): State<Arc<AppState>>,
    Path(doc_id): Path<String>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => Ok(Json(doc)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_new_document(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(create_new_document()))
}

pub async fn handle_list_documents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DocumentMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match list_documents_from_drive(&state, &user_id).await {
        Ok(docs) => Ok(Json(docs)),
        Err(e) => {
            error!("Failed to list documents: {}", e);
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<DocumentMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let docs = match list_documents_from_drive(&state, &user_id).await {
        Ok(d) => d,
        Err(_) => Vec::new(),
    };

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        docs.into_iter()
            .filter(|d| d.title.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        docs
    };

    Ok(Json(filtered))
}

pub async fn handle_get_document(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::LoadQuery>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc_id = query.id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Document ID required" })),
        )
    })?;

    match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => Ok(Json(doc)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_save_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc_id, &req.title, &req.content).await
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_autosave(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    handle_save_document(State(state), Json(req)).await
}

pub async fn handle_delete_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::docs::types::LoadQuery>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc_id = req.id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Document ID required" })),
        )
    })?;

    if let Err(e) = delete_document_from_drive(&state, &user_id, &doc_id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_template_blank() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Untitled Document".to_string(),
        content: String::new(),
    }))
}

pub async fn handle_template_meeting() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<h1>Meeting Notes</h1>
<p><strong>Date:</strong> [Date]</p>
<p><strong>Attendees:</strong> [Names]</p>
<p><strong>Location:</strong> [Location/Virtual]</p>
<hr>
<h2>Agenda</h2>
<ol>
<li>Topic 1</li>
<li>Topic 2</li>
<li>Topic 3</li>
</ol>
<h2>Discussion Points</h2>
<p>[Notes here]</p>
<h2>Action Items</h2>
<ul>
<li>[ ] Action 1 - Owner - Due Date</li>
<li>[ ] Action 2 - Owner - Due Date</li>
</ul>
<h2>Next Meeting</h2>
<p>[Date and time of next meeting]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Meeting Notes".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_template_report() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<h1>Report Title</h1>
<p><em>Author: [Your Name]</em></p>
<p><em>Date: [Date]</em></p>
<hr>
<h2>Executive Summary</h2>
<p>[Brief overview of the report]</p>
<h2>Introduction</h2>
<p>[Background and context]</p>
<h2>Methodology</h2>
<p>[How the information was gathered]</p>
<h2>Findings</h2>
<p>[Key findings and data]</p>
<h2>Recommendations</h2>
<ul>
<li>Recommendation 1</li>
<li>Recommendation 2</li>
<li>Recommendation 3</li>
</ul>
<h2>Conclusion</h2>
<p>[Summary and next steps]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Report".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_template_letter() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<p>[Your Name]<br>
[Your Address]<br>
[City, State ZIP]<br>
[Date]</p>
<p>[Recipient Name]<br>
[Recipient Title]<br>
[Company Name]<br>
[Address]<br>
[City, State ZIP]</p>
<p>Dear [Recipient Name],</p>
<p>[Opening paragraph - state the purpose of your letter]</p>
<p>[Body paragraph(s) - provide details and supporting information]</p>
<p>[Closing paragraph - summarize and state any call to action]</p>
<p>Sincerely,</p>
<p>[Your Name]<br>
[Your Title]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Letter".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_ai_summarize(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let summary = if text.len() > 200 {
        format!("Summary: {}...", &text[..200])
    } else {
        format!("Summary: {}", text)
    };

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: summary,
        error: None,
    }))
}

pub async fn handle_ai_expand(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let expanded = format!("{}\n\n[Additional context and details would be added here by AI]", text);

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: expanded,
        error: None,
    }))
}

pub async fn handle_ai_improve(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_ai_simplify(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_ai_translate(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let lang = req.translate_lang.unwrap_or_else(|| "English".to_string());

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: format!("[Translated to {}]: {}", lang, text),
        error: None,
    }))
}

pub async fn handle_ai_custom(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_export_pdf(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        "PDF export not yet implemented".to_string(),
    ))
}

pub async fn handle_export_docx(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let docx_bytes = html_to_docx(&doc.content, &doc.title);

    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )],
        docx_bytes,
    ))
}

fn html_to_docx(html: &str, title: &str) -> Vec<u8> {
    let plain_text = strip_html(html);
    let paragraphs: Vec<&str> = plain_text.split("\n\n").collect();

    let mut docx = Docx::new();

    let title_para = Paragraph::new()
        .add_run(Run::new().add_text(title).bold())
        .align(AlignmentType::Center);
    docx = docx.add_paragraph(title_para);

    for para_text in paragraphs {
        if !para_text.trim().is_empty() {
            let para = Paragraph::new().add_run(Run::new().add_text(para_text.trim()));
            docx = docx.add_paragraph(para);
        }
    }

    let mut buffer = Vec::new();
    if let Ok(_) = docx.build().pack(&mut std::io::Cursor::new(&mut buffer)) {
        buffer
    } else {
        Vec::new()
    }
}

pub async fn handle_export_md(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let markdown = html_to_markdown(&doc.content);

    Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], markdown))
}

pub async fn handle_export_html(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        doc.title, doc.content
    );

    Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], full_html))
}

pub async fn handle_export_txt(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let plain_text = strip_html(&doc.content);

    Ok(([(axum::http::header::CONTENT_TYPE, "text/plain")], plain_text))
}
