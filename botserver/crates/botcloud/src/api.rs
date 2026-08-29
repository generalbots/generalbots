use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, middleware, routing::{get, post, put, delete}, Json, Router};
use axum::response::{IntoResponse, Response};
use axum::body::Body;
use diesel::deserialize::QueryableByName;
use diesel::prelude::*;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(QueryableByName, Debug)]
struct OrgRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(QueryableByName, Debug)]
struct BranchIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

use crate::{integration, notifier, CalculatorPayload, SaasConfig, SaasService};
use botproviders::{ComputeProvider, MachineSpec};
use botproviders::vast::VastAiProvider;
use botproviders::contabo::ContaboProvider;

/// JWT authentication middleware for cloud API routes.
/// Validates Bearer token on all routes except `/api/cloud/auth/*`.
async fn cloud_jwt_middleware(
    axum::Extension(jwt_secret): axum::Extension<String>,
    request: axum::http::Request<Body>,
    next: middleware::Next,
) -> Response {
    let path = request.uri().path().to_string();

    // Skip auth for public endpoints
    if path.starts_with("/api/cloud/auth/")
        || path.starts_with("/api/domains/resolve")
        || path == "/api/cloud/tenant/settings/oauth/google/callback"
    {
        return next.run(request).await;
    }

    // Public GET endpoints for anonymous product/plan browsing
    if request.method() == "GET" {
        if path.starts_with("/api/cloud/store")
            || path.starts_with("/api/cloud/plans")
            || path.starts_with("/api/cloud/offers")
            || path.starts_with("/api/cloud/llm-providers")
            || path.starts_with("/api/products/")
            || path.starts_with("/api/catalog/")
        {
            return next.run(request).await;
        }
    }

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match auth_header {
        Some(token) => {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                let err = serde_json::json!({"error": "Invalid token format"});
                return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
            }
            let (header_b64, _payload_b64, sig_b64) = (parts[0], parts[1], parts[2]);
            let message = format!("{}.{}", header_b64, parts[1]);
            let expected_sig = jwt_sign_inner(&message, jwt_secret.as_bytes());
            if sig_b64 != expected_sig {
                let err = serde_json::json!({"error": "Invalid token signature"});
                return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
            }
            next.run(request).await
        }
        None => {
            let err = serde_json::json!({"error": "Missing Authorization header"});
            (StatusCode::UNAUTHORIZED, Json(err)).into_response()
        }
    }
}

/// HMAC-SHA256 sign a message and return the base64url-encoded signature.
fn jwt_sign_inner(message: &str, secret: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    mac.update(message.as_bytes());
    base64_url_encode(&mac.finalize().into_bytes())
}

#[derive(diesel::QueryableByName, Debug)]
struct ProviderKeyRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    key: String,
}

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
    pub bot_name: Option<String>,
    pub password: Option<String>,
    pub plan: Option<String>,
    pub template: Option<String>,
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
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgBody {
    pub name: Option<String>,
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

pub fn configure_cloud_api_routes(config: SaasConfig) -> Router<Arc<SaasService>> {
    let jwt_secret = config.jwt_secret.clone();
    Router::new()
        // Auth
        .route("/api/cloud/auth/login", post(handle_login))
        .route("/api/cloud/auth/signup", post(handle_signup))
        // Checkout / Plans
        .route("/api/cloud/checkout", post(handle_checkout))
        .route("/api/cloud/checkout/success", get(checkout_success))
        .route("/api/cloud/plans", get(list_plans))
        .route("/api/cloud/plans/:plan_id", get(get_plan_detail))
        // Organizations
        .route("/api/cloud/organizations", get(list_organizations).post(create_organization))
        .route("/api/cloud/organizations/:org_id", get(get_organization).put(update_organization).delete(delete_organization))
        .route("/api/cloud/organizations/:org_id/billing", get(org_billing_portal))
        // Branches per organization
        .route("/api/cloud/organizations/:org_id/branches", get(list_branches).post(create_branch_handler))
        .route("/api/cloud/organizations/:org_id/branches/:branch_id", put(update_branch_handler).delete(delete_branch_handler))
        // Workspaces per organization
        .route("/api/cloud/organizations/:org_id/workspaces", get(list_workspaces).post(create_workspace))
        .route("/api/cloud/organizations/:org_id/workspaces/:ws_id", put(update_workspace).delete(delete_workspace))
        // Resources per workspace
        .route("/api/cloud/organizations/:org_id/workspaces/:ws_id/resources", get(list_workspace_resources).post(assign_resource))
        .route("/api/cloud/organizations/:org_id/workspaces/:ws_id/resources/:res_id", delete(remove_resource))
        // Services (purchased add-ons)
        .route("/api/cloud/bots", get(list_bots))
        .route("/api/cloud/services", get(list_services))
        .route("/api/cloud/services/:id/cancel", post(cancel_service))
        // Invoices
        .route("/api/cloud/invoices", get(list_invoices))
        // Vouchers
        .route("/api/cloud/vouchers", post(crate::vouchers::create_voucher).get(crate::vouchers::list_vouchers))
        .route("/api/cloud/vouchers/redeem", post(crate::vouchers::redeem_voucher))
        .route("/api/cloud/vouchers/my", get(crate::vouchers::get_my_redemptions))
        // Payment cards (Stripe SetupIntent / hosted Checkout setup mode)
        .route("/api/cloud/payment-cards", get(crate::payment_cards::list_payment_cards))
        .route("/api/cloud/payment-cards/setup", post(crate::payment_cards::create_payment_card_setup))
        .route("/api/cloud/payment-cards/setup-intent", post(crate::payment_cards::create_payment_card_setup_intent))
        .route("/api/cloud/payment-cards/:pm_id/default", post(crate::payment_cards::set_default_payment_card))
        .route("/api/cloud/payment-cards/:pm_id", delete(crate::payment_cards::delete_payment_card))
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
        // Offers (combo bundles)
        .route("/api/cloud/offers", get(list_offers))
        // LLM Providers catalog
        .route("/api/cloud/llm-providers", get(list_llm_providers))
        // BYOK (Bring Your Own Key) — encrypted server-side storage
        .route("/api/cloud/tenant/settings/byok", post(handle_save_byok))
        .route("/api/cloud/tenant/settings/oauth/:provider/start", get(handle_oauth_start))
        .route("/api/cloud/tenant/settings/oauth/google/callback", get(handle_google_oauth_callback))
        // Admin
        .route("/api/cloud/admin/server-capacity", get(get_server_capacity))
        // Domains (CRUD — admin only, all require JWT)
        .route("/api/cloud/domains", get(crate::domains::list_domains).post(crate::domains::create_domain))
        .route("/api/cloud/domains/:id", put(crate::domains::update_domain).delete(crate::domains::delete_domain))
        // Domain resolution (public — no JWT required)
        .route("/api/domains/resolve", get(crate::domains::resolve_domain))
        // JWT auth middleware — protects all routes except /api/cloud/auth/*
        .layer(middleware::from_fn(cloud_jwt_middleware))
        .layer(axum::Extension(jwt_secret))
}

/// `POST /api/cloud/auth/signup`
///
/// Creates organization in DB + contact in CRM, returning the IDs.
async fn handle_signup(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<SignupBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let bot_name = body.bot_name.as_deref()
        .map(|n| n.trim().to_lowercase().replace(' ', "-"))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| {
            body.email.split('@').next().unwrap_or("default").to_lowercase()
        });

    // 7. Determine plan from body (default: free) — checked early for capacity gate
    let chosen_plan = body.plan.as_deref()
        .map(|p| p.to_lowercase())
        .filter(|p| p == "free" || p == "shared" || p == "private-cloud")
        .unwrap_or_else(|| "free".to_string());

    // Auto-pause free signups when server is under pressure
    // Can be disabled via SAAS_DISABLE_CAPACITY_CHECK=1 for dev/testing
    if (chosen_plan == "free" || chosen_plan == "shared")
        && std::env::var("SAAS_DISABLE_CAPACITY_CHECK").as_deref() != Ok("1")
    {
        let capacity = botbilling::server_capacity::calculate_server_capacity(
            &botbilling::server_capacity::ServerCapacityConfig::default(),
            0, 0,
        );
        if !capacity.new_signups_allowed {
            return Err((StatusCode::SERVICE_UNAVAILABLE, format!(
                "{{ \"error\": \"server_at_capacity\", \"message\": \"{} plan temporarily unavailable. Please try again later or upgrade.\", \"capacity_health\": \"{}\" }}",
                chosen_plan, capacity.capacity_health
            )));
        }
    }

    // Determine plan config (no DB needed — done before transaction)
    let product_config = botbilling::default_product_config();
    let plan_config = product_config.plans.get(&chosen_plan)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid plan".to_string()))?;

    let is_custom_plan = matches!(plan_config.price, botbilling::PlanPrice::Custom);
    let is_free_plan = matches!(plan_config.price, botbilling::PlanPrice::Free);
    let trial_days = plan_config.trial_days.unwrap_or(0);


    // Get a single DB connection for the entire signup transaction (raw SQL tx)
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    use diesel::sql_query;
    sql_query("BEGIN").execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("BEGIN: {e}")))?;

    let tx_result = (|| -> Result<(Uuid, Uuid, Uuid, String, Uuid, Option<Uuid>), String> {
        // 1. Get or create tenant
        let tenant_id = integration::get_or_create_default_tenant_inner(&mut conn)?;

        // 2. Create organization named after the bot
        let org_domain = format!("{bot_name}.org.pragmatismo.com.br");
        let org_id = integration::create_organization_inner(&mut conn, &bot_name, Some(&org_domain))?;
        integration::link_org_to_tenant_inner(&mut conn, org_id, tenant_id)?;

        // 3. Create branch with same name
        let branch_id = integration::create_branch_inner(&mut conn, org_id, tenant_id, &bot_name)?;

        // 4. Create bot record
        let (new_bot_id, org_slug) = integration::create_bot_inner(&mut conn, org_id, branch_id, &bot_name)?;

        // 5. Create CRM contact (password managed by Zitadel, not stored in DB)
        let contact_id = integration::create_crm_contact_inner(
            &mut conn, branch_id, new_bot_id, &body.name, &body.email, None::<&str>,
        )?;

        // 6. Create subscription
        // Note: org_id column in billing_recurring now references branches(id) per migration 9.16
        let subscription_id = if is_custom_plan {
            None
        } else if is_free_plan {
            Some(integration::create_free_subscription_inner(
                &mut conn, branch_id, new_bot_id, &body.name, &body.email,
            )?)
        } else {
            Some(integration::create_trial_subscription_inner(
                &mut conn, branch_id, new_bot_id, &body.name, &body.email,
                &chosen_plan, trial_days as i32,
            )?)
        };

        // 7. For free plan, create a $0 invoice to populate billing/ERP
        if is_free_plan {
            let now = chrono::Utc::now();
            let inv_id = Uuid::new_v4();
            let inv_number = botbilling::api_models::generate_invoice_number(&mut conn, branch_id);
            diesel::sql_query(
                r#"INSERT INTO billing_invoices
                   (id, org_id, bot_id, branch_id, invoice_number, customer_name, customer_email,
                    status, issue_date, due_date, subtotal, total, amount_due, amount_paid,
                    currency, notes, paid_at, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, 'paid', $8, $9, 0, 0, 0, 0,
                           'usd', 'Free Plan activation', $10, $11, $11)"#,
            )
            .bind::<diesel::sql_types::Uuid, _>(inv_id)
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .bind::<diesel::sql_types::Uuid, _>(new_bot_id)
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .bind::<diesel::sql_types::Text, _>(&inv_number)
            .bind::<diesel::sql_types::Text, _>(&body.name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(body.email.clone()))
            .bind::<diesel::sql_types::Date, _>(now.date_naive())
            .bind::<diesel::sql_types::Date, _>(now.date_naive())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(Some(now))
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)
            .map_err(|e| format!("Insert free plan invoice: {e}"))?;
        }

        // 8. Create default cloud workspace
        integration::create_cloud_workspace_inner(&mut conn, branch_id, &bot_name)?;

        Ok((org_id, branch_id, new_bot_id, org_slug, contact_id, subscription_id))
    })();

    let (org_id, branch_id, new_bot_id, org_slug, contact_id, subscription_id) = match tx_result {
        Ok(result) => {
            sql_query("COMMIT").execute(&mut conn)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("COMMIT: {e}")))?;
            result
        }
        Err(e) => {
            sql_query("ROLLBACK").execute(&mut conn).ok();
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        }
    };

    // 8. Seed default cloud CRM products (non-fatal — outside tx)
    #[cfg(feature = "saas")]
    {
        use botproducts::seed::seed_default_products;
        seed_default_products(&mut conn, branch_id);
    }

    // 9. Create org bucket `.gborg` in MinIO with bot files inside (non-fatal — outside tx)
    if let Err(e) = integration::create_bot_bucket(
        &service.config, &org_slug, &org_slug, &bot_name, body.template.as_deref(),
    ) {
        tracing::warn!("MinIO bucket creation skipped (non-fatal): {e}");
    }

    // 10. Create identity in directory (Zitadel) if configured (non-fatal — outside tx)
    if let (Some(dir_url), Some(dir_token)) = (&service.config.directory_api_url, &service.config.directory_service_token) {
        let parts: Vec<&str> = body.name.splitn(2, ' ').collect();
        let first_name = parts.first().unwrap_or(&"");
        let last_name = parts.get(1).unwrap_or(&"");
        let client = reqwest::Client::new();

        let mut create_req = client
            .post(format!("{dir_url}/management/v1/users/human"))
            .header("Authorization", format!("Bearer {dir_token}"))
            .json(&serde_json::json!({
                "userName": &body.email,
                "profile": { "firstName": first_name, "lastName": last_name, "displayName": &body.name },
                "email": { "email": &body.email, "isVerified": true },
                "password": body.password.as_deref().unwrap_or(""),
            }));
        if let Some(host) = &service.config.directory_external_domain {
            create_req = create_req.header("Host", host);
        }
        let create_resp = create_req.send().await;

        match create_resp {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(user_id) = data.get("userId").and_then(|v| v.as_str()) {
                        if let Some(password) = &body.password {
                            let mut pw_req = client
                                .post(format!("{dir_url}/v2/users/{user_id}/password"))
                                .header("Authorization", format!("Bearer {dir_token}"))
                                .json(&serde_json::json!({
                                    "newPassword": { "password": password, "changeRequired": false }
                                }));
                            if let Some(host) = &service.config.directory_external_domain {
                                pw_req = pw_req.header("Host", host);
                            }
                            let _ = pw_req.send().await
                                .map(|r| {
                                    if !r.status().is_success() {
                                        tracing::warn!("Zitadel password set returned {}", r.status());
                                    }
                                })
                                .unwrap_or_else(|e| tracing::warn!("Zitadel password set failed: {e}"));
                        }
                    }
                }
            }
            Ok(resp) => tracing::warn!("Zitadel user creation returned {}", resp.status()),
            Err(e) => tracing::warn!("Zitadel user creation failed: {e}"),
        }
    }

    notifier::notify_welcome(&notifier::EmailVars::new(
        &body.name, &body.email, &chosen_plan, 0.0, "USD",
    ));

    let header = base64_url_encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
    let now_ts = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp();
    let payload = base64_url_encode(
        format!(
            "{{\"sub\":\"{}\",\"email\":\"{}\",\"org_id\":\"{}\",\"branch_id\":\"{}\",\"bot_id\":\"{}\",\"bucket\":\"{}.gborg\",\"exp\":{}}}",
            new_bot_id, body.email, org_id, branch_id, new_bot_id, org_slug, now_ts,
        ).as_bytes()
    );
    let token = jwt_sign(&header, &payload, service.config.jwt_secret.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Store the JWT in the global session cache so /api/auth/me recognizes it.
    // Persist to login_sessions so the session survives botserver restarts.
    {
        use botcoredirectory::auth_routes::{SESSION_CACHE, persist_session};
        let mut cache = SESSION_CACHE.write().await;
        let session_user = botcoredirectory::auth_routes::SessionUserData {
            user_id: new_bot_id.to_string(),
            email: body.email.clone(),
            username: bot_name.clone(),
            first_name: Some(body.name.clone()),
            last_name: None,
            display_name: Some(body.name.clone()),
            organization_id: Some(org_id.to_string()),
            roles: resolve_rbac_roles(&mut conn, &new_bot_id.to_string()),
            bucket: Some(format!("{}.gborg", org_slug)),
            created_at: chrono::Utc::now().timestamp(),
        };
        cache.insert(token.clone(), session_user.clone());
        persist_session(&token, &session_user);
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "account": { "email": body.email, "name": body.name },
        "org_id": org_id,
        "branch_id": branch_id,
        "bot_id": new_bot_id,
        "bucket": format!("{}.gborg", org_slug),
        "contact_id": contact_id,
        "subscription_id": subscription_id,
        "plan": chosen_plan,
        "trial_days": trial_days,
        "token": token,
    })))
}

/// `GET /api/cloud/admin/server-capacity`
///
/// Returns real-time server capacity metrics for SaaS admin dashboard.
async fn get_server_capacity(
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

/// `POST /api/cloud/checkout`
///
/// Creates invoice in billing + contact/deal in CRM + Stripe session.
async fn handle_checkout(
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
        .unwrap_or_else(|| format!("{} Customer", &payload.plan));

    let total_cents = (payload.total * 100.0) as u64;
    let total_value = total_cents as f64;
    let invoice_id = Uuid::new_v4();
    let invoice_number = botbilling::api_models::generate_invoice_number(&mut conn, effective_branch_id);

    let invoice = botbilling::api_models::BillingInvoice {
        id: invoice_id, branch_id: effective_branch_id, invoice_number,
        customer_id: None,
        customer_name: Some(customer_name.clone()),
        customer_email: Some(customer_email.clone()),
        customer_address: None, status: Some("draft".to_string()),
        issue_date: now.date_naive(),
        due_date: Some((now + chrono::Duration::days(30)).date_naive()),
        subtotal: botbilling::api_models::bd(total_value),
        tax_rate: botbilling::api_models::bd(0.0),
        tax_amount: botbilling::api_models::bd(0.0),
        discount_percent: botbilling::api_models::bd(0.0),
        discount_amount: botbilling::api_models::bd(0.0),
        total: Some(botbilling::api_models::bd(total_value)),
        amount_paid: botbilling::api_models::bd(0.0),
        amount_due: botbilling::api_models::bd(total_value),
        currency: Some(payload.currency.clone()),
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

    // --- Notification: invoice generated ---
    let mut vars = notifier::EmailVars::new(
        &customer_name, &customer_email, &payload.plan, total_value, &payload.currency,
    );
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

    // Persist the Stripe Customer mapping so SetupIntent card management and
    // webhook events can resolve the owning branch later.
    crate::payment_cards::persist_customer_mapping(
        &service,
        effective_branch_id,
        &stripe_customer.id,
        &customer_email,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe customer mapping: {e}")))?;

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

    // --- Post-payment integrations ---
    if is_complete {
        let invoice = {
            let mut conn = service.pool().get()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;
            botbilling::schema::billing_invoices::table
                .filter(botbilling::schema::billing_invoices::id.eq(invoice_id))
                .first::<botbilling::api_models::BillingInvoice>(&mut conn)
                .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?
        };

        let total = invoice.total.as_ref().map(|t| botbilling::api_models::bd_to_f64(t)).unwrap_or(0.0);

        // 1. CRM: marca deal como ganho
        let deal_name = format!("Assinatura - {}", invoice.customer_name.as_deref().unwrap_or(""));
        let _ = integration::win_crm_deal(
            service.pool(),
            invoice.branch_id,
            &deal_name,
        ).map_err(|e| tracing::warn!("CRM win deal failed: {e}"));

        // 2. ERP (GL): posts accounting entry
        let _ = integration::create_gl_entry_for_invoice(
            service.pool(),
            invoice_id,
            total,
            invoice.customer_name.as_deref().unwrap_or(""),
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
            invoice.branch_id,
            invoice.customer_name.as_deref().unwrap_or(""),
            invoice.customer_email.as_deref().unwrap_or(""),
            &plan_label,
            total,
            invoice.currency.as_deref().unwrap_or(""),
            invoice_id,
            "monthly",
        ).map_err(|e| tracing::warn!("Subscription creation failed: {e}"));

        // --- Post-payment notifications ---
        let mut vars = notifier::EmailVars::new(
            invoice.customer_name.as_deref().unwrap_or(""),
            invoice.customer_email.as_deref().unwrap_or(""),
            &plan_label,
            total,
            invoice.currency.as_deref().unwrap_or(""),
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
/// Validates credentials and returns a JWT token for the management portal.
/// Uses Zitadel sessions API when directory is configured; falls back to
/// local argon2 hash (dev mode) when Zitadel is not available.
async fn handle_login(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Minimum email validation
    if !body.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Invalid email".to_string()));
    }

    // Try Zitadel v2 sessions API for password verification.
    // Returns (stable Zitadel user id, password-verified flag): this Zitadel
    // build omits session factors, so a created session proves the password but
    // may not expose the user id — in that case callers resolve the stable id
    // from the users table keyed by the verified email.
    let (zitadel_user_id, zitadel_password_verified) = match (&service.config.directory_api_url, &service.config.directory_service_token) {
        (Some(dir_url), Some(dir_token)) => {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build().ok();
            match client {
                Some(c) => {
                    // Zitadel matches `loginName` against a user's login names
                    // (username + org-domain forms). For accounts created with a
                    // bare username, sending the full email fails; try the email
                    // first, then fall back to the username prefix.
                    let username = body.email.split('@').next().unwrap_or(&body.email).to_string();
                    let login_names = [body.email.clone(), username];

                    let mut user_id: Option<String> = None;
                    let mut verified = false;

                    for login_name in &login_names {
                        // Use the v2 `checks` wrapper so the password is actually
                        // verified. A bare `{loginName, password}` flat shape makes
                        // Zitadel start a session WITHOUT checking credentials, so
                        // ANY password would mint a valid JWT. The checks wrapper
                        // runs the user + password checks and only returns a
                        // session when they pass (wrong password -> COMMAND-3M0fs).
                        let mut rb = c.post(format!("{dir_url}/v2/sessions"))
                            .header("Authorization", format!("Bearer {dir_token}"))
                            .json(&serde_json::json!({
                                "checks": {
                                    "user": { "loginName": login_name },
                                    "password": { "password": body.password }
                                }
                            }));
                        if let Some(host) = &service.config.directory_external_domain {
                            rb = rb.header("Host", host);
                        }
                        match rb.send().await {
                            Ok(r) if r.status().is_success() => {
                                verified = true;
                                // On a successful checks wrapper, the response carries
                                // the verified user factor directly; use it as the
                                // stable JWT subject (RBAC derives UUIDv5 from it).
                                let session = r.json::<serde_json::Value>().await.ok();
                                user_id = session.as_ref()
                                    .and_then(|v| v.get("factors").cloned())
                                    .and_then(|f| f.get("user").cloned())
                                    .and_then(|u| u.get("userId").or_else(|| u.get("id")).cloned())
                                    .and_then(|uid| uid.as_str().map(|s| s.to_string()))
                                    .or_else(|| {
                                        session.as_ref()
                                            .and_then(|v| v.get("sessionId").cloned())
                                            .and_then(|sid| sid.as_str().map(|s| s.to_string()))
                                    });
                                break;
                            }
                            Ok(_) => {
                                tracing::warn!("Zitadel session check rejected credentials for loginName '{}'", login_name);
                            }
                            Err(e) => {
                                tracing::warn!("Zitadel session check failed for loginName '{}': {}", login_name, e);
                            }
                        }
                    }

                    (user_id, verified)
                }
                None => (None, false),
            }
        }
        _ => (None, false),
    };

    // Look up user in CRM contacts for JWT claims
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, id, first_name, last_name, branch_id};
    let contact_opt = crm_contacts
        .filter(email.eq(&body.email))
        .select((id, first_name, last_name, email, branch_id))
        .first::<(Uuid, Option<String>, Option<String>, String, Uuid)>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    // Generate JWT (HMAC-SHA256 with configured secret)
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() + 86400 * 7; // 7 days

    let header  = base64_url_encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
    // Auth gate: a token is only issued after password verification.
    // 1) Zitadel v2 session check succeeded (sessionId), or
    // 2) dev-only bootstrap admin (admin-credentials.json, present only in dev).
    // NEVER fall back to an unverified CRM contact row.
    let sub = zitadel_user_id
        .or_else(|| {
            if zitadel_password_verified {
                // Zitadel verified the password but this build does not expose
                // the user id via session factors. Derive a stable subject from
                // the users table (keyed by the verified email) so JWT subs and
                // the derived UUIDs used by RBAC/org membership stay constant
                // across logins.
                #[derive(QueryableByName)]
                struct StableUserRow {
                    #[diesel(sql_type = diesel::sql_types::Uuid)]
                    id: Uuid,
                }
                diesel::sql_query(
                    "SELECT id FROM users WHERE email = $1 AND is_active = true LIMIT 1",
                )
                .bind::<diesel::sql_types::Text, _>(body.email.as_str())
                .get_result::<StableUserRow>(&mut conn)
                .optional()
                .ok()
                .flatten()
                .map(|r| r.id.to_string())
            } else {
                None
            }
        })
        .or_else(|| lookup_admin_credentials_user_id(&body.email, &body.password))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({ "detail": "Invalid credentials" }).to_string(),
            )
        })?;

    // Mint verified tenant scope claims (issue #736): the branch comes from
    // the CRM contact owned by the authenticated identity — never from the
    // client. When the user has no CRM contact row, fall back to their org
    // membership binding (users → user_organizations → branches) so suite
    // apps scope to the caller's own workspace (issue #808 fix: prod admin
    // without a crm_contacts row got nil-branch scoping → empty grids).
    // The org owning the branch is resolved from the branches table.
    let branch_scope: Option<Uuid> = contact_opt
        .as_ref()
        .map(|(_, _, _, _, br)| *br)
        .or_else(|| resolve_branch_from_user_binding(&mut conn, &body.email));
    let org_scope: Option<Uuid> = branch_scope.and_then(|b| {
        #[derive(diesel::QueryableByName)]
        struct OrgRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(b)
            .get_result::<OrgRow>(&mut conn)
            .optional()
            .ok()
            .flatten()
            .map(|r| r.org_id)
    });

    let payload_body = match (org_scope, branch_scope) {
        (Some(org_id), Some(branch)) => format!(
            "{{\"sub\":\"{}\",\"email\":\"{}\",\"exp\":{},\"org_id\":\"{}\",\"branch_id\":\"{}\"}}",
            sub, body.email, exp, org_id, branch
        ),
        _ => format!(
            "{{\"sub\":\"{}\",\"email\":\"{}\",\"exp\":{}}}",
            sub,
            body.email,
            exp
        ),
    };
    let payload = base64_url_encode(payload_body.as_bytes());
    let token = jwt_sign(&header, &payload, service.config.jwt_secret.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    tracing::info!(
        "cloud login minted JWT for {} (secret fp {:?})",
        body.email,
        &service.config.jwt_secret[..8.min(service.config.jwt_secret.len())]
    );

    let (user_name, found) = if let Some((_, fn_, ln_, _, _)) = &contact_opt {
        let n = [fn_.as_deref().unwrap_or(""), ln_.as_deref().unwrap_or("")]
            .iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ");
        (if n.is_empty() { body.email.split('@').next().unwrap_or("User").to_string() } else { n }, true)
    } else {
        (body.email.split('@').next().unwrap_or("User").to_string(), false)
    };

    // Cache the session so /api/auth/me resolves the bearer token to a real
    // user (signup already stores it; login must too — otherwise the suite
    // treats freshly logged-in users as anonymous, issue #808 report).
    // Persist to login_sessions so the session survives botserver restarts
    // (same durability the suite-sso hop provides).
    {
        use botcoredirectory::auth_routes::{SESSION_CACHE, persist_session};
        let mut cache = SESSION_CACHE.write().await;
        let session_user = botcoredirectory::auth_routes::SessionUserData {
            user_id: sub.clone(),
            email: body.email.clone(),
            username: body.email.split('@').next().unwrap_or("user").to_string(),
            first_name: contact_opt.as_ref().and_then(|(_, fn_, _, _, _)| fn_.clone()),
            last_name: contact_opt.as_ref().and_then(|(_, _, ln_, _, _)| ln_.clone()),
            display_name: Some(user_name.clone()),
            organization_id: org_scope.map(|o| o.to_string()),
            roles: resolve_rbac_roles(&mut conn, &sub),
            bucket: None,
            created_at: chrono::Utc::now().timestamp(),
        };
        cache.insert(token.clone(), session_user.clone());
        persist_session(&token, &session_user);
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "token": token,
        "email": body.email,
        "name": user_name,
        "is_new": !found,
    })))
}

fn base64_url_encode(input: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD
        .encode(input)
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

/// Resolves the caller's workspace branch from their user→org membership
/// binding (users → user_organizations → branches). Returns `None` when the
/// user has no verified org binding — callers then omit the branch claim
/// rather than minting an unverified scope.
fn resolve_branch_from_user_binding(conn: &mut diesel::PgConnection, email: &str) -> Option<Uuid> {
    #[derive(diesel::QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let user_id = diesel::sql_query("SELECT id FROM users WHERE email = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(email)
        .get_result::<UserRow>(conn)
        .optional()
        .ok()
        .flatten()?
        .id;

    #[derive(diesel::QueryableByName)]
    struct BindingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    let org_id = diesel::sql_query(
        "SELECT org_id FROM user_organizations WHERE user_id = $1 ORDER BY is_default DESC, joined_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<BindingRow>(conn)
    .optional()
    .ok()
    .flatten()?
    .org_id;

    #[derive(diesel::QueryableByName)]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    diesel::sql_query(
        "SELECT id FROM branches WHERE org_id = $1 AND is_active = true ORDER BY created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .get_result::<BranchRow>(conn)
    .optional()
    .ok()
    .flatten()
    .map(|r| r.id)
}

/// Resolves the effective role vector from RBAC group membership (fix #843).
/// Maps non-UUID identity ids (Zitadel numeric) through the same stable UUID
/// derivation used by `resolve_user_role` in the main crate. Falls back to
/// the plain "user" role when the user has no admin group.
fn resolve_rbac_roles(conn: &mut diesel::PgConnection, user_id: &str) -> Vec<String> {
    let stable = match Uuid::parse_str(user_id) {
        Ok(u) => u,
        Err(_) => Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{user_id}").as_bytes()),
    };
    #[derive(diesel::QueryableByName)]
    struct GroupName {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let names: Vec<GroupName> = diesel::sql_query(
        "SELECT g.name FROM rbac_groups g \
         JOIN rbac_user_groups ug ON ug.group_id = g.id \
         WHERE ug.user_id = $1 AND g.is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(stable)
    .load(conn)
    .unwrap_or_default();
    if names.iter().any(|g| g.name.to_lowercase().contains("admin")) {
        vec!["admin".to_string()]
    } else {
        vec!["user".to_string()]
    }
}

/// Fallback for local dev: resolves the Zitadel user_id for the bootstrap admin
/// from `admin-credentials.json` when the Zitadel sessions API is unavailable.
/// Requires the password to match the file (or the dev bootstrap password for
/// admin@localhost) — never an email-only match.
fn lookup_admin_credentials_user_id(email: &str, password: &str) -> Option<String> {
    let base = std::env::current_dir().ok()?;
    let candidates = [
        base.join("botserver-stack/conf/directory/admin-credentials.json"),
        base.join("../botserver-stack/conf/directory/admin-credentials.json"),
    ];
    for candidate in candidates {
        let content = std::fs::read_to_string(&candidate).ok()?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let cred_email = json.get("email").and_then(|v| v.as_str()).unwrap_or("");
            if cred_email.eq_ignore_ascii_case(email) {
                let file_pw = json.get("password").and_then(|v| v.as_str()).unwrap_or("");
                let dev_pw_ok = email.eq_ignore_ascii_case("admin@localhost") && password == "dev";
                if (password == file_pw) || dev_pw_ok {
                    if let Some(user_id) = json.get("user_id").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                        return Some(user_id.to_string());
                    }
                }
            }
        }
    }
    None
}

fn jwt_sign(header: &str, payload: &str, secret: &[u8]) -> Result<String, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|e| format!("HMAC init: {e}"))?;
    let signing_input = format!("{header}.{payload}");
    mac.update(signing_input.as_bytes());
    let signature = base64_url_encode(&mac.finalize().into_bytes());
    Ok(format!("{signing_input}.{signature}"))
}

pub(crate) fn base64_url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    // Convert URL-safe base64 to standard base64
    let raw = input.replace('-', "+").replace('_', "/");
    let raw = match raw.len() % 4 {
        2 => raw + "==",
        3 => raw + "=",
        0 => raw,
        _ => return Err("invalid base64 input length"),
    };
    // Decode base64 manually (no external dep needed)
    let chars: Vec<char> = raw.chars().collect();
    let mut out = Vec::with_capacity(chars.len() / 4 * 3);
    let alphabet: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '=' { break; }
        let mut vals = [0u8; 4];
        let mut valid = 0usize;
        for j in 0..4 {
            if i + j >= chars.len() || chars[i + j] == '=' { break; }
            if let Some(pos) = alphabet.iter().position(|&a| a as char == chars[i + j]) {
                vals[valid] = pos as u8;
                valid += 1;
            }
        }
        if valid == 0 { break; }
        let n = ((vals[0] as u32) << 18)
            | (if valid > 1 { (vals[1] as u32) << 12 } else { 0 })
            | (if valid > 2 { (vals[2] as u32) << 6 } else { 0 })
            | (if valid > 3 { vals[3] as u32 } else { 0 });
        out.push((n >> 16) as u8);
        if valid > 2 { out.push((n >> 8) as u8); }
        if valid > 3 { out.push(n as u8); }
        i += 4;
    }
    Ok(out)
}

pub fn get_branch_id_from_jwt(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Result<Option<Uuid>, String> {
    use diesel::prelude::*;
    if let Some(auth_val) = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        let parts: Vec<&str> = auth_val.split('.').collect();
        if parts.len() == 3 {
            if let Ok(decoded) = base64_url_decode(parts[1]) {
                if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                    if let Some(user_email) = payload.get("email").and_then(|v| v.as_str()) {
                        use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, branch_id};
                        let contact_branch: Option<Uuid> = crm_contacts
                            .filter(email.eq(user_email))
                            .select(branch_id)
                            .first(conn)
                            .optional()
                            .map_err(|e| format!("Query: {e}"))?;
                        return Ok(contact_branch);
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Check if the authenticated user is the SaaS super-admin.
/// A user is super-admin if their CRM org is the default organization
/// (slug = 'default'). In development, the bootstrap admin email
/// (admin@localhost) is also treated as super-admin.
pub fn is_super_admin(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Result<bool, String> {
    // Extract email from JWT
    let user_email = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() == 3 {
                base64_url_decode(parts[1]).ok()
                    .and_then(|decoded| serde_json::from_slice::<serde_json::Value>(&decoded).ok())
                    .and_then(|payload| payload.get("email").and_then(|v| v.as_str()).map(|s| s.to_string()))
            } else { None }
        });

    // Dev mode: admin@localhost is always super-admin
    if let Some(ref email) = user_email {
        if email == "admin@localhost" {
            return Ok(true);
        }
    }

    // Check if user's organization is the default org
    let user_org_id = get_branch_id_from_jwt(headers, conn)?;

    #[derive(diesel::QueryableByName)]
    struct DefaultOrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }

    let default_org_id: Option<Uuid> = diesel::sql_query(
        "SELECT org_id FROM organizations WHERE slug = 'default' LIMIT 1"
    )
    .get_result::<DefaultOrgRow>(conn)
    .optional()
    .map_err(|e| format!("Query default org: {e}?"))?
    .map(|r| r.org_id);

    match (user_org_id, default_org_id) {
        (Some(uid), Some(did)) => Ok(uid == did),
        _ => Ok(false),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Organizations
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/organizations`
async fn list_organizations(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);

    #[derive(diesel::QueryableByName, Debug)]
    struct OrgWithCounts {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        domain: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        plan_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        branches_count: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        bots_count: i64,
    }

    let base_query = r#"
        SELECT o.org_id AS id, o.name, o.domain,
            (SELECT br.description FROM billing_recurring br
             JOIN branches b2 ON b2.id = br.org_id
             WHERE b2.org_id = o.org_id
             ORDER BY br.created_at DESC LIMIT 1) AS plan_name,
            COUNT(DISTINCT b.id) AS branches_count,
            COUNT(DISTINCT bt.id) AS bots_count
        FROM organizations o
        LEFT JOIN branches b ON b.org_id = o.org_id
        LEFT JOIN bots bt ON bt.branch_id = b.id
    "#;

    let orgs: Vec<OrgWithCounts> = if admin {
        diesel::sql_query(format!(
            "{base_query} WHERE o.org_id <> '00000000-0000-0000-0000-000000000000' GROUP BY o.org_id, o.name ORDER BY o.name"
        ))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    } else if let Some(bid) = user_branch_id {
        diesel::sql_query(format!("{base_query} WHERE o.org_id = (SELECT org_id FROM branches WHERE id = $1) GROUP BY o.org_id, o.name"))
        .bind::<diesel::sql_types::Uuid, _>(bid)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    } else {
        diesel::sql_query(format!(
            "{base_query} WHERE o.org_id <> '00000000-0000-0000-0000-000000000000' GROUP BY o.org_id, o.name ORDER BY o.name"
        ))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    };

    let result: Vec<serde_json::Value> = orgs.into_iter().map(|r| {
        let plan = if admin {
            "private-cloud".to_string()
        } else {
            r.plan_name.as_deref()
                .and_then(|d| d.split_once(" - ").map(|(p, _)| p).or(Some(d)))
                .map(|p| p.to_lowercase().replace(' ', "-"))
                .filter(|p| p == "free" || p == "shared" || p == "private-cloud")
                .unwrap_or_else(|| "free".to_string())
        };
        serde_json::json!({
            "id": r.id,
            "name": r.name,
            "plan": plan,
            "status": "active",
            "domain": r.domain,
            "branches_count": r.branches_count,
            "bots_count": r.bots_count,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "organizations": result, "is_admin": admin })))
}

/// `POST /api/cloud/organizations`
async fn create_organization(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<CreateOrgBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Organization name is required".to_string()));
    }

    let org_id = integration::create_organization(service.pool(), &body.name, body.domain.as_deref())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let plan = body.plan.unwrap_or_else(|| "free".to_string());
    let period = body.period.unwrap_or_else(|| "monthly".to_string());
    let storage = body.storage_gb.unwrap_or(5.0);

    // If paid plan, create checkout session
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
    // Redirects to Stripe customer billing portal
    let portal_url = format!(
        "{}/api/billing/portal?org_id={}",
        service.config.base_url, org_id
    );
    Ok(Json(serde_json::json!({ "url": portal_url, "org_id": org_id })))
}

/// `GET /api/cloud/organizations/{org_id}`
async fn get_organization(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let rows: Vec<OrgRow> = diesel::sql_query("SELECT org_id AS id, name FROM organizations WHERE org_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let org = rows.into_iter().next().ok_or_else(|| {
        (StatusCode::NOT_FOUND, "Organization not found".to_string())
    })?;

    Ok(Json(serde_json::json!({
        "id": org.id,
        "name": org.name,
        "plan": "free",
        "status": "active",
    })))
}

/// `PUT /api/cloud/organizations/{org_id}`
async fn update_organization(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
    Json(body): Json<UpdateOrgBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Organization name cannot be empty".to_string()));
        }
        let slug = name.to_lowercase().replace(' ', "-").replace(|c: char| !c.is_alphanumeric() && c != '-', "");
        let affected = diesel::sql_query("UPDATE organizations SET name = $1, slug = $2, updated_at = NOW() WHERE org_id = $3")
            .bind::<diesel::sql_types::Text, _>(name.trim())
            .bind::<diesel::sql_types::Text, _>(&slug)
            .bind::<diesel::sql_types::Uuid, _>(org_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;

        if affected == 0 {
            return Err((StatusCode::NOT_FOUND, "Organization not found".to_string()));
        }
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/organizations/{org_id}`
async fn delete_organization(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Remove workspace resources and workspaces for this org
    let branch_id: Option<Uuid> = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .get_result::<BranchIdRow>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?
        .map(|r| r.id);

    if let Some(bid) = branch_id {
        diesel::sql_query(
            "DELETE FROM workspace_resources WHERE workspace_id IN (SELECT id FROM cloud_workspaces WHERE branch_id = $1)"
        )
        .bind::<diesel::sql_types::Uuid, _>(bid)
        .execute(&mut conn)
        .ok();

        use crate::schema_ext::cloud_workspaces::dsl as cw;
        diesel::delete(cw::cloud_workspaces.filter(cw::branch_id.eq(bid)))
            .execute(&mut conn)
            .ok();
    }

    // Remove organization record
    let affected = diesel::sql_query("DELETE FROM organizations WHERE org_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Organization not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Workspaces
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceBody {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignResourceBody {
    pub store_item_id: String,
    pub name: Option<String>,
}

/// `GET /api/cloud/organizations/{org_id}/workspaces`
async fn list_workspaces(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    use crate::schema_ext::cloud_workspaces::dsl as cw;
    let rows = cw::cloud_workspaces
        .filter(cw::branch_id.eq(branch_id))
        .order(cw::created_at.desc())
        .load::<(Uuid, Uuid, Uuid, String, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|(wid, _, _, wname, wdesc, wicon, _, _)| {
        serde_json::json!({
            "id": wid,
            "name": wname,
            "description": wdesc,
            "icon": wicon,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "workspaces": result })))
}

/// `POST /api/cloud/organizations/{org_id}/workspaces`
async fn create_workspace(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
    Json(body): Json<CreateWorkspaceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Workspace name is required".to_string()));
    }

    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    let now = chrono::Utc::now();
    let ws_id = Uuid::new_v4();

    use crate::schema_ext::cloud_workspaces::dsl as cw;
    diesel::insert_into(cw::cloud_workspaces)
        .values((
            cw::id.eq(ws_id),
            cw::org_id.eq(org_id_param),
            cw::branch_id.eq(branch_id),
            cw::name.eq(body.name.trim()),
            cw::description.eq(body.description),
            cw::icon.eq(body.icon),
            cw::created_at.eq(now),
            cw::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert: {e}")))?;

    Ok(Json(serde_json::json!({
        "id": ws_id,
        "name": body.name.trim(),
        "org_id": org_id_param,
    })))
}

/// `PUT /api/cloud/organizations/{org_id}/workspaces/{ws_id}`
async fn update_workspace(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateWorkspaceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    use crate::schema_ext::cloud_workspaces::dsl as cw;
    let filter = cw::id.eq(ws_id_param).and(cw::branch_id.eq(branch_id));

    if let Some(n) = &body.name {
        diesel::update(cw::cloud_workspaces).filter(filter).set(cw::name.eq(n.trim())).execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;
    }
    if let Some(d) = &body.description {
        diesel::update(cw::cloud_workspaces).filter(filter).set(cw::description.eq(d)).execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update desc: {e}")))?;
    }
    if let Some(i) = &body.icon {
        diesel::update(cw::cloud_workspaces).filter(filter).set(cw::icon.eq(i)).execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update icon: {e}")))?;
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/organizations/{org_id}/workspaces/{ws_id}`
async fn delete_workspace(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    // Remove resources first
    use crate::schema_ext::workspace_resources::dsl as wr;
    diesel::delete(wr::workspace_resources.filter(wr::workspace_id.eq(ws_id_param)))
        .execute(&mut conn)
        .ok();

    use crate::schema_ext::cloud_workspaces::dsl as cw;
    let deleted = diesel::delete(cw::cloud_workspaces.filter(cw::id.eq(ws_id_param).and(cw::branch_id.eq(branch_id))))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Workspace not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/// `GET /api/cloud/organizations/{org_id}/workspaces/{ws_id}/resources`
async fn list_workspace_resources(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((_org_id, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::workspace_resources::dsl as wr;
    let rows = wr::workspace_resources
        .filter(wr::workspace_id.eq(ws_id_param))
        .order(wr::created_at.desc())
        .load::<(Uuid, Uuid, Uuid, String, String, String, String, Option<serde_json::Value>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|(rid, _, _, sid, rname, rtype, rstatus, rconfig, rprov, _, _)| {
        serde_json::json!({
            "id": rid,
            "store_item_id": sid,
            "name": rname,
            "resource_type": rtype,
            "status": rstatus,
            "config": rconfig,
            "provisioned_at": rprov,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "resources": result })))
}

/// `POST /api/cloud/organizations/{org_id}/workspaces/{ws_id}/resources`
async fn assign_resource(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
    Json(body): Json<AssignResourceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let restype = if body.store_item_id.starts_with("vps-") || body.store_item_id.starts_with("gpu-") {
        "compute"
    } else if body.store_item_id.starts_with("storage-") {
        "storage"
    } else if body.store_item_id.starts_with("number-") {
        "phone"
    } else if body.store_item_id.starts_with("domain-") || body.store_item_id.starts_with("calls-") {
        "comms"
    } else {
        "other"
    };


    let now = chrono::Utc::now();
    let res_id = Uuid::new_v4();

    use crate::schema_ext::workspace_resources::dsl as wr;
    diesel::insert_into(wr::workspace_resources)
        .values((
            wr::id.eq(res_id),
            wr::workspace_id.eq(ws_id_param),
            wr::org_id.eq(org_id_param),
            wr::store_item_id.eq(&body.store_item_id),
            wr::name.eq(body.name.unwrap_or_else(|| body.store_item_id.clone())),
            wr::resource_type.eq(restype),
            wr::status.eq("provisioning"),
            wr::provisioned_at.eq(now),
            wr::created_at.eq(now),
            wr::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert: {e}")))?;

    let db_pool = service.pool().clone();
    let item_id = body.store_item_id.clone();
    let rid = res_id;
    let wid = ws_id_param;
    let oid = org_id_param;

    tokio::spawn(async move {
        if let Err(e) = provision_compute_resource(&db_pool, &item_id, rid, wid, oid).await {
            tracing::error!("Provisioning failed for resource {rid} ({item_id}): {e}");
        }
    });

    Ok(Json(serde_json::json!({
        "id": res_id,
        "store_item_id": body.store_item_id,
        "resource_type": restype,
        "status": "provisioning",
    })))
}

fn store_item_to_spec(store_item_id: &str) -> Option<(MachineSpec, Vec<&'static str>)> {
    match store_item_id {
        "vps-small" => Some((MachineSpec { cpu_cores: 4, ram_gb: 8, disk_gb: 100, gpu_type: None, gpu_count: 0, bandwidth_tb: 2 }, vec!["vast", "contabo"])),
        "vps-medium" => Some((MachineSpec { cpu_cores: 6, ram_gb: 16, disk_gb: 200, gpu_type: None, gpu_count: 0, bandwidth_tb: 4 }, vec!["vast", "contabo"])),
        "vps-large" => Some((MachineSpec { cpu_cores: 8, ram_gb: 32, disk_gb: 400, gpu_type: None, gpu_count: 0, bandwidth_tb: 8 }, vec!["contabo"])),
        "vps-xl" => Some((MachineSpec { cpu_cores: 16, ram_gb: 64, disk_gb: 800, gpu_type: None, gpu_count: 0, bandwidth_tb: 16 }, vec!["contabo"])),
        "gpu-basic" => Some((MachineSpec { cpu_cores: 4, ram_gb: 8, disk_gb: 50, gpu_type: Some("RTX 3060 12 GB".into()), gpu_count: 1, bandwidth_tb: 2 }, vec!["vast"])),
        "gpu-pro" => Some((MachineSpec { cpu_cores: 8, ram_gb: 32, disk_gb: 200, gpu_type: Some("RTX 4090 24 GB".into()), gpu_count: 1, bandwidth_tb: 4 }, vec!["vast", "contabo"])),
        "gpu-enterprise" => Some((MachineSpec { cpu_cores: 16, ram_gb: 64, disk_gb: 500, gpu_type: Some("A100".into()), gpu_count: 1, bandwidth_tb: 8 }, vec!["vast", "contabo"])),
        _ => None,
    }
}

async fn provision_compute_resource(
    pool: &crate::DbPool,
    store_item_id: &str,
    resource_id: Uuid,
    _workspace_id: Uuid,
    org_id: Uuid,
) -> Result<(), String> {
    let (spec, candidates) = store_item_to_spec(store_item_id)
        .ok_or_else(|| format!("Unknown store item: {store_item_id}"))?;

    use crate::schema_ext::workspace_resources::dsl as wr;
    let mut conn = pool.get().map_err(|e| format!("DB: {e}"))?;

    let org_provider_key: Option<String> = {
        diesel::sql_query(
            "SELECT COALESCE(config->>'provider_api_key', '') AS key FROM cloud_organizations WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .get_result::<ProviderKeyRow>(&mut conn)
        .ok()
        .map(|r| r.key)
    };

    let api_key = match org_provider_key {
        Some(k) if !k.is_empty() => k,
        _ => {
            diesel::update(wr::workspace_resources.filter(wr::id.eq(resource_id)))
                .set(wr::status.eq("provisioning_no_key"))
                .execute(&mut conn)
                .ok();
            return Err("No provider API key configured for organization".into());
        }
    };

    let preferred_provider = std::env::var("GB_PROVIDER").unwrap_or_else(|_| "vast".into());
    let provider_name = if candidates.contains(&preferred_provider.as_str()) {
        preferred_provider.clone()
    } else {
        candidates.first().copied().unwrap_or("vast").to_string()
    };

    let provider: Box<dyn ComputeProvider> = match provider_name.as_str() {
        "vast" => Box::new(VastAiProvider::new()),
        "contabo" => Box::new(ContaboProvider::new()),
        other => return Err(format!("Unknown provider: {other}")),
    };

    match provider.provision(&spec, "US", &api_key).await {
        Ok(result) => {
            let config: serde_json::Value = serde_json::json!({
                "provider": result.provider,
                "instance_id": result.instance_id,
                "ip": result.ip_address,
                "region": result.region,
                "hourly_cost": result.hourly_cost,
            });

            diesel::update(wr::workspace_resources.filter(wr::id.eq(resource_id)))
                .set((
                    wr::status.eq("active"),
                    wr::config.eq(Some(config)),
                ))
                .execute(&mut conn)
                .ok();
            tracing::info!("Provisioned {store_item_id} via {provider_name}: instance={}", result.instance_id);
        }
        Err(e) => {
            let err_msg = e.to_string();
            let config: serde_json::Value = serde_json::json!({ "error": err_msg });
            diesel::update(wr::workspace_resources.filter(wr::id.eq(resource_id)))
                .set((
                    wr::status.eq("provisioning_failed"),
                    wr::config.eq(Some(config)),
                ))
                .execute(&mut conn)
                .ok();
            tracing::warn!("Failed to provision {store_item_id} via {provider_name}: {err_msg}");
            return Err(err_msg);
        }
    }

    Ok(())
}

/// `DELETE /api/cloud/organizations/{org_id}/workspaces/{ws_id}/resources/{res_id}`
async fn remove_resource(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((_org_id, _ws_id, res_id_param)): axum::extract::Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::workspace_resources::dsl as wr;
    diesel::delete(wr::workspace_resources.filter(wr::id.eq(res_id_param)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "removed" })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Branches per organization
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateBranchBody {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBranchBody {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// `GET /api/cloud/organizations/{org_id}/branches`
async fn list_branches(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    #[derive(diesel::QueryableByName, Debug)]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        slug: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        bots_count: i64,
    }

    let rows: Vec<BranchRow> = diesel::sql_query(
        r#"SELECT b.id, b.name, b.slug, b.description, b.is_active,
                  COUNT(DISTINCT bt.id) AS bots_count
           FROM branches b
           LEFT JOIN bots bt ON bt.branch_id = b.id
           WHERE b.org_id = $1
           GROUP BY b.id ORDER BY b.name"#,
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id_param)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "name": r.name,
            "slug": r.slug,
            "description": r.description,
            "is_active": r.is_active,
            "bots_count": r.bots_count,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "branches": result })))
}

/// `POST /api/cloud/organizations/{org_id}/branches`
async fn create_branch_handler(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
    Json(body): Json<CreateBranchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Branch name is required".to_string()));
    }

    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Get tenant_id for this organization
    #[derive(diesel::QueryableByName, Debug)]
    struct TenantIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        tenant_id: Uuid,
    }

    let tenant_row: Option<TenantIdRow> = diesel::sql_query(
        "SELECT tenant_id FROM organizations WHERE org_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id_param)
    .get_result(&mut conn)
    .ok();

    let tenant_id = match tenant_row {
        Some(row) => row.tenant_id,
        None => return Err((StatusCode::NOT_FOUND, "Organization not found".to_string())),
    };

    let branch_id = integration::create_branch_inner(&mut conn, org_id_param, tenant_id, body.name.trim())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "id": branch_id,
        "name": body.name.trim(),
        "org_id": org_id_param,
    })))
}

/// `PUT /api/cloud/organizations/{org_id}/branches/{branch_id}`
async fn update_branch_handler(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, branch_id_param)): axum::extract::Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateBranchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    if let Some(ref n) = body.name {
        if n.trim().is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Branch name cannot be empty".to_string()));
        }
        let slug = n.trim().to_lowercase().replace(' ', "-");
        let affected = diesel::sql_query(
            "UPDATE branches SET name = $1, slug = $2, updated_at = NOW() WHERE id = $3 AND org_id = $4",
        )
        .bind::<diesel::sql_types::Text, _>(n.trim())
        .bind::<diesel::sql_types::Text, _>(&slug)
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;

        if affected == 0 {
            return Err((StatusCode::NOT_FOUND, "Branch not found".to_string()));
        }
    }

    if let Some(ref desc) = body.description {
        diesel::sql_query(
            "UPDATE branches SET description = $1, updated_at = NOW() WHERE id = $2 AND org_id = $3",
        )
        .bind::<diesel::sql_types::Text, _>(desc)
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update desc: {e}")))?;
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/organizations/{org_id}/branches/{branch_id}`
async fn delete_branch_handler(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, branch_id_param)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Unlink bots first
    diesel::sql_query("UPDATE bots SET branch_id = NULL WHERE branch_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .execute(&mut conn)
        .ok();

    let deleted = diesel::sql_query("DELETE FROM branches WHERE id = $1 AND org_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Branch not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Services (add-ons purchased)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/services`
/// Returns provisioned services (active subscriptions).
/// `GET /api/cloud/bots`
async fn list_bots(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);

    #[derive(QueryableByName, Debug)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        slug: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let rows: Vec<BotRow> = if admin {
        diesel::sql_query("SELECT id, name, slug, description, is_active FROM bots ORDER BY name")
            .load(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    } else if let Some(bid) = user_branch_id {
        diesel::sql_query(
            "SELECT id, name, slug, description, is_active FROM bots WHERE branch_id = $1 ORDER BY name"
        )
        .bind::<diesel::sql_types::Uuid, _>(bid)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    } else {
        Vec::new()
    };

    let result: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "name": r.name,
            "slug": r.slug,
            "description": r.description,
            "is_active": r.is_active,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "bots": result })))
}

async fn list_services(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);

    use crate::schema_ext::billing_recurring::dsl::*;
    let mut query = billing_recurring
        .select((id, customer_name, description, frequency, status, amount, currency, interval_count, start_date, created_at))
        .into_boxed();

    if let Some(bid) = user_branch_id {
        query = query.filter(branch_id.eq(bid));
    }

    let subs = query
        .load::<(Uuid, String, Option<String>, String, String, bigdecimal::BigDecimal, String, i32, chrono::NaiveDate, chrono::NaiveDateTime)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = subs.into_iter().map(|(sid, cname, desc, freq, stat, amt, cur, _interval, sdate, cdate)| {
        let plan_label = desc.as_deref().unwrap_or(&cname);
        let friendly_desc = if stat == "active" && amt == bigdecimal::BigDecimal::from(0) {
            "GBO Free Service".to_string()
        } else if stat == "trialing" {
            format!("GBO {} Service (Trial)", plan_label)
        } else if stat == "active" {
            format!("GBO {} Service", plan_label)
        } else {
            plan_label.to_string()
        };
        serde_json::json!({
            "id": sid,
            "name": friendly_desc,
            "description": desc,
            "status": stat,
            "amount": amt.to_string(),
            "currency": cur,
            "period": if _interval > 1 { format!("every {} {}", _interval, freq) } else { freq.to_string() },
            "created_at": cdate.and_utc().to_rfc3339(),
            "expires_at": sdate,
        })
    }).collect();

    let mut result = result;

    // When accessing the cloud for the default org (base system), the host that
    // runs the entire stack is itself the VPS — surface it as a service.
    #[derive(QueryableByName)]
    struct DefaultBranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        value: Uuid,
    }
    let default_branch_id: Option<Uuid> = diesel::sql_query(
        "SELECT b.id AS value FROM branches b JOIN organizations o ON o.org_id = b.org_id \
         WHERE o.slug = 'default' LIMIT 1"
    )
    .get_result::<DefaultBranchRow>(&mut conn)
    .optional()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Default branch query: {e}")))?
    .map(|r| r.value);

    let is_default_access = admin
        || user_branch_id.is_none()
        || (user_branch_id.is_some() && default_branch_id == user_branch_id);

    if is_default_access {
        let scheme = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty())
            .unwrap_or("http");
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let dashboard_url = if !host.is_empty() {
            Some(format!("{scheme}://{host}"))
        } else if !service.config.base_url.is_empty() {
            Some(service.config.base_url.clone())
        } else {
            None
        };
        result.insert(0, serde_json::json!({
            "id": Uuid::nil(),
            "name": "Base System VPS (Own Host)",
            "description": "The host that runs the General Bots base system",
            "status": "active",
            "amount": "0",
            "currency": "USD",
            "period": "monthly",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "expires_at": null,
            "is_base_system": true,
            "dashboard_url": dashboard_url,
        }));
    }

    Ok(Json(serde_json::json!({ "services": result })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Invoices
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/invoices`
async fn list_invoices(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    use botbilling::schema::billing_invoices::dsl::*;
    let mut query = billing_invoices
        .select((id, invoice_number, customer_name, total, status, issue_date, due_date))
        .into_boxed();

    if let Some(bid) = user_branch_id {
        query = query.filter(branch_id.eq(bid));
    }

    let invs = query
        .order(issue_date.desc())
        .limit(50)
        .load::<(Uuid, String, Option<String>, Option<bigdecimal::BigDecimal>, Option<String>, chrono::NaiveDate, Option<chrono::NaiveDate>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = invs.into_iter().map(|(iid, inum, cname, tot, stat, idate, ddate)| {
        serde_json::json!({
            "id": iid,
            "number": inum,
            "customer": cname,
            "total": tot.map(|t| t.to_string()),
            "status": stat,
            "issue_date": idate.to_string(),
            "due_date": ddate.map(|d| d.to_string()),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "invoices": result })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Payment Cards (Stripe SetupIntent) — implemented in `crate::payment_cards`
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Store catalogue (products available for purchase)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/cloud/store`
async fn list_store_items(
    State(_service): State<Arc<SaasService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Static catalog with doubled prices, invisible providers
    let items = serde_json::json!({
        "items": [
            // ── VPS ──
            { "id":"vps-small",  "category":"compute", "name":"VPS Small",  "icon":"🖥️", "price_type":"fixed", "amount":999,  "currency":"usd", "period":"mo", "description":"4 vCPU · 8 GB RAM · 100 GB NVMe · 2 TB BW" },
            { "id":"vps-medium", "category":"compute", "name":"VPS Medium", "icon":"🖥️", "price_type":"fixed", "amount":1999, "currency":"usd", "period":"mo", "description":"6 vCPU · 16 GB RAM · 200 GB NVMe · 4 TB BW" },
            { "id":"vps-large",  "category":"compute", "name":"VPS Large",  "icon":"🖥️", "price_type":"fixed", "amount":3999, "currency":"usd", "period":"mo", "description":"8 vCPU · 32 GB RAM · 400 GB NVMe · 8 TB BW" },
            { "id":"vps-xl",     "category":"compute", "name":"VPS XL",     "icon":"🖥️", "price_type":"fixed", "amount":7999, "currency":"usd", "period":"mo", "description":"16 vCPU · 64 GB RAM · 800 GB NVMe · 16 TB BW" },
            // ── GPU ──
            { "id":"gpu-basic",      "category":"compute", "name":"GPU Basic",      "icon":"⚡", "price_type":"fixed", "amount":3999,  "currency":"usd", "period":"mo", "description":"RTX 3060 12 GB · 4 vCPU · 8 GB RAM" },
            { "id":"gpu-pro",        "category":"compute", "name":"GPU Pro",        "icon":"⚡", "price_type":"fixed", "amount":9999,  "currency":"usd", "period":"mo", "description":"RTX 4090 (24 GB VRAM) · 8 vCPU · 32 GB RAM" },
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
    // In production: decode JWT from Authorization header and fetch profile
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
    // Stub: update name/org in DB associated with the authenticated user's JWT
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

    let branch_id = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    let effective_branch_id = body.org_id.unwrap_or(branch_id);
    let now = chrono::Utc::now();
    let zero = bigdecimal::BigDecimal::from(0);

    let invoice_id = Uuid::new_v4();
    let invoice_num = botbilling::api_models::generate_invoice_number(&mut conn, effective_branch_id);

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
            botbilling::schema::billing_invoices::branch_id.eq(effective_branch_id),
            botbilling::schema::billing_invoices::invoice_number.eq(&invoice_num),
            botbilling::schema::billing_invoices::customer_name.eq(Some(&contact_name)),
            botbilling::schema::billing_invoices::customer_email.eq(Some(body.email)),
            botbilling::schema::billing_invoices::status.eq(Some("draft")),
            botbilling::schema::billing_invoices::issue_date.eq(now.date_naive()),
            botbilling::schema::billing_invoices::due_date.eq(Some((now + chrono::Duration::days(30)).date_naive())),
            botbilling::schema::billing_invoices::subtotal.eq(&zero),
            botbilling::schema::billing_invoices::total.eq(Some(&zero)),
            botbilling::schema::billing_invoices::amount_due.eq(&zero),
            botbilling::schema::billing_invoices::currency.eq(Some("usd")),
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
/// Creates an invoice for the app store publishing consultancy service.
/// Payment is processed through the existing checkout flow.
async fn handle_appstore_purchase(
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
        .map(|(fn_, ln_)| [fn_.unwrap_or_default(), ln_.unwrap_or_default()].join(" "))
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
async fn list_offers() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
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

/// `GET /api/cloud/llm-providers`
async fn list_llm_providers() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Try to read from llm_releases.json for the complete catalog
    let json_path = PathBuf::from("3rdparty/llm_releases.json");
    if let Ok(content) = tokio::fs::read_to_string(&json_path).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(providers) = val.get("providers") {
                return Ok(Json(serde_json::json!({ "providers": providers })));
            }
        }
    }

    // Fallback: return hardcoded providers so the frontend always works
    Ok(Json(serde_json::json!({
        "providers": [
            {
                "id": "zhipu", "name": "GLM (Zhipu AI)",
                "description": "Chinese models from Zhipu AI with excellent reasoning performance and long context.",
                "website": "https://open.bigmodel.cn", "requires_byok": false, "icon": "glm",
                "models": [
                    {"id": "glm-5.2", "name": "GLM-5.2", "context": 262144, "description": "Flagship model with deep reasoning and agentic capabilities", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools","vision"]},
                    {"id": "glm-5.2-air", "name": "GLM-5.2-Air", "context": 131072, "description": "Lightweight and fast for chatbots", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools"]},
                    {"id": "glm-4-flash", "name": "GLM-4-Flash", "context": 131072, "description": "Free tier with high request rate", "pricing": "free-tier", "capabilities": ["chat"]}
                ]
            },
            {
                "id": "alibaba", "name": "Qwen (Alibaba Cloud)",
                "description": "Alibaba's Qwen 3.6 family — latest-generation open models with breakthrough performance.",
                "website": "https://tongyi.aliyun.com", "requires_byok": false, "icon": "qwen",
                "models": [
                    {"id": "qwen-3.6-max", "name": "Qwen 3.6-Max", "context": 262144, "description": "Most powerful in the 3.6 family", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools","reasoning"]},
                    {"id": "qwen-3.6-plus", "name": "Qwen 3.6-Plus", "context": 131072, "description": "Performance-cost balance", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools"]},
                    {"id": "qwen-3.6-turbo", "name": "Qwen 3.6-Turbo", "context": 131072, "description": "Fast and economical", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat"]}
                ]
            },
            {
                "id": "deepseek", "name": "DeepSeek",
                "description": "Deep reasoning models from DeepSeek (深度求索).",
                "website": "https://platform.deepseek.com", "requires_byok": false, "icon": "deepseek",
                "models": [
                    {"id": "deepseek-v4-flash", "name": "DeepSeek V4 Flash", "context": 131072, "description": "Latest generation — fast and powerful reasoning model", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools","reasoning"]},
                    {"id": "deepseek-r1", "name": "DeepSeek-R1", "context": 65536, "description": "Reasoning with chain-of-thought", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","reasoning"]},
                    {"id": "deepseek-v3", "name": "DeepSeek-V3", "context": 65536, "description": "Previous gen — still available for cost savings", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools"]}
                ]
            },
            {
                "id": "minimax", "name": "MiniMax",
                "description": "Chinese models with up to 1M token context.",
                "website": "https://www.minimaxi.com", "requires_byok": false, "icon": "minimax",
                "models": [
                    {"id": "minimax-text-01", "name": "MiniMax-Text-01", "context": 1048576, "description": "1M token context", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools"]},
                    {"id": "minimax-abab-6.5", "name": "MiniMax-abab6.5", "context": 131072, "description": "Efficient for conversation", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat"]}
                ]
            },
            {
                "id": "yi", "name": "Yi (01.AI)",
                "description": "Models from 01.AI (Kai-Fu Lee) with multilingual performance.",
                "website": "https://www.lingyiwanwu.com", "requires_byok": false, "icon": "yi",
                "models": [
                    {"id": "yi-lightning", "name": "Yi-Lightning", "context": 131072, "description": "Flagship with advanced reasoning", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools"]},
                    {"id": "yi-lightning-fast", "name": "Yi-Lightning-Fast", "context": 32768, "description": "Optimized for low latency", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat"]}
                ]
            },
            {
                "id": "openai", "name": "OpenAI",
                "description": "Frontier models: GPT-5.5, GPT-5.4 and o-5 reasoning.",
                "website": "https://platform.openai.com", "requires_byok": false, "icon": "openai",
                "models": [
                    {"id": "gpt-5.5", "name": "GPT-5.5", "context": 1048576, "description": "Frontier multimodal intelligence — 1M context", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","vision","tools","reasoning"]},
                    {"id": "gpt-5.4", "name": "GPT-5.4", "context": 262144, "description": "Previous frontier — still excellent for production", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","vision","tools","reasoning"]},
                    {"id": "o-5", "name": "o-5", "context": 524288, "description": "Advanced reasoning with full chain-of-thought", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","reasoning"]}
                ]
            },
            {
                "id": "anthropic", "name": "Anthropic",
                "description": "Claude Fable 5 (Mythos-class), Opus 4.8, Sonnet 4.6, Haiku 4.5 — no legacy 3.x.",
                "website": "https://console.anthropic.com", "requires_byok": false, "icon": "anthropic",
                "models": [
                    {"id": "claude-fable-5", "name": "Claude Fable 5", "context": 1048576, "description": "Mythos-class — Anthropic's most capable model", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools","vision","reasoning"]},
                    {"id": "claude-opus-4-8", "name": "Claude Opus 4.8", "context": 1048576, "description": "Top Opus-tier — complex reasoning & agentic coding", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools","vision","reasoning"]},
                    {"id": "claude-sonnet-4-6", "name": "Claude Sonnet 4.6", "context": 1048576, "description": "Best speed-intelligence balance for production", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools","vision"]},
                    {"id": "claude-haiku-4-5", "name": "Claude Haiku 4.5", "context": 204800, "description": "Fastest — high-volume, cost-sensitive tasks", "pricing": "token-package", "package_url": "/cloud/store?calc=1#calc-llm-grid", "capabilities": ["chat","tools"]}
                ]
            },
            {
                "id": "google", "name": "Google",
                "description": "Gemini 3.5 Flash and 3.1 Pro — agentic frontier. Token packages available.",
                "website": "https://ai.google.dev", "requires_byok": false, "icon": "google",
                "models": [
                    {"id": "gemini-3.5-flash", "name": "Gemini 3.5 Flash", "context": 1048576, "description": "GA — frontier agentic performance, 1M context", "pricing": "token-package", "capabilities": ["chat","vision","tools","reasoning","code-execution"]},
                    {"id": "gemini-3.1-pro", "name": "Gemini 3.1 Pro", "context": 1048576, "description": "Preview — advanced reasoning for complex tasks", "pricing": "token-package", "capabilities": ["chat","vision","tools","reasoning"]}
                ]
            },
            {
                "id": "groq", "name": "Groq",
                "description": "Ultra-fast inference on LPU. No Llama, no Mistral — only GPT-OSS and Qwen.",
                "website": "https://groq.com", "requires_byok": false, "icon": "groq",
                "models": [
                    {"id": "gpt-oss-120b", "name": "GPT-OSS 120B", "context": 131072, "description": "120B MoE — reasoning at 500 tok/s on LPU", "pricing": "token-package", "capabilities": ["chat","tools","reasoning"]},
                    {"id": "qwen-3.6-27b", "name": "Qwen 3.6-27B", "context": 131072, "description": "Best open 27B — 500 tok/s on Groq", "pricing": "token-package", "capabilities": ["chat","tools","reasoning"]},
                    {"id": "gpt-oss-20b", "name": "GPT-OSS 20B", "context": 131072, "description": "20B at 1000 tok/s — fastest option", "pricing": "token-package", "capabilities": ["chat","tools"]}
                ]
            },
            {
                "id": "generalbots", "name": "General Bots (Own GPU)",
                "description": "Open-weight models running on General Bots' own GPU infrastructure. Included in all plans.",
                "website": "https://generalbots.com.br", "requires_byok": false, "icon": "gb",
                "models": [
                    {"id": "qwen-3.6-27b", "name": "Qwen 3.6-27B", "context": 131072, "description": "27B parameters — best open model in its class", "pricing": "included", "capabilities": ["chat","tools","reasoning"]},
                    {"id": "deepseek-r1-distill-qwen", "name": "DeepSeek-R1-Distill-Qwen-1.5B", "context": 32768, "description": "Lightweight reasoning included in all plans", "pricing": "included", "capabilities": ["chat","reasoning"]},
                    {"id": "gpt-oss-20b", "name": "GPT-OSS 20B", "context": 32768, "description": "20B parameters on dedicated GPU", "pricing": "included", "capabilities": ["chat","tools"]}
                ]
            }
        ]
    })))
}

async fn handle_topup(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<TopupBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Connection: {e}")))?;

    // Retrieve default branch_id
    let branch_id = botbilling::get_bot_context(&service.billing_state.pool, &service.billing_state.get_default_bot);
    let effective_branch_id = if branch_id == Uuid::nil() {
        Uuid::nil()
    } else {
        branch_id
    };

    use std::str::FromStr;
    let decimal_amount = bigdecimal::BigDecimal::from_str(&format!("{:.2}", body.amount))
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid amount: {e}")))?;

    let zero = bigdecimal::BigDecimal::from(0);

    // Generate unique invoice number
    use rand::Rng;
    let mut rng = rand::rng();
    let num: u32 = rng.random_range(100_000..999_999);
    let invoice_num = format!("INV-TOPUP-{}", num);

    // Insert top-up invoice in database as paid
    let new_invoice_id = Uuid::new_v4();
    
    // Get contact name corresponding to the email, or use email as fallback
    use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, first_name, last_name};
    let contact_name = crm_contacts
        .filter(email.eq(&body.email))
        .select((first_name, last_name))
        .first::<(Option<String>, Option<String>)>(&mut conn)
        .map(|(fn_, ln_)| [fn_.unwrap_or_default(), ln_.unwrap_or_default()].join(" "))
        .map(|s| if s.trim().is_empty() { body.email.split('@').next().unwrap_or("Customer").to_string() } else { s })
        .unwrap_or_else(|_| body.email.split('@').next().unwrap_or("Customer").to_string());

    // Insert record in billing_invoices table
    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values((
            botbilling::schema::billing_invoices::id.eq(new_invoice_id),
            botbilling::schema::billing_invoices::branch_id.eq(effective_branch_id),
            botbilling::schema::billing_invoices::invoice_number.eq(&invoice_num),
            botbilling::schema::billing_invoices::customer_name.eq(Some(&contact_name)),
            botbilling::schema::billing_invoices::customer_email.eq(Some(body.email)),
            botbilling::schema::billing_invoices::status.eq(Some("paid")),
            botbilling::schema::billing_invoices::issue_date.eq(chrono::Local::now().date_naive()),
            botbilling::schema::billing_invoices::due_date.eq(Some(chrono::Local::now().date_naive())),
            botbilling::schema::billing_invoices::subtotal.eq(&decimal_amount),
            botbilling::schema::billing_invoices::tax_rate.eq(&zero),
            botbilling::schema::billing_invoices::tax_amount.eq(&zero),
            botbilling::schema::billing_invoices::discount_percent.eq(&zero),
            botbilling::schema::billing_invoices::discount_amount.eq(&zero),
            botbilling::schema::billing_invoices::total.eq(Some(&decimal_amount)),
            botbilling::schema::billing_invoices::amount_paid.eq(&decimal_amount),
            botbilling::schema::billing_invoices::amount_due.eq(&zero),
            botbilling::schema::billing_invoices::currency.eq(Some("usd")),
            botbilling::schema::billing_invoices::notes.eq(Some("Account balance top-up via Special Offers".to_string())),
            botbilling::schema::billing_invoices::paid_at.eq(Some(chrono::Utc::now())),
            botbilling::schema::billing_invoices::created_at.eq(chrono::Utc::now()),
            botbilling::schema::billing_invoices::updated_at.eq(chrono::Utc::now()),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create topup invoice: {e}")))?;

    // Create payment in billing_payments to record the transaction
    let payment_id = Uuid::new_v4();
    let payment_num = format!("PAY-TOPUP-{}", num);
    diesel::insert_into(botbilling::schema::billing_payments::table)
        .values((
            botbilling::schema::billing_payments::id.eq(payment_id),
            botbilling::schema::billing_payments::branch_id.eq(effective_branch_id),
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

// ─────────────────────────────────────────────────────────────────────────────
// BYOK: Bring Your Own Key — Encrypted Server-Side Storage
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ByokSaveBody {
    keys: std::collections::HashMap<String, String>,
}

/// `POST /api/cloud/tenant/settings/byok`
///
/// Receives BYOK API keys from the frontend and stores them in Vault.
async fn handle_save_byok(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Json(body): Json<ByokSaveBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (org_id, branch_id) = {
        let mut conn = service.pool().get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB connection: {e}")))?;
        let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "No tenant is associated with this account".to_string()))?;
        #[derive(diesel::QueryableByName)]
        struct TenantOrg {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        let org_id = diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .get_result::<TenantOrg>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tenant lookup: {e}")))?
            .org_id;
        (org_id, branch_id)
    };
    let path = format!("gbo/{org_id}/{branch_id}/{}", uuid::Uuid::nil());
    let sm = botcoresecrets::manager::SecretsManager::get_clone()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault: {e}")))?;

    let mut saved_count = 0u32;
    let mut secrets = match sm.get_secret(&path).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("No existing tenant LLM credentials at {path}: {e}");
            std::collections::HashMap::new()
        }
    };
    for (provider_id, api_key) in &body.keys {
        if api_key.trim().is_empty() {
            continue;
        }
        if provider_id.len() > 64 || !provider_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err((StatusCode::BAD_REQUEST, "Invalid provider id".to_string()));
        }
        if api_key.len() > 8192 {
            return Err((StatusCode::BAD_REQUEST, "API key is too large".to_string()));
        }
        let config_key = format!("byok_{provider_id}");
        secrets.insert(config_key, api_key.clone());
        saved_count += 1;
    }
    sm.put_secret(&path, secrets).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault write: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "saved": saved_count,
        "providers": body.keys.keys().collect::<Vec<&String>>(),
    })))
}

#[derive(Debug, Serialize, Deserialize)]
struct LlmOAuthState {
    provider: String,
    org_id: Uuid,
    branch_id: Uuid,
    return_url: String,
    exp: i64,
    nonce: Uuid,
}

#[derive(Debug, Deserialize)]
struct LlmOAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    token_type: Option<String>,
    scope: Option<String>,
}

fn request_origin(headers: &HeaderMap, fallback: &str) -> String {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.contains('/') && !value.contains('\\'));
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .filter(|value| *value == "http" || *value == "https")
        .unwrap_or("http");
    host.map(|value| format!("{scheme}://{value}"))
        .unwrap_or_else(|| fallback.trim_end_matches('/').to_string())
}

async fn google_oauth_client() -> Result<std::collections::HashMap<String, String>, (StatusCode, String)> {
    let manager = botcoresecrets::manager::SecretsManager::get_clone()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault: {e}")))?;
    let config = manager.get_secret("gbo/cloud/oauth/google").await
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE,
            "Google OAuth requires administrator configuration".to_string()))?;
    let has_id = config.get("client_id").is_some_and(|value| !value.trim().is_empty());
    let has_secret = config.get("client_secret").is_some_and(|value| !value.trim().is_empty());
    if !has_id || !has_secret {
        return Err((StatusCode::SERVICE_UNAVAILABLE,
            "Google OAuth requires administrator configuration".to_string()));
    }
    Ok(config)
}

fn decode_llm_oauth_state(token: &str, secret: &str) -> Result<LlmOAuthState, (StatusCode, String)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()));
    }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    if jwt_sign_inner(&signing_input, secret.as_bytes()) != parts[2] {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()));
    }
    let payload = base64_url_decode(parts[1])
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()))?;
    let state: LlmOAuthState = serde_json::from_slice(&payload)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()))?;
    if state.exp < chrono::Utc::now().timestamp() {
        return Err((StatusCode::BAD_REQUEST, "OAuth state expired".to_string()));
    }
    Ok(state)
}

async fn handle_oauth_start(
    State(service): State<Arc<SaasService>>,
    Path(provider): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if provider != "google" {
        return Err((StatusCode::BAD_REQUEST, "OAuth is not available for this provider".to_string()));
    }
    let (org_id, branch_id) = {
        let mut conn = service.pool().get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB connection: {e}")))?;
        let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "No tenant is associated with this account".to_string()))?;
        #[derive(diesel::QueryableByName)]
        struct TenantOrg {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        let org_id = diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .get_result::<TenantOrg>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tenant lookup: {e}")))?
            .org_id;
        (org_id, branch_id)
    };
    let config = google_oauth_client().await?;
    let origin = request_origin(&headers, &service.config.base_url);
    let redirect_uri = config.get("redirect_uri").cloned().unwrap_or_else(|| {
        format!("{origin}/api/cloud/tenant/settings/oauth/google/callback")
    });
    let state = LlmOAuthState {
        provider: provider.clone(),
        org_id,
        branch_id,
        return_url: format!("{origin}/llm?connected=google"),
        exp: chrono::Utc::now().timestamp() + 600,
        nonce: Uuid::new_v4(),
    };
    let state_json = serde_json::to_vec(&state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth state: {e}")))?;
    let header = base64_url_encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = base64_url_encode(&state_json);
    let signed_state = jwt_sign(&header, &payload, service.config.jwt_secret.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let client_id = config.get("client_id")
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth requires administrator configuration".to_string()))?;
    let mut auth_url = url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth URL: {e}")))?;
    auth_url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/generative-language.retriever")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &signed_state);
    Ok(Json(serde_json::json!({ "authorization_url": auth_url.as_str() })))
}

async fn handle_google_oauth_callback(
    State(service): State<Arc<SaasService>>,
    Query(query): Query<LlmOAuthCallbackQuery>,
) -> Result<Response, (StatusCode, String)> {
    if query.error.is_some() {
        return Err((StatusCode::BAD_REQUEST, "Google OAuth authorization was not completed".to_string()));
    }
    let code = query.code.filter(|value| !value.trim().is_empty())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing OAuth authorization code".to_string()))?;
    let encoded_state = query.state
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing OAuth state".to_string()))?;
    let state = decode_llm_oauth_state(&encoded_state, &service.config.jwt_secret)?;
    if state.provider != "google" {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth provider".to_string()));
    }
    let config = google_oauth_client().await?;
    let client_id = config.get("client_id")
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth requires administrator configuration".to_string()))?;
    let client_secret = config.get("client_secret")
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth requires administrator configuration".to_string()))?;
    let redirect_uri = config.get("redirect_uri").cloned().unwrap_or_else(|| {
        let origin = state.return_url.split("/llm").next().unwrap_or("");
        format!("{origin}/api/cloud/tenant/settings/oauth/google/callback")
    });
    let response = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Google OAuth token exchange failed: {e}")))?;
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "Google OAuth token exchange rejected");
        return Err((StatusCode::BAD_GATEWAY, "Google OAuth token exchange was rejected".to_string()));
    }
    let token: GoogleTokenResponse = response.json().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Invalid Google OAuth response: {e}")))?;
    let manager = botcoresecrets::manager::SecretsManager::get_clone()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault: {e}")))?;
    let path = format!("gbo/{}/{}/{}", state.org_id, state.branch_id, Uuid::nil());
    let mut secrets = match manager.get_secret(&path).await {
        Ok(values) => values,
        Err(error) => {
            tracing::warn!("No existing tenant LLM credentials at {path}: {error}");
            std::collections::HashMap::new()
        }
    };
    secrets.insert("oauth_google_access_token".to_string(), token.access_token);
    if let Some(refresh_token) = token.refresh_token {
        secrets.insert("oauth_google_refresh_token".to_string(), refresh_token);
    }
    if let Some(expires_in) = token.expires_in {
        secrets.insert("oauth_google_expires_at".to_string(),
            (chrono::Utc::now().timestamp() + expires_in).to_string());
    }
    if let Some(token_type) = token.token_type {
        secrets.insert("oauth_google_token_type".to_string(), token_type);
    }
    if let Some(scope) = token.scope {
        secrets.insert("oauth_google_scope".to_string(), scope);
    }
    manager.put_secret(&path, secrets).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault write: {e}")))?;
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("location", state.return_url)
        .body(Body::empty())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth redirect: {e}")))
}

