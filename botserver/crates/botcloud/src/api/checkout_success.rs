use super::*;

/// `GET /api/cloud/checkout/success`
///
/// Confirms a Stripe checkout only when the session fetched server-side, the
/// authenticated JWT tenant and the local customer mapping all point at the
/// same invoice. #2665: the query `invoice` id alone never authorizes a
/// transition — an unrelated client-supplied id is refused with 403.
pub(crate) async fn checkout_success(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    fn forbidden() -> (StatusCode, String) {
        (StatusCode::FORBIDDEN, "checkout session does not authorize this invoice".to_string())
    }
    fn db_error(context: &str, e: impl std::fmt::Display) -> (StatusCode, String) {
        tracing::error!("checkout_success {context}: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "billing database unavailable".to_string())
    }

    let session_id = params.get("session_id")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing session_id".to_string()))?;
    let invoice_id = params.get("invoice")
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing invoice".to_string()))?;

    // Never trust client-supplied session data: retrieve it from Stripe with
    // our API key. metadata["invoice_id"] was set by handle_checkout.
    let session = service.stripe
        .retrieve_checkout_session(session_id)
        .await
        .map_err(|e| {
            tracing::error!("checkout session lookup failed for {session_id}: {e}");
            (StatusCode::BAD_GATEWAY, "checkout session could not be verified".to_string())
        })?;

    let (invoice, authenticated_branch, authenticated_org, mapped_customer) = {
        let mut conn = service.pool().get()
            .map_err(|e| db_error("pool", e))?;

        let authenticated_branch = get_branch_id_from_jwt(&headers, &mut conn)
            .map_err(|e| {
                tracing::warn!("checkout JWT scope rejected: {e}");
                (StatusCode::UNAUTHORIZED, "unable to resolve tenant".to_string())
            })?
            .ok_or_else(|| (StatusCode::FORBIDDEN, "no billing scope for this account".to_string()))?;

        let invoice = botbilling::schema::billing_invoices::table
            .filter(botbilling::schema::billing_invoices::id.eq(invoice_id))
            .first::<botbilling::api_models::BillingInvoice>(&mut conn)
            .optional()
            .map_err(|e| db_error("invoice lookup", e))?
            .ok_or_else(forbidden)?;

        #[derive(diesel::QueryableByName)]
        struct TenantOrg {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        let authenticated_org = diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(authenticated_branch)
            .get_result::<TenantOrg>(&mut conn)
            .optional()
            .map_err(|e| db_error("tenant lookup", e))?
            .map(|row| row.org_id)
            .ok_or_else(forbidden)?;

        use crate::schema_ext::billing_customers::dsl::{
            billing_customers, branch_id as bc_branch, stripe_customer_id as bc_stripe,
        };
        let mapped_customer = billing_customers
            .filter(bc_branch.eq(invoice.branch_id))
            .select(bc_stripe)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| db_error("customer mapping", e))?;

        (invoice, authenticated_branch, authenticated_org, mapped_customer)
    };

    let binding = super::checkout_binding::CheckoutBinding {
        requested_session_id: session_id,
        verified_session_id: session.id.as_str(),
        session_mode: session.mode.as_str(),
        metadata_invoice_id: session.metadata.as_ref()
            .and_then(|m| m.get("invoice_id"))
            .map(String::as_str),
        requested_invoice_id: invoice_id,
        session_customer: session.customer.as_deref(),
        mapped_customer: mapped_customer.as_deref(),
        authenticated_branch_id: Some(authenticated_branch),
        invoice_branch_id: invoice.branch_id,
        invoice_org_id: invoice.org_id,
        authenticated_org_id: authenticated_org,
    };
    if let Some(reason) = super::checkout_binding::binding_failure(&binding) {
        tracing::warn!(
            invoice_id = %invoice_id,
            session_id = %session_id,
            branch_id = %authenticated_branch,
            reason,
            "checkout_success rejected invoice transition"
        );
        return Err(forbidden());
    }

    // A non-complete session is not a payment: audit and leave the invoice
    // untouched (the old code wrote paid_at on a "pending" invoice).
    if session.status != "complete" {
        tracing::info!(invoice_id = %invoice_id, session_id = %session_id, "checkout not complete; no transition");
        return Ok(Json(serde_json::json!({
            "status": "pending",
            "customer": session.customer,
            "subscription": session.subscription,
        })));
    }

    let now = chrono::Utc::now();
    let transitioned = {
        let mut conn = service.pool().get()
            .map_err(|e| db_error("update pool", e))?;
        if invoice.status == "paid" {
            0
        } else {
            diesel::update(
                botbilling::schema::billing_invoices::table
                    .filter(botbilling::schema::billing_invoices::id.eq(invoice_id))
                    .filter(botbilling::schema::billing_invoices::status.ne("paid")),
            )
            .set((
                botbilling::schema::billing_invoices::status.eq("paid"),
                botbilling::schema::billing_invoices::paid_at.eq(Some(now)),
                botbilling::schema::billing_invoices::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| db_error("invoice update", e))?
        }
    };

    // Audit every transition (and every no-op) with the verified identities.
    tracing::info!(
        invoice_id = %invoice_id,
        session_id = %session_id,
        branch_id = %invoice.branch_id,
        org_id = %invoice.org_id,
        stripe_customer = ?session.customer,
        from_status = %invoice.status,
        to_status = "paid",
        transitioned,
        "checkout_success audit"
    );
    if transitioned == 0 {
        return Ok(Json(serde_json::json!({
            "status": "completed",
            "customer": session.customer,
            "subscription": session.subscription,
        })));
    }

    // Integrations run only for the request that won the transition, so GL,
    // subscription and notifications fire exactly once for this invoice.
    let total = botbilling::api_models::bd_to_f64(&invoice.total);
    let plan_label = session.metadata.as_ref()
        .and_then(|m| m.get("plan"))
        .map(String::as_str)
        .unwrap_or("unknown");

    let deal_name = format!("Assinatura - {}", invoice.customer_name.as_str());
    if let Err(e) = integration::win_crm_deal(service.pool(), invoice.branch_id, &deal_name) {
        tracing::warn!("CRM win deal failed: {e}");
    }

    if let Err(e) = integration::create_gl_entry_for_invoice(
        service.pool(), invoice_id, total, invoice.customer_name.as_str(),
    ) {
        tracing::warn!("GL entry creation failed: {e}");
    }

    if let Err(e) = integration::create_billing_subscription(integration::NewSubscription {
        pool: service.pool(),
        branch_id: invoice.branch_id,
        customer_name: invoice.customer_name.as_str(),
        customer_email: invoice.customer_email.as_deref().unwrap_or(""),
        plan: plan_label,
        total,
        currency: invoice.currency.as_str(),
        invoice_id,
        period: "monthly",
    }) {
        tracing::warn!("Subscription creation failed: {e}");
    }

    let mut vars = notifier::EmailVars::new(
        invoice.customer_name.as_str(),
        invoice.customer_email.as_deref().unwrap_or(""),
        plan_label,
        total,
        invoice.currency.as_str(),
    )
    .with_locale(notifier::recipient_locale(params.get("locale").map(String::as_str)));
    vars.invoice_id = invoice_id.to_string();
    notifier::notify_payment_success(&vars);
    notifier::notify_subscription_activated(&vars);

    Ok(Json(serde_json::json!({
        "status": "completed",
        "customer": session.customer,
        "subscription": session.subscription,
    })))
}
