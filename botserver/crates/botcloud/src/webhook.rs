//! Webhook Stripe — processa eventos assíncronos de pagamento.
//!
//! Eventos tratados:
//! - `checkout.session.completed` → fatura paga, CRM deal won, GL entry, subscription
//! - `invoice.paid` → cobrança recorrente paga
//! - `invoice.payment_failed` → falha de pagamento
//! - `customer.subscription.*` → ciclo de vida da assinatura

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use std::sync::Arc;
use uuid::Uuid;

use crate::{integration, notifier, SaasService};



pub fn configure_webhook_routes() -> Router<Arc<SaasService>> {
    Router::new()
        .route("/api/billing/webhook", post(handle_stripe_webhook))
}

/// `POST /api/billing/webhook`
///
/// Recebe eventos do Stripe, verifica assinatura HMAC e processa.
async fn handle_stripe_webhook(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing stripe-signature header".to_string()))?;

    // Verifica assinatura HMAC
    let event = service
        .stripe
        .verify_webhook_signature(&body, signature)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid signature: {e}")))?;

    // Parseia o tipo de evento
    let event_type = service
        .stripe
        .parse_webhook_event(&event)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid event: {e}")))?;

    use botbilling::stripe_integration::WebhookEventType;

    match event_type {
        WebhookEventType::CheckoutCompleted(session) => {
            process_checkout_completed(&service, &session).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        WebhookEventType::InvoicePaid(invoice) => {
            process_invoice_paid(&service, &invoice).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        WebhookEventType::InvoicePaymentFailed(invoice) => {
            process_invoice_payment_failed(&service, &invoice).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        WebhookEventType::SubscriptionCreated(sub) => {
            process_subscription_lifecycle(
                &service,
                &sub.id,
                &sub.customer,
                &sub.metadata,
                "active",
            )
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        WebhookEventType::SubscriptionUpdated(sub) => {
            let status = match sub.status {
                botbilling::stripe_integration::StripeSubscriptionStatus::Active
                | botbilling::stripe_integration::StripeSubscriptionStatus::Trialing => "active",
                botbilling::stripe_integration::StripeSubscriptionStatus::PastDue => "past_due",
                botbilling::stripe_integration::StripeSubscriptionStatus::Canceled
                | botbilling::stripe_integration::StripeSubscriptionStatus::IncompleteExpired
                | botbilling::stripe_integration::StripeSubscriptionStatus::Unpaid => "cancelled",
                botbilling::stripe_integration::StripeSubscriptionStatus::Paused => "paused",
                botbilling::stripe_integration::StripeSubscriptionStatus::Incomplete => {
                    "incomplete"
                }
            };
            process_subscription_lifecycle(
                &service,
                &sub.id,
                &sub.customer,
                &sub.metadata,
                status,
            )
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        WebhookEventType::SubscriptionCanceled(sub) => {
            process_subscription_lifecycle(
                &service,
                &sub.id,
                &sub.customer,
                &sub.metadata,
                "cancelled",
            )
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        WebhookEventType::Unknown(event_type) => {
            tracing::info!("Ignoring unhandled Stripe event: {event_type}");
        }
    }

    Ok(Json(serde_json::json!({ "received": true })))
}

/// Processa `checkout.session.completed`.
///
/// 1. Atualiza fatura para "paid"
/// 2. Marca CRM deal como "won"
/// 3. Cria lançamento contábil no GL
/// 4. Cria assinatura recorrente em billing_recurring
async fn process_checkout_completed(
    service: &SaasService,
    session: &botbilling::stripe_integration::StripeCheckoutSession,
) -> Result<(), String> {
    // Extrai invoice_id e plan do metadata da sessão
    let metadata = session
        .metadata
        .as_ref()
        .ok_or_else(|| "missing metadata on checkout session".to_string())?;

    let invoice_id_str = metadata
        .get("invoice_id")
        .ok_or_else(|| "missing invoice_id in metadata".to_string())?;
    let invoice_id = Uuid::parse_str(invoice_id_str)
        .map_err(|e| format!("invalid invoice_id: {e}"))?;

    let now = chrono::Utc::now();

    // 1. Atualiza fatura para "paid"
    {
        let mut conn = service
            .pool()
            .get()
            .map_err(|e| format!("DB pool: {e}"))?;

        diesel::update(
            botbilling::schema::billing_invoices::table
                .filter(botbilling::schema::billing_invoices::id.eq(invoice_id)),
        )
        .set((
            botbilling::schema::billing_invoices::status.eq("paid"),
            botbilling::schema::billing_invoices::paid_at.eq(Some(now)),
            botbilling::schema::billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("update invoice: {e}"))?;
    }

    // Busca invoice para ter dados do cliente
    let invoice = {
        let mut conn = service
            .pool()
            .get()
            .map_err(|e| format!("DB pool: {e}"))?;
        botbilling::schema::billing_invoices::table
            .filter(botbilling::schema::billing_invoices::id.eq(invoice_id))
            .first::<botbilling::api_models::BillingInvoice>(&mut conn)
            .map_err(|e| format!("invoice not found: {e}"))?
    };

    let total = botbilling::api_models::bd_to_f64(&invoice.total);
    let plan_label = metadata.get("plan").cloned().unwrap_or_else(|| "unknown".to_string());

    // 2. CRM: marca deal como ganho
    let deal_name = format!("Assinatura {} - {}", plan_label, invoice.customer_name);
    let _ = integration::win_crm_deal(service.pool(), invoice.org_id, &deal_name)
        .map_err(|e| tracing::warn!("CRM win deal failed: {e}"));

    // 3. ERP (GL): lança entrada contábil
    let _ = integration::create_gl_entry_for_invoice(
        service.pool(),
        invoice_id,
        total,
        &invoice.customer_name,
    )
    .map_err(|e| tracing::warn!("GL entry failed: {e}"));

    // 4. Subscription: cria ou atualiza billing_recurring
    let email_ref = invoice.customer_email.clone().unwrap_or_default();
    let _ = integration::create_billing_subscription(
        service.pool(),
        invoice.org_id,
        invoice.bot_id,
        &invoice.customer_name,
        &email_ref,
        &plan_label,
        total,
        &invoice.currency,
        invoice_id,
        "monthly",
    )
    .map_err(|e| tracing::warn!("Subscription creation failed: {e}"));

    // --- Notificações ---
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

    tracing::info!(
        "checkout.session.completed: invoice={invoice_id}, plan={plan_label}, total={total}"
    );

    Ok(())
}

/// Processa `invoice.paid` — cobrança recorrente paga com sucesso.
///
/// 1. Gera nova fatura para o próximo período
/// 2. Atualiza billing_recurring
/// 3. Cria lançamento contábil no GL
async fn process_invoice_paid(
    service: &SaasService,
    stripe_invoice: &botbilling::stripe_integration::StripeInvoice,
) -> Result<(), String> {
    let now = chrono::Utc::now();
    let customer_email = stripe_invoice
        .customer_email
        .as_deref()
        .ok_or_else(|| "invoice has no customer_email".to_string())?;

    let mut conn = service
        .pool()
        .get()
        .map_err(|e| format!("DB pool: {e}"))?;

    // Busca a subscription pelo email do cliente (único por email no modelo single-tenant)
    let recurring = botbilling::schema::billing_recurring::table
        .filter(botbilling::schema::billing_recurring::customer_email.eq(Some(customer_email.to_string())))
        .filter(botbilling::schema::billing_recurring::status.eq("active"))
        .first::<botbilling::api_models::BillingRecurring>(&mut conn)
        .map_err(|_| format!("billing_recurring not found for email {customer_email}"))?;

    let total = botbilling::api_models::bd_to_f64(&recurring.amount);

    // Gera nova fatura para o período
    let new_invoice_id = Uuid::new_v4();
    let invoice_number =
        botbilling::api_models::generate_invoice_number(&mut conn, recurring.org_id);

    let new_invoice = botbilling::api_models::BillingInvoice {
        id: new_invoice_id,
        org_id: recurring.org_id,
        bot_id: recurring.bot_id,
        invoice_number,
        customer_id: None,
        customer_name: recurring.customer_name.clone(),
        customer_email: recurring.customer_email.clone(),
        customer_address: None,
        status: "paid".to_string(),
        issue_date: now.date_naive(),
        due_date: (now + chrono::Duration::days(30)).date_naive(),
        subtotal: recurring.amount.clone(),
        tax_rate: botbilling::api_models::bd(0.0),
        tax_amount: botbilling::api_models::bd(0.0),
        discount_percent: botbilling::api_models::bd(0.0),
        discount_amount: botbilling::api_models::bd(0.0),
        total: recurring.amount.clone(),
        amount_paid: recurring.amount.clone(),
        amount_due: botbilling::api_models::bd(0.0),
        currency: recurring.currency.clone(),
        notes: Some(format!("Cobrança recorrente - {}", recurring.description.as_deref().unwrap_or("assinatura"))),
        terms: None,
        footer: None,
        paid_at: Some(now),
        sent_at: None,
        voided_at: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(botbilling::schema::billing_invoices::table)
        .values(&new_invoice)
        .execute(&mut conn)
        .map_err(|e| format!("insert invoice: {e}"))?;

    let line_item = botbilling::api_models::BillingInvoiceItem {
        id: Uuid::new_v4(),
        invoice_id: new_invoice_id,
        product_id: None,
        description: recurring.description.clone().unwrap_or_else(|| "Assinatura".to_string()),
        quantity: botbilling::api_models::bd(1.0),
        unit_price: recurring.amount.clone(),
        discount_percent: botbilling::api_models::bd(0.0),
        tax_rate: botbilling::api_models::bd(0.0),
        amount: recurring.amount.clone(),
        sort_order: 0,
        created_at: now,
    };

    diesel::insert_into(botbilling::schema::billing_invoice_items::table)
        .values(&line_item)
        .execute(&mut conn)
        .map_err(|e| format!("insert invoice item: {e}"))?;

    // Atualiza billing_recurring
    let _next_date = if recurring.frequency == "yearly" {
        now.date_naive() + chrono::Duration::days(365)
    } else {
        now.date_naive() + chrono::Duration::days(30)
    };

    diesel::update(
        botbilling::schema::billing_recurring::table
            .filter(botbilling::schema::billing_recurring::id.eq(recurring.id)),
    )
    .set((
        botbilling::schema::billing_recurring::last_invoice_date.eq(Some(now.date_naive())),
        botbilling::schema::billing_recurring::last_invoice_id.eq(Some(new_invoice_id)),
        botbilling::schema::billing_recurring::next_invoice_date.eq(now.date_naive()),
        botbilling::schema::billing_recurring::invoices_generated
            .eq(recurring.invoices_generated + 1),
        botbilling::schema::billing_recurring::updated_at.eq(now),
    ))
    .execute(&mut conn)
    .map_err(|e| format!("update recurring: {e}"))?;

    // GL
    let _ = integration::create_gl_entry_for_invoice(
        service.pool(),
        new_invoice_id,
        total,
        &recurring.customer_name,
    )
    .map_err(|e| tracing::warn!("GL entry for recurring failed: {e}"));

    // --- Notificação: cobrança recorrente ---
    let mut vars = notifier::EmailVars::new(
        &recurring.customer_name,
        &recurring.customer_email.as_deref().unwrap_or(""),
        &recurring.description.clone().unwrap_or_else(|| "assinatura".to_string()),
        total,
        &recurring.currency,
    );
    vars.invoice_id = new_invoice_id.to_string();
    notifier::notify_recurring_charge(&vars);

    tracing::info!(
        "invoice.paid: stripe={}, new_invoice={}, amount={}",
        stripe_invoice.id,
        new_invoice_id,
        total
    );

    Ok(())
}

/// Processa `invoice.payment_failed`.
///
/// Marca a fatura como "overdue" e a subscription como "past_due".
async fn process_invoice_payment_failed(
    service: &SaasService,
    stripe_invoice: &botbilling::stripe_integration::StripeInvoice,
) -> Result<(), String> {
    let now = chrono::Utc::now();
    let customer_email = stripe_invoice
        .customer_email
        .as_deref()
        .ok_or_else(|| "invoice has no customer_email".to_string())?;

    let mut conn = service
        .pool()
        .get()
        .map_err(|e| format!("DB pool: {e}"))?;

    let recurring = botbilling::schema::billing_recurring::table
        .filter(botbilling::schema::billing_recurring::customer_email.eq(Some(customer_email.to_string())))
        .filter(botbilling::schema::billing_recurring::status.eq("active"))
        .first::<botbilling::api_models::BillingRecurring>(&mut conn)
        .map_err(|_| format!("billing_recurring not found for email {customer_email}"))?;

    // Marca a última invoice como overdue
    if let Some(last_inv_id) = recurring.last_invoice_id {
        let _ = diesel::update(
            botbilling::schema::billing_invoices::table
                .filter(botbilling::schema::billing_invoices::id.eq(last_inv_id)),
        )
        .set((
            botbilling::schema::billing_invoices::status.eq("overdue"),
            botbilling::schema::billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn);
    }

    // Marca subscription como past_due
    let _ = diesel::update(
        botbilling::schema::billing_recurring::table
            .filter(botbilling::schema::billing_recurring::id.eq(recurring.id)),
    )
    .set((
        botbilling::schema::billing_recurring::status.eq("past_due"),
        botbilling::schema::billing_recurring::updated_at.eq(now),
    ))
    .execute(&mut conn);

    let mut vars = notifier::EmailVars::new(
        &recurring.customer_name,
        &recurring.customer_email.as_deref().unwrap_or(""),
        &recurring.description.clone().unwrap_or_else(|| "assinatura".to_string()),
        botbilling::api_models::bd_to_f64(&recurring.amount),
        &recurring.currency,
    );
    vars.invoice_id = recurring.last_invoice_id.unwrap_or(Uuid::nil()).to_string();
    notifier::notify_payment_failed(&vars);

    tracing::warn!(
        "invoice.payment_failed: stripe={}, email={customer_email}",
        stripe_invoice.id,
    );

    Ok(())
}

/// Processa eventos de ciclo de vida da subscription.
///
/// Cria ou atualiza `billing_recurring` com os dados da assinatura Stripe.
async fn process_subscription_lifecycle(
    service: &SaasService,
    stripe_subscription_id: &str,
    stripe_customer_id: &str,
    metadata: &std::collections::HashMap<String, String>,
    status: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now();
    let mut conn = service
        .pool()
        .get()
        .map_err(|e| format!("DB pool: {e}"))?;

    // Tenta encontrar billing_recurring pelo stripe_subscription_id armazenado
    // Como não temos coluna para isso, usamos o customer email do metadata
    let account_email = metadata
        .get("account_email")
        .cloned()
        .unwrap_or_default();

    if status == "cancelled" || status == "past_due" || status == "paused" {
        let _ = diesel::update(
            botbilling::schema::billing_recurring::table
                .filter(botbilling::schema::billing_recurring::customer_email.eq(Some(account_email.clone()))),
        )
        .set((
            botbilling::schema::billing_recurring::status.eq(status),
            botbilling::schema::billing_recurring::updated_at.eq(now),
        ))
        .execute(&mut conn);

        if status == "cancelled" && !account_email.is_empty() {
            let vars = notifier::EmailVars::new(
                &account_email, &account_email, "unknown", 0.0, "USD",
            );
            notifier::notify_subscription_cancelled(&vars);
        }
    }

    tracing::info!(
        "subscription.lifecycle: stripe_sub={stripe_subscription_id}, customer={stripe_customer_id}, status={status}"
    );

    Ok(())
}
