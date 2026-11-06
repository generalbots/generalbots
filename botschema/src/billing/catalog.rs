diesel::table! {
    // Physical order: `6.0.10-01-products` creates the core columns, later
    // ALTERs append the tax/inventory/e-commerce columns, `6.5.23` appends
    // `branch_id`. `price`/`currency`/`stock_quantity`/`attributes`/`is_active`
    // are NOT NULL; `sku` is nullable.
    products (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        bot_id -> Nullable<Uuid>,
        sku -> Nullable<Varchar>,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        product_type -> Varchar,
        price -> Numeric,
        cost -> Nullable<Numeric>,
        currency -> Varchar,
        tax_rate -> Numeric,
        unit -> Varchar,
        stock_quantity -> Int4,
        low_stock_threshold -> Int4,
        is_active -> Bool,
        images -> Jsonb,
        attributes -> Jsonb,
        weight -> Nullable<Numeric>,
        dimensions -> Nullable<Jsonb>,
        barcode -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        tax_code -> Nullable<Varchar>,
        tax_class -> Nullable<Varchar>,
        global_trade_number -> Nullable<Varchar>,
        net_weight -> Nullable<Numeric>,
        gross_weight -> Nullable<Numeric>,
        width -> Nullable<Numeric>,
        height -> Nullable<Numeric>,
        length -> Nullable<Numeric>,
        brand -> Nullable<Varchar>,
        model -> Nullable<Varchar>,
        color -> Nullable<Varchar>,
        size -> Nullable<Varchar>,
        material -> Nullable<Varchar>,
        gender -> Nullable<Varchar>,
        expiration_date -> Nullable<Date>,
        min_stock -> Nullable<Int4>,
        max_stock -> Nullable<Int4>,
        reorder_point -> Nullable<Int4>,
        shipping_cost -> Nullable<Numeric>,
        external_id -> Nullable<Varchar>,
        external_metadata -> Nullable<Jsonb>,
        slug -> Nullable<Varchar>,
        meta_description -> Nullable<Text>,
        tags -> Nullable<Array<Nullable<Text>>>,
        branch_id -> Uuid,
        category_id -> Nullable<Uuid>,
        is_public -> Bool,
    }
}

// `services` is created by `6.0.10-01-products`; `6.5.23-branch-scope-cleanup`
// adds `branch_id` (NOT NULL) and `6.5.40-services-tax` adds `tax_rate`.
diesel::table! {
    services (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        branch_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        service_type -> Varchar,
        hourly_rate -> Nullable<Numeric>,
        fixed_price -> Nullable<Numeric>,
        currency -> Varchar,
        duration_minutes -> Nullable<Int4>,
        is_active -> Bool,
        attributes -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        tax_rate -> Numeric,
    }
}

// `product_categories` is created by `6.0.10-01-products`; `6.5.15.1-consolidated`
// and `6.5.98-product-categories-display-order` add `display_order` + `updated_at`
// and make `org_id`/`bot_id` nullable (the botproducts seed omits them), while
// `6.5.23-branch-scope-cleanup` adds `branch_id` as NOT NULL.
diesel::table! {
    product_categories (id) {
        id -> Uuid,
        org_id -> Nullable<Uuid>,
        bot_id -> Nullable<Uuid>,
        branch_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        parent_id -> Nullable<Uuid>,
        slug -> Nullable<Varchar>,
        image_url -> Nullable<Text>,
        sort_order -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
        display_order -> Nullable<Int4>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    price_lists (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    #[max_length = 255]
    name -> Varchar,
    description -> Nullable<Text>,
    #[max_length = 3]
    currency -> Varchar,
    is_default -> Bool,
    valid_from -> Nullable<Date>,
    valid_until -> Nullable<Date>,
    #[max_length = 100]
    customer_group -> Nullable<Varchar>,
    discount_percent -> Numeric,
    is_active -> Bool,
    created_at -> Timestamptz,
    updated_at -> Timestamptz,
    branch_id -> Uuid,
    }
}


diesel::table! {
    price_list_items (id) {
        id -> Uuid,
        price_list_id -> Uuid,
        product_id -> Nullable<Uuid>,
        service_id -> Nullable<Uuid>,
        price -> Numeric,
        min_quantity -> Int4,
        created_at -> Timestamptz,
    }
}
