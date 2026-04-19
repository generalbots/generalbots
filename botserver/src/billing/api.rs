use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

use bigdecimal::BigDecimal;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::{
    billing_invoice_items, billing_invoices, billing_payments, billing_quote_items,
    billing_quotes, billing_recurring, billing_tax_rates,
};
use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = billing_invoices)]
pub struct BillingInvoice {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub invoice_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub status: String,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub subtotal: BigDecimal,
    pub tax_rate: BigDecimal,
    pub tax_amount: BigDecimal,
    pub discount_percent: BigDecimal,
    pub discount_amount: BigDecimal,
    pub total: BigDecimal,
    pub amount_paid: BigDecimal,
    pub amount_due: BigDecimal,
    pub currency: String,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub footer: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub voided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = billing_invoice_items)]
pub struct BillingInvoiceItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub product_id: Option<Uuid>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: BigDecimal,
    pub tax_rate: BigDecimal,
    pub amount: BigDecimal,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = billing_payments)]
pub struct BillingPayment {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub payment_number: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub payment_method: String,
    pub payment_reference: Option<String>,
    pub status: String,
    pub payer_name: Option<String>,
    pub payer_email: Option<String>,
    pub notes: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub refund_amount: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = billing_quotes)]
pub struct BillingQuote {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub quote_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub status: String,
    pub issue_date: NaiveDate,
    pub valid_until: NaiveDate,
    pub subtotal: BigDecimal,
    pub tax_rate: BigDecimal,
    pub tax_amount: BigDecimal,
    pub discount_percent: BigDecimal,
    pub discount_amount: BigDecimal,
    pub total: BigDecimal,
    pub currency: String,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub converted_invoice_id: Option<Uuid>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = billing_quote_items)]
pub struct BillingQuoteItem {
    pub id: Uuid,
    pub quote_id: Uuid,
    pub product_id: Option<Uuid>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: BigDecimal,
    pub tax_rate: BigDecimal,
    pub amount: BigDecimal,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = billing_recurring)]
pub struct BillingRecurring {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub status: String,
    pub frequency: String,
    pub interval_count: i32,
    pub amount: BigDecimal,
    pub currency: String,
    pub description: Option<String>,
    pub next_invoice_date: NaiveDate,
    pub last_invoice_date: Option<NaiveDate>,
    pub last_invoice_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub invoices_generated: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = billing_tax_rates)]
pub struct BillingTaxRate {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub rate: BigDecimal,
    pub description: Option<String>,
    pub region: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub customer_id: Option<Uuid>,
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub tax_rate: Option<f64>,
    pub discount_percent: Option<f64>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub items: Option<Vec<InvoiceItemRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceItemRequest {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_percent: Option<f64>,
    pub tax_rate: Option<f64>,
    pub product_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvoiceRequest {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub due_date: Option<String>,
    pub tax_rate: Option<f64>,
    pub discount_percent: Option<f64>,
    pub notes: Option<String>,
    pub terms: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordPaymentRequest {
    pub invoice_id: Option<Uuid>,
    pub amount: f64,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub payer_name: Option<String>,
    pub payer_email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuoteRequest {
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub customer_id: Option<Uuid>,
    pub issue_date: Option<String>,
    pub valid_until: Option<String>,
    pub tax_rate: Option<f64>,
    pub discount_percent: Option<f64>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub items: Option<Vec<InvoiceItemRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BillingStats {
    pub total_revenue: f64,
    pub revenue_this_month: f64,
    pub pending_amount: f64,
    pub overdue_amount: f64,
    pub paid_this_month: f64,
    pub invoice_count: i64,
    pub payment_count: i64,
    pub overdue_count: i64,
}

#[derive(Debug, Serialize)]
pub struct InvoiceWithItems {
    pub invoice: BillingInvoice,
    pub items: Vec<BillingInvoiceItem>,
}

#[derive(Debug, Serialize)]
pub struct QuoteWithItems {
    pub quote: BillingQuote,
    pub items: Vec<BillingQuoteItem>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn generate_invoice_number(conn: &mut diesel::PgConnection, org_id: Uuid) -> String {
    let count: i64 = billing_invoices::table
        .filter(billing_invoices::org_id.eq(org_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);
    format!("INV-{:06}", count + 1)
}

fn generate_payment_number(conn: &mut diesel::PgConnection, org_id: Uuid) -> String {
    let count: i64 = billing_payments::table
        .filter(billing_payments::org_id.eq(org_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);
    format!("PAY-{:06}", count + 1)
}

fn generate_quote_number(conn: &mut diesel::PgConnection, org_id: Uuid) -> String {
    let count: i64 = billing_quotes::table
        .filter(billing_quotes::org_id.eq(org_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);
    format!("QTE-{:06}", count + 1)
}

fn bd(val: f64) -> BigDecimal {
    BigDecimal::from_str(&val.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
}

fn bd_to_f64(val: &BigDecimal) -> f64 {
    val.to_string().parse::<f64>().unwrap_or(0.0)
}

pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let invoice_number = generate_invoice_number(&mut conn, org_id);

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
        subtotal = subtotal + bd(item_amount - item_discount);
    }

    let discount_amount = &subtotal * &discount_percent / bd(100.0);
    let taxable = &subtotal - &discount_amount;
    let tax_amount = &taxable * &tax_rate / bd(100.0);
    let total = &taxable + &tax_amount;

    let invoice = BillingInvoice {
        id,
        org_id,
        bot_id,
        invoice_number,
        customer_id: req.customer_id,
        customer_name: req.customer_name,
        customer_email: req.customer_email,
        customer_address: req.customer_address,
        status: "draft".to_string(),
        issue_date,
        due_date,
        subtotal: subtotal.clone(),
        tax_rate,
        tax_amount,
        discount_percent,
        discount_amount,
        total: total.clone(),
        amount_paid: bd(0.0),
        amount_due: total,
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        notes: req.notes,
        terms: req.terms,
        footer: None,
        paid_at: None,
        sent_at: None,
        voided_at: None,
        created_at: now,
        updated_at: now,
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BillingInvoice>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = billing_invoices::table
        .filter(billing_invoices::org_id.eq(org_id))
        .filter(billing_invoices::bot_id.eq(bot_id))
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceWithItems>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateInvoiceRequest>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingInvoice>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(billing_invoices::table.filter(billing_invoices::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn record_payment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecordPaymentRequest>,
) -> Result<Json<BillingPayment>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let payment_number = generate_payment_number(&mut conn, org_id);

    let payment = BillingPayment {
        id,
        org_id,
        bot_id,
        invoice_id: req.invoice_id,
        payment_number,
        amount: bd(req.amount),
        currency: "USD".to_string(),
        payment_method: req.payment_method.unwrap_or_else(|| "other".to_string()),
        payment_reference: req.payment_reference,
        status: "completed".to_string(),
        payer_name: req.payer_name,
        payer_email: req.payer_email,
        notes: req.notes,
        paid_at: now,
        refunded_at: None,
        refund_amount: None,
        created_at: now,
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
        let new_due = &invoice.total - &new_paid;

        let new_status = if bd_to_f64(&new_due) <= 0.0 {
            "paid"
        } else if bd_to_f64(&new_paid) > 0.0 {
            "partial"
        } else {
            &invoice.status
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BillingPayment>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = billing_payments::table
        .filter(billing_payments::org_id.eq(org_id))
        .filter(billing_payments::bot_id.eq(bot_id))
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingPayment>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let payment: BillingPayment = billing_payments::table
        .filter(billing_payments::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Payment not found".to_string()))?;

    Ok(Json(payment))
}

pub async fn create_quote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateQuoteRequest>,
) -> Result<Json<BillingQuote>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let quote_number = generate_quote_number(&mut conn, org_id);

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
        subtotal = subtotal + bd(item_amount - item_discount);
    }

    let discount_amount = &subtotal * &discount_percent / bd(100.0);
    let taxable = &subtotal - &discount_amount;
    let tax_amount = &taxable * &tax_rate / bd(100.0);
    let total = &taxable + &tax_amount;

    let quote = BillingQuote {
        id,
        org_id,
        bot_id,
        quote_number,
        customer_id: req.customer_id,
        customer_name: req.customer_name,
        customer_email: req.customer_email,
        customer_address: req.customer_address,
        status: "draft".to_string(),
        issue_date,
        valid_until,
        subtotal,
        tax_rate,
        tax_amount,
        discount_percent,
        discount_amount,
        total,
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        notes: req.notes,
        terms: req.terms,
        accepted_at: None,
        rejected_at: None,
        converted_invoice_id: None,
        sent_at: None,
        created_at: now,
        updated_at: now,
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BillingQuote>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = billing_quotes::table
        .filter(billing_quotes::org_id.eq(org_id))
        .filter(billing_quotes::bot_id.eq(bot_id))
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<QuoteWithItems>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingQuote>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BillingQuote>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(billing_quotes::table.filter(billing_quotes::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_billing_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BillingStats>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let today = Utc::now().date_naive();

    let invoices: Vec<BillingInvoice> = billing_invoices::table
        .filter(billing_invoices::org_id.eq(org_id))
        .filter(billing_invoices::bot_id.eq(bot_id))
        .load(&mut conn)
        .unwrap_or_default();

    let mut total_revenue = 0.0;
    let mut pending_amount = 0.0;
    let mut overdue_amount = 0.0;
    let mut overdue_count = 0i64;

    for inv in &invoices {
        if inv.status == "paid" {
            total_revenue += bd_to_f64(&inv.total);
        }
        if inv.status != "paid" && inv.status != "voided" {
            pending_amount += bd_to_f64(&inv.amount_due);
            if inv.due_date < today {
                overdue_amount += bd_to_f64(&inv.amount_due);
                overdue_count += 1;
            }
        }
    }

    let payments: Vec<BillingPayment> = billing_payments::table
        .filter(billing_payments::org_id.eq(org_id))
        .filter(billing_payments::bot_id.eq(bot_id))
        .filter(billing_payments::status.eq("completed"))
        .load(&mut conn)
        .unwrap_or_default();

    let paid_this_month: f64 = payments
        .iter()
        .filter(|p| p.paid_at.date_naive().month() == today.month() && p.paid_at.date_naive().year() == today.year())
        .map(|p| bd_to_f64(&p.amount))
        .sum();

    let revenue_this_month: f64 = invoices
        .iter()
        .filter(|i| i.status == "paid" && i.paid_at.map(|d| d.date_naive().month() == today.month() && d.date_naive().year() == today.year()).unwrap_or(false))
        .map(|i| bd_to_f64(&i.total))
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BillingInvoice>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let today = Utc::now().date_naive();

    let invoices: Vec<BillingInvoice> = billing_invoices::table
        .filter(billing_invoices::org_id.eq(org_id))
        .filter(billing_invoices::bot_id.eq(bot_id))
        .filter(billing_invoices::status.ne("paid"))
        .filter(billing_invoices::status.ne("voided"))
        .filter(billing_invoices::due_date.lt(today))
        .order(billing_invoices::due_date.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(invoices))
}

pub async fn list_tax_rates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BillingTaxRate>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let rates: Vec<BillingTaxRate> = billing_tax_rates::table
        .filter(billing_tax_rates::org_id.eq(org_id))
        .filter(billing_tax_rates::bot_id.eq(bot_id))
        .filter(billing_tax_rates::is_active.eq(true))
        .order(billing_tax_rates::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(rates))
}

pub async fn list_recurring(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BillingRecurring>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let recurring: Vec<BillingRecurring> = billing_recurring::table
        .filter(billing_recurring::org_id.eq(org_id))
        .filter(billing_recurring::bot_id.eq(bot_id))
        .filter(billing_recurring::status.eq("active"))
        .order(billing_recurring::next_invoice_date.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(recurring))
}

pub fn configure_billing_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/billing/invoices", get(list_invoices).post(create_invoice))
        .route("/api/billing/invoices/overdue", get(list_overdue_invoices))
        .route("/api/billing/invoices/:id", get(get_invoice).put(update_invoice).delete(delete_invoice))
        .route("/api/billing/invoices/:id/send", put(send_invoice))
        .route("/api/billing/invoices/:id/void", put(void_invoice))
        .route("/api/billing/payments", get(list_payments).post(record_payment))
        .route("/api/billing/payments/:id", get(get_payment))
        .route("/api/billing/quotes", get(list_quotes).post(create_quote))
        .route("/api/billing/quotes/:id", get(get_quote).delete(delete_quote))
        .route("/api/billing/quotes/:id/accept", put(accept_quote))
        .route("/api/billing/quotes/:id/reject", put(reject_quote))
        .route("/api/billing/stats", get(get_billing_stats))
        .route("/api/billing/tax-rates", get(list_tax_rates))
        .route("/api/billing/recurring", get(list_recurring))
}
