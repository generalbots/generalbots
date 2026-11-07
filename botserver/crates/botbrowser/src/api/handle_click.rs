use super::*;

pub async fn handle_click(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
    Json(payload): Json<ClickRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;

    // The VNC-like surface sends page coordinates because the user sees a
    // screenshot, not the DOM. Selector clicks stay supported for automation.
    if let (Some(x), Some(y)) = (payload.x, payload.y) {
        return match session.click_at(x, y).await {
            Ok(()) => Ok(Json(serde_json::json!({
                "status": "clicked",
                "x": x,
                "y": y
            }))),
            Err(e) => Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Click failed: {e}"),
            )),
        };
    }

    let selector = payload.selector.ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            "click requires a selector or x/y coordinates".to_string(),
        )
    })?;
    match session.click(&selector).await {
        Ok(state) => Ok(Json(serde_json::to_value(state).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Click failed: {e}"),
        )),
    }
}
