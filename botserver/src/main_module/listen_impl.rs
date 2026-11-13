use super::shutdown::shutdown_signal;
use axum::Router;
use log::{error, info};
use std::net::SocketAddr;

pub(crate) async fn listen(app: Router, port: u16, no_security: bool) -> std::io::Result<()> {
    let stack = botcore::shared::utils::get_stack_path();
    let cert_dir = std::path::PathBuf::from(format!("{}/conf/system/certificates", stack));
    let cert_path = cert_dir.join("api/server.crt");
    let key_path = cert_dir.join("api/server.key");

    // In single-user desktop/no_security mode the API must not be reachable
    // from the network. Bind to loopback only and refuse explicit non-loopback
    // host configuration to avoid exposing an unauthenticated surface.
    let host = std::env::var("BOTSERVER_HOST").unwrap_or_default();
    if no_security && !host.is_empty() {
        let host_trimmed = host.trim();
        let is_loopback_host = matches!(
            host_trimmed,
            "127.0.0.1" | "localhost" | "::1" | "[::1]"
        );
        if !is_loopback_host {
            return Err(std::io::Error::other(format!(
                "Refusing to start in no_security mode: BOTSERVER_HOST '{}' is not loopback",
                host_trimmed
            )));
        }
    }

    let ip = if no_security {
        std::net::IpAddr::from([127, 0, 0, 1])
    } else {
        std::net::IpAddr::from([0, 0, 0, 0])
    };
    let addr = SocketAddr::new(ip, port);

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
