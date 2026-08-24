diesel::table! {
    skill_packages (id) {
        id -> Uuid,
        slug -> Text,
        name -> Text,
        description -> Nullable<Text>,
        latest_version -> Nullable<Text>,
        publisher_org_id -> Nullable<Uuid>,
        publisher_name -> Nullable<Text>,
        visibility -> Text,
        review_status -> Text,
        downloads -> BigInt,
        icon_glyph -> Nullable<Text>,
        tags -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    skill_versions (id) {
        id -> Uuid,
        package_id -> Uuid,
        version -> Text,
        manifest -> Jsonb,
        object_key -> Text,
        changelog -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    skill_installs (id) {
        id -> Uuid,
        package_id -> Uuid,
        version_id -> Uuid,
        org_id -> Nullable<Uuid>,
        branch_id -> Nullable<Uuid>,
        bot_id -> Uuid,
        installed_by -> Nullable<Uuid>,
        status -> Text,
        installed_at -> Timestamptz,
    }
}
