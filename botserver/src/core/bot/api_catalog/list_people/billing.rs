use super::super::*;

/// Lists invoices (branch-scoped, read-only).
pub(super) async fn list_invoices(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<InvoiceRow> = diesel::sql_query(
        "SELECT id AS invoice_id, invoice_number, customer_name, status, total FROM billing_invoices \
         WHERE branch_id = $1 ORDER BY issue_date DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "invoice_id": r.invoice_id,
                "invoice_number": r.invoice_number,
                "customer_name": r.customer_name,
                "status": r.status,
                "total": r.total.to_string(),
                "deep_link": format!("app://billing?invoice_id={}", r.invoice_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "invoices": items }))
}

/// Lists payments (branch-scoped, read-only).
pub(super) async fn list_payments(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<PaymentRow> = diesel::sql_query(
        "SELECT id AS payment_id, payment_number, payer_name, status, amount, invoice_id \
         FROM billing_payments WHERE branch_id = $1 ORDER BY paid_at DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "payment_id": r.payment_id,
                "payment_number": r.payment_number,
                "payer_name": r.payer_name.unwrap_or_default(),
                "status": r.status,
                "amount": r.amount.to_string(),
                "invoice_id": r.invoice_id,
                "deep_link": "app://billing?view=payments",
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "payments": items }))
}

/// Lists quotes (branch-scoped, read-only).
pub(super) async fn list_quotes(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<QuoteRow> = diesel::sql_query(
        "SELECT id AS quote_id, quote_number, customer_name, status, total FROM billing_quotes \
         WHERE branch_id = $1 ORDER BY issue_date DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "quote_id": r.quote_id,
                "quote_number": r.quote_number,
                "customer_name": r.customer_name,
                "status": r.status,
                "total": r.total.to_string(),
                "deep_link": format!("app://billing?view=quotes&quote_id={}", r.quote_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "quotes": items }))
}
