// use crate::core::shared::schema::core::{organizations, bots};

diesel::table! {
    learn_courses (id) {
        id -> Uuid,
        organization_id -> Nullable<Uuid>,
        title -> Text,
        description -> Nullable<Text>,
        category -> Text,
        difficulty -> Text,
        duration_minutes -> Int4,
        thumbnail_url -> Nullable<Text>,
        is_mandatory -> Bool,
        due_days -> Nullable<Int4>,
        is_published -> Bool,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    learn_lessons (id) {
        id -> Uuid,
        course_id -> Uuid,
        title -> Text,
        content -> Nullable<Text>,
        content_type -> Text,
        lesson_order -> Int4,
        duration_minutes -> Int4,
        video_url -> Nullable<Text>,
        attachments -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    learn_quizzes (id) {
        id -> Uuid,
        lesson_id -> Nullable<Uuid>,
        course_id -> Uuid,
        title -> Text,
        passing_score -> Int4,
        time_limit_minutes -> Nullable<Int4>,
        max_attempts -> Nullable<Int4>,
        questions -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    learn_user_progress (id) {
        id -> Uuid,
        user_id -> Uuid,
        course_id -> Uuid,
        lesson_id -> Nullable<Uuid>,
        status -> Text,
        quiz_score -> Nullable<Int4>,
        quiz_attempts -> Int4,
        time_spent_minutes -> Int4,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        last_accessed_at -> Timestamptz,
    }
}

diesel::table! {
    learn_course_assignments (id) {
        id -> Uuid,
        course_id -> Uuid,
        user_id -> Uuid,
        assigned_by -> Nullable<Uuid>,
        due_date -> Nullable<Timestamptz>,
        is_mandatory -> Bool,
        assigned_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        reminder_sent -> Bool,
        reminder_sent_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    learn_certificates (id) {
        id -> Uuid,
        user_id -> Uuid,
        course_id -> Uuid,
        issued_at -> Timestamptz,
        score -> Int4,
        certificate_url -> Nullable<Text>,
        verification_code -> Text,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    learn_categories (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        icon -> Nullable<Text>,
        color -> Nullable<Text>,
        parent_id -> Nullable<Uuid>,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    learn_courses,
    learn_lessons,
    learn_quizzes,
    learn_user_progress,
    learn_course_assignments,
    learn_certificates,
    learn_categories,
);
