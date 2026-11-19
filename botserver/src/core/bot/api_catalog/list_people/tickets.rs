use super::super::*;

/// Lists support tickets (branch-scoped, read-only).
pub(in crate::core::bot::api_catalog) async fn list_tickets(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<TicketRow> = diesel::sql_query(
        "SELECT id AS ticket_id, ticket_number, subject, status, priority FROM support_tickets \
         WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "ticket_id": r.ticket_id,
                "ticket_number": r.ticket_number,
                "subject": r.subject,
                "status": r.status,
                "priority": r.priority,
                "deep_link": format!("app://tickets?ticket_id={}", r.ticket_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "tickets": items }))
}
