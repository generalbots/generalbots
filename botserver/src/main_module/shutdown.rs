//! Shutdown signal handling

use log::{error, info, warn};

pub fn print_shutdown_message() {
    println!();
    println!("Thank you for using General Bots!");
    println!();
}

pub async fn shutdown_signal() {
    info!("Shutdown signal handler installed, waiting for SIGINT or SIGTERM...");

    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed to install Ctrl+C handler: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                info!("SIGTERM handler installed successfully");
                signal.recv().await;
            }
            Err(e) => {
                error!("Failed to install SIGTERM handler: {}", e);
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM (systemctl stop), initiating graceful shutdown...");
        }
    }

    info!("Shutdown signal received - server will stop accepting new connections");
    warn!("Graceful shutdown timeout is 10s for HTTPS, after which process will exit");

    print_shutdown_message();

    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        warn!("Graceful shutdown exceeded 15s - forcing process exit to prevent hang");
        std::process::exit(0);
    });
}
