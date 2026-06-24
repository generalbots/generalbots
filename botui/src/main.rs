use log::info;
use std::net::SocketAddr;

mod shared;
mod ui_server;

fn init_logging() {
    botlib::logging::init_compact_logger("info");
}

fn get_port() -> u16 {
    std::env::var("BOTUI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000)
}

fn get_cloud_port() -> u16 {
    std::env::var("BOTUI_CLOUD_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4000)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let version = env!("CARGO_PKG_VERSION");
    info!("BotUI {version} starting...");

    let app = ui_server::configure_router();
    let port = get_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("UI server listening on http://{addr}");

    let cloud_app = ui_server::configure_cloud_router();
    let cloud_port = get_cloud_port();
    let cloud_addr = SocketAddr::from(([0, 0, 0, 0], cloud_port));

    let cloud_listener = tokio::net::TcpListener::bind(cloud_addr).await?;
    info!("Cloud server listening on http://{cloud_addr}");

    let suite_server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());
    let cloud_server = axum::serve(cloud_listener, cloud_app).with_graceful_shutdown(shutdown_signal());

    tokio::try_join!(
        async { suite_server.await.map_err(|e| anyhow::anyhow!(e)) },
        async { cloud_server.await.map_err(|e| anyhow::anyhow!(e)) }
    )?;

    info!("BotUI shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            log::error!("Failed to install Ctrl+C handler: {e}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                log::error!("Failed to install SIGTERM handler: {e}");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => info!("Received Ctrl+C, shutting down..."),
        () = terminate => info!("Received SIGTERM, shutting down..."),
    }
}
