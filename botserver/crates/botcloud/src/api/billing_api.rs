use super::*;


// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct TopupBody {
    pub org_id: Uuid,
    pub amount: f64,
    pub email: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Services — Cancel
// ─────────────────────────────────────────────────────────────────────────────

/// `POST /api/cloud/services/{id}/cancel`
pub(crate) async fn cancel_service(
    State(_service): State<Arc<SaasService>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("Service cancellation requested: {id}");
    Ok(Json(serde_json::json!({
        "status": "cancelled",
        "service_id": id,
        "message": "Service cancellation initiated. You will receive a confirmation email shortly."
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Store — Purchase
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct StorePurchaseBody {
    pub item_id: String,
    pub email: String,
    pub org_id: Option<Uuid>,
}

/// Pure tenant-scope predicate for store purchases (#2666).
///
/// The client-supplied `org_id` is never used to choose the write target: it
/// is accepted only when absent or exactly equal to the caller's trusted
/// branch id (resolved server-side by `get_bot_context`). A different value is
/// a cross-tenant forgery and is rejected with 403 so the write fails closed.
fn tenant_scope_allows(caller_branch_id: Uuid, requested_org_id: Option<Uuid>) -> bool {
    match requested_org_id {
        None => true,
        Some(requested) => requested == caller_branch_id,
    }
}

/// `POST /api/cloud/store/purchase`
pub(crate) async fn handle_store_purchase(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<StorePurchaseBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    // #2666: a request body must never redirect the write into another tenant.
    // A forged `org_id` is rejected; the invoice always uses the trusted,
    // server-resolved branch.
    if !tenant_scope_allows(branch_id, body.org_id) {
        return Err((
            StatusCode::FORBIDDEN,
            "Target organization is outside the authenticated tenant scope".to_string(),
        ));
    }
    let now = chrono::Utc::now();
    let zero = bigdecimal::BigDecimal::from(0);

    let invoice_id = Uuid::new_v4();
    let invoice_num = botbilling::api_models::generate_invoice_number(&mut conn, branch_id);

    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, first_name, last_name};
    let contact_name = crm_contacts
        .filter(email.eq(&body.email))
        .select((first_name, last_name))
        .first::<(Option<String>, Option<String>)>(&mut conn)
        .map(|(fn_, ln_)| [fn_.unwrap_or_else(String::new), ln_.unwrap_or_else(String::new)].join(" "))
        .map(|s| if s.trim().is_empty() { body.email.split('@').next().unwrap_or("Customer").to_string() } else { s })
        .unwrap_or_else(|_| body.email.split('@').next().unwrap_or("Customer").to_string());

    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values((
            botbilling::schema::billing_invoices::id.eq(invoice_id),
            botbilling::schema::billing_invoices::branch_id.eq(branch_id),
            botbilling::schema::billing_invoices::invoice_number.eq(&invoice_num),
            botbilling::schema::billing_invoices::customer_name.eq(&contact_name),
            botbilling::schema::billing_invoices::customer_email.eq(Some(body.email)),
            botbilling::schema::billing_invoices::status.eq("draft"),
            botbilling::schema::billing_invoices::issue_date.eq(now.date_naive()),
            botbilling::schema::billing_invoices::due_date.eq((now + chrono::Duration::days(30)).date_naive()),
            botbilling::schema::billing_invoices::subtotal.eq(&zero),
            botbilling::schema::billing_invoices::total.eq(&zero),
            botbilling::schema::billing_invoices::amount_due.eq(&zero),
            botbilling::schema::billing_invoices::currency.eq("usd"),
            botbilling::schema::billing_invoices::notes.eq(Some(format!("Store purchase: {}", body.item_id))),
            botbilling::schema::billing_invoices::created_at.eq(now),
            botbilling::schema::billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "created",
        "invoice_id": invoice_id,
        "invoice_number": invoice_num,
        "item_id": body.item_id,
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Billing Portal
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/billing-portal`
pub(crate) async fn billing_portal(
    State(service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let portal_url = format!("{}/api/billing/portal", service.config.base_url);
    Ok(Json(serde_json::json!({
        "url": portal_url,
        "message": "Redirect to Stripe Customer Portal to manage payment methods and invoices."
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// App Store Publishing Consultancy
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AppStorePurchaseBody {
    pub store: String,
    pub amount: f64,
    pub email: String,
    pub description: Option<String>,
}

/// `POST /api/cloud/appstore/purchase`
///
/// Creates an invoice for the app store publishing consultancy service.
/// Payment is processed through the existing checkout flow.
pub(crate) async fn handle_appstore_purchase(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<AppStorePurchaseBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Connection: {e}")))?;

    let branch_id = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    let effective_branch_id = if branch_id == Uuid::nil() { Uuid::nil() } else { branch_id };
    let now = chrono::Utc::now();

    use std::str::FromStr;
    let decimal_amount = bigdecimal::BigDecimal::from_str(&format!("{:.2}", body.amount))
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid amount: {e}")))?;
    let zero = bigdecimal::BigDecimal::from(0);

    use rand::Rng;
    let mut rng = rand::rng();
    let num: u32 = rng.random_range(100_000..999_999);
    let invoice_num = format!("INV-APPSTORE-{}", num);

    let desc = body.description.clone().unwrap_or_else(|| format!("App Store Publishing: {}", body.store));

    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, first_name, last_name};
    let contact_name = crm_contacts
        .filter(email.eq(&body.email))
        .select((first_name, last_name))
        .first::<(Option<String>, Option<String>)>(&mut conn)
        .map(|(fn_, ln_)| [fn_.unwrap_or_else(String::new), ln_.unwrap_or_else(String::new)].join(" "))
        .map(|s| if s.trim().is_empty() { body.email.split('@').next().unwrap_or("Customer").to_string() } else { s })
        .unwrap_or_else(|_| body.email.split('@').next().unwrap_or("Customer").to_string());

    let invoice_id = Uuid::new_v4();

    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values((
            botbilling::schema::billing_invoices::id.eq(invoice_id),
            botbilling::schema::billing_invoices::branch_id.eq(effective_branch_id),
            botbilling::schema::billing_invoices::invoice_number.eq(&invoice_num),
            botbilling::schema::billing_invoices::invoice_number.eq(&invoice_num),
            botbilling::schema::billing_invoices::customer_name.eq(&contact_name),
            botbilling::schema::billing_invoices::customer_email.eq(Some(body.email.clone())),
            botbilling::schema::billing_invoices::status.eq("draft"),
            botbilling::schema::billing_invoices::issue_date.eq(now.date_naive()),
            botbilling::schema::billing_invoices::due_date.eq((now + chrono::Duration::days(30)).date_naive()),
            botbilling::schema::billing_invoices::subtotal.eq(&decimal_amount),
            botbilling::schema::billing_invoices::tax_rate.eq(&zero),
            botbilling::schema::billing_invoices::tax_amount.eq(&zero),
            botbilling::schema::billing_invoices::discount_percent.eq(&zero),
            botbilling::schema::billing_invoices::discount_amount.eq(&zero),
            botbilling::schema::billing_invoices::total.eq(&decimal_amount),
            botbilling::schema::billing_invoices::amount_paid.eq(&zero),
            botbilling::schema::billing_invoices::amount_due.eq(&decimal_amount),
            botbilling::schema::billing_invoices::currency.eq("usd"),
            botbilling::schema::billing_invoices::notes.eq(Some(format!("App Store Publishing Consultancy - {} - {}", body.store, desc))),
            botbilling::schema::billing_invoices::created_at.eq(now),
            botbilling::schema::billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create invoice: {e}")))?;

    let line_item = botbilling::api_models::BillingInvoiceItem {
        id: Uuid::new_v4(), invoice_id,
        product_id: None,
        description: desc,
        quantity: botbilling::api_models::bd(1.0),
        unit_price: decimal_amount.clone(),
        discount_percent: zero.clone(),
        tax_rate: zero.clone(),
        amount: decimal_amount.clone(),
        sort_order: 0, created_at: now,
    };

    diesel::insert_into(botbilling::schema::billing_invoice_items::table)
        .values(&line_item)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert item: {e}")))?;

    notifier::notify_invoice_created(&notifier::EmailVars::new(
        &contact_name, &body.email, "appstore-publishing", body.amount, "USD",
    ));

    tracing::info!("App Store publishing invoice created: {invoice_num} for {store} - ${amount}", store=body.store, amount=body.amount);

    Ok(Json(serde_json::json!({
        "status": "ok",
        "invoice_id": invoice_id,
        "invoice_number": invoice_num,
        "amount": decimal_amount.to_string(),
        "store": body.store,
        "customer": contact_name,
    })))
}

/// `GET /api/cloud/offers`
pub(crate) async fn list_offers() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let offers = serde_json::json!([
        {
            "id": "shared-solo",
            "name": "Shared Solo",
            "description": "Standard Shared subscription with 5 workspaces, 5 organizations and 50GB shared storage.",
            "base": "shared",
            "addons": [],
            "monthly_price": 3.99,
            "original_price": 3.99,
            "savings_percent": 0,
            "highlight": false,
        },
        {
            "id": "shared-domain",
            "name": "Shared + Domain",
            "description": "Shared + 1 .com domain. Ideal for professional web presence.",
            "base": "shared",
            "addons": ["domain_com"],
            "monthly_price": 5.82,
            "original_price": 5.82,
            "savings_percent": 0,
            "highlight": false,
        },
        {
            "id": "shared-storage",
            "name": "Shared + 50GB",
            "description": "Shared + 50GB extra storage. Perfect for bots with many documents and files.",
            "base": "shared",
            "addons": ["storage_50gb"],
            "monthly_price": 12.49,
            "original_price": 13.98,
            "savings_percent": 11,
            "highlight": false,
        },
        {
            "id": "shared-phone",
            "name": "Shared + Telefone",
            "description": "Shared + 1 local number. Connect your bot to phone with SMS and calls.",
            "base": "shared",
            "addons": ["local_number"],
            "monthly_price": 8.99,
            "original_price": 9.98,
            "savings_percent": 10,
            "highlight": false,
        },
        {
            "id": "shared-domain-storage",
            "name": "Shared + Domain + 50GB",
            "description": "The essential combo: professional domain + extra storage + Shared.",
            "base": "shared",
            "addons": ["domain_com", "storage_50gb"],
            "monthly_price": 15.81,
            "original_price": 15.81,
            "savings_percent": 0,
            "highlight": false,
        },
        {
            "id": "shared-phone-storage",
            "name": "Shared + Telefone + 50GB",
            "description": "Complete communication: phone + storage + Shared.",
            "base": "shared",
            "addons": ["local_number", "storage_50gb"],
            "monthly_price": 17.49,
            "original_price": 19.97,
            "savings_percent": 12,
            "highlight": false,
        },

    ]);
    Ok(Json(serde_json::json!({ "offers": offers })))
}

#[cfg(test)]
mod store_purchase_tenant_scope_tests {
    use super::*;

    #[test]
    fn no_explicit_target_is_allowed() {
        assert!(tenant_scope_allows(Uuid::new_v4(), None));
    }

    #[test]
    fn target_equal_to_caller_branch_is_allowed() {
        let branch = Uuid::new_v4();
        assert!(tenant_scope_allows(branch, Some(branch)));
    }

    #[test]
    fn forged_foreign_branch_is_rejected() {
        // Two-tenant case: attacker branch must not write into victim branch.
        let attacker_branch = Uuid::new_v4();
        let victim_branch = Uuid::new_v4();
        assert!(!tenant_scope_allows(attacker_branch, Some(victim_branch)));
    }

    #[test]
    fn explicit_target_is_rejected_when_caller_is_unresolved() {
        // Fail closed: an unresolved (nil) caller scope accepts no target.
        assert!(!tenant_scope_allows(Uuid::nil(), Some(Uuid::new_v4())));
    }
}
