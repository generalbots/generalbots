//! HTTP surface for the #1185 proactivity scheduler.

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::proactivity::{ProactivityRef, RegisterTriggerRequest, SuggestionCard, TriggerDef};

#[derive(Debug, Serialize)]
pub struct ProactivityResponse {
    pub success: bool,
    pub trigger: Option<TriggerDef>,
    pub triggers: Option<Vec<TriggerDef>>,
    pub cards: Option<Vec<SuggestionCard>>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<ProactivityResponse>);

fn ok_trigger(trigger: TriggerDef) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProactivityResponse { success: true, trigger: Some(trigger), triggers: None, cards: None, error: None }),
    )
}

fn ok_triggers(triggers: Vec<TriggerDef>) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProactivityResponse { success: true, trigger: None, triggers: Some(triggers), cards: None, error: None }),
    )
}

fn ok_cards(cards: Vec<SuggestionCard>) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProactivityResponse { success: true, trigger: None, triggers: None, cards: Some(cards), error: None }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe proactivity API error: {msg}");
    (
        StatusCode::OK,
        Json(ProactivityResponse { success: false, trigger: None, triggers: None, cards: None, error: Some(msg) }),
    )
}

#[derive(Debug, serde::Deserialize)]
pub struct CardsQuery {
    #[serde(default)]
    pub include_seen: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct ConsentRequest {
    pub consented: bool,
}

pub fn proactivity_router(engine: ProactivityRef) -> Router {
    Router::new()
        .route("/api/vibe/proactivity/triggers", get(list_triggers).post(register_trigger))
        .route("/api/vibe/proactivity/triggers/:trigger_id/consent", post(set_consent))
        .route("/api/vibe/proactivity/cards", get(list_cards))
        .route("/api/vibe/proactivity/cards/:card_id/seen", post(mark_seen))
        .route("/api/vibe/proactivity/cards", axum::routing::delete(clear_cards))
        .layer(Extension(engine))
}

async fn register_trigger(
    Extension(engine): Extension<ProactivityRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<RegisterTriggerRequest>,
) -> ApiResult {
    if req.category.trim().is_empty() {
        return err("category is required".to_string());
    }
    ok_trigger(engine.register(&req).await)
}

async fn list_triggers(
    Extension(engine): Extension<ProactivityRef>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> ApiResult {
    ok_triggers(engine.list_triggers().await)
}

async fn set_consent(
    Extension(engine): Extension<ProactivityRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(trigger_id): Path<Uuid>,
    Json(req): Json<ConsentRequest>,
) -> ApiResult {
    if engine.set_consent(&trigger_id, req.consented).await {
        ok_triggers(engine.list_triggers().await)
    } else {
        err(format!("trigger {trigger_id} not found"))
    }
}

async fn list_cards(
    Extension(engine): Extension<ProactivityRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    axum::extract::Query(q): axum::extract::Query<CardsQuery>,
) -> ApiResult {
    ok_cards(engine.cards(q.include_seen).await)
}

async fn mark_seen(
    Extension(engine): Extension<ProactivityRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(card_id): Path<Uuid>,
) -> ApiResult {
    if engine.mark_seen(&card_id).await {
        ok_cards(engine.cards(false).await)
    } else {
        err(format!("card {card_id} not found"))
    }
}

async fn clear_cards(
    Extension(engine): Extension<ProactivityRef>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> ApiResult {
    engine.clear_cards().await;
    ok_cards(Vec::new())
}
