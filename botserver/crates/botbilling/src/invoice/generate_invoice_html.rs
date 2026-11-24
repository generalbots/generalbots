use super::*;

/// Escapes text before it is interpolated into the invoice's HTML markup.
///
/// `&` is replaced first so a pre-encoded entity such as `&lt;` cannot be
/// decoded twice back into markup; the remaining characters cover both
/// element context (`<`/`>`) and attribute context (`"`/`'`). Every value that
/// reaches the invoice template — customer, business, line-item, notes and
/// currency-derived strings — goes through this helper.
fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl InvoiceService {
    pub(super) fn generate_invoice_html(&self, invoice: &Invoice) -> String {
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
                    escape_html(&item.description),
                    item.quantity,
                    escape_html(&self.format_currency(item.unit_price, &invoice.currency)),
                    escape_html(&self.format_currency(item.amount, &invoice.currency))
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
                    escape_html(&d.description),
                    escape_html(&self.format_currency(d.amount, &invoice.currency))
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
                    escape_html(&t.description),
                    escape_html(&self.format_currency(t.amount, &invoice.currency))
                )
            })
            .collect();

        let status_class = match invoice.status {
            InvoiceStatus::Paid => "status-paid",
            InvoiceStatus::PastDue => "status-overdue",
            InvoiceStatus::Void => "status-void",
            _ => "status-open",
        };

        // Escape every caller-supplied value once, up front, so the template
        // below cannot accidentally interpolate a raw string.
        let number = escape_html(&invoice.number);
        let company_name = escape_html(&self.company_name);
        let company_line1 = escape_html(self.company_address.line1.as_deref().unwrap_or(""));
        let company_city = escape_html(self.company_address.city.as_deref().unwrap_or(""));
        let company_state = escape_html(self.company_address.state.as_deref().unwrap_or(""));
        let company_postal = escape_html(self.company_address.postal_code.as_deref().unwrap_or(""));
        let company_country = escape_html(&self.company_address.country);
        let company_tax_id = self
            .company_tax_id
            .as_ref()
            .map(|id| format!("<div>Tax ID: {}</div>", escape_html(id)))
            .unwrap_or_else(String::new);
        let customer_name = escape_html(&invoice.customer.name);
        let customer_email = escape_html(&invoice.customer.email);
        let customer_address = invoice
            .customer
            .address
            .as_ref()
            .map(|a| {
                format!(
                    "<div>{}, {} {} {}</div>",
                    escape_html(a.line1.as_deref().unwrap_or("")),
                    escape_html(a.city.as_deref().unwrap_or("")),
                    escape_html(a.state.as_deref().unwrap_or("")),
                    escape_html(a.postal_code.as_deref().unwrap_or(""))
                )
            })
            .unwrap_or_else(String::new);
        let notes = invoice
            .notes
            .as_ref()
            .map(|n| {
                format!(
                    r#"<div class="notes"><strong>Notes:</strong><br>{}</div>"#,
                    escape_html(n)
                )
            })
            .unwrap_or_else(String::new);
        let footer = escape_html(
            invoice
                .footer
                .as_deref()
                .unwrap_or("Thank you for your business!"),
        );

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
            number,
            company_name,
            company_line1,
            company_city,
            company_state,
            company_postal,
            company_country,
            company_tax_id,
            number,
            invoice.created_at.format("%B %d, %Y"),
            invoice.due_date.format("%B %d, %Y"),
            status_class,
            invoice.status,
            customer_name,
            customer_email,
            customer_address,
            line_items_html,
            discounts_html,
            taxes_html,
            escape_html(&self.format_currency(invoice.subtotal, &invoice.currency)),
            escape_html(&self.format_currency(invoice.total_discount, &invoice.currency)),
            escape_html(&self.format_currency(invoice.total_tax, &invoice.currency)),
            escape_html(&self.format_currency(invoice.total, &invoice.currency)),
            escape_html(&self.format_currency(invoice.amount_paid, &invoice.currency)),
            escape_html(&self.format_currency(invoice.amount_due, &invoice.currency)),
            notes,
            footer
        )
    }

    pub(super) fn format_currency(&self, amount: i64, currency: &str) -> String {
        let symbol = match currency.to_lowercase().as_str() {
            "usd" => "$",
            "eur" => "\u{20ac}",
            "gbp" => "\u{a3}",
            "jpy" => "\u{a5}",
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

    pub(crate) async fn html_to_pdf(&self, _html: &str) -> Result<Vec<u8>, InvoiceError> {
        Ok(Vec::new())
    }

    pub(crate) async fn create_stripe_invoice(
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

#[cfg(test)]
mod tests {
    use super::*;

    const IMG_PAYLOAD: &str = "<img src=x onerror=alert(1)>";
    const QUOTE_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    fn service() -> InvoiceService {
        InvoiceService::new(
            None,
            "Acme <Corp>".to_string(),
            CustomerAddress {
                line1: Some("1 <Main> St".to_string()),
                line2: None,
                city: Some("Sao <Paulo>".to_string()),
                state: Some("SP".to_string()),
                postal_code: Some("01000".to_string()),
                country: "BR".to_string(),
            },
        )
    }

    fn invoice(description: &str, customer_name: &str) -> Invoice {
        let now = Utc::now();
        Invoice {
            id: Uuid::new_v4(),
            number: "INV-<1>".to_string(),
            organization_id: Uuid::nil(),
            customer: InvoiceCustomer {
                id: Uuid::new_v4(),
                name: customer_name.to_string(),
                email: "billing@example.test".to_string(),
                tax_id: None,
                address: Some(CustomerAddress {
                    line1: Some("2 \"Elm\" St".to_string()),
                    line2: None,
                    city: Some("City".to_string()),
                    state: Some("ST".to_string()),
                    postal_code: Some("12345".to_string()),
                    country: "US".to_string(),
                }),
            },
            status: InvoiceStatus::Draft,
            invoice_type: InvoiceType::Subscription,
            currency: "usd".to_string(),
            line_items: vec![InvoiceLineItem {
                id: Uuid::new_v4(),
                description: description.to_string(),
                quantity: 1.0,
                unit_price: 1000,
                amount: 1000,
                currency: "usd".to_string(),
                period_start: None,
                period_end: None,
                proration: false,
                metadata: HashMap::new(),
            }],
            discounts: vec![InvoiceDiscount {
                id: Uuid::new_v4(),
                coupon_id: None,
                description: IMG_PAYLOAD.to_string(),
                amount: 0,
                percent_off: None,
            }],
            taxes: vec![InvoiceTax {
                id: Uuid::new_v4(),
                description: IMG_PAYLOAD.to_string(),
                rate: 0.0,
                amount: 0,
                jurisdiction: None,
                tax_type: "sales".to_string(),
            }],
            subtotal: 1000,
            total_discount: 0,
            total_tax: 0,
            total: 1000,
            amount_paid: 0,
            amount_due: 1000,
            period_start: now,
            period_end: now,
            due_date: now,
            paid_at: None,
            voided_at: None,
            created_at: now,
            updated_at: now,
            payment_attempts: Vec::new(),
            stripe_invoice_id: None,
            hosted_invoice_url: None,
            pdf_url: None,
            notes: Some(IMG_PAYLOAD.to_string()),
            footer: Some(IMG_PAYLOAD.to_string()),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn line_item_description_cannot_inject_an_image_tag() {
        let html = service().generate_invoice_html(&invoice(IMG_PAYLOAD, "Acme"));

        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!html.contains(IMG_PAYLOAD));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn customer_name_breakout_is_escaped() {
        let html =
            service().generate_invoice_html(&invoice("item", "</td><img src=x onerror=alert(1)>"));

        assert!(html.contains("&lt;/td&gt;&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!html.contains("</td><img"));
    }

    #[test]
    fn quote_breakout_and_dangerous_characters_are_escaped() {
        let html = service().generate_invoice_html(&invoice(QUOTE_PAYLOAD, QUOTE_PAYLOAD));

        assert!(html.contains("&quot; onmouseover=&quot;alert(1)"));
        assert!(!html.contains(QUOTE_PAYLOAD));
        assert_eq!(escape_html("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
        assert_eq!(escape_html("&lt;"), "&amp;lt;");
    }

    #[test]
    fn business_and_invoice_fields_are_escaped_too() {
        let html = service().generate_invoice_html(&invoice("item", "Acme"));

        // company name, address, invoice number, notes and footer all carried
        // angle brackets in the fixtures and must never appear raw.
        assert!(!html.contains("<Corp>"));
        assert!(!html.contains("<Main>"));
        assert!(!html.contains("INV-<1>"));
        assert!(!html.contains("<script"));
        assert!(html.contains("&lt;Corp&gt;"));
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
    }
}
