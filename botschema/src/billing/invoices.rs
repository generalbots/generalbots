diesel::table! {
    billing_invoices (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 50]
    invoice_number -> Varchar,
    customer_id -> Nullable<Uuid>,
    #[max_length = 255]
    customer_name -> Varchar,
    #[max_length = 255]
    customer_email -> Nullable<Varchar>,
    customer_address -> Nullable<Text>,
    #[max_length = 50]
    status -> Varchar,
    issue_date -> Date,
    due_date -> Date,
    subtotal -> Numeric,
    tax_rate -> Numeric,
    tax_amount -> Numeric,
    discount_percent -> Numeric,
    discount_amount -> Numeric,
    total -> Numeric,
    amount_paid -> Numeric,
    amount_due -> Numeric,
    #[max_length = 3]
    currency -> Varchar,
    notes -> Nullable<Text>,
    terms -> Nullable<Text>,
    footer -> Nullable<Text>,
    paid_at -> Nullable<Timestamptz>,
    sent_at -> Nullable<Timestamptz>,
    voided_at -> Nullable<Timestamptz>,
    created_at -> Timestamptz,
    updated_at -> Timestamptz,
    branch_id -> Uuid,
    }
}


diesel::table! {
    billing_invoice_items (id) {
        id -> Uuid,
        invoice_id -> Uuid,
        product_id -> Nullable<Uuid>,
        description -> Varchar,
        quantity -> Numeric,
        unit_price -> Numeric,
        discount_percent -> Numeric,
        tax_rate -> Numeric,
        amount -> Numeric,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}
