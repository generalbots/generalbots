use std::net::SocketAddr;
use axum::Router;
use log::{error, info};
use super::shutdown::shutdown_signal;

pub(crate) async fn listen(app: Router, port: u16) -> std::io::Result<()> {
    let stack = botcore::shared::utils::get_stack_path();
    let cert_dir = std::path::PathBuf::from(format!("{}/conf/system/certificates", stack));
    let cert_path = cert_dir.join("api/server.crt");
    let key_path = cert_dir.join("api/server.key");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let disable_tls = std::env::var("BOTSERVER_DISABLE_TLS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if !disable_tls && cert_path.exists() && key_path.exists() {
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(std::io::Error::other)?;

        info!("HTTPS server listening on {} with TLS", addr);

        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();

        tokio::spawn(async move {
            shutdown_signal().await;
            info!("Shutting down HTTPS server - draining active connections (10s timeout)...");
            handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
            info!("HTTPS graceful shutdown initiated, waiting for connections to drain...");
        });

        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .map_err(|e| {
                error!("HTTPS server failed on {}: {}", addr, e);
                std::io::Error::other(e)
            })?;
    } else {
        if disable_tls {
            info!("TLS disabled via BOTSERVER_DISABLE_TLS environment variable");
        } else {
            info!("TLS certificates not found, using HTTP");
        }

        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!(
                    "Failed to bind to {}: {} - is another instance running?",
                    addr, e
                );
                return Err(e);
            }
        };
        info!("HTTP server listening on {}", addr);
        info!("Server ready - shutdown via SIGINT (Ctrl+C) or SIGTERM (systemctl stop)");
        let result = axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await;
        match &result {
            Ok(()) => info!("HTTP server shut down gracefully"),
            Err(e) => error!("HTTP server shutdown with error: {}", e),
        }
        result.map_err(std::io::Error::other)?;
    }

    Ok(())
}
