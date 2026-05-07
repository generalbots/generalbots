use crate::cli::{generate_access_key, generate_password, generate_secret_key};
use crate::{InstallMode, PackageManager};
use botcoresecrets::{SecretPaths, SecretsManager};
use botlib::security::SafeCommand;
use anyhow::Result;
use std::collections::HashMap;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_usage() {
    println!("BotServer CLI v{}", VERSION);
    println!();
    println!("Usage: botserver <command> [options]");
    println!();
    println!("Commands:");
    println!(" install <component> Install a component");
    println!(" remove <component> Remove a component");
    println!(" list List all components");
    println!(" status <component> Check component status");
    println!(" start Start all installed components");
    println!(" stop Stop all components");
    println!(" restart Restart all components");
    println!(" setup-env Generate .env from running vault container");
    println!(" vault <subcommand> Manage Vault secrets");
    println!(" rotate-secret <comp> Rotate a component's credentials");
    println!(" (tables, drive, cache, email, directory, encryption, jwt)");
    println!(" rotate-secrets --all Rotate ALL credentials (dangerous!)");
    println!(" version [--all] Show version information");
    println!(" --version, -v Show version");
    println!(" --help, -h Show this help");
    println!();
    println!("Options:");
    println!(" --container Use container mode (LXC)");
    println!(" --container-only Create container only, don't complete installation");
    println!(" --tenant <name> Specify tenant name");
    println!();
    println!("Security Protection (requires root):");
    println!(" sudo botserver install protection Install security tools + sudoers");
    println!(" sudo botserver remove protection Remove sudoers configuration");
    println!(" botserver status protection Check protection tools status");
    println!();
    println!("Vault subcommands:");
    println!(" vault migrate [.env] Migrate .env secrets to Vault");
    println!(" vault put <path> k=v Store secrets in Vault");
    println!(" vault get <path> Get secrets from Vault");
    println!(" vault list List all secret paths");
    println!(" vault health Check Vault health");
}

pub fn print_vault_usage() {
    println!("Vault Secret Management");
    println!();
    println!("Usage: botserver vault <subcommand> [options]");
    println!();
    println!("Subcommands:");
    println!(" migrate [file] Migrate secrets from .env to Vault");
    println!(" Default file: .env");
    println!();
    println!(" put <path> k=v... Store key-value pairs at path");
    println!(" Example: botserver vault put gbo/email user=x pass=y");
    println!();
    println!(" get <path> [key] Get all secrets or specific key from path");
    println!(" Example: botserver vault get gbo/tables password");
    println!();
    println!(" list List all configured secret paths");
    println!();
    println!(" health Check Vault connection status");
    println!();
    println!("Secret paths:");
    println!(" gbo/tables Database credentials");
    println!(" gbo/drive S3/MinIO credentials");
    println!(" gbo/email SMTP credentials");
    println!(" gbo/cache Redis credentials");
    println!(" gbo/directory Zitadel credentials");
    println!(" gbo/llm AI API keys");
    println!(" gbo/encryption Encryption keys");
    println!(" gbo/stripe Payment credentials");
    println!(" gbo/vectordb Qdrant credentials");
    println!();
    println!("Environment:");
    println!(" VAULT_ADDR Vault server address");
    println!(" VAULT_TOKEN Vault authentication token");
}

pub fn vault_list() -> Result<()> {
    println!("Configured secret paths:");
    println!(" {} - Database credentials", SecretPaths::TABLES);
    println!(" {} - S3/MinIO credentials", SecretPaths::DRIVE);
    println!(" {} - Redis credentials", SecretPaths::CACHE);
    println!(" {} - SMTP credentials", SecretPaths::EMAIL);
    println!(" {} - Zitadel credentials", SecretPaths::DIRECTORY);
    println!(" {} - AI API keys", SecretPaths::LLM);
    println!(" {} - Encryption keys", SecretPaths::ENCRYPTION);
    println!(" {} - LiveKit credentials", SecretPaths::MEET);
    println!(" {} - Forgejo credentials", SecretPaths::ALM);
    println!(" {} - Qdrant credentials", SecretPaths::VECTORDB);
    println!(" {} - InfluxDB credentials", SecretPaths::OBSERVABILITY);
    println!(" gbo/stripe - Payment credentials");
    println!(" gbo/custom - Custom database");

    Ok(())
}

pub async fn print_version(show_all: bool) -> Result<()> {
    println!("botserver {}", VERSION);

    if show_all {
        println!();
        println!("Build Information:");
        println!(" rustc: {}", rustc_version());
        println!(" target: {}", std::env::consts::ARCH);
        println!(" os: {}", std::env::consts::OS);
        println!();

        let mode = InstallMode::Local;
        if let Ok(pm) = PackageManager::new(mode, None) {
            println!("Installed Components:");
            let components = pm.list();
            let mut installed_count = 0;
            for component in &components {
                if pm.is_installed(component) {
                    println!(" * {} (installed)", component);
                    installed_count += 1;
                }
            }
            if installed_count == 0 {
                println!(" (none)");
            }
            println!();
            println!("Available Components: {}", components.len());
        }

        println!();
        println!("Secrets:");
        if let Ok(manager) = SecretsManager::get() {
            if manager.is_enabled() {
                match manager.health_check().await {
                    Ok(true) => println!(" Vault: connected"),
                    _ => println!(" Vault: not reachable"),
                }
            } else {
                println!(" Vault: not configured");
            }
        } else {
            println!(" Vault: not configured");
        }
    }

    Ok(())
}

fn rustc_version() -> String {
    SafeCommand::new("rustc")
        .and_then(|c| c.arg("--version"))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn vault_migrate(env_file: &str) -> Result<()> {
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

    let manager = SecretsManager::get()?.clone();
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
        println!(" * Migrated tables credentials");
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
        println!(" * Migrated custom database credentials");
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
        println!(" * Migrated drive credentials");
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
        println!(" * Migrated email credentials");
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
        println!(" * Migrated stripe credentials");
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
        println!(" * Migrated LLM credentials");
    }

    println!();
    println!("Migration complete!");
    println!();
    println!(
        "You can now remove secrets from {} and keep only:",
        env_file
    );
    println!(" RUST_LOG=info");
    println!(" VAULT_ADDR=<vault-address>");
    println!(" VAULT_TOKEN=<vault-token>");
    println!(" SERVER_HOST=0.0.0.0");
    println!(" SERVER_PORT=5858");

    Ok(())
}
