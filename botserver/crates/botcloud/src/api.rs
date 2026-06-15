use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use diesel::deserialize::QueryableByName;
use diesel::prelude::*;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(QueryableByName, Debug)]
struct OrgRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

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

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgBody {
    pub name: String,
    pub plan: Option<String>,
    pub period: Option<String>,
    pub storage_gb: Option<f64>,
    pub ai_addons: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub plan: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfileUpdateBody {
    pub name: Option<String>,
    pub organization: Option<String>,
}

pub fn configure_cloud_api_routes() -> Router<Arc<SaasService>> {
    Router::new()
        // Auth
        .route("/api/cloud/auth/login", post(handle_login))
        .route("/api/cloud/auth/signup", post(handle_signup))
        // Checkout / Plans
        .route("/api/cloud/checkout", post(handle_checkout))
        .route("/api/cloud/checkout/success", get(checkout_success))
        .route("/api/cloud/plans", get(list_plans))
        .route("/api/cloud/plans/{plan_id}", get(get_plan_detail))
        // Organizations
        .route("/api/cloud/organizations", get(list_organizations).post(create_organization))
        .route("/api/cloud/organizations/{org_id}/billing", get(org_billing_portal))
        // Services (purchased add-ons)
        .route("/api/cloud/services", get(list_services))
        .route("/api/cloud/services/{id}/cancel", post(cancel_service))
        // Invoices
        .route("/api/cloud/invoices", get(list_invoices))
        // Payment cards (stub — real impl via Stripe SetupIntent)
        .route("/api/cloud/payment-cards", get(list_payment_cards))
        // Store items
        .route("/api/cloud/store", get(list_store_items))
        .route("/api/cloud/store/purchase", post(handle_store_purchase))
        .route("/api/cloud/billing-portal", get(billing_portal))
        // Profile
        .route("/api/cloud/profile", get(get_profile).post(update_profile).put(update_profile))
        // Top-up (Special Offers)
        .route("/api/cloud/topup", post(handle_topup))
        // App Store Publishing Consultancy
        .route("/api/cloud/appstore/purchase", post(handle_appstore_purchase))
}

/// `POST /api/cloud/auth/signup`
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

/// `POST /api/cloud/checkout`
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

    let cancel_url = format!("{}/cloud/checkout/cancel", service.config.base_url);

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

/// `GET /api/cloud/checkout/success`
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
            invoice.customer_email.as_deref().unwrap_or(""),
            &plan_label,
            total,
            &invoice.currency,
            invoice_id,
            "monthly",
        ).map_err(|e| tracing::warn!("Subscription creation failed: {e}"));

        // --- Notificações de pós-pagamento ---
        let mut vars = notifier::EmailVars::new(
            &invoice.customer_name,
            invoice.customer_email.as_deref().unwrap_or(""),
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

/// `GET /api/cloud/plans`
async fn list_plans(
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
async fn get_plan_detail(
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
/// Valida credenciais e devolve um token JWT de acesso ao portal de gestão.
/// Em produção, delegar ao provedor OIDC configurado (Zitadel).
async fn handle_login(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Validação mínima de email
    if !body.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Invalid email".to_string()));
    }

    // Verificar se existe contato com este email na base
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, id, first_name, last_name};
    let contact_opt = crm_contacts
        .filter(email.eq(&body.email))
        .select((id, first_name, last_name, email))
        .first::<(Uuid, Option<String>, Option<String>, String)>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    // Gerar token JWT simples (HMAC-SHA256 com o secret configurado)
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() + 86400 * 7; // 7 dias

    let header  = base64_url_encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
    let payload = base64_url_encode(
        format!(
            "{{\"sub\":\"{}\",\"email\":\"{}\",\"exp\":{}}}",
            contact_opt.as_ref().map(|c| c.0.to_string()).unwrap_or_else(|| "guest".to_string()),
            body.email,
            exp
        ).as_bytes()
    );
    // Signature stub (real HMAC signing requires a crypto crate)
    let signature = base64_url_encode(service.config.jwt_secret.as_bytes());
    let token = format!("{header}.{payload}.{signature}");

    let (user_name, found) = if let Some((_, fn_, ln_, _)) = &contact_opt {
        let n = [fn_.as_deref().unwrap_or(""), ln_.as_deref().unwrap_or("")]
            .iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ");
        (if n.is_empty() { body.email.split('@').next().unwrap_or("User").to_string() } else { n }, true)
    } else {
        (body.email.split('@').next().unwrap_or("User").to_string(), false)
    };

    Ok(Json(serde_json::json!({
        "status": "ok",
        "token": token,
        "email": body.email,
        "name": user_name,
        "is_new": !found,
    })))
}

fn base64_url_encode(input: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let _ = write!(out, "{}{}{}{}", chars[((n >> 18) & 63) as usize] as char,
            chars[((n >> 12) & 63) as usize] as char,
            if chunk.len() > 1 { chars[((n >> 6) & 63) as usize] as char } else { '=' },
            if chunk.len() > 2 { chars[(n & 63) as usize] as char } else { '=' });
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Organizations
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/organizations`
async fn list_organizations(
    State(service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let orgs: Vec<OrgRow> = diesel::sql_query("SELECT org_id AS id, name FROM organizations")
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = orgs.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "name": r.name,
            "plan": "personal",
            "status": "active",
        })
    }).collect();

    Ok(Json(serde_json::json!({ "organizations": result })))
}

/// `POST /api/cloud/organizations`
async fn create_organization(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<CreateOrgBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Organization name is required".to_string()));
    }

    let org_id = integration::create_organization(service.pool(), &body.name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let plan = body.plan.unwrap_or_else(|| "personal".to_string());
    let period = body.period.unwrap_or_else(|| "monthly".to_string());
    let storage = body.storage_gb.unwrap_or(5.0);

    // Se houver plano pago, criar sessão de checkout
    let config = botbilling::default_product_config();
    if let Some(plan_cfg) = config.plans.get(&plan) {
        if matches!(plan_cfg.price, botbilling::PlanPrice::Fixed { .. }) {
            let total = match &plan_cfg.price {
                botbilling::PlanPrice::Fixed { amount, .. } => *amount as f64 / 100.0,
                _ => 0.0,
            };
            let payload_json = serde_json::json!({
                "plan": plan, "period": period,
                "storage": storage, "ai": [], "total": total, "currency": "usd"
            });
            return Ok(Json(serde_json::json!({
                "status": "checkout_required",
                "org_id": org_id,
                "checkout_payload": payload_json,
                "checkout_url": format!("/cloud/checkout?payload={}", url::form_urlencoded::byte_serialize(payload_json.to_string().as_bytes()).collect::<String>()),
            })));
        }
    }

    Ok(Json(serde_json::json!({
        "status": "created",
        "org_id": org_id,
        "name": body.name,
        "plan": plan,
    })))
}

/// `GET /api/cloud/organizations/{org_id}/billing`
async fn org_billing_portal(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Redireciona para o portal de faturamento Stripe do cliente
    let portal_url = format!(
        "{}/api/billing/portal?org_id={}",
        service.config.base_url, org_id
    );
    Ok(Json(serde_json::json!({ "url": portal_url, "org_id": org_id })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Services (add-ons purchased)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/services`
///
/// Retorna os serviços provisionados (subscriptions activas).
async fn list_services(
    State(service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::billing_recurring::dsl::*;
    let subs = billing_recurring
        .select((id, customer_name, frequency, status, amount, currency, interval_count))
        .load::<(Uuid, String, String, String, bigdecimal::BigDecimal, String, i32)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = subs.into_iter().map(|(sid, cname, freq, stat, amt, cur, _interval)| {
        serde_json::json!({
            "id": sid,
            "name": cname,
            "plan_id": freq,
            "status": stat,
            "amount": amt.to_string(),
            "currency": cur,
            "period": if _interval > 1 { format!("every {} {}", _interval, freq) } else { freq.to_string() },
        })
    }).collect();

    Ok(Json(serde_json::json!({ "services": result })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Invoices
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/invoices`
async fn list_invoices(
    State(service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use botbilling::schema::billing_invoices::dsl::*;
    let invs = billing_invoices
        .select((id, invoice_number, customer_name, total, status, issue_date, due_date))
        .order(issue_date.desc())
        .limit(50)
        .load::<(Uuid, String, String, bigdecimal::BigDecimal, String, chrono::NaiveDate, chrono::NaiveDate)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = invs.into_iter().map(|(iid, inum, cname, tot, stat, idate, ddate)| {
        serde_json::json!({
            "id": iid,
            "number": inum,
            "customer": cname,
            "total": tot.to_string(),
            "status": stat,
            "issue_date": idate.to_string(),
            "due_date": ddate.to_string(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "invoices": result })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Payment Cards (Stripe — lista os métodos de pagamento do cliente)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/payment-cards`
async fn list_payment_cards(
    State(_service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Stub: em produção, listar payment methods do Stripe Customer ligado ao usuário autenticado
    Ok(Json(serde_json::json!({
        "cards": [],
        "message": "Payment cards are managed via Stripe. Add a card during checkout."
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Store catalogue (produtos disponíveis para compra)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/store`
async fn list_store_items(
    State(_service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Catálogo estático com preços dobrados, providers invisíveis
    let items = serde_json::json!({
        "items": [
            // ── VPS ──
            { "id":"vps-small",  "category":"compute", "name":"VPS Small",  "icon":"🖥️", "price_type":"fixed", "amount":999,  "currency":"usd", "period":"mo", "description":"4 vCPU · 8 GB RAM · 100 GB NVMe · 2 TB BW" },
            { "id":"vps-medium", "category":"compute", "name":"VPS Medium", "icon":"🖥️", "price_type":"fixed", "amount":1999, "currency":"usd", "period":"mo", "description":"6 vCPU · 16 GB RAM · 200 GB NVMe · 4 TB BW" },
            { "id":"vps-large",  "category":"compute", "name":"VPS Large",  "icon":"🖥️", "price_type":"fixed", "amount":3999, "currency":"usd", "period":"mo", "description":"8 vCPU · 32 GB RAM · 400 GB NVMe · 8 TB BW" },
            { "id":"vps-xl",     "category":"compute", "name":"VPS XL",     "icon":"🖥️", "price_type":"fixed", "amount":7999, "currency":"usd", "period":"mo", "description":"16 vCPU · 64 GB RAM · 800 GB NVMe · 16 TB BW" },
            // ── GPU ──
            { "id":"gpu-basic",      "category":"compute", "name":"GPU Basic",      "icon":"⚡", "price_type":"fixed", "amount":2999,  "currency":"usd", "period":"mo", "description":"GT 730 · 4 vCPU · 8 GB RAM" },
            { "id":"gpu-pro",        "category":"compute", "name":"GPU Pro",        "icon":"⚡", "price_type":"fixed", "amount":9999,  "currency":"usd", "period":"mo", "description":"RTX 4090 · 8 vCPU · 32 GB RAM" },
            { "id":"gpu-enterprise", "category":"compute", "name":"GPU Enterprise", "icon":"⚡", "price_type":"fixed", "amount":29999, "currency":"usd", "period":"mo", "description":"A100 80 GB · 16 vCPU · 64 GB RAM" },
            // ── Storage ──
            { "id":"storage-50",   "category":"storage", "name":"Storage 50 GB",  "icon":"💾", "price_type":"fixed", "amount":999,  "currency":"usd", "period":"mo", "description":"S3-compatible · 100 GB egress" },
            { "id":"storage-250",  "category":"storage", "name":"Storage 250 GB", "icon":"💾", "price_type":"fixed", "amount":2999, "currency":"usd", "period":"mo", "description":"S3-compatible · 500 GB egress · Versioning" },
            { "id":"storage-1tb",  "category":"storage", "name":"Storage 1 TB",   "icon":"💾", "price_type":"fixed", "amount":5999, "currency":"usd", "period":"mo", "description":"S3-compatible · 2 TB egress · Lifecycle mgmt" },
            { "id":"storage-10tb", "category":"storage", "name":"Storage 10 TB",  "icon":"💾", "price_type":"fixed", "amount":19999,"currency":"usd", "period":"mo", "description":"S3-compatible · 20 TB egress · Priority support" },
            // ── Numbers ──
            { "id":"number-local",    "category":"comms", "name":"Local Number",   "icon":"📞", "price_type":"fixed", "amount":599,  "currency":"usd", "period":"mo", "description":"1 number · SMS + Voice · WhatsApp-ready" },
            { "id":"number-global",   "category":"comms", "name":"Global Bundle",  "icon":"📞", "price_type":"fixed", "amount":1999, "currency":"usd", "period":"mo", "description":"3 numbers · Different countries" },
            { "id":"number-business", "category":"comms", "name":"Business Pack",  "icon":"📞", "price_type":"fixed", "amount":4999, "currency":"usd", "period":"mo", "description":"10 numbers · Any countries" },
            // ── Calls ──
            { "id":"calls-100",  "category":"comms", "name":"100 Min Bundle",  "icon":"📱", "price_type":"fixed", "amount":999,  "currency":"usd", "period":"once", "description":"100 outbound minutes · Global coverage" },
            { "id":"calls-500",  "category":"comms", "name":"500 Min Bundle",  "icon":"📱", "price_type":"fixed", "amount":3999, "currency":"usd", "period":"once", "description":"500 outbound minutes · Priority routing" },
            { "id":"calls-1000", "category":"comms", "name":"1000+ Min Bundle","icon":"📱", "price_type":"fixed", "amount":6999, "currency":"usd", "period":"once", "description":"1000 outbound minutes · Dedicated routes" },
            // ── Domains ──
            { "id":"domain-com",  "category":"domains", "name":".com Domain", "icon":"🌐", "price_type":"fixed", "amount":2199, "currency":"usd", "period":"yr", "description":"Free WHOIS privacy · Managed DNS" },
            { "id":"domain-io",   "category":"domains", "name":".io Domain",  "icon":"🌐", "price_type":"fixed", "amount":7199, "currency":"usd", "period":"yr", "description":"Free WHOIS privacy · Managed DNS" },
            { "id":"domain-ai",   "category":"domains", "name":".ai Domain",  "icon":"🌐", "price_type":"fixed", "amount":15999,"currency":"usd", "period":"yr", "description":"Free WHOIS privacy · Managed DNS" },
        ]
    });
    Ok(Json(items))
}

// ─────────────────────────────────────────────────────────────────────────────
// Profile
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/profile`
async fn get_profile(
    State(_service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Em produção: decodificar JWT do header Authorization e buscar perfil
    Ok(Json(serde_json::json!({
        "name": "",
        "email": "",
        "organization": "",
    })))
}

/// `POST /api/cloud/profile`
async fn update_profile(
    State(_service): State<Arc<SaasService>>,
    Json(body): Json<ProfileUpdateBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Stub: atualizar nome/org no banco associado ao JWT do usuário autenticado
    Ok(Json(serde_json::json!({
        "status": "ok",
        "name": body.name.unwrap_or_default(),
        "organization": body.organization.unwrap_or_default(),
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Top-up (Special Offers)
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
async fn cancel_service(
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

/// `POST /api/cloud/store/purchase`
async fn handle_store_purchase(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<StorePurchaseBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let (org_id, bot_id) = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    let effective_org_id = body.org_id.unwrap_or(org_id);
    let now = chrono::Utc::now();
    let zero = bigdecimal::BigDecimal::from(0);

    let invoice_id = Uuid::new_v4();
    let invoice_num = botbilling::api_models::generate_invoice_number(&mut conn, effective_org_id);

    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, first_name, last_name};
    let contact_name = crm_contacts
        .filter(email.eq(&body.email))
        .select((first_name, last_name))
        .first::<(Option<String>, Option<String>)>(&mut conn)
        .map(|(fn_, ln_)| [fn_.unwrap_or_default(), ln_.unwrap_or_default()].join(" "))
        .map(|s| if s.trim().is_empty() { body.email.split('@').next().unwrap_or("Customer").to_string() } else { s })
        .unwrap_or_else(|_| body.email.split('@').next().unwrap_or("Customer").to_string());

    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values((
            botbilling::schema::billing_invoices::id.eq(invoice_id),
            botbilling::schema::billing_invoices::org_id.eq(effective_org_id),
            botbilling::schema::billing_invoices::bot_id.eq(bot_id),
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
async fn billing_portal(
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
/// Cria uma fatura para o serviço de consultoria de publicação em lojas de aplicativos.
/// O pagamento é processado via o fluxo de checkout existente.
async fn handle_appstore_purchase(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<AppStorePurchaseBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Connection: {e}")))?;

    let (org_id, bot_id) = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    let effective_org_id = if org_id == Uuid::nil() { Uuid::nil() } else { org_id };
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
        .map(|(fn_, ln_)| [fn_.unwrap_or_default(), ln_.unwrap_or_default()].join(" "))
        .map(|s| if s.trim().is_empty() { body.email.split('@').next().unwrap_or("Customer").to_string() } else { s })
        .unwrap_or_else(|_| body.email.split('@').next().unwrap_or("Customer").to_string());

    let invoice_id = Uuid::new_v4();

    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values((
            botbilling::schema::billing_invoices::id.eq(invoice_id),
            botbilling::schema::billing_invoices::org_id.eq(effective_org_id),
            botbilling::schema::billing_invoices::bot_id.eq(bot_id),
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
async fn handle_topup(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<TopupBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Connection: {e}")))?;

    // Recuperar bot_id padrão
    let (_, bot_id) = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    let target_bot_id = if bot_id == Uuid::nil() {
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    } else {
        bot_id
    };

    use std::str::FromStr;
    let decimal_amount = bigdecimal::BigDecimal::from_str(&format!("{:.2}", body.amount))
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid amount: {e}")))?;

    let zero = bigdecimal::BigDecimal::from(0);

    // Gerar número de fatura único
    use rand::Rng;
    let mut rng = rand::rng();
    let num: u32 = rng.random_range(100_000..999_999);
    let invoice_num = format!("INV-TOPUP-{}", num);

    // Inserir a fatura de recarga no banco de dados como paga (paid)
    let new_invoice_id = Uuid::new_v4();
    
    // Obter o nome do contato correspondente ao email, ou usar email
    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, first_name, last_name};
    let contact_name = crm_contacts
        .filter(email.eq(&body.email))
        .select((first_name, last_name))
        .first::<(Option<String>, Option<String>)>(&mut conn)
        .map(|(fn_, ln_)| [fn_.unwrap_or_default(), ln_.unwrap_or_default()].join(" "))
        .map(|s| if s.trim().is_empty() { body.email.split('@').next().unwrap_or("Customer").to_string() } else { s })
        .unwrap_or_else(|_| body.email.split('@').next().unwrap_or("Customer").to_string());

    // Inserir registro na tabela billing_invoices
    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values((
            botbilling::schema::billing_invoices::id.eq(new_invoice_id),
            botbilling::schema::billing_invoices::org_id.eq(body.org_id),
            botbilling::schema::billing_invoices::bot_id.eq(target_bot_id),
            botbilling::schema::billing_invoices::invoice_number.eq(&invoice_num),
            botbilling::schema::billing_invoices::customer_name.eq(&contact_name),
            botbilling::schema::billing_invoices::customer_email.eq(Some(body.email)),
            botbilling::schema::billing_invoices::status.eq("paid"),
            botbilling::schema::billing_invoices::issue_date.eq(chrono::Local::now().date_naive()),
            botbilling::schema::billing_invoices::due_date.eq(chrono::Local::now().date_naive()),
            botbilling::schema::billing_invoices::subtotal.eq(&decimal_amount),
            botbilling::schema::billing_invoices::tax_rate.eq(&zero),
            botbilling::schema::billing_invoices::tax_amount.eq(&zero),
            botbilling::schema::billing_invoices::discount_percent.eq(&zero),
            botbilling::schema::billing_invoices::discount_amount.eq(&zero),
            botbilling::schema::billing_invoices::total.eq(&decimal_amount),
            botbilling::schema::billing_invoices::amount_paid.eq(&decimal_amount),
            botbilling::schema::billing_invoices::amount_due.eq(&zero),
            botbilling::schema::billing_invoices::currency.eq("usd"),
            botbilling::schema::billing_invoices::notes.eq(Some("Account balance top-up via Special Offers".to_string())),
            botbilling::schema::billing_invoices::paid_at.eq(Some(chrono::Utc::now())),
            botbilling::schema::billing_invoices::created_at.eq(chrono::Utc::now()),
            botbilling::schema::billing_invoices::updated_at.eq(chrono::Utc::now()),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create topup invoice: {e}")))?;

    // Criar o pagamento em billing_payments para registrar a transação
    let payment_id = Uuid::new_v4();
    let payment_num = format!("PAY-TOPUP-{}", num);
    diesel::insert_into(botbilling::schema::billing_payments::table)
        .values((
            botbilling::schema::billing_payments::id.eq(payment_id),
            botbilling::schema::billing_payments::org_id.eq(body.org_id),
            botbilling::schema::billing_payments::bot_id.eq(target_bot_id),
            botbilling::schema::billing_payments::invoice_id.eq(Some(new_invoice_id)),
            botbilling::schema::billing_payments::payment_number.eq(&payment_num),
            botbilling::schema::billing_payments::amount.eq(&decimal_amount),
            botbilling::schema::billing_payments::currency.eq("usd"),
            botbilling::schema::billing_payments::payment_method.eq("offline_topup"),
            botbilling::schema::billing_payments::status.eq("completed"),
            botbilling::schema::billing_payments::payer_name.eq(Some(&contact_name)),
            botbilling::schema::billing_payments::paid_at.eq(chrono::Utc::now()),
            botbilling::schema::billing_payments::created_at.eq(chrono::Utc::now()),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to register payment: {e}")))?;

    tracing::info!("Top-up success: Organization {} added credits of ${:.2}", body.org_id, body.amount);

    Ok(Json(serde_json::json!({
        "status": "ok",
        "invoice_id": new_invoice_id,
        "invoice_number": invoice_num,
        "amount": decimal_amount.to_string(),
        "customer": contact_name,
    })))
}

