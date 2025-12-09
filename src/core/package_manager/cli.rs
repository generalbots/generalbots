use crate::package_manager::{get_all_components, InstallMode, PackageManager};
use anyhow::Result;
use std::env;
use std::process::Command;
pub async fn run() -> Result<()> {
    // Logger is already initialized in main.rs, don't initialize again
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    use tracing::info;
    fn print_usage() {
        info!("usage: botserver <command> [options]")
    }
    let command = &args[1];
    match command.as_str() {
        "start" => {
            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode, tenant)?;
            println!("Starting all installed components...");
            let components = get_all_components();
            for component in components {
                if pm.is_installed(component.name) {
                    match pm.start(component.name) {
                        Ok(_) => println!("* Started {}", component.name),
                        Err(e) => eprintln!("x Failed to start {}: {}", component.name, e),
                    }
                }
            }
            println!("* BotServer components started");
        }
        "stop" => {
            println!("Stopping all components...");
            let components = get_all_components();
            for component in components {
                let _ = Command::new("pkill")
                    .arg("-f")
                    .arg(component.termination_command)
                    .output();
            }
            println!("* BotServer components stopped");
        }
        "restart" => {
            println!("Restarting BotServer...");
            let components = get_all_components();
            for component in components {
                let _ = Command::new("pkill")
                    .arg("-f")
                    .arg(component.termination_command)
                    .output();
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode, tenant)?;
            let components = get_all_components();
            for component in components {
                if pm.is_installed(component.name) {
                    let _ = pm.start(component.name);
                }
            }
            println!("* BotServer restarted");
        }
        "install" => {
            if args.len() < 3 {
                eprintln!("Usage: botserver install <component> [--container] [--tenant <name>]");
                return Ok(());
            }
            let component = &args[2];
            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode, tenant)?;
            pm.install(component).await?;
            println!("* Component '{}' installed successfully", component);
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: botserver remove <component> [--container] [--tenant <name>]");
                return Ok(());
            }
            let component = &args[2];
            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode, tenant)?;
            pm.remove(component)?;
            println!("* Component '{}' removed successfully", component);
        }
        "list" => {
            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode, tenant)?;
            println!("Available components:");
            for component in pm.list() {
                let status = if pm.is_installed(&component) {
                    "* installed"
                } else {
                    "  available"
                };
                println!(" {} {}", status, component);
            }
        }
        "status" => {
            if args.len() < 3 {
                eprintln!("Usage: botserver status <component> [--container] [--tenant <name>]");
                return Ok(());
            }
            let component = &args[2];
            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode, tenant)?;
            if pm.is_installed(component) {
                println!("* Component '{}' is installed", component);
            } else {
                println!("x Component '{}' is not installed", component);
            }
        }
        "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }
    Ok(())
}
