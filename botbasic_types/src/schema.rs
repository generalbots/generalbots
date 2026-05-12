pub mod tables {
    diesel::table! {
        bots (id) {
            id -> Uuid,
            name -> Varchar,
            description -> Nullable<Text>,
            llm_provider -> Varchar,
            llm_config -> Jsonb,
            context_provider -> Varchar,
            context_config -> Jsonb,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            is_active -> Nullable<Bool>,
            org_id -> Nullable<Uuid>,
            database_name -> Nullable<Varchar>,
            is_public -> Bool,
        }
    }

    diesel::table! {
        user_sessions (id) {
            id -> Uuid,
            user_id -> Uuid,
            bot_id -> Uuid,
            title -> Varchar,
            context_data -> Jsonb,
            current_tool -> Nullable<Varchar>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        bot_memories (id) {
            id -> Uuid,
            bot_id -> Uuid,
            key -> Varchar,
            value -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        bot_shared_memory (id) {
            id -> Uuid,
            bot_id -> Uuid,
            key -> Varchar,
            value -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        system_automations (id) {
            id -> Uuid,
            bot_id -> Uuid,
            trigger_kind -> Varchar,
            trigger_data -> Jsonb,
            script_path -> Varchar,
            is_active -> Bool,
            schedule -> Nullable<Varchar>,
            param -> Varchar,
            last_triggered -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        session_tool_associations (id) {
            id -> Uuid,
            session_id -> Uuid,
            tool_name -> Varchar,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        message_history (id) {
            id -> Uuid,
            session_id -> Uuid,
            user_id -> Uuid,
            role -> Int4,
            content_encrypted -> Text,
            message_type -> Int4,
            message_index -> Int4,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        workflow_executions (id) {
            id -> Uuid,
            bot_id -> Uuid,
            workflow_name -> Varchar,
            status -> Varchar,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        workflow_events (id) {
            id -> Uuid,
            execution_id -> Uuid,
            event_type -> Varchar,
            payload -> Jsonb,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        calendar_events (id) {
            id -> Uuid,
            org_id -> Uuid,
            bot_id -> Uuid,
            calendar_id -> Uuid,
            owner_id -> Uuid,
            title -> Varchar,
            description -> Nullable<Text>,
            location -> Nullable<Varchar>,
            start_time -> Timestamptz,
            end_time -> Timestamptz,
            all_day -> Bool,
            recurrence_rule -> Nullable<Text>,
            recurrence_id -> Nullable<Uuid>,
            status -> Varchar,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        bot_configuration (id) {
            id -> Uuid,
            bot_id -> Uuid,
            config_key -> Varchar,
            config_value -> Text,
            is_encrypted -> Bool,
            config_type -> Varchar,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }
}

pub use tables::*;
