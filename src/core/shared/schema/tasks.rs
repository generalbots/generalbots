diesel::table! {
    tasks (id) {
        id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        status -> Text,
        priority -> Text,
        assignee_id -> Nullable<Uuid>,
        reporter_id -> Nullable<Uuid>,
        project_id -> Nullable<Uuid>,
        due_date -> Nullable<Timestamptz>,
        tags -> Array<Text>,
        dependencies -> Array<Uuid>,
        estimated_hours -> Nullable<Float8>,
        actual_hours -> Nullable<Float8>,
        progress -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

pub use self::tasks::*;
