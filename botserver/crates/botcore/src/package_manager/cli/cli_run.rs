use super::*;

use crate::i18n::cli_locale::{t, t_args};

pub async fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
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
            println!("{}", t("cli-start-starting"));
            let components = get_all_components();
            for component in components {
                if pm.is_installed(component.name) {
                    match pm.start(component.name) {
                        Ok(_) => println!(
                            "* {}",
                            t_args("cli-component-started", &[("component", component.name)])
                        ),
                        Err(e) => eprintln!(
                            "x {}",
                            t_args(
                                "cli-component-start-failed",
                                &[("component", component.name), ("error", &e.to_string())]
                            )
                        ),
                    }
                }
            }
            println!("* {}", t("cli-start-done"));
        }
        "stop" => {
            println!("{}", t("cli-stop-stopping"));
            let components = get_all_components();
            for component in components {
                safe_pkill(&["-f", component.termination_command]);
            }
            println!("* {}", t("cli-stop-done"));
        }
        "restart" => {
            println!("{}", t("cli-restart-restarting"));
            let components = get_all_components();
            for component in components {
                safe_pkill(&["-f", component.termination_command]);
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
            println!("* {}", t("cli-restart-done"));
        }
        "install" => {
            if args.len() < 3 {
                eprintln!("{}", t("cli-usage-install-synopsis"));
                return Ok(());
            }
            let component = &args[2];

            if component == "protection" {
                install_protection()?;
                return Ok(());
            }

            let mode = if args.contains(&"--container".to_string()) {
                InstallMode::Container
            } else {
                InstallMode::Local
            };
            let container_only = args.contains(&"--container-only".to_string());
            let tenant = if let Some(idx) = args.iter().position(|a| a == "--tenant") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            let pm = PackageManager::new(mode.clone(), tenant)?;

            let result = if container_only && mode == InstallMode::Container {
                Some(pm.install_container_only(component)?)
            } else {
                pm.install(component).await?
            };

            if let Some(install_result) = result {
                install_result.print();
                if container_only {
                    println!("\n* {}", t("cli-container-only-created"));
                    println!("* {}", t("cli-container-only-next"));
                }
            }
        }
        "setup-env" => {
            let vault_container = args.get(2).map(|s| s.as_str()).unwrap_or("vault");
            let container_name = vault_container.to_string();

            println!(
                "* {}",
                t_args("cli-setup-env-generating", &[("container", &container_name)])
            );

            match PackageManager::generate_env_from_vault(&container_name) {
                Ok(env_vars) => {
                    println!("* {}", t("cli-setup-env-done"));
                    println!("\n{}", t("cli-setup-env-config"));
                    println!("{}", env_vars);
                    println!("\n{}", t("cli-setup-env-next"));
                }
                Err(e) => {
                    eprintln!(
                        "x {}",
                        t_args("cli-setup-env-failed", &[("error", &e.to_string())])
                    );
                    return Err(anyhow::anyhow!("Setup env failed: {}", e));
                }
            }
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("{}", t("cli-usage-remove-synopsis"));
                return Ok(());
            }
            let component = &args[2];

            if component == "protection" {
                remove_protection()?;
                return Ok(());
            }

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
            println!(
                "* {}",
                t_args("cli-component-removed", &[("component", component.as_str())])
            );
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
            println!("{}", t("cli-list-title"));
            for component in pm.list() {
                let status = if pm.is_installed(&component) {
                    format!("* {}", t("cli-list-installed"))
                } else {
                    format!("  {}", t("cli-list-available"))
                };
                println!(" {} {}", status, component);
            }
        }
        "status" => {
            if args.len() < 3 {
                eprintln!("{}", t("cli-usage-status-synopsis"));
                return Ok(());
            }
            let component = &args[2];

            if component == "protection" {
                let _ = verify_protection();
                return Ok(());
            }

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
                println!(
                    "* {}",
                    t_args("cli-component-installed", &[("component", component.as_str())])
                );
            } else {
                println!(
                    "x {}",
                    t_args("cli-component-not-installed", &[("component", component.as_str())])
                );
            }
        }
        "version" => {
            let show_all = args.contains(&"--all".to_string());
            print_version(show_all).await?;
        }
        "rotate-secret" => {
            if args.len() < 3 {
                eprintln!("{}", t("cli-usage-rotate-secret-synopsis"));
                eprintln!("{}", t("cli-rotate-components"));
                return Ok(());
            }
            let component = &args[2];
            rotate_secret(component).await?;
        }
        "rotate-secrets" => {
            let rotate_all = args.contains(&"--all".to_string());
            if rotate_all {
                rotate_all_secrets().await?;
            } else {
                eprintln!("{}", t("cli-usage-rotate-secrets-synopsis"));
                eprintln!("{}", t("cli-rotate-secrets-warning"));
            }
        }
        "certs" => {
            if args.len() < 3 {
                print_certs_usage();
                return Ok(());
            }
            let subcommand = &args[2];
            match subcommand.as_str() {
                "generate" => {
                    let mode = if args.contains(&"--container".to_string()) {
                        InstallMode::Container
                    } else {
                        InstallMode::Local
                    };
                    let tenant = args.iter().position(|a| a == "--tenant")
                        .and_then(|i| args.get(i + 1).cloned());
                    let pm = PackageManager::new(mode, tenant)?;
                    println!("{}", t("cli-certs-generating"));
                    certs_utils::generate_all_certs(&pm.base_path)?;
                    println!("* {}", t("cli-certs-generated"));
                }
                "push" => {
                    let ssh_host = if let Some(idx) = args.iter().position(|a| a == "--ssh-host") {
                        args.get(idx + 1).map(|s| s.as_str())
                    } else {
                        Some(DEFAULT_SSH_HOST)
                    };
                    if let Some(host) = ssh_host {
                        let mode = if args.contains(&"--container".to_string()) {
                            InstallMode::Container
                        } else {
                            InstallMode::Local
                        };
                        let tenant = args.iter().position(|a| a == "--tenant")
                            .and_then(|i| args.get(i + 1).cloned());
                        let pm = PackageManager::new(mode, tenant)?;
                        println!(
                            "{}",
                            t_args("cli-certs-pushing", &[("host", host)])
                        );
                        certs_utils::push_all_certs_ssh(&pm.base_path, host)?;
                        println!("* {}", t("cli-certs-pushed"));
                    } else {
                        eprintln!("{}", t("cli-usage-certs-push-synopsis"));
                    }
                }
                "provision" => {
                    let ssh_host = if let Some(idx) = args.iter().position(|a| a == "--ssh-host") {
                        args.get(idx + 1).map(|s| s.as_str())
                    } else {
                        Some(DEFAULT_SSH_HOST)
                    };
                    if let Some(host) = ssh_host {
                        let mode = if args.contains(&"--container".to_string()) {
                            InstallMode::Container
                        } else {
                            InstallMode::Local
                        };
                        let tenant = args.iter().position(|a| a == "--tenant")
                            .and_then(|i| args.get(i + 1).cloned());
                        let pm = PackageManager::new(mode, tenant)?;
                        println!("{}", t("cli-certs-provision-title"));
                        println!("{}", t("cli-certs-phase1"));
                        certs_utils::generate_all_certs(&pm.base_path)?;
                        println!(
                            "{}",
                            t_args("cli-certs-phase2", &[("host", host)])
                        );
                        certs_utils::push_all_certs_ssh(&pm.base_path, host)?;
                        println!("* {}", t("cli-certs-provision-done"));
                        println!();
                        println!("{}", t("cli-certs-next-steps"));
                        println!("  {}", t("cli-certs-next-restart"));
                        println!("  {}", t("cli-certs-next-connection"));
                        println!("  {}", t("cli-certs-next-verify"));
                    } else {
                        eprintln!("{}", t("cli-usage-certs-provision-synopsis"));
                    }
                }
                _ => {
                    eprintln!(
                        "{}",
                        t_args("cli-certs-unknown", &[("command", subcommand.as_str())])
                    );
                    print_certs_usage();
                }
            }
        }
        "vault" => {
            if args.len() < 3 {
                print_vault_usage();
                return Ok(());
            }
            let subcommand = &args[2];
            match subcommand.as_str() {
                "migrate" => {
                    let env_file = args.get(3).map(|s| s.as_str()).unwrap_or(".env");
                    vault_migrate(env_file).await?;
                }
                "put" => {
                    if args.len() < 4 {
                        eprintln!("{}", t("cli-usage-vault-put-synopsis"));
                        eprintln!("       {}", t("cli-usage-vault-put-interactive"));
                        return Ok(());
                    }
                    let path = &args[3];
                    let kvs: Vec<&str> = args[4..].iter().map(|s| s.as_str()).collect();
                    vault_put(path, &kvs).await?;
                }
                "get" => {
                    if args.len() < 4 {
                        eprintln!("{}", t("cli-usage-vault-get-synopsis"));
                        return Ok(());
                    }
                    let path = &args[3];
                    let key = args.get(4).map(|s| s.as_str());
                    vault_get(path, key).await?;
                }
                "list" => {
                    vault_list()?;
                }
                "health" => {
                    vault_health().await?;
                }
                _ => {
                    eprintln!(
                        "{}",
                        t_args("cli-vault-unknown", &[("command", subcommand.as_str())])
                    );
                    print_vault_usage();
                }
            }
        }
        "--version" | "-v" => {
            print_version(false).await?;
        }
        "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!(
                "{}",
                t_args("cli-unknown-command", &[("command", command.as_str())])
            );
            print_usage();
        }
    }
    Ok(())
}
