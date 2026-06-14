use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{integration, notifier, CalculatorPayload, SaasService};

#[derive(Debug, Deserialize)]
pub struct CheckoutBody {
    pub payload: String,
    pub email: String,
    pub organization_name: Option<String>,
    pub return_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupBody {
    pub email: String,
    pub name: String,
    pub password: Option<String>,
}

pub fn configure_management_api_routes() -> Router<Arc<SaasService>> {
    Router::new()
        .route("/api/management/checkout", post(handle_checkout))
        .route("/api/management/checkout/success", get(checkout_success))
        .route("/api/management/plans", get(list_plans))
        .route("/api/management/plans/{plan_id}", get(get_plan_detail))
        .route("/api/management/auth/signup", post(handle_signup))
}

/// `POST /api/management/auth/signup`
///
/// Cria organização no banco + contato no CRM, retornando os IDs.
async fn handle_signup(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<SignupBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (org_id, bot_id) = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);

    let effective_org_id = if org_id == Uuid::nil() {
        integration::create_organization(service.pool(), &body.name)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
    } else {
        org_id
    };

    let contact_id = integration::create_crm_contact(
        service.pool(),
        effective_org_id,
        bot_id,
        &body.name,
        &body.email,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    notifier::notify_welcome(&notifier::EmailVars::new(
        &body.name, &body.email, "free", 0.0, "USD",
    ));

    Ok(Json(serde_json::json!({
        "status": "ok",
        "account": { "email": body.email, "name": body.name },
        "org_id": effective_org_id,
        "contact_id": contact_id,
        "token": "placeholder-token",
    })))
}

/// `POST /api/management/checkout`
///
/// Cria fatura no billing + contato/deal no CRM + sessão no Stripe.
async fn handle_checkout(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<CheckoutBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload: CalculatorPayload = serde_json::from_str(&body.payload)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid payload: {e}")))?;

    let billing = &service.billing_state;
    let mut conn = billing.pool.get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let (org_id, bot_id) = botbilling::get_bot_context(&billing.pool, &billing.get_default_bot);
    let now = chrono::Utc::now();

    let effective_org_id = if org_id == Uuid::nil() {
        Uuid::nil()
    } else {
        org_id
    };

    let customer_email = body.email.clone();
    let customer_name = body.organization_name.clone()
        .unwrap_or_else(|| format!("{} Customer", &payload.plan));

    let total_cents = (payload.total * 100.0) as u64;
    let total_value = total_cents as f64;
    let invoice_id = Uuid::new_v4();
    let invoice_number = botbilling::api_models::generate_invoice_number(&mut conn, effective_org_id);

    let invoice = botbilling::api_models::BillingInvoice {
        id: invoice_id, org_id: effective_org_id, bot_id, invoice_number,
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
        currency: payload.currency.clone(),
        notes: Some(format!("SaaS: plan={}, period={}, storage={}GB", payload.plan, payload.period, payload.storage)),
        terms: None, footer: None, paid_at: None, sent_at: None, voided_at: None,
        created_at: now, updated_at: now,
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

    // --- Notificação: fatura gerada ---
    let mut vars = notifier::EmailVars::new(
        &customer_name, &customer_email, &payload.plan, total_value, &payload.currency,
    );
    vars.invoice_id = invoice_id.to_string();
    notifier::notify_invoice_created(&vars);

    // --- Integração CRM: cria contato e deal (oportunidade) ---
    let contact_id = integration::create_crm_contact(
        service.pool(),
        effective_org_id,
        bot_id,
        &customer_name,
        &customer_email,
    )
    .map_err(|e| {
        tracing::warn!("CRM contact creation failed (non-fatal): {e}");
        Uuid::nil()
    });

    let _deal_id = integration::create_crm_deal(
        service.pool(),
        effective_org_id,
        bot_id,
        contact_id.unwrap_or(Uuid::nil()),
        invoice_id,
        &format!("Assinatura {} - {}", payload.plan, customer_name),
        total_value,
        &payload.currency,
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

    let plan_config = botbilling::default_product_config();
    let plan = plan_config.plans.get(&payload.plan)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Plan '{}' not found", payload.plan)))?;

    let return_url = body.return_url.clone()
        .unwrap_or_else(|| format!("{}/management/dashboard", service.config.base_url));
    let cancel_url = format!("{}/management/checkout/cancel", service.config.base_url);

    let session = service.stripe
        .create_checkout_session(
            botbilling::stripe_integration::CreateCheckoutSessionParams {
                customer_id: stripe_customer.id,
                price_id: payload.plan.clone(),
                success_url: format!("{}/management/checkout/success?session_id={{CHECKOUT_SESSION_ID}}&invoice={}", service.config.base_url, invoice_id),
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

/// `GET /api/management/checkout/success`
///
/// Confirma pagamento Stripe, atualiza fatura, cria GL entry + subscription.
async fn checkout_success(
    State(service): State<Arc<SaasService>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let session_id = params.get("session_id")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing session_id".to_string()))?;
    let invoice_id = params.get("invoice")
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing invoice".to_string()))?;

    let session = service.stripe
        .retrieve_checkout_session(session_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe: {e}")))?;

    let is_complete = session.status == "complete";
    let now = chrono::Utc::now();

    {
        let mut conn = service.pool().get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

        diesel::update(
            botbilling::schema::billing_invoices::table
                .filter(botbilling::schema::billing_invoices::id.eq(invoice_id))
        )
        .set((
            botbilling::schema::billing_invoices::status.eq(
                if is_complete { "paid" } else { "pending" }
            ),
            botbilling::schema::billing_invoices::paid_at.eq(Some(now)),
            botbilling::schema::billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;
    }

    // --- Integrações pós-pagamento ---
    if is_complete {
        let invoice = {
            let mut conn = service.pool().get()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;
            botbilling::schema::billing_invoices::table
                .filter(botbilling::schema::billing_invoices::id.eq(invoice_id))
                .first::<botbilling::api_models::BillingInvoice>(&mut conn)
                .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?
        };

        let total = botbilling::api_models::bd_to_f64(&invoice.total);

        // 1. CRM: marca deal como ganho
        let deal_name = format!("Assinatura - {}", invoice.customer_name);
        let _ = integration::win_crm_deal(
            service.pool(),
            invoice.org_id,
            &deal_name,
        ).map_err(|e| tracing::warn!("CRM win deal failed: {e}"));

        // 2. ERP (GL): lança entrada contábil
        let _ = integration::create_gl_entry_for_invoice(
            service.pool(),
            invoice_id,
            total,
            &invoice.customer_name,
        ).map_err(|e| tracing::warn!("GL entry creation failed: {e}"));

        // 3. Subscription: cria registro de assinatura recorrente
        let plan_label = session
            .metadata
            .as_ref()
            .and_then(|m| m.get("plan"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let _ = integration::create_billing_subscription(
            service.pool(),
            invoice.org_id,
            invoice.bot_id,
            &invoice.customer_name,
            &invoice.customer_email.unwrap_or_default(),
            &plan_label,
            total,
            &invoice.currency,
            invoice_id,
            "monthly",
        ).map_err(|e| tracing::warn!("Subscription creation failed: {e}"));

        // --- Notificações de pós-pagamento ---
        let mut vars = notifier::EmailVars::new(
            &invoice.customer_name,
            &invoice.customer_email.as_deref().unwrap_or(""),
            &plan_label,
            total,
            &invoice.currency,
        );
        vars.invoice_id = invoice_id.to_string();
        notifier::notify_payment_success(&vars);
        notifier::notify_subscription_activated(&vars);
    }

    Ok(Json(serde_json::json!({
        "status": if is_complete { "completed" } else { "pending" },
        "customer": session.customer,
        "subscription": session.subscription,
    })))
}

/// `GET /api/management/plans`
async fn list_plans(
    State(service): State<Arc<SaasService>>,
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

/// `GET /api/management/plans/{plan_id}`
async fn get_plan_detail(
    State(service): State<Arc<SaasService>>,
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
