//! Skills discovery for the chat catalogue (#1681 follow-up).
//!
//! The prompt advertises every installed skill by name and description; the
//! full `SKILL.md` body is only pulled when the model asks for it, so the
//! prompt stays compact no matter how large the marketplace grows.

use super::*;

#[derive(diesel::QueryableByName)]
struct SkillRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    content: String,
}

/// Lists installed, enabled skills. With a query, only the best matches are
/// returned together with their full package content.
pub(super) fn installed_skills(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    query: Option<String>,
) -> Value {
    let q = query
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());
    let Ok(mut conn) = state.conn.get() else {
        return json!({ "error": "database unavailable" });
    };
    let rows: Vec<SkillRow> = match &q {
        Some(term) => diesel::sql_query(
            "SELECT name, description, content FROM bot_skills
             WHERE bot_id = $1 AND enabled = TRUE
               AND (LOWER(name) LIKE $2
                    OR LOWER(COALESCE(description, '')) LIKE $2
                    OR LOWER(content) LIKE $2)
             ORDER BY name ASC LIMIT 5",
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .bind::<diesel::sql_types::Text, _>(format!("%{term}%"))
        .load(&mut conn),
        None => diesel::sql_query(
            "SELECT name, description, content FROM bot_skills
             WHERE bot_id = $1 AND enabled = TRUE ORDER BY name ASC",
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .load(&mut conn),
    }
    .unwrap_or_default();
    json!({
        "count": rows.len(),
        "skills": rows
            .iter()
            .map(|row| json!({
                "name": row.name,
                "description": row.description,
                "content": row.content,
            }))
            .collect::<Vec<_>>(),
    })
}
