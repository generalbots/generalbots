use crate::handlers::*;
use crate::state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

const API_SOURCES_KB: &str = "/api/sources/kb";
const API_SOURCES_MCP: &str = "/api/sources/mcp";
const API_UI_SOURCES: &str = "/api/ui/sources";

pub fn configure_sources_api_routes() -> Router<Arc<AppState>> {
    let kb_routes = Router::new()
        .route("/", get(handle_list_sources).post(handle_upload_document))
        .route("/query", axum::routing::post(handle_query_knowledge_base))
        .route("/reindex", axum::routing::post(handle_reindex_sources))
        .route("/stats", get(handle_get_stats))
        .route("/{id}", get(handle_get_source).delete(handle_delete_source));

    let mcp_routes = Router::new()
        .route(
            "/",
            get(handle_list_mcp_servers_json).post(handle_add_mcp_server),
        )
        .route("/scan", axum::routing::post(handle_scan_mcp_directory))
        .route("/examples", get(handle_get_mcp_examples))
        .route("/tools", get(handle_list_all_tools))
        .route(
            "/{name}",
            get(handle_get_mcp_server)
                .put(handle_update_mcp_server)
                .delete(handle_delete_mcp_server),
        )
        .route("/{name}/test", axum::routing::post(handle_test_mcp_server))
        .route("/{name}/tools", get(handle_list_mcp_server_tools))
        .route("/{name}/enable", axum::routing::post(handle_enable_mcp_server))
        .route(
            "/{name}/disable",
            axum::routing::post(handle_disable_mcp_server),
        );

    let ui_sources_routes = Router::new()
        .route("/repositories", get(handle_list_repositories))
        .route(
            "/repositories/{id}/connect",
            axum::routing::post(handle_connect_repository),
        )
        .route(
            "/repositories/{id}/disconnect",
            axum::routing::post(handle_disconnect_repository),
        )
        .route("/apps", get(handle_list_apps))
        .route("/prompts", get(handle_prompts))
        .route("/templates", get(handle_templates))
        .route("/news", get(handle_news))
        .route("/mcp-servers", get(handle_mcp_servers))
        .route("/llm-tools", get(handle_llm_tools))
        .route("/models", get(handle_models))
        .route("/search", get(handle_search))
        .route("/mentions", get(handle_mentions_autocomplete))
        .route("/api-keys", get(handle_list_api_keys))
        .route("/api-keys", axum::routing::post(handle_add_api_key))
        .route("/api-keys/{id}", axum::routing::delete(handle_delete_api_key));

    Router::new()
        .nest(API_SOURCES_KB, kb_routes)
        .nest(API_SOURCES_MCP, mcp_routes)
        .nest(API_UI_SOURCES, ui_sources_routes)
        .merge(crate::ui::configure_sources_ui_routes())
}
