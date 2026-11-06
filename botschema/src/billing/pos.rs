// Moved verbatim from `botproducts/src/schema.rs` (#1497) — column order
// preserved, so the consuming crate's `Queryable` structs are unaffected.
// Verified against the database by `scripts/schema_db_audit.py` (#1519).
diesel::table! {
    product_prices (id) {
        id -> Uuid,
        price_list_id -> Uuid,
        product_id -> Uuid,
        variation_id -> Nullable<Uuid>,
        price -> Numeric,
        min_quantity -> Numeric,
    }
}

// Product and POS tables (canonical, #1497).

diesel::table! {
    product_variations (id) {

    id -> Uuid,
    bot_id -> Uuid,
    product_id -> Uuid,
    #[max_length = 100]
    sku -> Varchar,
    #[max_length = 255]
    name -> Varchar,
    attributes -> Jsonb,
    price -> Numeric,
    cost_price -> Nullable<Numeric>,
    #[max_length = 100]
    barcode -> Nullable<Varchar>,
    weight -> Nullable<Numeric>,
    is_active -> Bool,
    created_at -> Timestamptz,
    branch_id -> Uuid,
    }
}

diesel::table! {
    product_stock (id) {

    id -> Uuid,
    bot_id -> Uuid,
    product_id -> Uuid,
    variation_id -> Nullable<Uuid>,
    branch_id -> Uuid,
    quantity -> Numeric,
    reserved -> Numeric,
    reorder_point -> Nullable<Numeric>,
    updated_at -> Timestamptz,
    }
}

diesel::table! {
    product_price_lists (id) {

    id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 100]
    name -> Varchar,
    #[max_length = 3]
    currency -> Bpchar,
    is_default -> Bool,
    valid_from -> Nullable<Date>,
    valid_until -> Nullable<Date>,
    created_at -> Timestamptz,
    branch_id -> Uuid,
    }
}

diesel::table! {
    product_promotions (id) {

    id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 255]
    name -> Varchar,
    #[max_length = 20]
    discount_type -> Varchar,
    discount_value -> Numeric,
    product_ids -> Nullable<Jsonb>,
    min_purchase -> Nullable<Numeric>,
    valid_from -> Date,
    valid_until -> Date,
    is_active -> Bool,
    created_at -> Timestamptz,
    branch_id -> Uuid,
    }
}

diesel::table! {
    pos_sessions (id) {

    id -> Uuid,
    bot_id -> Uuid,
    branch_id -> Uuid,
    operator_id -> Uuid,
    started_at -> Timestamptz,
    closed_at -> Nullable<Timestamptz>,
    opening_amount -> Numeric,
    closing_amount -> Nullable<Numeric>,
    #[max_length = 20]
    status -> Varchar,
    }
}

diesel::table! {
    pos_sales (id) {

    id -> Uuid,
    bot_id -> Uuid,
    session_id -> Uuid,
    sale_number -> Int4,
    items -> Jsonb,
    subtotal -> Numeric,
    discount -> Nullable<Numeric>,
    tax -> Nullable<Numeric>,
    total -> Numeric,
    #[max_length = 50]
    payment_method -> Varchar,
    nfse_id -> Nullable<Uuid>,
    created_at -> Timestamptz,
    branch_id -> Uuid,
    }
}
