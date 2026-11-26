//! Password recovery from this computer only (desktop edition).
//!
//! A second listener, bound to the loopback interface on a port the OS picks,
//! serves one route: `POST /local-recovery/reset`. It is not part of the main
//! router, so no public route list names it, and the network cannot reach it.
//! A request must also carry the one-use secret from `local-recovery.json`,
//! which only this Windows account can read: the desktop shell reads it and
//! calls the route; a web page cannot.

mod handler;
mod secret;

use axum::routing::post;
use axum::Router;
use botcore::shared::state::AppState;
use crate::security::headers::{with_security_headers, SecurityHeadersConfig};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

/// Starts the recovery listener in the desktop edition; elsewhere it does nothing.
pub fn spawn(app: Arc<AppState>) {
    if !botcoredirectory::auth_routes::is_desktop_edition() {
        return;
    }
    tokio::spawn(async move {
        if let Err(e) = serve(app).await {
            log::warn!("Local password recovery is unavailable: {e}");
        }
    });
}

async fn serve(app: Arc<AppState>) -> std::io::Result<()> {
    let path = secret::secret_path();
    // A file left by an earlier run names a port nobody may be serving now.
    if let Err(e) = std::fs::remove_file(&path)
    {
        log::debug!("stale recovery secret removal failed: {e}");
    }
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let port = listener.local_addr()?.port();
    let store = Arc::new(secret::SecretStore::create(path, port)?);
    // IMP-09 applies to every server, even this loopback-only one: the Tauri
    // shell calls it over plain HTTP, so HSTS is dropped to avoid pinning
    // 127.0.0.1 to HTTPS in the WebView.
    let router = with_security_headers(
        Router::new()
            .route("/local-recovery/reset", post(handler::reset))
            .with_state(handler::RecoveryState {
                app,
                store: Arc::clone(&store),
            }),
        SecurityHeadersConfig::from_env().disable_hsts(),
    );
    log::info!("Local password recovery listens on the loopback interface only");
    let served = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await;
    store.remove_file();
    served
}
