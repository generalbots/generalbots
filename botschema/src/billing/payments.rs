diesel::table! {
    billing_payments (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    invoice_id -> Nullable<Uuid>,
    #[max_length = 50]
    payment_number -> Varchar,
    amount -> Numeric,
    #[max_length = 3]
    currency -> Varchar,
    #[max_length = 50]
    payment_method -> Varchar,
    #[max_length = 255]
    payment_reference -> Nullable<Varchar>,
    #[max_length = 50]
    status -> Varchar,
    #[max_length = 255]
    payer_name -> Nullable<Varchar>,
    #[max_length = 255]
    payer_email -> Nullable<Varchar>,
    notes -> Nullable<Text>,
    paid_at -> Timestamptz,
    refunded_at -> Nullable<Timestamptz>,
    refund_amount -> Nullable<Numeric>,
    created_at -> Timestamptz,
    branch_id -> Uuid,
    }
}


diesel::table! {
    billing_quotes (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 50]
    quote_number -> Varchar,
    customer_id -> Nullable<Uuid>,
    #[max_length = 255]
    customer_name -> Varchar,
    #[max_length = 255]
    customer_email -> Nullable<Varchar>,
    customer_address -> Nullable<Text>,
    #[max_length = 50]
    status -> Varchar,
    issue_date -> Date,
    valid_until -> Date,
    subtotal -> Numeric,
    tax_rate -> Numeric,
    tax_amount -> Numeric,
    discount_percent -> Numeric,
    discount_amount -> Numeric,
    total -> Numeric,
    #[max_length = 3]
    currency -> Varchar,
    notes -> Nullable<Text>,
    terms -> Nullable<Text>,
    accepted_at -> Nullable<Timestamptz>,
    rejected_at -> Nullable<Timestamptz>,
    converted_invoice_id -> Nullable<Uuid>,
    sent_at -> Nullable<Timestamptz>,
    created_at -> Timestamptz,
    updated_at -> Timestamptz,
    branch_id -> Uuid,
    }
}


diesel::table! {
    billing_quote_items (id) {
        id -> Uuid,
        quote_id -> Uuid,
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

diesel::table! {
    billing_recurring (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    customer_id -> Nullable<Uuid>,
    #[max_length = 255]
    customer_name -> Varchar,
    #[max_length = 255]
    customer_email -> Nullable<Varchar>,
    #[max_length = 50]
    status -> Varchar,
    #[max_length = 50]
    frequency -> Varchar,
    interval_count -> Int4,
    amount -> Numeric,
    #[max_length = 3]
    currency -> Varchar,
    description -> Nullable<Text>,
    next_invoice_date -> Date,
    last_invoice_date -> Nullable<Date>,
    last_invoice_id -> Nullable<Uuid>,
    start_date -> Date,
    end_date -> Nullable<Date>,
    invoices_generated -> Int4,
    created_at -> Timestamptz,
    updated_at -> Timestamptz,
    branch_id -> Uuid,
    }
}


diesel::table! {
    billing_tax_rates (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 100]
    name -> Varchar,
    rate -> Numeric,
    description -> Nullable<Text>,
    #[max_length = 100]
    region -> Nullable<Varchar>,
    is_default -> Bool,
    is_active -> Bool,
    created_at -> Timestamptz,
    branch_id -> Uuid,
    }
}
