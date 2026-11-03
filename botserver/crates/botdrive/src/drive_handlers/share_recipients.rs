use super::*;

#[derive(Debug, Deserialize)]
pub struct RecipientsQuery {
    pub query: Option<String>,
}

/// #1639 — recipient picker for "Share with person". Returns at most 10 active
/// accounts whose email or username contains the typed text; the caller still
/// has to choose one, and the share row is written by `share_folder`.
pub async fn list_share_recipients(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<RecipientsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let query = params.query.unwrap_or_default().trim().to_lowercase();
    if query.len() < 2 {
        return Ok(Json(Vec::new()));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = Text)]
        username: String,
    }

    let rows = diesel::sql_query(
        "SELECT id::text, email, username FROM users \
         WHERE is_active IS NOT FALSE \
           AND (LOWER(email) LIKE $1 OR LOWER(username) LIKE $1) \
         ORDER BY email LIMIT 10",
    )
    .bind::<Text, _>(format!("%{query}%"))
    .load::<Row>(&mut conn)
    .unwrap_or_default();

    Ok(Json(
        rows.into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "email": row.email,
                    "username": row.username,
                })
            })
            .collect(),
    ))
}
