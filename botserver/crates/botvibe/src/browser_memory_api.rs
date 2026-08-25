//! HTTP surface for the #1175 agentic-browser memory.

use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::browser_memory::{AskRequest, BrowserMemoryRef, CitedAnswer, MemoryEntry, RememberRequest};

#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    pub success: bool,
    pub entry: Option<MemoryEntry>,
    pub chips: Option<Vec<MemoryEntry>>,
    pub answer: Option<CitedAnswer>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<MemoryResponse>);

fn ok_entry(entry: MemoryEntry) -> ApiResult {
    (
        StatusCode::OK,
        Json(MemoryResponse { success: true, entry: Some(entry), chips: None, answer: None, error: None }),
    )
}

fn ok_chips(chips: Vec<MemoryEntry>) -> ApiResult {
    (
        StatusCode::OK,
        Json(MemoryResponse { success: true, entry: None, chips: Some(chips), answer: None, error: None }),
    )
}

fn ok_answer(answer: CitedAnswer) -> ApiResult {
    (
        StatusCode::OK,
        Json(MemoryResponse { success: true, entry: None, chips: None, answer: Some(answer), error: None }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe browser-memory API error: {msg}");
    (
        StatusCode::OK,
        Json(MemoryResponse { success: false, entry: None, chips: None, answer: None, error: Some(msg) }),
    )
}

#[derive(Debug, Deserialize)]
pub struct ChipsQuery {
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClearQuery {
    #[serde(default)]
    pub domain: Option<String>,
}

pub fn browser_memory_router(memory: BrowserMemoryRef) -> Router {
    Router::new()
        .route("/api/vibe/browser-memory", get(chips).post(remember))
        .route("/api/vibe/browser-memory/ask", post(ask))
        .route("/api/vibe/browser-memory", delete(clear))
        .layer(Extension(memory))
}

async fn remember(
    Extension(memory): Extension<BrowserMemoryRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<RememberRequest>,
) -> ApiResult {
    if req.domain.trim().is_empty() || req.fact.trim().is_empty() {
        return err("domain and fact are required".to_string());
    }
    ok_entry(memory.remember(&req).await)
}

async fn chips(
    Extension(memory): Extension<BrowserMemoryRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(q): Query<ChipsQuery>,
) -> ApiResult {
    let domains = q.domain.as_ref().map(|d| vec![d.clone()]).unwrap_or_default();
    ok_chips(memory.chips(&domains).await)
}

async fn ask(
    Extension(memory): Extension<BrowserMemoryRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<AskRequest>,
) -> ApiResult {
    if req.question.trim().is_empty() {
        return err("question is required".to_string());
    }
    match memory.ask(&req).await {
        Ok(answer) => ok_answer(answer),
        Err(e) => err(e),
    }
}

async fn clear(
    Extension(memory): Extension<BrowserMemoryRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(q): Query<ClearQuery>,
) -> ApiResult {
    memory.clear(q.domain.as_deref()).await;
    (
        StatusCode::OK,
        Json(MemoryResponse { success: true, entry: None, chips: None, answer: None, error: None }),
    )
}
