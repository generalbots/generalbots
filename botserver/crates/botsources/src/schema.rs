use diesel::prelude::*;

table! {
    knowledge_sources (id) {
        id -> Text,
        name -> Text,
        source_type -> Text,
        file_path -> Nullable<Text>,
        url -> Nullable<Text>,
        content_hash -> Text,
        chunk_count -> Integer,
        status -> Text,
        collection -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        indexed_at -> Nullable<Timestamptz>,
    }
}

table! {
    knowledge_chunks (id) {
        id -> Text,
        source_id -> Text,
        chunk_index -> Integer,
        content -> Text,
        token_count -> Integer,
        created_at -> Timestamptz,
    }
}

table! {
    bot_configuration (id) {
        id -> Uuid,
        bot_id -> Uuid,
        config_key -> Text,
        config_value -> Text,
        config_type -> Text,
        is_encrypted -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(
    knowledge_sources,
    knowledge_chunks,
    bot_configuration,
);
