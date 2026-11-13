use super::*;
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
/// Lists products/services (branch-scoped, read-only).
pub(super) async fn list_products(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    category: Option<&str>,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    // A suite/default bot often has no tenant branch catalog; fall back to the
    // global (nil) catalog so chat lists the same seeded products the Store/HTTP
    // reads (suite token -> nil branch) instead of answering empty (#2204).
    let nil = Uuid::nil();
    let rows: Vec<ProductRow> = match category.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(cat) => diesel::sql_query(
            "SELECT id AS product_id, sku, name, price, product_type, stock_quantity FROM products \
             WHERE (branch_id = $1 OR ($1 <> $3 AND branch_id = $3)) AND category = $2 \
             ORDER BY (branch_id = $1) DESC, name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Text, _>(cat)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
        None => diesel::sql_query(
            "SELECT id AS product_id, sku, name, price, product_type, stock_quantity FROM products \
             WHERE branch_id = $1 OR ($1 <> $2 AND branch_id = $2) \
             ORDER BY (branch_id = $1) DESC, name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
    };
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "product_id": r.product_id,
                "sku": r.sku.unwrap_or_default(),
                "name": r.name,
                "price": r.price.to_string(),
                "product_type": r.product_type,
                "stock_quantity": r.stock_quantity,
                "deep_link": format!("app://products?product_id={}", r.product_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "products": items }))
}
/// Lists support tickets (branch-scoped, read-only).
pub(super) async fn list_tickets(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
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
/// Drive location for the bot a catalog command runs as.
///
/// The bucket cannot be derived from the bot name: an org-hosted bot lives in
/// `{slug}.gborg` with the bot directory as a key prefix, while a standalone
/// bot owns `{bot}.gbai`. Every `drive.*` command goes through this so the
/// chat side addresses exactly the Drive the UI and the file keywords use.
pub(super) fn catalog_drive_location(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
) -> Result<botbasic_core::utils::BotDriveLocation, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    Ok(botbasic_core::utils::bot_drive_location_for(
        &mut conn, *bot_uuid,
    ))
}
/// Lists drive files under a folder (read-only, via the configured S3 repository).
pub(super) async fn list_drive_files(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    path: Option<&str>,
    query: Option<&str>,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let loc = catalog_drive_location(state, bot_uuid)?;
    let bucket = loc.bucket.clone();
    let rel = normalize_drive_path(path.unwrap_or(""))?;
    let prefix = if rel.is_empty() {
        loc.key_prefix.clone()
    } else {
        loc.key_for(&rel)
    };
    let mut keys = drive
        .list_objects_with_metadata(&bucket, Some(&prefix))
        .await
        .map_err(|e| format!("drive list error: {e}"))?;
    if let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) {
        let needle = q.to_ascii_lowercase();
        keys.retain(|o| {
            let key = o.key.to_ascii_lowercase();
            let name = key.rsplit('/').next().unwrap_or(&key);
            needle
                .split_whitespace()
                .filter(|t| !t.is_empty())
                .all(|token| key.contains(token) || name.contains(token))
        });
    }
    keys.sort_by(|a, b| b.key.cmp(&a.key));
    let items: Vec<Value> = keys
        .into_iter()
        .take(100)
        .map(|o| {
            let ext = o.key.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
            let is_doc = matches!(
                ext.as_str(),
                "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "markdown" | "pdf" | "html" | "htm"
            );
            let file = urlencoding::encode(&o.key);
            let bucket_enc = urlencoding::encode(&bucket);
            json!({
                "path": o.key,
                "size": o.size,
                "etag": o.etag,
                "drive_link": format!("app://drive?path={file}&scope=bot&bucket={bucket_enc}"),
                "docs_link": if is_doc {
                    Value::String(format!("app://docs?file={file}&bucket={bucket_enc}"))
                } else {
                    Value::Null
                }
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "bucket": bucket, "files": items }))
}
#[cfg(all(feature = "banking", feature = "products", feature = "tax"))]
pub(super) async fn execute_tax(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    service: Option<String>,
    value: Option<String>,
) -> Result<Value, String> {
    let scope = botbanking::cashflow::resolve_bot_scope(&state.conn, bot_uuid, None)
        .ok_or_else(|| "bot not found".to_string())?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let result = match service {
        Some(service) => {
            botproducts::service_tax::calculate_service_tax_inner(
                &mut conn,
                scope.branch_id,
                &service,
                None,
                value.as_deref(),
            )
            .await
        }
        None => calculate_anonymous_service_tax(&mut conn, &scope.branch_id, value.as_deref()),
    };
    match result {
        Ok(v) => Ok(v),
        // The inner calculation returns a stable catalogue key (#2094); the
        // chat path has no request locale, so the default one is used.
        Err((_code, key)) => Err(botcore::i18n::t(&botcore::i18n::Locale::default(), key)),
    }
}
#[cfg(not(all(feature = "banking", feature = "products", feature = "tax")))]
pub(super) async fn execute_tax(
    _state: &Arc<AppState>,
    _bot_uuid: &Uuid,
    _service: Option<String>,
    _value: Option<String>,
) -> Result<Value, String> {
    Err("tax calculation requires banking, products and tax features".to_string())
}
#[cfg(all(feature = "banking", feature = "products", feature = "tax"))]
pub(super) fn calculate_anonymous_service_tax(
    conn: &mut diesel::PgConnection,
    branch_id: &Uuid,
    value: Option<&str>,
) -> Result<serde_json::Value, (axum::http::StatusCode, &'static str)> {
    use rust_decimal::Decimal;
    use std::str::FromStr as _;
    let service_value = value
        .map(Decimal::from_str)
        .transpose()
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "error-invalid-value"))?
        .ok_or_else(|| (axum::http::StatusCode::BAD_REQUEST, "error-value-required"))?;
    let loaded = bottax::storage::load_rates_from_billing(conn, branch_id);
    let rates = loaded.unwrap_or_default();
    let source = if loaded.is_some() {
        "billing_tax_rates"
    } else {
        "default"
    };
    let breakdown = bottax::calculator::calculate_service_tax(service_value, &rates);
    if let Err((_code, msg)) =
        bottax::handlers::persist_calculation(conn, branch_id, None, None, &breakdown, source)
    {
        log::warn!("service.tax audit insert skipped: {msg}");
    }
    Ok(serde_json::json!({
        "service": null,
        "service_value": breakdown.service_value.to_string(),
        "breakdown": breakdown,
        "rates": {
            "irpj_pct": rates.irpj_pct.to_string(),
            "csll_pct": rates.csll_pct.to_string(),
            "pis_cofins_pct": rates.pis_cofins_pct.to_string(),
            "iss_pct": rates.iss_pct.to_string(),
        },
        "rate_source": source,
    }))
}
pub(super) async fn write_drive_file(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    path: &str,
    content_b64: &str,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let data = base64::engine::general_purpose::STANDARD
        .decode(content_b64)
        .map_err(|e| format!("invalid base64 content: {e}"))?;
    let loc = catalog_drive_location(state, bot_uuid)?;
    let bucket = loc.bucket.clone();
    let key = loc.key_for(&normalize_drive_path(path)?);
    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("api_call drive.write -> {bucket}/{key}");
    Ok(json!({ "success": true, "bucket": bucket, "path": key }))
}
/// Copies a file from one drive path to another (e.g. an attached invoice
/// from `inbox/` into its `faturas/<month>/` folder).
pub(super) async fn move_drive_file(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    from: &str,
    to: &str,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let loc = catalog_drive_location(state, bot_uuid)?;
    let bucket = loc.bucket.clone();
    let from_key = loc.key_for(&normalize_drive_path(from)?);
    let to_key = loc.key_for(&normalize_drive_path(to)?);
    let bytes = drive
        .get_object(&bucket, &from_key)
        .await
        .map_err(|e| format!("not found at {from_key}: {e}"))?;
    drive
        .put_object(&bucket, &to_key, bytes, None)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("api_call drive.file -> {bucket}/{to_key}");
    Ok(json!({ "success": true, "bucket": bucket, "from": from_key, "to": to_key }))
}

/// Creates a product in the catalog (chat command, branch-scoped).
pub(super) async fn create_product_item(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "params.name is required".to_string())?;
    let price = params
        .get("price")
        .and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .ok_or_else(|| "params.price is required".to_string())?;
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let sku = params
        .get("sku")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let description = params
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let category = params
        .get("category")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let currency = params
        .get("currency")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("USD");
    let product_type = params
        .get("product_type")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("physical");
    let unit = params
        .get("unit")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("unit");
    let stock_quantity = params
        .get("stock_quantity")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let cost = params.get("cost").and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    });

    diesel::sql_query(
        "INSERT INTO products (id, org_id, bot_id, sku, name, description, category, product_type, \
         price, cost, currency, tax_rate, unit, stock_quantity, low_stock_threshold, is_active, \
         images, attributes, created_at, updated_at, branch_id, category_id, is_public) \
         VALUES ($1, NULL, NULL, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9, 0, $10, $11, 10, \
         true, '[]'::jsonb, '{}'::jsonb, $12, $12, $13, NULL, false)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(sku.map(|s| s.to_string()))
    .bind::<diesel::sql_types::Text, _>(name.to_string())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description.map(|s| s.to_string()))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(category.map(|s| s.to_string()))
    .bind::<diesel::sql_types::Text, _>(product_type.to_string())
    .bind::<diesel::sql_types::Text, _>(price.to_string())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(cost.map(|c| c.to_string()))
    .bind::<diesel::sql_types::Text, _>(currency.to_string())
    .bind::<diesel::sql_types::Text, _>(unit.to_string())
    .bind::<diesel::sql_types::Int4, _>(stock_quantity)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(|e| format!("Insert error: {e}"))?;

    Ok(json!({
        "success": true,
        "product_id": id,
        "name": name,
        "price": price,
        "deep_link": format!("app://products?product_id={id}"),
    }))
}

/// Updates a product in the catalog (chat command, branch-scoped).
pub(super) async fn update_product_item(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let nil = Uuid::nil();
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let product_id = params
        .get("product_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s.trim()).ok());
    let lookup_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let target_id: Uuid = if let Some(pid) = product_id {
        pid
    } else if let Some(ref n) = lookup_name {
        // Prefer exact/ilike match in bot branch, then global nil branch.
        let pattern = format!("%{n}%");
        let row: Option<IdRow> = diesel::sql_query(
            "SELECT id FROM products \
             WHERE (branch_id = $1 OR ($1 <> $2 AND branch_id = $2)) AND name ILIKE $3 \
             ORDER BY (branch_id = $1) DESC, (lower(name) = lower($4)) DESC, name ASC LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .bind::<diesel::sql_types::Text, _>(pattern)
        .bind::<diesel::sql_types::Text, _>(n.clone())
        .get_result(&mut conn)
        .optional()
        .map_err(|e| format!("Query error: {e}"))?;
        row.map(|r| r.id)
            .ok_or_else(|| format!("product not found: {n}"))?
    } else {
        return Err("params.product_id or params.name is required".to_string());
    };

    let now = chrono::Utc::now();
    let mut updates = 0_i64;

    if let Some(price) = params.get("price").and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    }) {
        updates += diesel::sql_query(
            "UPDATE products SET price = $1::numeric, updated_at = $2 \
             WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
        )
        .bind::<diesel::sql_types::Text, _>(price.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))? as i64;
    }
    if let Some(desc) = params.get("description").and_then(|v| v.as_str()) {
        updates += diesel::sql_query(
            "UPDATE products SET description = $1, updated_at = $2 \
             WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
        )
        .bind::<diesel::sql_types::Text, _>(desc.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))? as i64;
    }
    if let Some(active) = params.get("is_active").and_then(|v| v.as_bool()) {
        updates += diesel::sql_query(
            "UPDATE products SET is_active = $1, updated_at = $2 \
             WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
        )
        .bind::<diesel::sql_types::Bool, _>(active)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))? as i64;
    }
    // Rename: if name provided together with product_id, treat as new name.
    if product_id.is_some() {
        if let Some(new_name) = params
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            updates += diesel::sql_query(
                "UPDATE products SET name = $1, updated_at = $2 \
                 WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
            )
            .bind::<diesel::sql_types::Text, _>(new_name.to_string())
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Uuid, _>(target_id)
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .bind::<diesel::sql_types::Uuid, _>(nil)
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))? as i64;
        }
    }

    if updates == 0 {
        // Touch updated_at to confirm row exists/reachable when only lookup name matched without field changes.
        let touched = diesel::sql_query(
            "UPDATE products SET updated_at = $1 WHERE id = $2 AND (branch_id = $3 OR branch_id = $4)",
        )
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))?;
        if touched == 0 {
            return Err("product not found or not in scope".to_string());
        }
    }

    Ok(json!({
        "success": true,
        "product_id": target_id,
        "deep_link": format!("app://products?product_id={target_id}"),
    }))
}
