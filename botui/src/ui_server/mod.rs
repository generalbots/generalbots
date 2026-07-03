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
pub use self::assets::{handle_embedded_asset, handle_embedded_root_asset, handle_auth_asset};
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
