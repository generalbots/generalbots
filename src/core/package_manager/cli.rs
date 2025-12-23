use crate::core::secrets::{SecretPaths, SecretsManager};
use crate::package_manager::{get_all_components, InstallMode, PackageManager};
use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
            let result = pm.install(component).await?;
            println!("* Component '{}' installed successfully", component);


            if let Some(install_result) = result {
                install_result.print();
            }
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
        "version" => {
            let show_all = args.contains(&"--all".to_string());
            print_version(show_all).await?;
        }
        "rotate-secret" => {
            if args.len() < 3 {
                eprintln!("Usage: botserver rotate-secret <component>");
                eprintln!("Components: tables, drive, cache, email, directory, encryption");
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
                eprintln!("Usage: botserver rotate-secrets --all");
                eprintln!("This will rotate ALL secrets. Use with caution!");
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
                    if args.len() < 5 {
                        eprintln!("Usage: botserver vault put <path> <key=value> [key=value...]");
                        return Ok(());
                    }
                    let path = &args[3];
                    let kvs: Vec<&str> = args[4..].iter().map(|s| s.as_str()).collect();
                    vault_put(path, &kvs).await?;
                }
                "get" => {
                    if args.len() < 4 {
                        eprintln!("Usage: botserver vault get <path> [key]");
                        return Ok(());
                    }
                    let path = &args[3];
                    let key = args.get(4).map(|s| s.as_str());
                    vault_get(path, key).await?;
                }
                "list" => {
                    vault_list().await?;
                }
                "health" => {
                    vault_health().await?;
                }
                _ => {
                    eprintln!("Unknown vault command: {}", subcommand);
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
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }
    Ok(())
}

fn print_usage() {
    println!("BotServer CLI v{}", VERSION);
    println!();
    println!("Usage: botserver <command> [options]");
    println!();
    println!("Commands:");
    println!("  install <component>  Install a component");
    println!("  remove <component>   Remove a component");
    println!("  list                 List all components");
    println!("  status <component>   Check component status");
    println!("  start                Start all installed components");
    println!("  stop                 Stop all components");
    println!("  restart              Restart all components");
    println!("  vault <subcommand>   Manage Vault secrets");
    println!("  rotate-secret <comp> Rotate a component's credentials");
    println!("  rotate-secrets --all Rotate ALL credentials (dangerous!)");
    println!("  version [--all]      Show version information");
    println!("  --version, -v        Show version");
    println!("  --help, -h           Show this help");
    println!();
    println!("Options:");
    println!("  --container          Use container mode (LXC)");
    println!("  --tenant <name>      Specify tenant name");
    println!();
    println!("Vault subcommands:");
    println!("  vault migrate [.env] Migrate .env secrets to Vault");
    println!("  vault put <path> k=v Store secrets in Vault");
    println!("  vault get <path>     Get secrets from Vault");
    println!("  vault list           List all secret paths");
    println!("  vault health         Check Vault health");
}

fn print_vault_usage() {
    println!("Vault Secret Management");
    println!();
    println!("Usage: botserver vault <subcommand> [options]");
    println!();
    println!("Subcommands:");
    println!("  migrate [file]       Migrate secrets from .env to Vault");
    println!("                       Default file: .env");
    println!();
    println!("  put <path> k=v...    Store key-value pairs at path");
    println!("                       Example: botserver vault put gbo/email user=x pass=y");
    println!();
    println!("  get <path> [key]     Get all secrets or specific key from path");
    println!("                       Example: botserver vault get gbo/tables password");
    println!();
    println!("  list                 List all configured secret paths");
    println!();
    println!("  health               Check Vault connection status");
    println!();
    println!("Secret paths:");
    println!("  gbo/tables           Database credentials");
    println!("  gbo/drive            S3/MinIO credentials");
    println!("  gbo/email            SMTP credentials");
    println!("  gbo/cache            Redis credentials");
    println!("  gbo/directory        Zitadel credentials");
    println!("  gbo/llm              AI API keys");
    println!("  gbo/encryption       Encryption keys");
    println!("  gbo/stripe           Payment credentials");
    println!("  gbo/vectordb         Qdrant credentials");
    println!();
    println!("Environment:");
    println!("  VAULT_ADDR           Vault server address");
    println!("  VAULT_TOKEN          Vault authentication token");
}

async fn vault_migrate(env_file: &str) -> Result<()> {
    println!("Migrating secrets from {} to Vault...", env_file);


    let content = std::fs::read_to_string(env_file)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", env_file, e))?;


    let mut env_vars: HashMap<String, String> = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            env_vars.insert(key.trim().to_string(), value.trim().to_string());
        }
    }


    let manager = SecretsManager::from_env()?;
    if !manager.is_enabled() {
        return Err(anyhow::anyhow!(
            "Vault not configured. Set VAULT_ADDR and VAULT_TOKEN"
        ));
    }


    let mut tables: HashMap<String, String> = HashMap::new();
    if let Some(v) = env_vars.get("TABLES_SERVER") {
        tables.insert("host".into(), v.clone());
    }
    if let Some(v) = env_vars.get("TABLES_PORT") {
        tables.insert("port".into(), v.clone());
    }
    if let Some(v) = env_vars.get("TABLES_DATABASE") {
        tables.insert("database".into(), v.clone());
    }
    if let Some(v) = env_vars.get("TABLES_USERNAME") {
        tables.insert("username".into(), v.clone());
    }
    if let Some(v) = env_vars.get("TABLES_PASSWORD") {
        tables.insert("password".into(), v.clone());
    }
    if !tables.is_empty() {
        manager.put_secret(SecretPaths::TABLES, tables).await?;
        println!("  * Migrated tables credentials");
    }


    let mut custom: HashMap<String, String> = HashMap::new();
    if let Some(v) = env_vars.get("CUSTOM_SERVER") {
        custom.insert("host".into(), v.clone());
    }
    if let Some(v) = env_vars.get("CUSTOM_PORT") {
        custom.insert("port".into(), v.clone());
    }
    if let Some(v) = env_vars.get("CUSTOM_DATABASE") {
        custom.insert("database".into(), v.clone());
    }
    if let Some(v) = env_vars.get("CUSTOM_USERNAME") {
        custom.insert("username".into(), v.clone());
    }
    if let Some(v) = env_vars.get("CUSTOM_PASSWORD") {
        custom.insert("password".into(), v.clone());
    }
    if !custom.is_empty() {
        manager.put_secret("gbo/custom", custom).await?;
        println!("  * Migrated custom database credentials");
    }


    let mut drive: HashMap<String, String> = HashMap::new();
    if let Some(v) = env_vars.get("DRIVE_SERVER") {
        drive.insert("server".into(), v.clone());
    }
    if let Some(v) = env_vars.get("DRIVE_PORT") {
        drive.insert("port".into(), v.clone());
    }
    if let Some(v) = env_vars.get("DRIVE_USE_SSL") {
        drive.insert("use_ssl".into(), v.clone());
    }
    if let Some(v) = env_vars.get("DRIVE_ACCESSKEY") {
        drive.insert("accesskey".into(), v.clone());
    }
    if let Some(v) = env_vars.get("DRIVE_SECRET") {
        drive.insert("secret".into(), v.clone());
    }
    if let Some(v) = env_vars.get("DRIVE_ORG_PREFIX") {
        drive.insert("org_prefix".into(), v.clone());
    }
    if !drive.is_empty() {
        manager.put_secret(SecretPaths::DRIVE, drive).await?;
        println!("  * Migrated drive credentials");
    }


    let mut email: HashMap<String, String> = HashMap::new();
    if let Some(v) = env_vars.get("EMAIL_FROM") {
        email.insert("from".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMAIL_SERVER") {
        email.insert("server".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMAIL_PORT") {
        email.insert("port".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMAIL_USER") {
        email.insert("username".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMAIL_PASS") {
        email.insert("password".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMAIL_REJECT_UNAUTHORIZED") {
        email.insert("reject_unauthorized".into(), v.clone());
    }
    if !email.is_empty() {
        manager.put_secret(SecretPaths::EMAIL, email).await?;
        println!("  * Migrated email credentials");
    }


    let mut stripe: HashMap<String, String> = HashMap::new();
    if let Some(v) = env_vars.get("STRIPE_SECRET_KEY") {
        stripe.insert("secret_key".into(), v.clone());
    }
    if let Some(v) = env_vars.get("STRIPE_PROFESSIONAL_PLAN_PRICE_ID") {
        stripe.insert("professional_plan_price_id".into(), v.clone());
    }
    if let Some(v) = env_vars.get("STRIPE_PERSONAL_PLAN_PRICE_ID") {
        stripe.insert("personal_plan_price_id".into(), v.clone());
    }
    if !stripe.is_empty() {
        manager.put_secret("gbo/stripe", stripe).await?;
        println!("  * Migrated stripe credentials");
    }


    let mut llm: HashMap<String, String> = HashMap::new();
    if let Some(v) = env_vars.get("AI_KEY") {
        llm.insert("api_key".into(), v.clone());
    }
    if let Some(v) = env_vars.get("AI_LLM_MODEL") {
        llm.insert("model".into(), v.clone());
    }
    if let Some(v) = env_vars.get("AI_ENDPOINT") {
        llm.insert("endpoint".into(), v.clone());
    }
    if let Some(v) = env_vars.get("AI_EMBEDDING_MODEL") {
        llm.insert("embedding_model".into(), v.clone());
    }
    if let Some(v) = env_vars.get("AI_IMAGE_MODEL") {
        llm.insert("image_model".into(), v.clone());
    }
    if let Some(v) = env_vars.get("LLM_LOCAL") {
        llm.insert("local".into(), v.clone());
    }
    if let Some(v) = env_vars.get("LLM_CPP_PATH") {
        llm.insert("cpp_path".into(), v.clone());
    }
    if let Some(v) = env_vars.get("LLM_URL") {
        llm.insert("url".into(), v.clone());
    }
    if let Some(v) = env_vars.get("LLM_MODEL_PATH") {
        llm.insert("model_path".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMBEDDING_MODEL_PATH") {
        llm.insert("embedding_model_path".into(), v.clone());
    }
    if let Some(v) = env_vars.get("EMBEDDING_URL") {
        llm.insert("embedding_url".into(), v.clone());
    }
    if !llm.is_empty() {
        manager.put_secret(SecretPaths::LLM, llm).await?;
        println!("  * Migrated LLM credentials");
    }

    println!();
    println!("Migration complete!");
    println!();
    println!(
        "You can now remove secrets from {} and keep only:",
        env_file
    );
    println!("  RUST_LOG=info");
    println!("  VAULT_ADDR=<vault-address>");
    println!("  VAULT_TOKEN=<vault-token>");
    println!("  SERVER_HOST=0.0.0.0");
    println!("  SERVER_PORT=5858");

    Ok(())
}

async fn vault_put(path: &str, kvs: &[&str]) -> Result<()> {
    let manager = SecretsManager::from_env()?;
    if !manager.is_enabled() {
        return Err(anyhow::anyhow!("Vault not configured"));
    }

    let mut data: HashMap<String, String> = HashMap::new();
    for kv in kvs {
        if let Some((k, v)) = kv.split_once('=') {
            data.insert(k.to_string(), v.to_string());
        } else {
            eprintln!("Invalid key=value pair: {}", kv);
        }
    }

    if data.is_empty() {
        return Err(anyhow::anyhow!("No valid key=value pairs provided"));
    }

    manager.put_secret(path, data).await?;
    println!("* Stored {} key(s) at {}", kvs.len(), path);

    Ok(())
}

async fn vault_get(path: &str, key: Option<&str>) -> Result<()> {
    let manager = SecretsManager::from_env()?;
    if !manager.is_enabled() {
        return Err(anyhow::anyhow!("Vault not configured"));
    }

    let secrets = manager.get_secret(path).await?;

    if let Some(k) = key {
        if let Some(v) = secrets.get(k) {
            println!("{}", v);
        } else {
            eprintln!("Key '{}' not found at {}", k, path);
        }
    } else {
        println!("Secrets at {}:", path);
        for (k, v) in &secrets {

            let masked = if k.contains("password")
                || k.contains("secret")
                || k.contains("key")
                || k.contains("token")
            {
                format!("{}...", &v.chars().take(4).collect::<String>())
            } else {
                v.clone()
            };
            println!("  {}={}", k, masked);
        }
    }

    Ok(())
}

async fn vault_list() -> Result<()> {
    println!("Configured secret paths:");
    println!("  {} - Database credentials", SecretPaths::TABLES);
    println!("  {} - S3/MinIO credentials", SecretPaths::DRIVE);
    println!("  {} - Redis credentials", SecretPaths::CACHE);
    println!("  {} - SMTP credentials", SecretPaths::EMAIL);
    println!("  {} - Zitadel credentials", SecretPaths::DIRECTORY);
    println!("  {} - AI API keys", SecretPaths::LLM);
    println!("  {} - Encryption keys", SecretPaths::ENCRYPTION);
    println!("  {} - LiveKit credentials", SecretPaths::MEET);
    println!("  {} - Forgejo credentials", SecretPaths::ALM);
    println!("  {} - Qdrant credentials", SecretPaths::VECTORDB);
    println!("  {} - InfluxDB credentials", SecretPaths::OBSERVABILITY);
    println!("  gbo/stripe - Payment credentials");
    println!("  gbo/custom - Custom database");

    Ok(())
}

async fn print_version(show_all: bool) -> Result<()> {
    println!("botserver {}", VERSION);

    if show_all {
        println!();
        println!("Build Information:");
        println!("  rustc: {}", rustc_version());
        println!("  target: {}", std::env::consts::ARCH);
        println!("  os: {}", std::env::consts::OS);
        println!();


        let mode = InstallMode::Local;
        if let Ok(pm) = PackageManager::new(mode, None) {
            println!("Installed Components:");
            let components = pm.list();
            let mut installed_count = 0;
            for component in &components {
                if pm.is_installed(component) {
                    println!("  * {} (installed)", component);
                    installed_count += 1;
                }
            }
            if installed_count == 0 {
                println!("  (none)");
            }
            println!();
            println!("Available Components: {}", components.len());
        }


        println!();
        println!("Secrets:");
        if let Ok(manager) = SecretsManager::from_env() {
            if manager.is_enabled() {
                match manager.health_check().await {
                    Ok(true) => println!("  Vault: connected"),
                    _ => println!("  Vault: not reachable"),
                }
            } else {
                println!("  Vault: not configured");
            }
        } else {
            println!("  Vault: not configured");
        }
    }

    Ok(())
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn generate_password(length: usize) -> String {
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_access_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..20)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_secret_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut rng = rand::rng();
    (0..40)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

async fn rotate_secret(component: &str) -> Result<()> {
    let manager = SecretsManager::from_env()?;
    if !manager.is_enabled() {
        return Err(anyhow::anyhow!(
            "Vault not configured. Set VAULT_ADDR and VAULT_TOKEN"
        ));
    }

    println!("Rotating credentials for: {}", component);
    println!();

    match component {
        "tables" => {
            let new_password = generate_password(32);
            let mut secrets = manager
                .get_secret(SecretPaths::TABLES)
                .await
                .unwrap_or_default();
            let old_password = secrets.get("password").cloned().unwrap_or_default();
            secrets.insert("password".to_string(), new_password.clone());

            println!("⚠️  WARNING: You must update PostgreSQL with the new password!");
            println!();
            println!("Run this SQL command:");
            println!(
                "  ALTER USER {} WITH PASSWORD '{}';",
                secrets.get("username").unwrap_or(&"postgres".to_string()),
                new_password
            );
            println!();
            println!(
                "Old password: {}...",
                &old_password.chars().take(4).collect::<String>()
            );
            println!(
                "New password: {}...",
                &new_password.chars().take(4).collect::<String>()
            );

            print!("Save to Vault? [y/N]: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                manager.put_secret(SecretPaths::TABLES, secrets).await?;
                println!("✓ Credentials saved to Vault");
            } else {
                println!("✗ Aborted");
            }
        }
        "drive" => {
            let new_accesskey = generate_access_key();
            let new_secret = generate_secret_key();
            let mut secrets = manager
                .get_secret(SecretPaths::DRIVE)
                .await
                .unwrap_or_default();

            println!("⚠️  WARNING: You must update MinIO with the new credentials!");
            println!();
            println!("Run these commands:");
            println!(
                "  mc admin user add myminio {} {}",
                new_accesskey, new_secret
            );
            println!(
                "  mc admin policy attach myminio readwrite --user {}",
                new_accesskey
            );
            println!();
            println!("New access key: {}", new_accesskey);
            println!(
                "New secret key: {}...",
                &new_secret.chars().take(8).collect::<String>()
            );

            print!("Save to Vault? [y/N]: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                secrets.insert("accesskey".to_string(), new_accesskey);
                secrets.insert("secret".to_string(), new_secret);
                manager.put_secret(SecretPaths::DRIVE, secrets).await?;
                println!("✓ Credentials saved to Vault");
            } else {
                println!("✗ Aborted");
            }
        }
        "cache" => {
            let new_password = generate_password(32);
            let mut secrets: HashMap<String, String> = HashMap::new();

            println!("⚠️  WARNING: You must update Valkey/Redis with the new password!");
            println!();
            println!("Run this command:");
            println!("  redis-cli CONFIG SET requirepass '{}'", new_password);
            println!();
            println!(
                "New password: {}...",
                &new_password.chars().take(4).collect::<String>()
            );

            print!("Save to Vault? [y/N]: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                secrets.insert("password".to_string(), new_password);
                manager.put_secret(SecretPaths::CACHE, secrets).await?;
                println!("✓ Credentials saved to Vault");
            } else {
                println!("✗ Aborted");
            }
        }
        "email" => {
            let new_password = generate_password(24);
            let mut secrets = manager
                .get_secret(SecretPaths::EMAIL)
                .await
                .unwrap_or_default();

            println!("⚠️  WARNING: You must update the mail server with the new password!");
            println!();
            println!(
                "New password: {}...",
                &new_password.chars().take(4).collect::<String>()
            );

            print!("Save to Vault? [y/N]: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                secrets.insert("password".to_string(), new_password);
                manager.put_secret(SecretPaths::EMAIL, secrets).await?;
                println!("✓ Credentials saved to Vault");
            } else {
                println!("✗ Aborted");
            }
        }
        "encryption" => {
            let new_key = generate_password(64);
            let mut secrets: HashMap<String, String> = HashMap::new();

            println!("⚠️  CRITICAL WARNING: Rotating encryption key will make existing encrypted data unreadable!");
            println!("⚠️  Make sure to re-encrypt all data with the new key!");
            println!();
            println!(
                "New master key: {}...",
                &new_key.chars().take(8).collect::<String>()
            );

            print!("Are you ABSOLUTELY sure? Type 'ROTATE' to confirm: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "ROTATE" {
                secrets.insert("master_key".to_string(), new_key);
                manager.put_secret(SecretPaths::ENCRYPTION, secrets).await?;
                println!("✓ Encryption key saved to Vault");
            } else {
                println!("✗ Aborted");
            }
        }
        "directory" => {
            let new_secret = generate_password(48);
            let mut secrets = manager
                .get_secret(SecretPaths::DIRECTORY)
                .await
                .unwrap_or_default();

            println!("⚠️  WARNING: You must update Zitadel with the new client secret!");
            println!();
            println!(
                "New client secret: {}...",
                &new_secret.chars().take(8).collect::<String>()
            );

            print!("Save to Vault? [y/N]: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                secrets.insert("client_secret".to_string(), new_secret);
                manager.put_secret(SecretPaths::DIRECTORY, secrets).await?;
                println!("✓ Credentials saved to Vault");
            } else {
                println!("✗ Aborted");
            }
        }
        _ => {
            eprintln!("Unknown component: {}", component);
            eprintln!("Valid components: tables, drive, cache, email, directory, encryption");
        }
    }

    Ok(())
}

async fn rotate_all_secrets() -> Result<()> {
    println!("🔐 ROTATING ALL SECRETS");
    println!("========================");
    println!();
    println!("⚠️  CRITICAL WARNING!");
    println!("This will generate new credentials for ALL components.");
    println!("You MUST update each service manually after rotation.");
    println!();
    print!("Type 'ROTATE ALL' to continue: ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() != "ROTATE ALL" {
        println!("✗ Aborted");
        return Ok(());
    }

    let manager = SecretsManager::from_env()?;
    if !manager.is_enabled() {
        return Err(anyhow::anyhow!("Vault not configured"));
    }

    println!();
    println!("Generating new credentials...");
    println!();


    let tables_password = generate_password(32);
    let mut tables = manager
        .get_secret(SecretPaths::TABLES)
        .await
        .unwrap_or_default();
    tables.insert("password".to_string(), tables_password.clone());
    manager
        .put_secret(SecretPaths::TABLES, tables.clone())
        .await?;
    println!(
        "✓ tables: ALTER USER {} WITH PASSWORD '{}';",
        tables.get("username").unwrap_or(&"postgres".to_string()),
        tables_password
    );


    let drive_accesskey = generate_access_key();
    let drive_secret = generate_secret_key();
    let mut drive = manager
        .get_secret(SecretPaths::DRIVE)
        .await
        .unwrap_or_default();
    drive.insert("accesskey".to_string(), drive_accesskey.clone());
    drive.insert("secret".to_string(), drive_secret.clone());
    manager.put_secret(SecretPaths::DRIVE, drive).await?;
    println!(
        "✓ drive: mc admin user add myminio {} {}",
        drive_accesskey, drive_secret
    );


    let cache_password = generate_password(32);
    let mut cache: HashMap<String, String> = HashMap::new();
    cache.insert("password".to_string(), cache_password.clone());
    manager.put_secret(SecretPaths::CACHE, cache).await?;
    println!(
        "✓ cache: redis-cli CONFIG SET requirepass '{}'",
        cache_password
    );


    let email_password = generate_password(24);
    let mut email = manager
        .get_secret(SecretPaths::EMAIL)
        .await
        .unwrap_or_default();
    email.insert("password".to_string(), email_password.clone());
    manager.put_secret(SecretPaths::EMAIL, email).await?;
    println!("✓ email: new password = {}", email_password);


    let directory_secret = generate_password(48);
    let mut directory = manager
        .get_secret(SecretPaths::DIRECTORY)
        .await
        .unwrap_or_default();
    directory.insert("client_secret".to_string(), directory_secret.clone());
    manager
        .put_secret(SecretPaths::DIRECTORY, directory)
        .await?;
    println!(
        "✓ directory: new client_secret = {}...",
        &directory_secret.chars().take(12).collect::<String>()
    );

    println!();
    println!("========================");
    println!("✓ All secrets rotated and saved to Vault");
    println!();
    println!("⚠️  IMPORTANT: Run the commands above to update each service!");
    println!("⚠️  Then restart botserver: botserver restart");

    Ok(())
}

async fn vault_health() -> Result<()> {
    let manager = SecretsManager::from_env()?;

    if !manager.is_enabled() {
        println!("x Vault not configured");
        println!("  Set VAULT_ADDR and VAULT_TOKEN environment variables");
        return Ok(());
    }

    match manager.health_check().await {
        Ok(true) => {
            println!("* Vault is healthy");
            println!("  Address: {}", env::var("VAULT_ADDR").unwrap_or_default());
        }
        Ok(false) => {
            println!("x Vault health check failed");
        }
        Err(e) => {
            println!("x Vault error: {}", e);
        }
    }

    Ok(())
}
