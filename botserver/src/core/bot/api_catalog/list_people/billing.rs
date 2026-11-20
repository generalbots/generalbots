use super::super::*;

/// Lists invoices (branch-scoped, read-only).
pub(in crate::core::bot::api_catalog) async fn list_invoices(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
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

/// Creates an invoice (branch-scoped). #2203 — the catalog only exposed
/// `billing.invoice.list`, so an app-backed money write degraded into "open the
/// New Invoice form and claim the invoice was created".
pub(in crate::core::bot::api_catalog) async fn create_invoice(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let customer = params
        .get("customer")
        .or_else(|| params.get("customer_name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "params.customer is required".to_string())?;
    let amount = params
        .get("amount")
        .or_else(|| params.get("total"))
        .and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
        })
        .ok_or_else(|| "params.amount is required".to_string())?;
    let currency = params
        .get("currency")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("USD");
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    // The invoice row needs org_id, bot_id and branch_id (all NOT NULL).
    // `billing_invoices_org_id_fkey` references branches(id), so the owning
    // branch is the value that satisfies it — verified against the dev DB
    // (bots.org_id is a default uuid that is not a branch and is rejected).
    let org_id: Uuid = branch;
    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let invoice_bot_id: Uuid = match diesel::sql_query(
        "SELECT id FROM bots WHERE branch_id = $1 OR id = $2 \
         ORDER BY (id = $2) DESC, created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Uuid, _>(*bot_uuid)
    .get_result::<BotRow>(&mut conn)
    {
        Ok(row) => row.id,
        Err(e) => {
            log::warn!("billing.invoice.create: no bot row for branch {branch} ({e})");
            *bot_uuid
        }
    };

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }
    let existing: i64 = match diesel::sql_query(
        "SELECT COUNT(*) AS count FROM billing_invoices WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result::<CountRow>(&mut conn)
    {
        Ok(row) => row.count,
        Err(e) => {
            log::warn!("billing.invoice.create: invoice count failed for branch {branch} ({e})");
            0
        }
    };
    let now = chrono::Utc::now();
    let invoice_number = format!("INV-{}{:02}-{:05}", now.format("%Y"), now.format("%m"), existing + 1);
    let issue_date = now.date_naive();
    let due_date = params
        .get("due_date")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok())
        .unwrap_or_else(|| issue_date + chrono::Duration::days(7));
    let id = Uuid::new_v4();
    let customer_email = params.get("customer_email").and_then(|v| v.as_str());
    let notes = params.get("notes").and_then(|v| v.as_str());

    diesel::sql_query(
        "INSERT INTO billing_invoices (id, org_id, bot_id, branch_id, invoice_number, customer_name, \
         customer_email, status, issue_date, due_date, subtotal, tax_rate, tax_amount, discount_percent, \
         discount_amount, total, amount_paid, amount_due, currency, notes, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'draft', $8, $9, $10::numeric, 0, 0, 0, 0, \
         $10::numeric, 0, $10::numeric, $11, $12, $13, $13)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Uuid, _>(invoice_bot_id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(invoice_number.clone())
    .bind::<diesel::sql_types::Text, _>(customer.to_string())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(customer_email.map(str::to_string))
    .bind::<diesel::sql_types::Date, _>(issue_date)
    .bind::<diesel::sql_types::Date, _>(due_date)
    .bind::<diesel::sql_types::Text, _>(amount.to_string())
    .bind::<diesel::sql_types::Text, _>(currency.to_string())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(notes.map(str::to_string))
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Insert error: {e}"))?;

    Ok(json!({
        "success": true,
        "invoice_id": id,
        "invoice_number": invoice_number,
        "customer_name": customer,
        "total": amount,
        "currency": currency,
        "due_date": due_date.to_string(),
        "deep_link": format!("app://billing?invoice_id={id}"),
    }))
}

/// Lists payments (branch-scoped, read-only).
pub(in crate::core::bot::api_catalog) async fn list_payments(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
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
pub(in crate::core::bot::api_catalog) async fn list_quotes(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
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
