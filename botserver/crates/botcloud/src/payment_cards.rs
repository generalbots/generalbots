//! Stripe payment card management (SetupIntent flow).
//!
//! Cards are collected exclusively on Stripe's side — either via a hosted
//! Checkout session in `setup` mode (used by the cloud UI) or via a raw
//! SetupIntent `client_secret` for clients embedding Stripe Elements. Card
//! numbers never transit our servers or database (PCI SAQ-A); we persist only
//! display metadata (brand, last4, expiry) synced from Stripe webhooks.
//!
//! RBAC: cards are scoped to the authenticated user's branch. The branch owner
//! (or the SaaS super-admin) is the only actor allowed to add / set default /
//! delete cards; every mutation is recorded in `cloud_audit_log`.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::{base64_url_decode, get_branch_id_from_jwt, is_super_admin};
use crate::SaasService;

/// Extracts the `email` claim from the Bearer JWT, if present.
pub fn jwt_email(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() == 3 {
                base64_url_decode(parts[1])
                    .ok()
                    .and_then(|decoded| serde_json::from_slice::<serde_json::Value>(&decoded).ok())
                    .and_then(|payload| payload.get("email").and_then(|v| v.as_str()).map(str::to_string))
            } else {
                None
            }
        })
}

/// Records one durable billing audit event (`cloud_audit_log`).
pub fn record_cloud_audit(
    service: &SaasService,
    branch_id: Uuid,
    actor_email: Option<&str>,
    action: &str,
    entity: &str,
    entity_id: Option<&str>,
    details: Option<&str>,
) {
    let Some(mut conn) = service.pool().get().ok() else {
        tracing::warn!("billing audit: DB unavailable, event dropped ({action})");
        return;
    };
    let actor = actor_email.map(str::to_string);
    let entity_id_owned = entity_id.map(str::to_string);
    let details_owned = details.map(str::to_string);
    let result = diesel::sql_query(
        "INSERT INTO cloud_audit_log \
         (branch_id, actor_email, action, entity, entity_id, details) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(actor)
    .bind::<diesel::sql_types::Text, _>(action)
    .bind::<diesel::sql_types::Text, _>(entity)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity_id_owned)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(details_owned)
    .execute(&mut conn);
    if let Err(e) = result {
        tracing::error!("billing audit: failed to persist event ({action}): {e}");
    }
}

/// Resolves the Stripe Customer id for a branch, creating and persisting the
/// customer mapping when one does not exist yet. Persisted in `billing_customers`
/// so webhook events carrying only a `customer` id can be re-attributed.
pub async fn resolve_stripe_customer(
    service: &SaasService,
    email: &str,
    branch_id: Uuid,
) -> Result<String, String> {
    // 1. Cached mapping in our database.
    {
        let mut conn = service
            .pool()
            .get()
            .map_err(|e| format!("DB pool: {e}"))?;
        use crate::schema_ext::billing_customers::dsl::{
            billing_customers, branch_id as bc_branch, stripe_customer_id as bc_stripe,
        };
        let existing: Option<String> = billing_customers
            .filter(bc_branch.eq(branch_id))
            .select(bc_stripe)
            .first(&mut conn)
            .optional()
            .map_err(|e| format!("Query billing_customers: {e}"))?;
        if let Some(customer_id) = existing {
            return Ok(customer_id);
        }
    }

    // 2. Ask Stripe whether a customer for this email already exists.
    if let Some(customer) = service
        .stripe
        .find_customer_by_email(email)
        .await
        .map_err(|e| format!("Stripe lookup: {e}"))?
    {
        persist_customer_mapping(service, branch_id, &customer.id, email)?;
        return Ok(customer.id);
    }

    // 3. No customer yet — create one and remember it.
    let customer = service
        .stripe
        .create_customer(botbilling::stripe_integration::CreateCustomerParams {
            email: email.to_string(),
            name: None,
            organization_id: branch_id,
            metadata: std::collections::HashMap::new(),
        })
        .await
        .map_err(|e| format!("Stripe customer: {e}"))?;
    persist_customer_mapping(service, branch_id, &customer.id, email)?;
    Ok(customer.id)
}

pub(crate) fn persist_customer_mapping(
    service: &SaasService,
    branch_id: Uuid,
    stripe_customer_id: &str,
    email: &str,
) -> Result<(), String> {
    let mut conn = service.pool().get().map_err(|e| format!("DB pool: {e}"))?;
    use crate::schema_ext::billing_customers::dsl::{
        billing_customers, id as bc_id, branch_id as bc_branch, email as bc_email,
        stripe_customer_id as bc_stripe,
    };
    diesel::insert_into(billing_customers)
        .values((
            bc_id.eq(Uuid::new_v4()),
            bc_branch.eq(branch_id),
            bc_stripe.eq(stripe_customer_id),
            bc_email.eq(email),
        ))
        .on_conflict(bc_branch)
        .do_update()
        .set((
            bc_stripe.eq(stripe_customer_id),
            bc_email.eq(email),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Upsert billing_customers: {e}"))?;
    Ok(())
}

/// `GET /api/cloud/payment-cards`
///
/// Lists saved cards for the authenticated user's branch, read from our local
/// table (kept in sync by Stripe webhooks). Super-admins without a branch see
/// every saved card.
pub async fn list_payment_cards(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service
        .pool()
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);

    use crate::schema_ext::billing_payment_methods::dsl::{
        billing_payment_methods, id, stripe_pm_id, brand, last4, exp_month, exp_year, is_default,
        stripe_customer_id, branch_id, created_at,
    };
    let mut query = billing_payment_methods
        .select((
            id,
            stripe_pm_id,
            brand,
            last4,
            exp_month,
            exp_year,
            is_default,
            stripe_customer_id,
        ))
        .into_boxed();

    if let Some(branch) = user_branch_id {
        query = query.filter(branch_id.eq(branch));
    } else if !admin {
        return Err((StatusCode::FORBIDDEN, "No billing scope for this account".to_string()));
    }

    let rows = query
        .order(is_default.desc())
        .order(created_at.asc())
        .load::<(Uuid, String, String, String, i32, i32, bool, String)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let cards: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(card_id, pm_id, brand_name, last, exp_m, exp_y, is_def, cust)| {
            serde_json::json!({
                "id": card_id,
                "stripe_pm_id": pm_id,
                "brand": brand_name,
                "last4": last,
                "exp_month": exp_m,
                "exp_year": exp_y,
                "is_default": is_def,
                "customer_id": cust,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "cards": cards })))
}

/// `POST /api/cloud/payment-cards/setup`
///
/// Creates a hosted Stripe Checkout session in `setup` mode. The browser is
/// redirected to `url`; Stripe collects the card on its own domain and fires
/// `checkout.session.completed` (mode=setup) when done.
pub async fn create_payment_card_setup(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service
        .pool()
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;
    let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::FORBIDDEN, "No billing scope for this account".to_string()))?;
    let email = jwt_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing email claim".to_string()))?;

    let customer_id = resolve_stripe_customer(&service, &email, branch_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let base = service.config.base_url.trim_end_matches('/').to_string();
    let session = service
        .stripe
        .create_setup_checkout_session(
            botbilling::stripe_integration::CreateSetupCheckoutSessionParams {
                customer_id,
                success_url: format!("{base}/payment-cards?added=1"),
                cancel_url: format!("{base}/payment-cards?cancelled=1"),
                metadata: std::collections::HashMap::from([
                    ("branch_id".to_string(), branch_id.to_string()),
                    ("email".to_string(), email.clone()),
                ]),
            },
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe session: {e}")))?;

    record_cloud_audit(
        &service,
        branch_id,
        Some(email.as_str()),
        "card.add.initiated",
        "payment_method",
        None,
        Some(&format!("checkout_session={}", session.id)),
    );

    Ok(Json(serde_json::json!({ "url": session.url, "session_id": session.id })))
}

/// `POST /api/cloud/payment-cards/setup-intent`
///
/// Creates a raw SetupIntent and returns the `client_secret` for clients that
/// embed Stripe Elements directly (headless / native integrations). The card is
/// attached and synced via the `payment_method.attached` webhook.
pub async fn create_payment_card_setup_intent(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service
        .pool()
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;
    let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::FORBIDDEN, "No billing scope for this account".to_string()))?;
    let email = jwt_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing email claim".to_string()))?;

    let customer_id = resolve_stripe_customer(&service, &email, branch_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let intent = service
        .stripe
        .create_setup_intent(botbilling::stripe_integration::CreateSetupIntentParams {
            customer_id: customer_id.clone(),
            usage: "off_session".to_string(),
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe SetupIntent: {e}")))?;

    Ok(Json(serde_json::json!({
        "client_secret": intent.client_secret,
        "setup_intent_id": intent.id,
        "customer_id": customer_id,
    })))
}

/// `POST /api/cloud/payment-cards/:pm_id/default`
///
/// Sets the default payment method on the Stripe customer and mirrors the flag
/// locally. Only the branch owner (or super-admin) may change the default.
pub async fn set_default_payment_card(
    State(service): State<Arc<SaasService>>,
    Path(pm_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service
        .pool()
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;
    let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);
    let email = jwt_email(&headers);

    use crate::schema_ext::billing_payment_methods::dsl::{
        billing_payment_methods, branch_id as pm_branch, stripe_customer_id as pm_customer,
        stripe_pm_id, is_default, updated_at,
    };
    let row: Option<(Uuid, String)> = billing_payment_methods
        .filter(stripe_pm_id.eq(&pm_id))
        .select((pm_branch, pm_customer))
        .first(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let Some((card_branch, customer_id)) = row else {
        return Err((StatusCode::NOT_FOUND, "Card not found".to_string()));
    };
    if branch_id != Some(card_branch) && !admin {
        return Err((StatusCode::FORBIDDEN, "Card does not belong to this account".to_string()));
    }

    service
        .stripe
        .set_default_payment_method(&customer_id, &pm_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe: {e}")))?;

    // Clear the previous default within the branch, then mark the new one.
    let now = chrono::Utc::now();
    diesel::update(billing_payment_methods.filter(pm_branch.eq(card_branch)))
        .set(is_default.eq(false))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update defaults: {e}")))?;
    diesel::update(billing_payment_methods.filter(stripe_pm_id.eq(&pm_id)))
        .set((is_default.eq(true), updated_at.eq(now)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update card: {e}")))?;

    record_cloud_audit(
        &service,
        card_branch,
        email.as_deref(),
        "card.set_default",
        "payment_method",
        Some(&pm_id),
        None,
    );

    Ok(Json(serde_json::json!({ "ok": true, "default": pm_id })))
}

/// `DELETE /api/cloud/payment-cards/:pm_id`
///
/// Detaches the card from the Stripe customer and removes the local record.
/// The last card of a branch is never auto-unset; Stripe requires at least the
/// default handling on its side, which the portal covers.
pub async fn delete_payment_card(
    State(service): State<Arc<SaasService>>,
    Path(pm_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service
        .pool()
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;
    let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);
    let email = jwt_email(&headers);

    use crate::schema_ext::billing_payment_methods::dsl::{
        billing_payment_methods, branch_id as pm_branch, stripe_customer_id as pm_customer,
        stripe_pm_id, is_default, created_at,
    };
    let row: Option<(Uuid, String, bool)> = billing_payment_methods
        .filter(stripe_pm_id.eq(&pm_id))
        .select((pm_branch, pm_customer, is_default))
        .first(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let Some((card_branch, customer_id, was_default)) = row else {
        return Err((StatusCode::NOT_FOUND, "Card not found".to_string()));
    };
    if branch_id != Some(card_branch) && !admin {
        return Err((StatusCode::FORBIDDEN, "Card does not belong to this account".to_string()));
    }

    service
        .stripe
        .detach_payment_method(&pm_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe detach: {e}")))?;

    diesel::delete(billing_payment_methods.filter(stripe_pm_id.eq(&pm_id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    // If the default was deleted, promote the oldest remaining card.
    if was_default {
        let next: Option<String> = billing_payment_methods
            .filter(pm_branch.eq(card_branch))
            .select(stripe_pm_id)
            .order(created_at.asc())
            .first(&mut conn)
            .optional()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query next: {e}")))?;
        if let Some(next_pm) = next {
            if let Err(e) = service.stripe.set_default_payment_method(&customer_id, &next_pm).await {
                tracing::warn!("promote default after delete failed: {e}");
            } else {
                diesel::update(billing_payment_methods.filter(stripe_pm_id.eq(&next_pm)))
                    .set(is_default.eq(true))
                    .execute(&mut conn)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Promote: {e}")))?;
            }
        }
    }

    record_cloud_audit(
        &service,
        card_branch,
        email.as_deref(),
        "card.delete",
        "payment_method",
        Some(&pm_id),
        None,
    );

    Ok(Json(serde_json::json!({ "ok": true, "deleted": pm_id })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Webhook-driven sync (keeps our records consistent with Stripe's source of truth)
// ─────────────────────────────────────────────────────────────────────────────

/// Upserts (or removes) a `billing_payment_methods` row from a Stripe
/// PaymentMethod payload. Resolves the owning branch via `billing_customers`.
pub async fn sync_payment_method(
    service: &SaasService,
    pm: &botbilling::stripe_integration::StripePaymentMethod,
    detach: bool,
) -> Result<(), String> {
    let customer_id = pm
        .customer
        .as_deref()
        .ok_or_else(|| "payment method has no customer".to_string())?;

    let branch_id = {
        let mut conn = service.pool().get().map_err(|e| format!("DB pool: {e}"))?;
        use crate::schema_ext::billing_customers::dsl::{
            billing_customers, branch_id as bc_branch, stripe_customer_id as bc_stripe,
        };
        billing_customers
            .filter(bc_stripe.eq(customer_id))
            .select(bc_branch)
            .first::<Uuid>(&mut conn)
            .optional()
            .map_err(|e| format!("Query billing_customers: {e}"))?
    };
    let Some(branch_id) = branch_id else {
        tracing::warn!(
            "payment_method webhook: no local customer mapping for {customer_id}, skipping"
        );
        return Ok(());
    };

    let mut conn = service.pool().get().map_err(|e| format!("DB pool: {e}"))?;
    use crate::schema_ext::billing_payment_methods::dsl::{
        billing_payment_methods, stripe_pm_id, brand, last4, exp_month, exp_year, is_default,
        stripe_customer_id as pm_customer, branch_id as pm_branch, id, created_at, updated_at,
    };

    if detach {
        diesel::delete(billing_payment_methods.filter(stripe_pm_id.eq(&pm.id)))
            .execute(&mut conn)
            .map_err(|e| format!("Delete payment method: {e}"))?;
        record_cloud_audit(service, branch_id, None, "card.delete.webhook", "payment_method", Some(&pm.id), None);
        return Ok(());
    }

    let Some(card) = pm.card.as_ref() else {
        tracing::debug!("payment_method {} is not a card, skipping sync", pm.id);
        return Ok(());
    };

    let now = chrono::Utc::now();
    let brand_name = card.brand.clone();
    let last = card.last4.clone();
    let exp_m = card.exp_month as i32;
    let exp_y = card.exp_year as i32;

    let inserted = diesel::insert_into(billing_payment_methods)
        .values((
            id.eq(Uuid::new_v4()),
            pm_branch.eq(branch_id),
            pm_customer.eq(customer_id),
            stripe_pm_id.eq(&pm.id),
            brand.eq(&brand_name),
            last4.eq(&last),
            exp_month.eq(exp_m),
            exp_year.eq(exp_y),
            is_default.eq(false),
            created_at.eq(now),
            updated_at.eq(now),
        ))
        .on_conflict(stripe_pm_id)
        .do_update()
        .set((
            brand.eq(&brand_name),
            last4.eq(&last),
            exp_month.eq(exp_m),
            exp_year.eq(exp_y),
            updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Upsert payment method: {e}"))?;

    // First card of the branch becomes the default automatically.
    if inserted == 1 {
        let count: i64 = billing_payment_methods
            .filter(pm_branch.eq(branch_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| format!("Count cards: {e}"))?;
        if count == 1 {
            diesel::update(billing_payment_methods.filter(stripe_pm_id.eq(&pm.id)))
                .set(is_default.eq(true))
                .execute(&mut conn)
                .map_err(|e| format!("Set first default: {e}"))?;
        }
    }

    record_cloud_audit(service, branch_id, None, "card.add.webhook", "payment_method", Some(&pm.id), None);
    Ok(())
}

/// Handles `checkout.session.completed` for `mode=setup`: resolves the attached
/// payment method from the SetupIntent and syncs it locally.
pub async fn sync_checkout_setup(
    service: &SaasService,
    session: &botbilling::stripe_integration::StripeCheckoutSession,
) -> Result<(), String> {
    if session.mode != "setup" {
        return Ok(());
    }
    let setup_intent_id = session
        .setup_intent
        .as_deref()
        .ok_or_else(|| "setup session without setup_intent".to_string())?;

    let intent = service
        .stripe
        .retrieve_setup_intent(setup_intent_id)
        .await
        .map_err(|e| format!("Retrieve SetupIntent: {e}"))?;

    let pm_id = intent
        .payment_method
        .ok_or_else(|| "SetupIntent has no payment method".to_string())?;

    // The PaymentMethod is already attached to the customer by setup mode.
    // Fetch its details for the local mirror.
    let customer_id = intent
        .customer
        .as_deref()
        .ok_or_else(|| "SetupIntent has no customer".to_string())?;

    let methods = service
        .stripe
        .get_payment_methods(customer_id)
        .await
        .map_err(|e| format!("List payment methods: {e}"))?;
    let pm = methods
        .into_iter()
        .find(|m| m.id == pm_id)
        .ok_or_else(|| format!("payment method {pm_id} not found for customer"))?;

    sync_payment_method(service, &pm, false).await?;
    Ok(())
}
