pub(crate) use axum::{ extract::{Path, Query, State}, http::StatusCode, routing::{get, put}, Json, Router, };
pub(crate) use chrono::{Datelike, NaiveDate, Utc};
pub(crate) use diesel::prelude::*;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;
pub(crate) use crate::api_models::{ bd, bd_to_f64, generate_invoice_number, generate_payment_number, generate_quote_number, BillingInvoice, BillingInvoiceItem, BillingPayment, BillingQuote, BillingQuoteItem, BillingRecurring, BillingStats, BillingTaxRate, CreateInvoiceRequest, CreateQuoteRequest, InvoiceWithItems, ListQuery, QuoteWithItems, RecordPaymentRequest, UpdateInvoiceRequest, };
pub(crate) use crate::schema::{ billing_invoice_items, billing_invoices, billing_payments, billing_quote_items, billing_quotes, billing_recurring, billing_tax_rates, };
pub(crate) use crate::{get_bot_context, scope_for_branch, DbPool, GetDefaultBotFn};

pub struct BillingApiState {
    pub pool: Arc<DbPool>,
    pub get_default_bot: Option<GetDefaultBotFn>,
}

mod create_invoice;
mod record_payment;
mod accept_quote;

pub use create_invoice::*;
pub use record_payment::*;
pub use accept_quote::*;
