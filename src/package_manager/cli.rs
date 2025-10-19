use anyhow::Result;
use std::env;

use crate::package_manager::{InstallMode, PackageManager};

pub async fn run() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
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
            println!("✓ Component '{}' installed successfully", component);
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
            println!("✓ Component '{}' removed successfully", component);
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
                    "✓ installed"
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
                println!("✓ Component '{}' is installed", component);
            } else {
                println!("✗ Component '{}' is not installed", component);
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

fn print_usage() {
    println!("BotServer Package Manager\n\nUSAGE:\n  botserver <command> [options]\n\nCOMMANDS:\n  install <component>    Install component\n  remove <component>     Remove component\n  list                   List all components\n  status <component>     Check component status\n\nOPTIONS:\n  --container            Use container mode (LXC)\n  --tenant <name>        Specify tenant (default: 'default')\n\nCOMPONENTS:\n  Required: drive cache tables llm\n  Optional: email proxy directory alm alm-ci dns webmail meeting table-editor doc-editor desktop devtools bot system vector-db host\n\nEXAMPLES:\n  botserver install email\n  botserver install email --container --tenant myorg\n  botserver remove email\n  botserver list");
}
