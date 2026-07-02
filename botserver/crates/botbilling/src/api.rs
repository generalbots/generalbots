use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::api_models::{
    bd, bd_to_f64, generate_invoice_number, generate_payment_number, generate_quote_number,
    BillingInvoice, BillingInvoiceItem, BillingPayment, BillingQuote, BillingQuoteItem,
    BillingRecurring, BillingStats, BillingTaxRate, CreateInvoiceRequest, CreateQuoteRequest,
    InvoiceWithItems, ListQuery, QuoteWithItems, RecordPaymentRequest,
    UpdateInvoiceRequest,
};
use crate::schema::{
    billing_invoice_items, billing_invoices, billing_payments, billing_quote_items,
    billing_quotes, billing_recurring, billing_tax_rates,
};
use crate::{get_bot_context, DbPool, GetDefaultBotFn};

pub struct BillingApiState {
    pub pool: Arc<DbPool>,
    pub get_default_bot: Option<GetDefaultBotFn>,
}

pub async fn create_invoice(
    State(state): State<Arc<BillingApiState>>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let invoice_number = generate_invoice_number(&mut conn, branch_id);

    let issue_date = req
        .issue_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| now.date_naive());

    let due_date = req
        .due_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| issue_date + chrono::Duration::days(30));

    let tax_rate = bd(req.tax_rate.unwrap_or(0.0));
    let discount_percent = bd(req.discount_percent.unwrap_or(0.0));

    let mut subtotal = bd(0.0);
    let items = req.items.unwrap_or_default();
    for item in &items {
        let item_amount = item.quantity * item.unit_price;
        let item_discount = item_amount * item.discount_percent.unwrap_or(0.0) / 100.0;
        subtotal += bd(item_amount - item_discount);
    }

    let discount_amount = &subtotal * &discount_percent / bd(100.0);
    let taxable = &subtotal - &discount_amount;
    let tax_amount = &taxable * &tax_rate / bd(100.0);
    let total = &taxable + &tax_amount;

        let invoice = BillingInvoice {
        id,
        branch_id,
        invoice_number,
        customer_name: req.customer_name.clone(),
        customer_email: req.customer_email.clone(),
        status: Some("draft".to_string()),
        total: Some(total.clone()),
        currency: Some(req.currency.clone().unwrap_or_else(|| "USD".to_string())),
        due_date: Some(due_date),
        paid_at: None,
        notes: req.notes.clone(),
        created_at: now,
        updated_at: now,
        customer_id: req.customer_id,
        customer_address: req.customer_address.clone(),
        issue_date,
        subtotal: subtotal.clone(),
        tax_rate,
        tax_amount,
        discount_percent,
        discount_amount,
        amount_paid: bd(0.0),
        amount_due: total,
        terms: req.terms.clone(),
        footer: None,
        sent_at: None,
        voided_at: None,
    };

    diesel::insert_into(billing_invoices::table)
        .values(&invoice)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    for (idx, item) in items.iter().enumerate() {
        let item_amount = item.quantity * item.unit_price;
        let item_discount = item_amount * item.discount_percent.unwrap_or(0.0) / 100.0;
        let final_amount = item_amount - item_discount;

        let inv_item = BillingInvoiceItem {
            id: Uuid::new_v4(),
            invoice_id: id,
            product_id: item.product_id,
            description: item.description.clone(),
            quantity: bd(item.quantity),
            unit_price: bd(item.unit_price),
            discount_percent: bd(item.discount_percent.unwrap_or(0.0)),
            tax_rate: bd(item.tax_rate.unwrap_or(0.0)),
            amount: bd(final_amount),
            sort_order: idx as i32,
            created_at: now,
        };

        diesel::insert_into(billing_invoice_items::table)
            .values(&inv_item)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert item error: {e}")))?;
    }

    Ok(Json(invoice))
}

pub async fn list_invoices(
    State(state): State<Arc<BillingApiState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BillingInvoice>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = billing_invoices::table
        .filter(billing_invoices::branch_id.eq(branch_id))
        .into_boxed();

    if let Some(status) = query.status {
        if status != "all" {
            q = q.filter(billing_invoices::status.eq(status));
        }
    }

    if let Some(customer_id) = query.customer_id {
        q = q.filter(billing_invoices::customer_id.eq(customer_id));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            billing_invoices::invoice_number
                .ilike(pattern.clone())
                .or(billing_invoices::customer_name.ilike(pattern)),
        );
    }

    let invoices: Vec<BillingInvoice> = q
        .order(billing_invoices::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(invoices))
}

pub async fn get_invoice(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceWithItems>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let invoice: BillingInvoice = billing_invoices::table
        .filter(billing_invoices::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?;

    let items: Vec<BillingInvoiceItem> = billing_invoice_items::table
        .filter(billing_invoice_items::invoice_id.eq(id))
        .order(billing_invoice_items::sort_order.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(InvoiceWithItems { invoice, items }))
}

pub async fn update_invoice(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateInvoiceRequest>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(id)))
        .set(billing_invoices::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(customer_name) = req.customer_name {
        diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(id)))
            .set(billing_invoices::customer_name.eq(customer_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(customer_email) = req.customer_email {
        diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(id)))
            .set(billing_invoices::customer_email.eq(customer_email))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(notes) = req.notes {
        diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(id)))
            .set(billing_invoices::notes.eq(notes))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    let invoice: BillingInvoice = billing_invoices::table
        .filter(billing_invoices::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?;

    Ok(Json(invoice))
}

pub async fn send_invoice(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(id)))
        .set((
            billing_invoices::status.eq("sent"),
            billing_invoices::sent_at.eq(Some(now)),
            billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let invoice: BillingInvoice = billing_invoices::table
        .filter(billing_invoices::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?;

    Ok(Json(invoice))
}

pub async fn void_invoice(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(id)))
        .set((
            billing_invoices::status.eq("voided"),
            billing_invoices::voided_at.eq(Some(now)),
            billing_invoices::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let invoice: BillingInvoice = billing_invoices::table
        .filter(billing_invoices::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?;

    Ok(Json(invoice))
}

pub async fn delete_invoice(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(billing_invoices::table.filter(billing_invoices::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn record_payment(
    State(state): State<Arc<BillingApiState>>,
    Json(req): Json<RecordPaymentRequest>,
) -> Result<Json<BillingPayment>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let payment_number = generate_payment_number(&mut conn, branch_id);

    let payment = BillingPayment {
        id,
        branch_id,
        invoice_id: req.invoice_id,
        amount: bd(req.amount),
        currency: Some("USD".to_string()),
        payment_method: Some(req.payment_method.clone().unwrap_or_else(|| "other".to_string())),
        status: Some("completed".to_string()),
        paid_at: Some(now),
        gateway_response: None,
        created_at: now,
        updated_at: now,
        payment_number,
        payment_reference: req.payment_reference.clone(),
        payer_name: req.payer_name.clone(),
        payer_email: req.payer_email.clone(),
        notes: req.notes.clone(),
        refunded_at: None,
        refund_amount: None,
    };

    diesel::insert_into(billing_payments::table)
        .values(&payment)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    if let Some(invoice_id) = req.invoice_id {
        let invoice: BillingInvoice = billing_invoices::table
            .filter(billing_invoices::id.eq(invoice_id))
            .first(&mut conn)
            .map_err(|_| (StatusCode::NOT_FOUND, "Invoice not found".to_string()))?;

        let new_paid = &invoice.amount_paid + bd(req.amount);
        let total_val = invoice.total.unwrap_or_else(|| bd(0.0));
        let new_due = &total_val - &new_paid;

        let new_status = if bd_to_f64(&new_due) <= 0.0 {
            "paid"
        } else if bd_to_f64(&new_paid) > 0.0 {
            "partial"
        } else {
            invoice.status.as_deref().unwrap_or("")
        };

        let paid_at = if new_status == "paid" { Some(now) } else { invoice.paid_at };

        diesel::update(billing_invoices::table.filter(billing_invoices::id.eq(invoice_id)))
            .set((
                billing_invoices::amount_paid.eq(new_paid),
                billing_invoices::amount_due.eq(new_due),
                billing_invoices::status.eq(new_status),
                billing_invoices::paid_at.eq(paid_at),
                billing_invoices::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    Ok(Json(payment))
}

pub async fn list_payments(
    State(state): State<Arc<BillingApiState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BillingPayment>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = billing_payments::table
        .filter(billing_payments::branch_id.eq(branch_id))
        .into_boxed();

    if let Some(status) = query.status {
        if status != "all" {
            q = q.filter(billing_payments::status.eq(status));
        }
    }

    let payments: Vec<BillingPayment> = q
        .order(billing_payments::paid_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(payments))
}

pub async fn get_payment(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingPayment>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let payment: BillingPayment = billing_payments::table
        .filter(billing_payments::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Payment not found".to_string()))?;

    Ok(Json(payment))
}

pub async fn create_quote(
    State(state): State<Arc<BillingApiState>>,
    Json(req): Json<CreateQuoteRequest>,
) -> Result<Json<BillingQuote>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let quote_number = generate_quote_number(&mut conn, branch_id);

    let issue_date = req
        .issue_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| now.date_naive());

    let valid_until = req
        .valid_until
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| issue_date + chrono::Duration::days(30));

    let tax_rate = bd(req.tax_rate.unwrap_or(0.0));
    let discount_percent = bd(req.discount_percent.unwrap_or(0.0));

    let mut subtotal = bd(0.0);
    let items = req.items.unwrap_or_default();
    for item in &items {
        let item_amount = item.quantity * item.unit_price;
        let item_discount = item_amount * item.discount_percent.unwrap_or(0.0) / 100.0;
        subtotal += bd(item_amount - item_discount);
    }

    let discount_amount = &subtotal * &discount_percent / bd(100.0);
    let taxable = &subtotal - &discount_amount;
    let tax_amount = &taxable * &tax_rate / bd(100.0);
    let total = &taxable + &tax_amount;

    let quote = BillingQuote {
        id,
        branch_id,
        quote_number,
        customer_name: req.customer_name.clone(),
        customer_email: req.customer_email.clone(),
        items: None,
        total: Some(total),
        currency: Some(req.currency.clone().unwrap_or_else(|| "USD".to_string())),
        status: Some("draft".to_string()),
        valid_until: Some(valid_until),
        notes: req.notes.clone(),
        created_at: now,
        updated_at: now,
        customer_id: req.customer_id,
        customer_address: req.customer_address.clone(),
        issue_date,
        subtotal,
        tax_rate,
        tax_amount,
        discount_percent,
        discount_amount,
        terms: req.terms.clone(),
        accepted_at: None,
        rejected_at: None,
        converted_invoice_id: None,
        sent_at: None,
    };

    diesel::insert_into(billing_quotes::table)
        .values(&quote)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    for (idx, item) in items.iter().enumerate() {
        let item_amount = item.quantity * item.unit_price;
        let item_discount = item_amount * item.discount_percent.unwrap_or(0.0) / 100.0;
        let final_amount = item_amount - item_discount;

        let quote_item = BillingQuoteItem {
            id: Uuid::new_v4(),
            quote_id: id,
            product_id: item.product_id,
            description: item.description.clone(),
            quantity: bd(item.quantity),
            unit_price: bd(item.unit_price),
            discount_percent: bd(item.discount_percent.unwrap_or(0.0)),
            tax_rate: bd(item.tax_rate.unwrap_or(0.0)),
            amount: bd(final_amount),
            sort_order: idx as i32,
            created_at: now,
        };

        diesel::insert_into(billing_quote_items::table)
            .values(&quote_item)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert item error: {e}")))?;
    }

    Ok(Json(quote))
}

pub async fn list_quotes(
    State(state): State<Arc<BillingApiState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BillingQuote>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = billing_quotes::table
        .filter(billing_quotes::branch_id.eq(branch_id))
        .into_boxed();

    if let Some(status) = query.status {
        if status != "all" {
            q = q.filter(billing_quotes::status.eq(status));
        }
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            billing_quotes::quote_number
                .ilike(pattern.clone())
                .or(billing_quotes::customer_name.ilike(pattern)),
        );
    }

    let quotes: Vec<BillingQuote> = q
        .order(billing_quotes::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(quotes))
}

pub async fn get_quote(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<QuoteWithItems>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let quote: BillingQuote = billing_quotes::table
        .filter(billing_quotes::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Quote not found".to_string()))?;

    let items: Vec<BillingQuoteItem> = billing_quote_items::table
        .filter(billing_quote_items::quote_id.eq(id))
        .order(billing_quote_items::sort_order.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(QuoteWithItems { quote, items }))
}

pub async fn accept_quote(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingQuote>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(billing_quotes::table.filter(billing_quotes::id.eq(id)))
        .set((
            billing_quotes::status.eq("accepted"),
            billing_quotes::accepted_at.eq(Some(now)),
            billing_quotes::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let quote: BillingQuote = billing_quotes::table
        .filter(billing_quotes::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Quote not found".to_string()))?;

    Ok(Json(quote))
}

pub async fn reject_quote(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingQuote>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(billing_quotes::table.filter(billing_quotes::id.eq(id)))
        .set((
            billing_quotes::status.eq("rejected"),
            billing_quotes::rejected_at.eq(Some(now)),
            billing_quotes::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let quote: BillingQuote = billing_quotes::table
        .filter(billing_quotes::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Quote not found".to_string()))?;

    Ok(Json(quote))
}

pub async fn delete_quote(
    State(state): State<Arc<BillingApiState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(billing_quotes::table.filter(billing_quotes::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_billing_stats(
    State(state): State<Arc<BillingApiState>>,
) -> Result<Json<BillingStats>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let today = Utc::now().date_naive();

    let invoices: Vec<BillingInvoice> = billing_invoices::table
        .filter(billing_invoices::branch_id.eq(branch_id))
        .load(&mut conn)
        .unwrap_or_default();

    let mut total_revenue = 0.0;
    let mut pending_amount = 0.0;
    let mut overdue_amount = 0.0;
    let mut overdue_count = 0i64;

    for inv in &invoices {
        if inv.status.as_deref() == Some("paid") {
            total_revenue += inv.total.as_ref().map_or(0.0, |t| bd_to_f64(t));
        }
        if inv.status.as_deref() != Some("paid") && inv.status.as_deref() != Some("voided") {
            pending_amount += bd_to_f64(&inv.amount_due);
            if inv.due_date.unwrap_or(NaiveDate::MAX) < today {
                overdue_amount += bd_to_f64(&inv.amount_due);
                overdue_count += 1;
            }
        }
    }

    let payments: Vec<BillingPayment> = billing_payments::table
        .filter(billing_payments::branch_id.eq(branch_id))
        .filter(billing_payments::status.eq("completed"))
        .load(&mut conn)
        .unwrap_or_default();

    let paid_this_month: f64 = payments
        .iter()
        .filter(|p| p.paid_at.map(|d| d.date_naive().month() == today.month() && d.date_naive().year() == today.year()).unwrap_or(false))
        .map(|p| bd_to_f64(&p.amount))
        .sum();

    let revenue_this_month: f64 = invoices
        .iter()
        .filter(|i| i.status.as_deref() == Some("paid") && i.paid_at.map(|d| d.date_naive().month() == today.month() && d.date_naive().year() == today.year()).unwrap_or(false))
        .map(|i| i.total.as_ref().map_or(0.0, |t| bd_to_f64(t)))
        .sum();

    let stats = BillingStats {
        total_revenue,
        revenue_this_month,
        pending_amount,
        overdue_amount,
        paid_this_month,
        invoice_count: invoices.len() as i64,
        payment_count: payments.len() as i64,
        overdue_count,
    };

    Ok(Json(stats))
}

pub async fn list_overdue_invoices(
    State(state): State<Arc<BillingApiState>>,
) -> Result<Json<Vec<BillingInvoice>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);
    let today = Utc::now().date_naive();

    let invoices: Vec<BillingInvoice> = billing_invoices::table
        .filter(billing_invoices::branch_id.eq(branch_id))
        .filter(billing_invoices::status.ne("paid"))
        .filter(billing_invoices::status.ne("voided"))
        .filter(billing_invoices::due_date.lt(today))
        .order(billing_invoices::due_date.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(invoices))
}

pub async fn list_tax_rates(
    State(state): State<Arc<BillingApiState>>,
) -> Result<Json<Vec<BillingTaxRate>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);

    let rates: Vec<BillingTaxRate> = billing_tax_rates::table
        .filter(billing_tax_rates::branch_id.eq(branch_id))
        .filter(billing_tax_rates::is_active.eq(true))
        .order(billing_tax_rates::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(rates))
}

pub async fn list_recurring(
    State(state): State<Arc<BillingApiState>>,
) -> Result<Json<Vec<BillingRecurring>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = get_bot_context(&state.pool, &state.get_default_bot);

    let recurring: Vec<BillingRecurring> = billing_recurring::table
        .filter(billing_recurring::branch_id.eq(branch_id))
        .filter(billing_recurring::status.eq("active"))
        .order(billing_recurring::next_invoice_date.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(recurring))
}

pub fn configure_billing_api_routes() -> Router<Arc<BillingApiState>> {
    Router::new()
        .route("/api/billing/invoices", get(list_invoices).post(create_invoice))
        .route("/api/billing/invoices/overdue", get(list_overdue_invoices))
        .route("/api/billing/invoices/{id}", get(get_invoice).put(update_invoice).delete(delete_invoice))
        .route("/api/billing/invoices/{id}/send", put(send_invoice))
        .route("/api/billing/invoices/{id}/void", put(void_invoice))
        .route("/api/billing/payments", get(list_payments).post(record_payment))
        .route("/api/billing/payments/{id}", get(get_payment))
        .route("/api/billing/quotes", get(list_quotes).post(create_quote))
        .route("/api/billing/quotes/{id}", get(get_quote).delete(delete_quote))
        .route("/api/billing/quotes/{id}/accept", put(accept_quote))
        .route("/api/billing/quotes/{id}/reject", put(reject_quote))
        .route("/api/billing/stats", get(get_billing_stats))
        .route("/api/billing/tax-rates", get(list_tax_rates))
        .route("/api/billing/recurring", get(list_recurring))
}
