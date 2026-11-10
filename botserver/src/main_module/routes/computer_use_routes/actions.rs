use axum::extract::{Json, Path, Query};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::page::fetch_page;
use super::{api_error, api_error_key, resolve_user, ApiError};
use crate::core::computer_use;

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub session_id: String,
    /// `browser` or `terminal`.
    pub kind: String,
    pub action: Value,
}

/// POST /api/computer/actions
pub async fn create_action(
    headers: HeaderMap,
    Json(body): Json<CreateActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    if body.session_id.trim().is_empty() {
        return Err(api_error_key(
            &locale,
            StatusCode::BAD_REQUEST,
            "error-computer-use-session-required",
        ));
    }
    if body.kind != "browser" && body.kind != "terminal" {
        return Err(api_error_key(
            &locale,
            StatusCode::BAD_REQUEST,
            "error-computer-use-kind-invalid",
        ));
    }
    let action =
        computer_use::create(body.session_id.trim(), user_id, &body.kind, body.action).await;
    Ok(Json(action.to_json()))
}

#[derive(Debug, Deserialize)]
pub struct PendingQuery {
    pub session_id: String,
    #[serde(default)]
    pub kind: Option<String>,
}

/// GET /api/computer/actions/pending?session_id=&kind=
pub async fn pending(
    headers: HeaderMap,
    Query(query): Query<PendingQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let kind = query.kind.as_deref().unwrap_or("browser");
    let actions = computer_use::pending_for(query.session_id.trim(), user_id, kind).await;
    Ok(Json(json!({
        "actions": actions.iter().map(computer_use::ComputerAction::to_json).collect::<Vec<_>>(),
    })))
}

/// GET /api/computer/actions/:id
pub async fn get_action(headers: HeaderMap, Path(id): Path<Uuid>) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    match computer_use::get(id, user_id).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error_key(
            &locale,
            StatusCode::NOT_FOUND,
            "error-computer-use-action-not-found",
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApproveActionRequest {
    /// The client echoes the action it is asking the user about; a mismatch is
    /// refused so an approval cannot be replayed onto another action.
    #[serde(default)]
    pub action_id: Option<Uuid>,
}

/// POST /api/computer/actions/:id/approve — creates the consent record that
/// the executors require. Refusing is simply *not* calling this endpoint.
pub async fn approve_action(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ApproveActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    if let Some(body_id) = body.action_id {
        if body_id != id {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                "action_id does not match the path",
            ));
        }
    }
    match computer_use::approve(id, user_id).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error(
            StatusCode::NOT_FOUND,
            "action not found or already denied",
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExecuteActionRequest {
    /// Kept for compatibility with older clients; the server requires the
    /// stored consent record created by /approve.
    #[serde(default)]
    pub approved: bool,
}

/// POST /api/computer/actions/:id/execute — runs the browser actions that need
/// the server (fetching a page, extracting its title/text). Actions that act
/// on the user's own browser panel (click/fill) stay client-side.
pub async fn execute_action(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ExecuteActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    let action = computer_use::get(id, user_id).await.ok_or_else(|| {
        api_error_key(
            &locale,
            StatusCode::NOT_FOUND,
            "error-computer-use-action-not-found",
        )
    })?;
    if !body.approved || !computer_use::is_approved(id, user_id).await {
        return Err(api_error_key(
            &locale,
            StatusCode::FORBIDDEN,
            "error-computer-use-no-consent",
        ));
    }
    let name = action
        .action
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("");
    if name != "open_url" && name != "extract_text" {
        return Err(api_error_key(
            &locale,
            StatusCode::BAD_REQUEST,
            "error-computer-use-browser-only",
        ));
    }
    let url = action
        .action
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            api_error_key(
                &locale,
                StatusCode::BAD_REQUEST,
                "error-computer-use-url-missing",
            )
        })?;
    let page = fetch_page(url)
        .await
        .map_err(|e| api_error(StatusCode::BAD_GATEWAY, &e))?;
    let result = json!({
        "status": "ok",
        "url": page.url,
        "title": page.title,
        "text": page.text,
    });
    computer_use::complete(id, user_id, result.clone()).await;
    Ok(Json(result))
}
