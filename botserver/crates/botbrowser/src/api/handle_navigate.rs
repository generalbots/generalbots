use super::*;

pub async fn handle_navigate(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    caller: SessionCaller,
    Json(payload): Json<NavigateRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id, &caller).await?;
    // #2878 — fail closed with 400 before Chromium ever sees the URL; the
    // shared policy is also re-applied inside `navigate_with_result` and on the
    // final post-redirect URL.
    if let Err(reason) = crate::url_policy::validate_navigation_url(&payload.url) {
        return Err((axum::http::StatusCode::BAD_REQUEST, reason));
    }
    match session.navigate_with_result(&payload.url).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Navigate failed: {e}"),
        )),
    }
}
