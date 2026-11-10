use super::*;

fn customer() -> InvoiceCustomer {
    InvoiceCustomer {
        id: Uuid::new_v4(),
        name: "Acme".to_string(),
        email: "billing@acme.test".to_string(),
        tax_id: None,
        address: None,
    }
}

fn line_item(quantity: f64, unit_price: i64, amount: i64) -> InvoiceLineItem {
    InvoiceLineItem {
        id: Uuid::new_v4(),
        description: "item".to_string(),
        quantity,
        unit_price,
        amount,
        currency: "usd".to_string(),
        period_start: None,
        period_end: None,
        proration: false,
        metadata: HashMap::new(),
    }
}

fn discount(amount: i64, percent_off: Option<f64>) -> InvoiceDiscount {
    InvoiceDiscount {
        id: Uuid::new_v4(),
        coupon_id: None,
        description: "discount".to_string(),
        amount,
        percent_off,
    }
}

fn tax(rate: f64) -> InvoiceTax {
    InvoiceTax {
        id: Uuid::new_v4(),
        description: "tax".to_string(),
        rate,
        amount: 0,
        jurisdiction: None,
        tax_type: "sales".to_string(),
    }
}

fn invoice(line_items: Vec<InvoiceLineItem>) -> Invoice {
    let now = Utc::now();
    Invoice {
        id: Uuid::new_v4(),
        number: "INV-1".to_string(),
        organization_id: Uuid::nil(),
        customer: customer(),
        status: InvoiceStatus::Draft,
        invoice_type: InvoiceType::Subscription,
        currency: "usd".to_string(),
        line_items,
        discounts: Vec::new(),
        taxes: Vec::new(),
        subtotal: 0,
        total_discount: 0,
        total_tax: 0,
        total: 0,
        amount_paid: 0,
        amount_due: 0,
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
        notes: None,
        footer: None,
        metadata: HashMap::new(),
    }
}

fn service() -> InvoiceService {
    InvoiceService::new(
        None,
        "General Bots".to_string(),
        CustomerAddress {
            line1: None,
            line2: None,
            city: None,
            state: None,
            postal_code: None,
            country: "BR".to_string(),
        },
    )
}

#[test]
fn the_subtotal_is_the_sum_of_the_line_item_amounts() {
    let svc = service();
    let mut inv = invoice(vec![
        line_item(2.0, 1000, 2000),
        line_item(1.0, 500, 500),
    ]);

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.subtotal, 2500);
    assert_eq!(inv.total_discount, 0);
    assert_eq!(inv.total_tax, 0);
    assert_eq!(inv.total, 2500);
    assert_eq!(inv.amount_due, 2500);
}

#[test]
fn an_empty_invoice_totals_zero() {
    let svc = service();
    let mut inv = invoice(Vec::new());

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.subtotal, 0);
    assert_eq!(inv.total, 0);
    assert_eq!(inv.amount_due, 0);
}

#[test]
fn a_percent_discount_is_recomputed_from_the_subtotal() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 2000, 2000)]);
    inv.discounts = vec![discount(0, Some(10.0))];

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.discounts[0].amount, 200);
    assert_eq!(inv.total_discount, 200);
    assert_eq!(inv.total, 1800);
}

#[test]
fn a_fixed_discount_keeps_its_amount() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 2000, 2000)]);
    inv.discounts = vec![discount(250, None)];

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.total_discount, 250);
    assert_eq!(inv.total, 1750);
}

#[test]
fn multiple_discounts_are_summed() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 1000, 1000)]);
    inv.discounts = vec![discount(100, None), discount(0, Some(20.0))];

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.discounts[1].amount, 200);
    assert_eq!(inv.total_discount, 300);
    assert_eq!(inv.total, 700);
}

#[test]
fn tax_is_charged_on_the_discounted_amount() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 2000, 2000)]);
    inv.discounts = vec![discount(0, Some(10.0))];
    inv.taxes = vec![tax(10.0)];

    svc.recalculate_totals(&mut inv);

    // taxable = 2000 - 200 = 1800; tax = 180
    assert_eq!(inv.taxes[0].amount, 180);
    assert_eq!(inv.total_tax, 180);
    assert_eq!(inv.total, 1980);
}

#[test]
fn multiple_taxes_are_summed() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 1000, 1000)]);
    inv.taxes = vec![tax(10.0), tax(5.0)];

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.total_tax, 150);
    assert_eq!(inv.total, 1150);
}

#[test]
fn the_amount_due_subtracts_the_payments_already_made() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 1000, 1000)]);
    inv.amount_paid = 400;

    svc.recalculate_totals(&mut inv);

    assert_eq!(inv.total, 1000);
    assert_eq!(inv.amount_due, 600);
}

#[test]
fn a_discount_larger_than_the_subtotal_never_makes_the_total_negative() {
    let svc = service();
    let mut inv = invoice(vec![line_item(1.0, 100, 100)]);
    inv.discounts = vec![discount(150, None)];
    inv.taxes = vec![tax(10.0)];

    svc.recalculate_totals(&mut inv);

    // The taxable base floors at zero: no negative tax and no negative total.
    assert_eq!(inv.total_discount, 150);
    assert_eq!(inv.taxes[0].amount, 0);
    assert_eq!(inv.total, 0);
    assert_eq!(inv.amount_due, 0);
}
