use axum::{Router, routing::{get, post}};
use std::sync::Arc;
use botcore::shared::state::AppState;
use botcore::urls::ApiUrls;

pub fn setup_api_routes() -> Router<Arc<AppState>> {
    let mut api_router = Router::new()
        .route("/health", get(crate::health_check_simple))
        .route(ApiUrls::HEALTH, get(crate::health_check))
        .route("/api/config/reload", post(botcore::config_reload::reload_config))
        .route("/api/product", get(super::product_handlers::get_product_config))
        .route("/api/manifest", get(super::product_handlers::get_workspace_manifest))
        .route("/api/client-errors", post(crate::receive_client_errors))
        .route("/api/bot/config", get(crate::core::bot::get_bot_config))
        .route("/api/bots/:bot_name/access", get(crate::core::bot::check_access_handler))
        .route(ApiUrls::SESSIONS, post(crate::core::session::create_session))
        .route(ApiUrls::SESSION_START, post(crate::core::session::start_session))
        .route(ApiUrls::WS, get(crate::core::bot::websocket_handler))
        .route("/ws/:bot_name", get(crate::core::bot::websocket_handler_with_bot));

    #[cfg(feature = "drive")]
    {
        use axum::routing::{get as axum_get, post as axum_post};
        api_router = api_router
            .route("/api/files/list", axum_get(crate::drive::drive_handlers::list_files))
            .route("/api/files/buckets", axum_get(crate::drive::drive_handlers::list_buckets))
            .route("/api/files/quota", axum_get(crate::drive::drive_handlers::quota))
            .route("/api/files/recent", axum_get(crate::drive::drive_handlers::recent_files))
            .route("/api/files/search", axum_get(crate::drive::drive_handlers::search_files))
            .route("/api/files/write", axum_post(crate::drive::drive_handlers::upload_file_to_drive))
            .route("/api/files/download", axum_post(crate::drive::drive_handlers::download_file))
            .route("/api/files/download-binary", axum_post(crate::drive::drive_handlers::download_file_binary))
            .route("/api/files/delete", axum_post(crate::drive::drive_handlers::delete_file))
            .route("/api/files/createFolder", axum_post(crate::drive::drive_handlers::create_folder))
            .route("/api/files/copy", axum_post(crate::drive::drive_handlers::copy_file))
            .route("/api/files/move", axum_post(crate::drive::drive_handlers::move_file))
            .route("/api/files/open", axum_post(crate::drive::drive_handlers::open_file))
            .route("/api/files/ai/chat", axum_post(crate::drive::drive_handlers::ai_chat_handler))
            .route("/api/files/favorite", axum_get(crate::drive::drive_handlers::list_favorites))
            .route("/api/files/shared", axum_get(crate::drive::drive_handlers::list_shared));
    }

    api_router = api_router
        .route(ApiUrls::AUTH, get(super::anonymous_auth::anonymous_auth_handler));

    api_router
}

pub fn add_base_api_routes(api_router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    use super::org_handlers::*;

    api_router
        .nest("/api/directory", crate::directory::router::configure())
        .nest("/api/auth", crate::directory::auth_routes::configure())
        .route("/api/organizations/current", get(handle_get_organization).put(handle_update_organization).post(handle_update_organization).delete(handle_delete_organization))
        .route("/api/organizations/current/settings", get(handle_get_org_settings))
        .route("/api/organizations/current/stats", get(handle_get_org_stats))
        .route("/api/organizations/current/contact", post(handle_update_organization_contact))
        .route("/api/organizations/current/branding", post(handle_update_organization_branding))
        .route("/api/organizations/current/audit", get(handle_get_org_audit))
        .route("/api/organizations/current/export", get(handle_export_org_data))
        .route("/api/admin/migrate/office365", post(handle_office365_migration))
}
