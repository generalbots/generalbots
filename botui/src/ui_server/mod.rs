pub mod assets;
pub mod cloud;
pub mod constants;
pub mod login;
pub mod proxy;
pub mod suite;
pub mod suite_ops;
pub mod ws;

pub use self::assets::serve_favicon;
#[cfg(feature = "embed-ui")]
pub use self::assets::{handle_embedded_asset, handle_embedded_root_asset};
pub use self::cloud::*;
pub use self::constants::get_ui_root;
pub use self::login::{serve_login_index, serve_login_signup, serve_login_js, serve_login_images};
pub use self::proxy::*;
pub use self::suite::*;
pub use self::suite_ops::*;
pub use self::ws::*;

use axum::{
    routing::{any, get},
    Router,
};
#[cfg(not(feature = "embed-ui"))]
use log::info;
use std::path::Path;
#[cfg(not(feature = "embed-ui"))]
use tower_http::services::{ServeDir, ServeFile};

use crate::shared::AppState;
use crate::ui_server::constants::ROOT_FILES;
#[cfg(not(feature = "embed-ui"))]
use crate::ui_server::constants::SUITE_DIRS;

fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(api_health))
        .route("/client-error", axum::routing::post(handle_client_error))
        // Backend API WebSocket endpoints (/api/terminal/ws, /api/browser/.../ws)
        // need a real WS proxy — the reqwest-based fallback cannot tunnel
        // upgrades. These routes must come BEFORE the fallback.
        .route("/terminal/ws", get(api_ws_proxy))
        .route("/browser/session/:id/agent/ws", get(api_ws_proxy))
        .fallback(any(proxy_api))
}

fn create_ws_router() -> Router<AppState> {
    Router::new()
        .route("/task-progress", get(ws_task_progress_proxy))
        .route("/task-progress/:task_id", get(ws_task_progress_proxy))
        .route("/autotask", get(ws_task_progress_proxy))
        .fallback(any(ws_proxy))
}

fn create_apps_router() -> Router<AppState> {
    Router::new().fallback(any(proxy_api))
}

fn create_ui_router() -> Router<AppState> {
    Router::new().fallback(any(proxy_api))
}

fn add_static_routes(router: Router<AppState>, _suite_path: &Path) -> Router<AppState> {
    #[cfg(feature = "embed-ui")]
    {
        let mut r = router.route("/suite/:dir/*path", get(handle_embedded_asset));

        for file in ROOT_FILES {
            r = r.route(&format!("/suite/{}", file), get(handle_embedded_root_asset));
        }
        r
    }
    #[cfg(not(feature = "embed-ui"))]
    {
        let mut r = router;
        for dir in SUITE_DIRS {
            let path = _suite_path.join(dir);
            info!("Adding route for /suite/{} -> {:?}", dir, path);
            r = r.nest_service(&format!("/suite/{dir}"), ServeDir::new(path.clone()));
        }

        for file in ROOT_FILES {
            let path = _suite_path.join(file);
            r = r.nest_service(&format!("/suite/{}", file), ServeFile::new(path));
        }
        r
    }
}

pub fn configure_router() -> Router {
    let suite_path = get_ui_root().join("suite");
    let state = AppState::new();

    let mut router = Router::new()
        .route("/health", get(health))
        .route("/favicon.ico", get(serve_favicon))
        .nest("/api", create_api_router())
        .nest("/ui", create_ui_router())
        .nest("/ws", create_ws_router())
        .nest("/apps", create_apps_router())
        .route("/", get(index))
        .route("/minimal", get(serve_minimal))
        .route("/suite", get(serve_suite));

    // Server-generated HTMX fragments live in botserver, not as static
    // files. Register them with an explicit static dir segment so they win
    // over the /suite/{dir} ServeDir nests — matchit ranks static segments
    // above the `:dir` param, so the generic param form never matched and
    // the ServeDir returned 404.
    #[cfg(not(feature = "embed-ui"))]
    for dir in SUITE_DIRS {
        router = router
            .route(&format!("/suite/{dir}/fragments/*path"), any(proxy_api))
            .route(&format!("/suite/{dir}/modals/*path"), any(proxy_api))
            .route(&format!("/suite/{dir}/forms/*path"), any(proxy_api))
            .route(&format!("/suite/{dir}/partials/*path"), any(proxy_api));
    }
    #[cfg(feature = "embed-ui")]
    {
        router = router
            .route("/suite/:dir/fragments/*path", any(proxy_api))
            .route("/suite/:dir/modals/*path", any(proxy_api))
            .route("/suite/:dir/forms/*path", any(proxy_api))
            .route("/suite/:dir/partials/*path", any(proxy_api));
    }

    router = add_static_routes(router, &suite_path);

    router.fallback(get(index)).with_state(state)
}

pub fn configure_cloud_router() -> Router {
    let state = AppState::new();

    Router::new()
        .route("/health", get(health))
        // Root → store (no landing page)
        .route("/", get(redirect_to_store))
        // Legacy /cloud/* paths for static assets
        .route("/cloud", get(redirect_to_store))
        .route("/cloud/", get(redirect_to_store))
        .route("/cloud/login", get(redirect_to_login))
        .route("/cloud/login/", get(redirect_to_login))
        .route("/cloud/signup", get(redirect_to_signup))
        .route("/cloud/signup/", get(redirect_to_signup))
        .route("/cloud/*path", get(serve_cloud))
        // API, UI, WS nests
        .nest("/api", create_api_router())
        .nest("/ui", create_ui_router())
        .nest("/ws", create_ws_router())
        // All root-level paths resolve to cloud pages
        // e.g. /store → store.html, /plans → plans.html, /js/cloud.js → cloud/js/cloud.js
        .fallback(get(serve_cloud_fallback))
        .with_state(state)
}

pub fn configure_login_router() -> Router {
    let state = AppState::new();

    Router::new()
        .route("/health", get(health))
        .route("/", get(serve_login_index))
        .route("/signup", get(serve_login_signup))
        .route("/js/*path", get(serve_login_js))
        .route("/images/*path", get(serve_login_images))
        .nest("/api", create_api_router())
        .nest("/ws", create_ws_router())
        .fallback(get(serve_login_index))
        .with_state(state)
}
