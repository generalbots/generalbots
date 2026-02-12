use crate::core::shared::state::AppState;
use crate::docs::storage::{
    create_new_document, delete_document_from_drive, get_current_user_id,
    list_documents_from_drive, load_document_from_drive, save_document_to_drive,
};
use crate::docs::types::{DocsSaveRequest, DocsSaveResponse, Document, DocumentMetadata, SearchQuery};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

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
            log::error!("Failed to list documents: {}", e);
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
