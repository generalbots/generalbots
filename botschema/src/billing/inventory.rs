diesel::table! {
    inventory_movements (id) {

    id -> Uuid,
    org_id -> Uuid,
    bot_id -> Uuid,
    product_id -> Uuid,
    #[max_length = 50]
    movement_type -> Varchar,
    quantity -> Int4,
    #[max_length = 50]
    reference_type -> Nullable<Varchar>,
    reference_id -> Nullable<Uuid>,
    notes -> Nullable<Text>,
    created_by -> Nullable<Uuid>,
    created_at -> Timestamptz,
    branch_id -> Uuid,
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
