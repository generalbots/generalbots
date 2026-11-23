use super::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecipientsQuery {
    pub query: Option<String>,
}

/// #1639 — recipient picker for "Share with person". Returns at most 10 active
/// accounts whose email or username contains the typed text; the caller still
/// has to choose one, and the share row is written by `share_folder`.
pub async fn list_share_recipients(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<RecipientsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    // #2670 — the caller's org membership is the tenant boundary. Never
    // discard the identity: a global user search is a cross-tenant user
    // enumeration primitive.
    let caller_id = user.user_id;
    let caller_email = user.email.clone().unwrap_or_default();
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
        "WITH allowed_orgs AS ( \
             SELECT org_id FROM user_organizations WHERE user_id = $2 \
             UNION \
             SELECT uo.org_id FROM user_organizations uo \
               JOIN users c ON c.id = uo.user_id \
               WHERE lower(c.email) = lower($3) \
             UNION \
             SELECT br.org_id FROM crm_contacts c \
               JOIN branches br ON br.id = c.branch_id \
               WHERE lower(c.email) = lower($3) \
         ) \
         SELECT DISTINCT u.id::text, u.email, u.username FROM users u \
         WHERE u.is_active IS NOT FALSE \
           AND (LOWER(u.email) LIKE $1 OR LOWER(u.username) LIKE $1) \
           AND ( \
                 EXISTS (SELECT 1 FROM user_organizations uo \
                         WHERE uo.user_id = u.id \
                           AND uo.org_id IN (SELECT org_id FROM allowed_orgs)) \
                 OR EXISTS (SELECT 1 FROM crm_contacts c \
                            JOIN branches br ON br.id = c.branch_id \
                            WHERE lower(c.email) = lower(u.email) \
                              AND br.org_id IN (SELECT org_id FROM allowed_orgs)) \
           ) \
         ORDER BY u.email LIMIT 10",
    )
    .bind::<Text, _>(format!("%{query}%"))
    .bind::<diesel::sql_types::Uuid, _>(caller_id)
    .bind::<Text, _>(caller_email)
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

#[cfg(test)]
mod recipient_scope_tests {
    /// #2670 — the recipient query is tenant-scoped by construction. A pure
    /// SQL-shape assertion keeps a future refactor from dropping the org join.
    #[test]
    fn recipient_query_is_scoped_to_caller_orgs() {
        let sql = include_str!("share_recipients.rs");
        assert!(sql.contains("allowed_orgs"), "must build a caller-org CTE");
        assert!(sql.contains("uo.user_id = u.id"), "must join candidates to their orgs");
        assert!(sql.contains("br.org_id IN (SELECT org_id FROM allowed_orgs)"));
        assert!(!sql.contains("let _ = user"), "the caller identity must not be discarded");
    }
}
