pub mod safety_constraints {
    diesel::table! {
        safety_constraints (id) {
            id -> Text,
            bot_id -> Text,
            name -> Text,
            constraint_type -> Text,
            description -> Nullable<Text>,
            expression -> Nullable<Text>,
            threshold -> Nullable<Text>,
            severity -> Text,
            enabled -> Bool,
            applies_to -> Nullable<Text>,
        }
    }
}

pub mod audit_log {
    diesel::table! {
        audit_log (id) {
            id -> Text,
            timestamp -> Text,
            event_type -> Text,
            actor_type -> Text,
            actor_id -> Text,
            action -> Text,
            target_type -> Text,
            target_id -> Text,
            outcome_success -> Bool,
            details -> Text,
            session_id -> Text,
            bot_id -> Text,
            task_id -> Nullable<Text>,
            step_id -> Nullable<Text>,
            risk_level -> Text,
        }
    }
}

pub mod intent_classifications {
    diesel::table! {
        intent_classifications (id) {
            id -> Uuid,
            bot_id -> Uuid,
            session_id -> Uuid,
            original_text -> Text,
            intent_type -> Text,
            confidence -> Float8,
            entities -> Text,
            created_at -> Timestamptz,
        }
    }
}

pub mod compiled_intents {
    diesel::table! {
        compiled_intents (id) {
            id -> Text,
            bot_id -> Text,
            session_id -> Text,
            original_intent -> Text,
            basic_program -> Text,
            confidence -> Float8,
            compiled_at -> Timestamptz,
            data -> Text,
        }
    }
}

pub mod tasks {
    diesel::table! {
        tasks (id) {
            id -> Uuid,
            title -> Text,
            description -> Text,
            status -> Text,
            priority -> Text,
            created_at -> Timestamptz,
        }
    }
}

pub mod designer_changes {
    diesel::table! {
        designer_changes (id) {
            id -> Uuid,
            bot_id -> Uuid,
            change_type -> Text,
            description -> Text,
            file_path -> Text,
            original_content -> Text,
            new_content -> Text,
            created_at -> Timestamptz,
        }
    }
}

pub mod designer_pending_changes {
    diesel::table! {
        designer_pending_changes (id) {
            id -> Text,
            bot_id -> Uuid,
            analysis_json -> Text,
            expires_at -> Timestamptz,
        }
    }
}
