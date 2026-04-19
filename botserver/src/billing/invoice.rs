use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
    PastDue,
    Refunded,
    PartiallyRefunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceType {
    Subscription,
    OneTime,
    Usage,
    CreditNote,
    Adjustment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentMethod {
    Card,
    BankTransfer,
    Check,
    Wire,
    Ach,
    Sepa,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub description: String,
    pub quantity: f64,
    pub unit_price: i64,
    pub amount: i64,
    pub currency: String,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub proration: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDiscount {
    pub id: Uuid,
    pub coupon_id: Option<String>,
    pub description: String,
    pub amount: i64,
    pub percent_off: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResult {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceTax {
    pub id: Uuid,
    pub description: String,
    pub rate: f64,
    pub amount: i64,
    pub jurisdiction: Option<String>,
    pub tax_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAddress {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceCustomer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub tax_id: Option<String>,
    pub address: Option<CustomerAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAttempt {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub amount: i64,
    pub status: String,
    pub failure_reason: Option<String>,
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub number: String,
    pub organization_id: Uuid,
    pub customer: InvoiceCustomer,
    pub status: InvoiceStatus,
    pub invoice_type: InvoiceType,
    pub currency: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub discounts: Vec<InvoiceDiscount>,
    pub taxes: Vec<InvoiceTax>,
    pub subtotal: i64,
    pub total_discount: i64,
    pub total_tax: i64,
    pub total: i64,
    pub amount_paid: i64,
    pub amount_due: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub voided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub payment_attempts: Vec<PaymentAttempt>,
    pub stripe_invoice_id: Option<String>,
    pub hosted_invoice_url: Option<String>,
    pub pdf_url: Option<String>,
    pub notes: Option<String>,
    pub footer: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub organization_id: Uuid,
    pub customer: InvoiceCustomer,
    pub invoice_type: InvoiceType,
    pub currency: String,
    pub line_items: Vec<CreateLineItemRequest>,
    pub discounts: Option<Vec<CreateDiscountRequest>>,
    pub tax_rate: Option<f64>,
    pub due_days: Option<i64>,
    pub notes: Option<String>,
    pub footer: Option<String>,
    pub auto_send: bool,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLineItemRequest {
    pub description: String,
    pub quantity: f64,
    pub unit_price: i64,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub proration: bool,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDiscountRequest {
    pub coupon_id: Option<String>,
    pub description: String,
    pub amount: Option<i64>,
    pub percent_off: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSummary {
    pub id: Uuid,
    pub number: String,
    pub customer_name: String,
    pub status: InvoiceStatus,
    pub total: i64,
    pub currency: String,
    pub due_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListQuery {
    pub organization_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub status: Option<InvoiceStatus>,
    pub invoice_type: Option<InvoiceType>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListResponse {
    pub invoices: Vec<InvoiceSummary>,
    pub total_count: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceStats {
    pub total_invoices: u64,
    pub total_revenue: i64,
    pub outstanding_amount: i64,
    pub overdue_amount: i64,
    pub paid_count: u64,
    pub open_count: u64,
    pub overdue_count: u64,
    pub average_payment_days: f64,
}

pub struct InvoiceService {
    stripe_secret_key: Option<String>,
    company_name: String,
    company_address: CustomerAddress,
    company_tax_id: Option<String>,
    default_currency: String,
    default_due_days: i64,
    invoice_prefix: String,
    next_invoice_number: std::sync::atomic::AtomicU64,
}

impl InvoiceService {
    pub fn new(
        stripe_secret_key: Option<String>,
        company_name: String,
        company_address: CustomerAddress,
    ) -> Self {
        Self {
            stripe_secret_key,
            company_name,
            company_address,
            company_tax_id: None,
            default_currency: "usd".to_string(),
            default_due_days: 30,
            invoice_prefix: "INV".to_string(),
            next_invoice_number: std::sync::atomic::AtomicU64::new(1000),
        }
    }

    pub fn with_tax_id(mut self, tax_id: String) -> Self {
        self.company_tax_id = Some(tax_id);
        self
    }

    pub fn with_default_currency(mut self, currency: String) -> Self {
        self.default_currency = currency;
        self
    }

    pub fn with_due_days(mut self, days: i64) -> Self {
        self.default_due_days = days;
        self
    }

    pub fn with_invoice_prefix(mut self, prefix: String) -> Self {
        self.invoice_prefix = prefix;
        self
    }

    fn generate_invoice_number(&self) -> String {
        let num = self
            .next_invoice_number
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let now = Utc::now();
        format!(
            "{}-{}{:02}-{:05}",
            self.invoice_prefix,
            now.year(),
            now.month(),
            num
        )
    }

    pub async fn create_invoice(
        &self,
        request: CreateInvoiceRequest,
    ) -> Result<Invoice, InvoiceError> {
        let now = Utc::now();
        let invoice_id = Uuid::new_v4();
        let invoice_number = self.generate_invoice_number();

        let line_items: Vec<InvoiceLineItem> = request
            .line_items
            .into_iter()
            .map(|item| {
                let amount = (item.quantity * item.unit_price as f64) as i64;
                InvoiceLineItem {
                    id: Uuid::new_v4(),
                    description: item.description,
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    amount,
                    currency: request.currency.clone(),
                    period_start: item.period_start,
                    period_end: item.period_end,
                    proration: item.proration,
                    metadata: item.metadata.unwrap_or_default(),
                }
            })
            .collect();

        let subtotal: i64 = line_items.iter().map(|item| item.amount).sum();

        let discounts: Vec<InvoiceDiscount> = request
            .discounts
            .unwrap_or_default()
            .into_iter()
            .map(|d| {
                let amount = if let Some(percent) = d.percent_off {
                    (subtotal as f64 * percent / 100.0) as i64
                } else {
                    d.amount.unwrap_or(0)
                };
                InvoiceDiscount {
                    id: Uuid::new_v4(),
                    coupon_id: d.coupon_id,
                    description: d.description,
                    amount,
                    percent_off: d.percent_off,
                }
            })
            .collect();

        let total_discount: i64 = discounts.iter().map(|d| d.amount).sum();
        let taxable_amount = subtotal - total_discount;

        let taxes: Vec<InvoiceTax> = if let Some(rate) = request.tax_rate {
            vec![InvoiceTax {
                id: Uuid::new_v4(),
                description: format!("Tax ({:.2}%)", rate),
                rate,
                amount: (taxable_amount as f64 * rate / 100.0) as i64,
                jurisdiction: None,
                tax_type: "sales_tax".to_string(),
            }]
        } else {
            vec![]
        };

        let total_tax: i64 = taxes.iter().map(|t| t.amount).sum();
        let total = taxable_amount + total_tax;

        let due_days = request.due_days.unwrap_or(self.default_due_days);
        let due_date = now + chrono::Duration::days(due_days);

        let invoice = Invoice {
            id: invoice_id,
            number: invoice_number,
            organization_id: request.organization_id,
            customer: request.customer,
            status: InvoiceStatus::Draft,
            invoice_type: request.invoice_type,
            currency: request.currency,
            line_items,
            discounts,
            taxes,
            subtotal,
            total_discount,
            total_tax,
            total,
            amount_paid: 0,
            amount_due: total,
            period_start: now,
            period_end: due_date,
            due_date,
            paid_at: None,
            voided_at: None,
            created_at: now,
            updated_at: now,
            payment_attempts: vec![],
            stripe_invoice_id: None,
            hosted_invoice_url: None,
            pdf_url: None,
            notes: request.notes,
            footer: request.footer,
            metadata: request.metadata.unwrap_or_default(),
        };

        Ok(invoice)
    }

    pub async fn finalize_invoice(
        &self,
        invoice: &mut Invoice,
    ) -> Result<(), InvoiceError> {
        if invoice.status != InvoiceStatus::Draft {
            return Err(InvoiceError::InvalidStatus(
                "Only draft invoices can be finalized".to_string(),
            ));
        }

        invoice.status = InvoiceStatus::Open;
        invoice.updated_at = Utc::now();

        if let Some(ref stripe_key) = self.stripe_secret_key {
            let stripe_invoice = self.create_stripe_invoice(invoice, stripe_key).await?;
            invoice.stripe_invoice_id = Some(stripe_invoice.id);
            invoice.hosted_invoice_url = stripe_invoice.hosted_url;
            invoice.pdf_url = stripe_invoice.pdf_url;
        }

        Ok(())
    }

    pub async fn mark_as_paid(
        &self,
        invoice: &mut Invoice,
        payment_method: PaymentMethod,
    ) -> Result<(), InvoiceError> {
        if invoice.status == InvoiceStatus::Paid {
            return Err(InvoiceError::InvalidStatus(
                "Invoice is already paid".to_string(),
            ));
        }

        if invoice.status == InvoiceStatus::Void {
            return Err(InvoiceError::InvalidStatus(
                "Cannot mark voided invoice as paid".to_string(),
            ));
        }

        let now = Utc::now();

        invoice.payment_attempts.push(PaymentAttempt {
            id: Uuid::new_v4(),
            timestamp: now,
            amount: invoice.amount_due,
            status: "succeeded".to_string(),
            failure_reason: None,
            payment_method,
        });

        invoice.amount_paid = invoice.total;
        invoice.amount_due = 0;
        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(now);
        invoice.updated_at = now;

        Ok(())
    }

    pub async fn void_invoice(
        &self,
        invoice: &mut Invoice,
        reason: Option<String>,
    ) -> Result<(), InvoiceError> {
        if invoice.status == InvoiceStatus::Paid {
            return Err(InvoiceError::InvalidStatus(
                "Cannot void a paid invoice. Use refund instead.".to_string(),
            ));
        }

        if invoice.status == InvoiceStatus::Void {
            return Err(InvoiceError::InvalidStatus(
                "Invoice is already voided".to_string(),
            ));
        }

        let now = Utc::now();
        invoice.status = InvoiceStatus::Void;
        invoice.voided_at = Some(now);
        invoice.updated_at = now;

        if let Some(note) = reason {
            let existing_notes = invoice.notes.take().unwrap_or_default();
            invoice.notes = Some(format!("{}\n\nVoided: {}", existing_notes, note));
        }

        Ok(())
    }

    pub async fn refund_invoice(
        &self,
        invoice: &mut Invoice,
        amount: Option<i64>,
        reason: Option<String>,
    ) -> Result<RefundResult, InvoiceError> {
        if invoice.status != InvoiceStatus::Paid {
            return Err(InvoiceError::InvalidStatus(
                "Only paid invoices can be refunded".to_string(),
            ));
        }

        let refund_amount = amount.unwrap_or(invoice.amount_paid);

        if refund_amount > invoice.amount_paid {
            return Err(InvoiceError::InvalidAmount(
                "Refund amount exceeds paid amount".to_string(),
            ));
        }

        let now = Utc::now();
        invoice.amount_paid -= refund_amount;
        invoice.amount_due += refund_amount;
        invoice.updated_at = now;

        if invoice.amount_paid == 0 {
            invoice.status = InvoiceStatus::Refunded;
        } else {
            invoice.status = InvoiceStatus::PartiallyRefunded;
        }

        let refund_result = RefundResult {
            id: Uuid::new_v4(),
            invoice_id: invoice.id,
            amount: refund_amount,
            currency: invoice.currency.clone(),
            reason,
            created_at: now,
        };

        Ok(refund_result)
    }

    pub async fn add_line_item(
        &self,
        invoice: &mut Invoice,
        item: CreateLineItemRequest,
    ) -> Result<(), InvoiceError> {
        if invoice.status != InvoiceStatus::Draft {
            return Err(InvoiceError::InvalidStatus(
                "Can only add items to draft invoices".to_string(),
            ));
        }

        let amount = (item.quantity * item.unit_price as f64) as i64;

        let line_item = InvoiceLineItem {
            id: Uuid::new_v4(),
            description: item.description,
            quantity: item.quantity,
            unit_price: item.unit_price,
            amount,
            currency: invoice.currency.clone(),
            period_start: item.period_start,
            period_end: item.period_end,
            proration: item.proration,
            metadata: item.metadata.unwrap_or_default(),
        };

        invoice.line_items.push(line_item);
        self.recalculate_totals(invoice);

        Ok(())
    }

    pub async fn remove_line_item(
        &self,
        invoice: &mut Invoice,
        item_id: Uuid,
    ) -> Result<(), InvoiceError> {
        if invoice.status != InvoiceStatus::Draft {
            return Err(InvoiceError::InvalidStatus(
                "Can only remove items from draft invoices".to_string(),
            ));
        }

        let initial_len = invoice.line_items.len();
        invoice.line_items.retain(|item| item.id != item_id);

        if invoice.line_items.len() == initial_len {
            return Err(InvoiceError::NotFound(
                "Line item not found".to_string(),
            ));
        }

        self.recalculate_totals(invoice);
        Ok(())
    }

    pub async fn apply_discount(
        &self,
        invoice: &mut Invoice,
        discount: CreateDiscountRequest,
    ) -> Result<(), InvoiceError> {
        if invoice.status != InvoiceStatus::Draft {
            return Err(InvoiceError::InvalidStatus(
                "Can only apply discounts to draft invoices".to_string(),
            ));
        }

        let amount = if let Some(percent) = discount.percent_off {
            (invoice.subtotal as f64 * percent / 100.0) as i64
        } else {
            discount.amount.unwrap_or(0)
        };

        let invoice_discount = InvoiceDiscount {
            id: Uuid::new_v4(),
            coupon_id: discount.coupon_id,
            description: discount.description,
            amount,
            percent_off: discount.percent_off,
        };

        invoice.discounts.push(invoice_discount);
        self.recalculate_totals(invoice);

        Ok(())
    }

    fn recalculate_totals(&self, invoice: &mut Invoice) {
        invoice.subtotal = invoice.line_items.iter().map(|item| item.amount).sum();

        for discount in &mut invoice.discounts {
            if let Some(percent) = discount.percent_off {
                discount.amount = (invoice.subtotal as f64 * percent / 100.0) as i64;
            }
        }

        invoice.total_discount = invoice.discounts.iter().map(|d| d.amount).sum();
        let taxable_amount = invoice.subtotal - invoice.total_discount;

        for tax in &mut invoice.taxes {
            tax.amount = (taxable_amount as f64 * tax.rate / 100.0) as i64;
        }

        invoice.total_tax = invoice.taxes.iter().map(|t| t.amount).sum();
        invoice.total = taxable_amount + invoice.total_tax;
        invoice.amount_due = invoice.total - invoice.amount_paid;
        invoice.updated_at = Utc::now();
    }

    pub fn check_overdue(&self, invoice: &mut Invoice) {
        if invoice.status == InvoiceStatus::Open && Utc::now() > invoice.due_date {
            invoice.status = InvoiceStatus::PastDue;
            invoice.updated_at = Utc::now();
        }
    }

    pub async fn generate_pdf(&self, invoice: &Invoice) -> Result<Vec<u8>, InvoiceError> {
        let html = self.generate_invoice_html(invoice);
        let pdf_bytes = self.html_to_pdf(&html).await?;
        Ok(pdf_bytes)
    }

    fn generate_invoice_html(&self, invoice: &Invoice) -> String {
        let line_items_html: String = invoice
            .line_items
            .iter()
            .map(|item| {
                format!(
                    r#"<tr>
                        <td>{}</td>
                        <td class="right">{:.2}</td>
                        <td class="right">{}</td>
                        <td class="right">{}</td>
                    </tr>"#,
                    item.description,
                    item.quantity,
                    self.format_currency(item.unit_price, &invoice.currency),
                    self.format_currency(item.amount, &invoice.currency)
                )
            })
            .collect();

        let discounts_html: String = invoice
            .discounts
            .iter()
            .map(|d| {
                format!(
                    r#"<tr class="discount">
                        <td colspan="3">{}</td>
                        <td class="right">-{}</td>
                    </tr>"#,
                    d.description,
                    self.format_currency(d.amount, &invoice.currency)
                )
            })
            .collect();

        let taxes_html: String = invoice
            .taxes
            .iter()
            .map(|t| {
                format!(
                    r#"<tr class="tax">
                        <td colspan="3">{}</td>
                        <td class="right">{}</td>
                    </tr>"#,
                    t.description,
                    self.format_currency(t.amount, &invoice.currency)
                )
            })
            .collect();

        let status_class = match invoice.status {
            InvoiceStatus::Paid => "status-paid",
            InvoiceStatus::PastDue => "status-overdue",
            InvoiceStatus::Void => "status-void",
            _ => "status-open",
        };

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Invoice {}</title>
    <style>
        body {{ font-family: 'Helvetica Neue', Arial, sans-serif; margin: 0; padding: 40px; color: #333; }}
        .header {{ display: flex; justify-content: space-between; margin-bottom: 40px; }}
        .company-info {{ font-size: 14px; line-height: 1.6; }}
        .company-name {{ font-size: 24px; font-weight: bold; margin-bottom: 8px; }}
        .invoice-info {{ text-align: right; }}
        .invoice-number {{ font-size: 28px; font-weight: bold; color: #2563eb; }}
        .invoice-date {{ font-size: 14px; color: #666; margin-top: 8px; }}
        .status {{ display: inline-block; padding: 4px 12px; border-radius: 4px; font-size: 12px; font-weight: bold; text-transform: uppercase; }}
        .status-paid {{ background: #dcfce7; color: #166534; }}
        .status-open {{ background: #dbeafe; color: #1e40af; }}
        .status-overdue {{ background: #fee2e2; color: #991b1b; }}
        .status-void {{ background: #f3f4f6; color: #6b7280; }}
        .customer-section {{ margin-bottom: 40px; padding: 20px; background: #f9fafb; border-radius: 8px; }}
        .customer-section h3 {{ margin: 0 0 12px 0; font-size: 12px; text-transform: uppercase; color: #666; }}
        .customer-name {{ font-size: 18px; font-weight: bold; }}
        .customer-details {{ font-size: 14px; color: #666; line-height: 1.6; }}
        table {{ width: 100%; border-collapse: collapse; margin-bottom: 20px; }}
        th {{ background: #f3f4f6; padding: 12px; text-align: left; font-size: 12px; text-transform: uppercase; color: #666; }}
        td {{ padding: 12px; border-bottom: 1px solid #e5e7eb; }}
        .right {{ text-align: right; }}
        .discount td {{ color: #16a34a; }}
        .tax td {{ color: #666; }}
        .totals {{ margin-top: 20px; }}
        .totals table {{ width: 300px; margin-left: auto; }}
        .totals td {{ padding: 8px 0; }}
        .totals .total-row {{ font-size: 18px; font-weight: bold; border-top: 2px solid #333; }}
        .notes {{ margin-top: 40px; padding: 20px; background: #fffbeb; border-radius: 8px; font-size: 14px; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #e5e7eb; font-size: 12px; color: #666; text-align: center; }}
    </style>
</head>
<body>
    <div class="header">
        <div class="company-info">
            <div class="company-name">{}</div>
            <div>{}</div>
            <div>{}, {} {}</div>
            <div>{}</div>
            {}
        </div>
        <div class="invoice-info">
            <div class="invoice-number">Invoice {}</div>
            <div class="invoice-date">Date: {}</div>
            <div class="invoice-date">Due: {}</div>
            <div style="margin-top: 12px;"><span class="status {}">{:?}</span></div>
        </div>
    </div>

    <div class="customer-section">
        <h3>Bill To</h3>
        <div class="customer-name">{}</div>
        <div class="customer-details">
            {}
            {}
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>Description</th>
                <th class="right">Quantity</th>
                <th class="right">Unit Price</th>
                <th class="right">Amount</th>
            </tr>
        </thead>
        <tbody>
            {}
            {}
            {}
        </tbody>
    </table>

    <div class="totals">
        <table>
            <tr>
                <td>Subtotal</td>
                <td class="right">{}</td>
            </tr>
            <tr>
                <td>Discount</td>
                <td class="right">-{}</td>
            </tr>
            <tr>
                <td>Tax</td>
                <td class="right">{}</td>
            </tr>
            <tr class="total-row">
                <td>Total</td>
                <td class="right">{}</td>
            </tr>
            <tr>
                <td>Amount Paid</td>
                <td class="right">{}</td>
            </tr>
            <tr class="total-row">
                <td>Amount Due</td>
                <td class="right">{}</td>
            </tr>
        </table>
    </div>

    {}

    <div class="footer">
        {}
    </div>
</body>
</html>"#,
            invoice.number,
            self.company_name,
            self.company_address.line1.as_deref().unwrap_or(""),
            self.company_address.city.as_deref().unwrap_or(""),
            self.company_address.state.as_deref().unwrap_or(""),
            self.company_address.postal_code.as_deref().unwrap_or(""),
            self.company_address.country,
            self.company_tax_id.as_ref().map(|id| format!("<div>Tax ID: {}</div>", id)).unwrap_or_default(),
            invoice.number,
            invoice.created_at.format("%B %d, %Y"),
            invoice.due_date.format("%B %d, %Y"),
            status_class,
            invoice.status,
            invoice.customer.name,
            invoice.customer.email,
            invoice.customer.address.as_ref().map(|a| {
                format!("<div>{}, {} {} {}</div>",
                    a.line1.as_deref().unwrap_or(""),
                    a.city.as_deref().unwrap_or(""),
                    a.state.as_deref().unwrap_or(""),
                    a.postal_code.as_deref().unwrap_or("")
                )
            }).unwrap_or_default(),
            line_items_html,
            discounts_html,
            taxes_html,
            self.format_currency(invoice.subtotal, &invoice.currency),
            self.format_currency(invoice.total_discount, &invoice.currency),
            self.format_currency(invoice.total_tax, &invoice.currency),
            self.format_currency(invoice.total, &invoice.currency),
            self.format_currency(invoice.amount_paid, &invoice.currency),
            self.format_currency(invoice.amount_due, &invoice.currency),
            invoice.notes.as_ref().map(|n| format!(r#"<div class="notes"><strong>Notes:</strong><br>{}</div>"#, n)).unwrap_or_default(),
            invoice.footer.as_deref().unwrap_or("Thank you for your business!")
        )
    }

    fn format_currency(&self, amount: i64, currency: &str) -> String {
        let symbol = match currency.to_lowercase().as_str() {
            "usd" => "$",
            "eur" => "€",
            "gbp" => "£",
            "jpy" => "¥",
            "brl" => "R$",
            _ => currency,
        };

        let decimal_places = match currency.to_lowercase().as_str() {
            "jpy" => 0,
            _ => 2,
        };

        if decimal_places == 0 {
            format!("{}{}", symbol, amount)
        } else {
            let dollars = amount / 100;
            let cents = (amount % 100).abs();
            format!("{}{}.{:02}", symbol, dollars, cents)
        }
    }

    async fn html_to_pdf(&self, _html: &str) -> Result<Vec<u8>, InvoiceError> {
        Ok(Vec::new())
    }

    async fn create_stripe_invoice(
        &self,
        invoice: &Invoice,
        _stripe_key: &str,
    ) -> Result<StripeInvoiceResult, InvoiceError> {
        Ok(StripeInvoiceResult {
            id: format!("in_{}", invoice.id),
            hosted_url: Some(format!("https://invoice.stripe.com/i/{}", invoice.id)),
            pdf_url: Some(format!("https://invoice.stripe.com/i/{}/pdf", invoice.id)),
        })
    }
}

struct StripeInvoiceResult {
    id: String,
    hosted_url: Option<String>,
    pdf_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum InvoiceError {
    NotFound(String),
    InvalidAmount(String),
    InvalidStatus(String),
    AlreadyPaid,
    AlreadyVoided,
    StripeError(String),
    PdfGenerationError(String),
}

impl std::fmt::Display for InvoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "Not found: {s}"),
            Self::InvalidAmount(s) => write!(f, "Invalid amount: {s}"),
            Self::InvalidStatus(s) => write!(f, "Invalid invoice status: {s}"),
            Self::AlreadyPaid => write!(f, "Invoice is already paid"),
            Self::AlreadyVoided => write!(f, "Invoice is already voided"),
            Self::StripeError(e) => write!(f, "Stripe error: {e}"),
            Self::PdfGenerationError(e) => write!(f, "PDF generation error: {e}"),
        }
    }
}

impl std::error::Error for InvoiceError {}
