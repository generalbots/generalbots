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

    // A desktop edition is one person's computer: only the shell on the same
    // machine talks to the server, so it listens on loopback as well. Setting
    // BOTSERVER_HOST to a non-loopback address opens it to the network on
    // purpose (sign-in is required in that mode).
    let desktop_edition = botcore::product::PRODUCT_CONFIG
        .read()
        .map(|product| product.edition.as_deref() == Some("desktop"))
        .unwrap_or(false);
    let host_is_default = matches!(
        host.trim(),
        "" | "127.0.0.1" | "localhost" | "::1" | "[::1]"
    );
    // #2477 — the General Bots OS appliance is a laptop on café Wi-Fi. Shell
    // mode used to publish the login form, the API and the suite on every
    // interface because the product is not `no_security`; the bind address now
    // follows the same rule as the desktop edition and stays on loopback
    // unless the operator sets BOTSERVER_HOST explicitly *and* opts in with
    // GB_ALLOW_LAN=1 (an environment flag, off by default; the image's
    // nftables `gb_lan_allowed` set is the matching host-side gate — no
    // Settings switch exists yet).
    let lan_opt_in = std::env::var("GB_ALLOW_LAN")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let shell_edition = botcore::product::is_shell_mode();
    let ip = if no_security
        || ((desktop_edition || shell_edition) && host_is_default)
        || (shell_edition && !lan_opt_in)
    {
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

        // #2614 — ConnectInfo<SocketAddr> must be inserted or the per-IP rate
        // limiter's extractor gets None and every client shares one bucket.
        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
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
        // #2614 — same fix on the plain-HTTP path (see HTTPS branch above).
        let result = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
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

#[cfg(test)]
mod connect_info_tests {
    use super::*;
    use axum::{body::Body, extract::ConnectInfo, http::Request, routing::get};
    use tower::{Service, ServiceExt};

    async fn who(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
        addr.to_string()
    }

    /// #2614 — the make-service flavour used by both listener paths must
    /// insert a distinct ConnectInfo per source address, which is what the
    /// per-IP rate limiter keys on.
    #[tokio::test]
    async fn connect_info_is_inserted_per_source_address() {
        let app = Router::new().route("/who", get(who));
        let mut make = app.into_make_service_with_connect_info::<SocketAddr>();
        for ip in ["127.0.0.1", "127.0.0.2"] {
            let addr: SocketAddr = format!("{ip}:12345").parse().expect("addr");
            let svc = make.call(addr).await.expect("make service");
            let res = svc
                .oneshot(Request::builder().uri("/who").body(Body::empty()).expect("request"))
                .await
                .expect("response");
            let body = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .expect("body");
            assert!(
                String::from_utf8_lossy(&body).contains(ip),
                "handler did not see the source address {ip}: {}",
                String::from_utf8_lossy(&body)
            );
        }
    }
}
