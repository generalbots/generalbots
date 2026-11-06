use super::super::*;

pub async fn handle_instant_answer<S: ResearchState>(
    State(_state): State<Arc<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();

    if query.is_empty() {
        return Json(serde_json::json!({
            "answer": null,
            "type": "none"
        }));
    }

    if let Some(answer) = get_instant_answer(&query).await {
        Json(serde_json::json!({
            "answer": answer.0,
            "type": answer.1,
            "source": "duckduckgo"
        }))
    } else {
        Json(serde_json::json!({
            "answer": null,
            "type": "none"
        }))
    }
}
