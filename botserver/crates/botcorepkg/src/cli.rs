use crate::{generate_random_string, InstallMode, PackageManager};
use botlib::security::SafeCommand;
use anyhow::Result;
use rand::Rng;
use std::env;

pub struct ComponentInfo {
    pub name: &'static str,
    pub termination_command: &'static str,
}

pub fn get_all_components() -> Vec<ComponentInfo> {
    vec![
        ComponentInfo {
            name: "tables",
            termination_command: "postgres",
        },
        ComponentInfo {
            name: "cache",
            termination_command: "redis-server",
        },
        ComponentInfo {
            name: "drive",
            termination_command: "minio",
        },
        ComponentInfo {
            name: "llm",
            termination_command: "llama-server",
        },
    ]
}

fn safe_pkill(args: &[&str]) {
    if let Ok(cmd) = SafeCommand::new("pkill").and_then(|c| c.args(args)) {
        let _ = cmd.execute();
    }
}

fn parse_mode(args: &[String]) -> InstallMode {
    if args.contains(&"--container".to_string()) {
        InstallMode::Container
    } else {
        InstallMode::Local
    }
}

fn parse_tenant(args: &[String]) -> Option<String> {
    if let Some(idx) = args.iter().position(|a| a == "--tenant") {
        args.get(idx + 1).cloned()
    } else {
        None
    }
}

pub async fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        super::cli_display::print_usage();
        return Ok(());
    }

    let command = &args[1];
    match command.as_str() {
        "start" => {
            let mode = parse_mode(&args);
            let tenant = parse_tenant(&args);
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
                safe_pkill(&["-f", component.termination_command]);
            }
            println!("* BotServer components stopped");
        }
        "restart" => {
            println!("Restarting BotServer...");
            let components = get_all_components();
            for component in components {
                safe_pkill(&["-f", component.termination_command]);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let mode = parse_mode(&args);
            let tenant = parse_tenant(&args);
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
                eprintln!("Usage: botserver install <component> [--container] [--container-only] [--tenant <name>]");
                return Ok(());
            }
            let component = &args[2];

            if component == "protection" {
                super::cli_ops::install_protection()?;
                return Ok(());
            }

            let mode = parse_mode(&args);
            let container_only = args.contains(&"--container-only".to_string());
            let tenant = parse_tenant(&args);
            let pm = PackageManager::new(mode.clone(), tenant)?;

            let result = if container_only && mode == InstallMode::Container {
                Some(pm.install_container_only(component)?)
            } else {
                pm.install(component).await?
            };

            if let Some(install_result) = result {
                install_result.print();
                if container_only {
                    println!("\n* Container created successfully (--container-only mode)");
                    println!("* Run without --container-only to complete installation");
                }
            }
        }
        "setup-env" => {
            let vault_container = args.get(2).map(|s| s.as_str()).unwrap_or("vault");
            let container_name = vault_container.to_string();

            println!("* Generating .env from vault container: {}", container_name);

            match PackageManager::generate_env_from_vault(&container_name) {
                Ok(env_vars) => {
                    println!("* Successfully generated .env from vault");
                    println!("\nGenerated configuration:");
                    println!("{}", env_vars);
                    println!("\nYou can now start botserver with: botserver start");
                }
                Err(e) => {
                    eprintln!("x Failed to generate .env: {}", e);
                    return Err(anyhow::anyhow!("Setup env failed: {}", e));
                }
            }
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: botserver remove <component> [--container] [--tenant <name>]");
                return Ok(());
            }
            let component = &args[2];

            if component == "protection" {
                super::cli_ops::remove_protection()?;
                return Ok(());
            }

            let mode = parse_mode(&args);
            let tenant = parse_tenant(&args);
            let pm = PackageManager::new(mode, tenant)?;
            pm.remove(component)?;
            println!("* Component '{}' removed successfully", component);
        }
        "list" => {
            let mode = parse_mode(&args);
            let tenant = parse_tenant(&args);
            let pm = PackageManager::new(mode, tenant)?;
            println!("Available components:");
            for component in pm.list() {
                let status = if pm.is_installed(&component) {
                    "* installed"
                } else {
                    " available"
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

            if component == "protection" {
                let result = super::cli_ops::verify_protection();
                println!("{}", result);
                return Ok(());
            }

            let mode = parse_mode(&args);
            let tenant = parse_tenant(&args);
            let pm = PackageManager::new(mode, tenant)?;
            if pm.is_installed(component) {
                println!("* Component '{}' is installed", component);
            } else {
                println!("x Component '{}' is not installed", component);
            }
        }
        "version" => {
            let show_all = args.contains(&"--all".to_string());
            super::cli_display::print_version(show_all).await?;
        }
        "rotate-secret" => {
            if args.len() < 3 {
                eprintln!("Usage: botserver rotate-secret <component>");
                eprintln!("Components: tables, drive, cache, email, directory, encryption, jwt");
                return Ok(());
            }
            let component = &args[2];
            super::cli_secrets::rotate_secret(component).await?;
        }
        "rotate-secrets" => {
            let rotate_all = args.contains(&"--all".to_string());
            if rotate_all {
                super::cli_secrets::rotate_all_secrets().await?;
            } else {
                eprintln!("Usage: botserver rotate-secrets --all");
                eprintln!("This will rotate ALL secrets. Use with caution!");
            }
        }
        "vault" => {
            if args.len() < 3 {
                super::cli_display::print_vault_usage();
                return Ok(());
            }
            let subcommand = &args[2];
            match subcommand.as_str() {
                "migrate" => {
                    let env_file = args.get(3).map(|s| s.as_str()).unwrap_or(".env");
                    super::cli_display::vault_migrate(env_file).await?;
                }
                "put" => {
                    if args.len() < 4 {
                        eprintln!("Usage: botserver vault put <path> [key=value] [key=value...]");
                        eprintln!(" botserver vault put <path> (interactive mode - prompts for keys)");
                        return Ok(());
                    }
                    let path = &args[3];
                    let kvs: Vec<&str> = args[4..].iter().map(|s| s.as_str()).collect();
                    super::cli_ops::vault_put(path, &kvs).await?;
                }
                "get" => {
                    if args.len() < 4 {
                        eprintln!("Usage: botserver vault get <path> [key]");
                        return Ok(());
                    }
                    let path = &args[3];
                    let key = args.get(4).map(|s| s.as_str());
                    super::cli_ops::vault_get(path, key).await?;
                }
                "list" => {
                    super::cli_display::vault_list()?;
                }
                "health" => {
                    super::cli_ops::vault_health().await?;
                }
                _ => {
                    eprintln!("Unknown vault command: {}", subcommand);
                    super::cli_display::print_vault_usage();
                }
            }
        }
        "--version" | "-v" => {
            super::cli_display::print_version(false).await?;
        }
        "--help" | "-h" => {
            super::cli_display::print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            super::cli_display::print_usage();
        }
    }
    Ok(())
}

pub fn generate_password(length: usize) -> String {
    generate_random_string(length)
}

pub fn generate_access_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..20)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn generate_secret_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut rng = rand::rng();
    (0..40)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
