use diesel::prelude::*;

diesel::table! {
    channel_accounts (id) {
        id -> Uuid,
        organization_id -> Uuid,
        name -> Text,
        channel_type -> Text,
        credentials -> Jsonb,
        settings -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(channel_accounts -> crate::core::shared::schema::core::organizations (organization_id));

diesel::allow_tables_to_appear_in_same_query!(
    channel_accounts,
);
