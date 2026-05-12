diesel::table! {
    products (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
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
    }
}

diesel::table! {
    services (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
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
    }
}

diesel::table! {
    product_categories (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        parent_id -> Nullable<Uuid>,
        slug -> Nullable<Varchar>,
        image_url -> Nullable<Text>,
        sort_order -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    price_lists (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        currency -> Varchar,
        is_default -> Bool,
        valid_from -> Nullable<Date>,
        valid_until -> Nullable<Date>,
        customer_group -> Nullable<Varchar>,
        discount_percent -> Numeric,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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

diesel::table! {
    inventory_movements (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        product_id -> Uuid,
        movement_type -> Varchar,
        quantity -> Int4,
        reference_type -> Nullable<Varchar>,
        reference_id -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    product_variants (id) {
        id -> Uuid,
        product_id -> Uuid,
        sku -> Nullable<Varchar>,
        name -> Varchar,
        price_adjustment -> Numeric,
        stock_quantity -> Int4,
        attributes -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        global_trade_number -> Nullable<Varchar>,
        net_weight -> Nullable<Numeric>,
        gross_weight -> Nullable<Numeric>,
        width -> Nullable<Numeric>,
        height -> Nullable<Numeric>,
        length -> Nullable<Numeric>,
        color -> Nullable<Varchar>,
        size -> Nullable<Varchar>,
        images -> Nullable<Jsonb>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
    }
}

diesel::table! {
    bots (id) {
        id -> Uuid,
    }
}

diesel::joinable!(products -> organizations (org_id));
diesel::joinable!(products -> bots (bot_id));
diesel::joinable!(services -> organizations (org_id));
diesel::joinable!(services -> bots (bot_id));
diesel::joinable!(product_categories -> organizations (org_id));
diesel::joinable!(product_categories -> bots (bot_id));
diesel::joinable!(price_lists -> organizations (org_id));
diesel::joinable!(price_lists -> bots (bot_id));
diesel::joinable!(price_list_items -> price_lists (price_list_id));
diesel::joinable!(price_list_items -> products (product_id));
diesel::joinable!(price_list_items -> services (service_id));
diesel::joinable!(inventory_movements -> organizations (org_id));
diesel::joinable!(inventory_movements -> bots (bot_id));
diesel::joinable!(inventory_movements -> products (product_id));
diesel::joinable!(product_variants -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(
    products,
    services,
    product_categories,
    price_lists,
    price_list_items,
    inventory_movements,
    product_variants,
    organizations,
    bots,
);
