use super::*;


/// `GET /api/cloud/admin/server-capacity`
///
/// Returns real-time server capacity metrics for SaaS admin dashboard.
pub(crate) async fn get_server_capacity(
    State(_service): State<Arc<SaasService>>,
) -> Json<serde_json::Value> {
    let capacity = botbilling::server_capacity::calculate_server_capacity(
        &botbilling::server_capacity::ServerCapacityConfig::default(),
        0, 0,
    );
    Json(serde_json::json!({
        "server": {
            "cpu_cores": capacity.cpu_cores,
            "cpu_usage_pct": (capacity.cpu_usage_pct * 100.0).round() / 100.0,
            "ram_total_gb": (capacity.ram_total_gb * 100.0).round() / 100.0,
            "ram_used_gb": (capacity.ram_used_gb * 100.0).round() / 100.0,
            "ram_available_gb": (capacity.ram_available_gb * 100.0).round() / 100.0,
        },
        "saas_capacity": {
            "available_free_slots": capacity.available_free_slots,
            "available_shared_slots": capacity.available_shared_slots,
            "new_signups_allowed": capacity.new_signups_allowed,
            "capacity_health": capacity.capacity_health,
            "pressure_index": (capacity.pressure_index * 100.0).round() / 100.0,
        },
    }))
}

/// #2669 — the only source of persisted checkout amounts: the server catalog.
/// Client payload values are deliberately not parameters of this function.
fn server_price_for_plan(plan: &str) -> Option<(f64, String)> {
    let config = botbilling::default_product_config();
    let plan = config.plans.get(plan)?;
    let price = match &plan.price {
        botbilling::PlanPrice::Fixed { amount, currency, .. } => {
            (*amount as f64 / 100.0, currency.clone())
        }
        botbilling::PlanPrice::Free
        | botbilling::PlanPrice::Custom => (0.0, "usd".to_string()),
    };
    Some(price)
}

/// `POST /api/cloud/checkout`
///
/// Creates invoice in billing + contact/deal in CRM + Stripe session.
pub(crate) async fn handle_checkout(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<CheckoutBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload: CalculatorPayload = serde_json::from_str(&body.payload)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid payload: {e}")))?;

    let billing = &service.billing_state;
    let mut conn = billing.pool.get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id = botbilling::get_bot_context(&billing.pool, &billing.get_default_bot);
    let now = chrono::Utc::now();

    let effective_branch_id = if branch_id == Uuid::nil() {
        Uuid::nil()
    } else {
        branch_id
    };

    let customer_email = body.email.clone();
    let customer_name = body.organization_name.clone()
        .unwrap_or_else(|| format!("{} Customer", payload.plan));

    // #2669 — resolve the plan from the server catalog FIRST. `payload.total`
    // and `payload.currency` are client-controlled and must never reach an
    // invoice, GL entry or subscription amount. Stripe charges by `price_id`
    // (= payload.plan), which this same catalog lookup validates.
    let (total_value, server_currency) = server_price_for_plan(&payload.plan)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Plan '{}' not found", payload.plan)))?;
    let invoice_id = Uuid::new_v4();
    let invoice_number = botbilling::api_models::generate_invoice_number(&mut conn, effective_branch_id);

    let (org_id, bot_id) = botbilling::scope_for_branch(&mut conn, effective_branch_id);

    let invoice = botbilling::api_models::BillingInvoice {
        id: invoice_id, branch_id: effective_branch_id, invoice_number,
        customer_id: None,
        customer_name: customer_name.clone(),
        customer_email: Some(customer_email.clone()),
        customer_address: None, status: "draft".to_string(),
        issue_date: now.date_naive(),
        due_date: (now + chrono::Duration::days(30)).date_naive(),
        subtotal: botbilling::api_models::bd(total_value),
        tax_rate: botbilling::api_models::bd(0.0),
        tax_amount: botbilling::api_models::bd(0.0),
        discount_percent: botbilling::api_models::bd(0.0),
        discount_amount: botbilling::api_models::bd(0.0),
        total: botbilling::api_models::bd(total_value),
        amount_paid: botbilling::api_models::bd(0.0),
        amount_due: botbilling::api_models::bd(total_value),
        currency: server_currency.clone(),
        notes: Some(format!("SaaS: plan={}, period={}, storage={}GB", payload.plan, payload.period, payload.storage)),
        terms: None, footer: None, paid_at: None, sent_at: None, voided_at: None,
        created_at: now, updated_at: now,
        org_id, bot_id,
    };

    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values(&invoice)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert: {e}")))?;

    let line_item = botbilling::api_models::BillingInvoiceItem {
        id: Uuid::new_v4(), invoice_id,
        product_id: None,
        description: format!("{} - {} ({}GB, AI: {:?})", payload.plan, payload.period, payload.storage, payload.ai),
        quantity: botbilling::api_models::bd(1.0),
        unit_price: botbilling::api_models::bd(total_value),
        discount_percent: botbilling::api_models::bd(0.0),
        tax_rate: botbilling::api_models::bd(0.0),
        amount: botbilling::api_models::bd(total_value),
        sort_order: 0, created_at: now,
    };

    diesel::insert_into(botbilling::schema::billing_invoice_items::table)
        .values(&line_item)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert item: {e}")))?;

    // --- Notification: invoice generated ---
    let mut vars = notifier::EmailVars::new(
        &customer_name, &customer_email, &payload.plan, total_value, &server_currency,
    )
    .with_locale(notifier::recipient_locale(body.locale.as_deref()));
    vars.invoice_id = invoice_id.to_string();
    notifier::notify_invoice_created(&vars);

    // --- CRM integration: creates contact and deal ---
    let contact_id = integration::create_crm_contact(
        service.pool(),
        effective_branch_id,
        &customer_name,
        &customer_email,
        None,  // checkout: no password set
    )
    .map_err(|e| {
        tracing::warn!("CRM contact creation failed (non-fatal): {e}");
        Uuid::nil()
    });

    let _deal_id = integration::create_crm_deal(
        service.pool(),
        effective_branch_id,
        contact_id.unwrap_or(Uuid::nil()),
        invoice_id,
        &format!("Assinatura {} - {}", payload.plan, customer_name),
        total_value,
        &server_currency,
    )
    .map_err(|e| {
        tracing::warn!("CRM deal creation failed (non-fatal): {e}");
        Uuid::nil()
    });

    // --- Stripe ---
    let stripe_customer = service.stripe
        .create_customer(botbilling::stripe_integration::CreateCustomerParams {
            email: customer_email.clone(),
            name: Some(customer_name.clone()),
            organization_id: invoice_id,
            metadata: std::collections::HashMap::new(),
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe customer: {e}")))?;

    // Persist the Stripe Customer mapping so SetupIntent card management and
    // webhook events can resolve the owning branch later.
    crate::payment_cards::persist_customer_mapping(
        &service,
        effective_branch_id,
        &stripe_customer.id,
        &customer_email,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe customer mapping: {e}")))?;

    let cancel_url = format!("{}/cloud/checkout/cancel", service.config.base_url);
    // Server catalog, never client input — same lookup that validated the plan.
    let plan = botbilling::default_product_config()
        .plans
        .get(&payload.plan)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Plan '{}' not found", payload.plan)))?
        .clone();

    let session = service.stripe
        .create_checkout_session(
            botbilling::stripe_integration::CreateCheckoutSessionParams {
                customer_id: stripe_customer.id,
                price_id: payload.plan.clone(),
                success_url: format!("{}/cloud/checkout/success?session_id={{CHECKOUT_SESSION_ID}}&invoice={}", service.config.base_url, invoice_id),
                cancel_url,
                trial_days: plan.trial_days,
                metadata: std::collections::HashMap::from([
                    ("invoice_id".to_string(), invoice_id.to_string()),
                    ("plan".to_string(), payload.plan.clone()),
                    ("account_email".to_string(), customer_email.clone()),
                ]),
            },
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe session: {e}")))?;

    Ok(Json(serde_json::json!({
        "checkout_url": session.url,
        "session_id": session.id,
        "invoice_id": invoice_id,
        "status": "redirecting",
    })))
}


/// `GET /api/cloud/plans`
pub(crate) async fn list_plans(
    State(_service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let config = botbilling::default_product_config();
    let mut plans = serde_json::Map::new();

    for (id, plan) in &config.plans {
        let price = match &plan.price {
            botbilling::PlanPrice::Free => serde_json::json!({"type": "free"}),
            botbilling::PlanPrice::Fixed { amount, currency, period } => serde_json::json!({
                "type": "fixed", "amount": amount, "currency": currency, "period": period,
            }),
            botbilling::PlanPrice::Custom => serde_json::json!({"type": "custom"}),
        };

        plans.insert(id.clone(), serde_json::json!({
            "name": plan.name, "description": plan.description,
            "price": price, "features": plan.features,
            "trial_days": plan.trial_days,
            "limits": {
                "messages_per_day": plan.limits.messages_per_day.value(),
                "storage_mb": plan.limits.storage_mb.value(),
                "bots": plan.limits.bots.value(),
                "users": plan.limits.users.value(),
            },
        }));
    }

    Ok(Json(serde_json::json!({ "branding": config.branding, "plans": plans })))
}

/// `GET /api/cloud/plans/{plan_id}`
pub(crate) async fn get_plan_detail(
    State(_service): State<Arc<SaasService>>,
    axum::extract::Path(plan_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let config = botbilling::default_product_config();
    let plan = config.plans.get(&plan_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Plan '{}' not found", plan_id)))?;

    let price = match &plan.price {
        botbilling::PlanPrice::Free => serde_json::json!({"type": "free"}),
        botbilling::PlanPrice::Fixed { amount, currency, period } => serde_json::json!({
            "type": "fixed", "amount": amount, "currency": currency, "period": period,
        }),
        botbilling::PlanPrice::Custom => serde_json::json!({"type": "custom"}),
    };

    Ok(Json(serde_json::json!({
        "id": plan_id,
        "name": plan.name,
        "description": plan.description,
        "price": price,
        "features": plan.features,
        "trial_days": plan.trial_days,
        "limits": {
            "messages_per_day": plan.limits.messages_per_day.value(),
            "storage_mb": plan.limits.storage_mb.value(),
            "bots": plan.limits.bots.value(),
            "users": plan.limits.users.value(),
            "api_calls_per_day": plan.limits.api_calls_per_day.value(),
            "kb_documents": plan.limits.kb_documents.value(),
            "apps": plan.limits.apps.value(),
        },
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Auth: Login (JWT stub — real auth via Zitadel/OIDC in production)
// ─────────────────────────────────────────────────────────────────────────────

/// `POST /api/cloud/auth/login`
///
/// Validates credentials and returns a JWT token for the management portal.
/// #1262 — `resolve_directory_service_token` (boot) only accepts candidates
/// from `directory_config.json` and `conf/directory/admin-pat.txt`. When the
/// config file loses its `service_token` key (observed on prod), login
/// silently skipped the Zitadel password check and every correct password
/// was rejected with "Invalid credentials" and no log line. This fallback
/// reads the canonical Vault secret (`gbo/directory`, `service_token`) so
/// password verification survives a config regression.
pub(crate) fn directory_token_from_vault() -> Option<String> {
    let sm = botcoresecrets::manager::SecretsManager::get_clone().ok()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move {
                sm.get_secret(botcoresecrets::paths::SecretPaths::DIRECTORY)
                    .await
                    .ok()
                    .and_then(|s| s.get("service_token").cloned())
                    .filter(|t| !t.is_empty())
            })
        } else {
            None
        };
        if let Err(e) = tx.send(result) {
            tracing::warn!("directory token worker: result send failed: {e}");
        }
    });
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .ok()
        .flatten()
}

#[cfg(test)]
mod checkout_price_tests {
    use super::server_price_for_plan;

    #[test]
    fn tampered_client_total_cannot_change_persisted_amount() {
        // The helper takes only a plan id — there is no parameter through which
        // a client-supplied `payload.total` / `payload.currency` could flow.
        let (price, currency) = server_price_for_plan("shared").expect("catalog plan");
        assert_eq!(price, 3.99);
        assert_eq!(currency, "usd");
        assert_eq!(server_price_for_plan("free").map(|p| p.0), Some(0.0));
        assert!(server_price_for_plan("does-not-exist").is_none());
    }
}
