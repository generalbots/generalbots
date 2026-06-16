diesel::table! {
    crm_contacts (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        email -> Varchar,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_subscriptions (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        customer_name -> Varchar,
        customer_email -> Nullable<Varchar>,
        plan_id -> Varchar,
        status -> Varchar,
        amount -> Numeric,
        currency -> Varchar,
        billing_period -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    billing_recurring (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        customer_id -> Nullable<Uuid>,
        customer_name -> Varchar,
        customer_email -> Nullable<Varchar>,
        status -> Varchar,
        frequency -> Varchar,
        interval_count -> Int4,
        amount -> Numeric,
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
    }
}

diesel::table! {
    billing_payments (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        invoice_id -> Nullable<Uuid>,
        payment_number -> Varchar,
        amount -> Numeric,
        currency -> Varchar,
        payment_method -> Varchar,
        status -> Varchar,
        payer_name -> Nullable<Varchar>,
        paid_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    cloud_workspaces (id) {
        id -> Uuid,
        org_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        icon -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_resources (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        org_id -> Uuid,
        store_item_id -> Varchar,
        name -> Varchar,
        resource_type -> Varchar,
        status -> Varchar,
        config -> Nullable<Jsonb>,
        provisioned_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
