//! Per-user AI accounts (#1654). Credentials are never stored in this table.


diesel::table! {
    user_llm_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        provider -> Text,
        status -> Text,
        vm_name -> Nullable<Text>,
        account_email -> Nullable<Text>,
        plan -> Nullable<Text>,
        vault_path -> Nullable<Text>,
        host_id -> Nullable<Text>,
        client_id -> Nullable<Text>,
        scopes -> Nullable<Text>,
        linked_at -> Nullable<Timestamptz>,
        last_verified_at -> Nullable<Timestamptz>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
