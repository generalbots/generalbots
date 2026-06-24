// @generated automatically by Diesel CLI.

diesel::table! {
    projects (id) {
        id -> Uuid,
        #[max_length = 255]
        org -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 50]
        project_type -> Varchar,
        #[max_length = 50]
        deploy_target -> Varchar,
        repo_url -> Nullable<Text>,
        deploy_url -> Nullable<Text>,
        #[max_length = 255]
        container_name -> Nullable<Varchar>,
        #[max_length = 255]
        custom_domain -> Nullable<Varchar>,
        #[max_length = 50]
        environment -> Varchar,
        #[max_length = 50]
        status -> Varchar,
        #[max_length = 100]
        framework -> Nullable<Varchar>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
