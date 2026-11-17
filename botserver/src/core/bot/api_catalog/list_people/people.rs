use super::super::*;
use diesel::OptionalExtension;

/// Lists CRM/people contacts (read-only), scoped to the bot's branch.
pub(super) async fn list_people(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    search: Option<&str>,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let term = search
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<PersonRow> = match &term {
        Some(t) => {
            let pattern = format!("%{t}%");
            diesel::sql_query(
                "SELECT id AS person_id, first_name, last_name, email, status FROM crm_contacts \
                 WHERE branch_id = $1 AND (first_name ILIKE $2 OR last_name ILIKE $2 OR email ILIKE $2) \
                 ORDER BY last_name ASC LIMIT 25",
            )
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .bind::<diesel::sql_types::Text, _>(pattern)
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?
        }
        None => diesel::sql_query(
            "SELECT id AS person_id, first_name, last_name, email, status FROM crm_contacts \
             WHERE branch_id = $1 ORDER BY last_name ASC LIMIT 25",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
    };
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            let name = format!(
                "{} {}",
                r.first_name.unwrap_or_default(),
                r.last_name.unwrap_or_default()
            )
            .trim()
            .to_string();
            json!({
                "person_id": r.person_id,
                "name": if name.is_empty() { "Unnamed".to_string() } else { name },
                "email": r.email.unwrap_or_default(),
                "status": r.status,
                "deep_link": format!("app://people?person_id={}", r.person_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "people": items }))
}
