diesel::table! {
    workspaces (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        settings -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_members (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        role -> Varchar,
        invited_by -> Nullable<Uuid>,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_pages (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        title -> Varchar,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        content -> Jsonb,
        properties -> Jsonb,
        is_template -> Bool,
        template_id -> Nullable<Uuid>,
        is_public -> Bool,
        public_edit -> Bool,
        position -> Int4,
        created_by -> Uuid,
        last_edited_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_page_versions (id) {
        id -> Uuid,
        page_id -> Uuid,
        version_number -> Int4,
        title -> Varchar,
        content -> Jsonb,
        change_summary -> Nullable<Text>,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_page_permissions (id) {
        id -> Uuid,
        page_id -> Uuid,
        user_id -> Nullable<Uuid>,
        role -> Nullable<Varchar>,
        permission -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_comments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        page_id -> Uuid,
        block_id -> Nullable<Uuid>,
        parent_comment_id -> Nullable<Uuid>,
        author_id -> Uuid,
        content -> Text,
        resolved -> Bool,
        resolved_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_comment_reactions (id) {
        id -> Uuid,
        comment_id -> Uuid,
        user_id -> Uuid,
        emoji -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_templates (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        category -> Nullable<Varchar>,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        content -> Jsonb,
        is_system -> Bool,
        usage_count -> Int4,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(workspaces -> organizations (org_id));
diesel::joinable!(workspaces -> bots (bot_id));
diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::joinable!(workspace_pages -> workspaces (workspace_id));
diesel::joinable!(workspace_page_versions -> workspace_pages (page_id));
diesel::joinable!(workspace_page_permissions -> workspace_pages (page_id));
diesel::joinable!(workspace_comments -> workspaces (workspace_id));
diesel::joinable!(workspace_comments -> workspace_pages (page_id));
diesel::joinable!(workspace_comment_reactions -> workspace_comments (comment_id));
diesel::joinable!(workspace_templates -> organizations (org_id));
diesel::joinable!(workspace_templates -> bots (bot_id));
