//! `botserver vault migrate` — moves `.env` secrets into Vault (#2094).
//!
//! Only the progress lines an operator reads are translated; the secret paths
//! and environment names stay verbatim so they can be copied into a shell.

use super::*;

use crate::i18n::cli_locale::{t, t_args};

pub(super) async fn vault_migrate(env_file: &str) -> Result<()> {
    println!("{}", t_args("cli-vault-migrating", &[("file", env_file)]));

    let content = std::fs::read_to_string(env_file).map_err(|e| {
        anyhow::anyhow!(
            "{}",
            t_args(
                "cli-vault-read-failed",
                &[("file", env_file), ("error", &e.to_string())]
            )
        )
    })?;

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
        return Err(anyhow::anyhow!("{}", t("cli-vault-not-configured")));
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
        report_migrated("cli-secret-tables");
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
        report_migrated("cli-secret-custom");
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
        report_migrated("cli-secret-drive");
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
        report_migrated("cli-secret-email");
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
        report_migrated("cli-secret-stripe");
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
        report_migrated("cli-secret-llm");
    }

    println!();
    println!("{}", t("cli-vault-migration-complete"));
    println!();
    println!(
        "{}",
        t_args("cli-vault-now-remove", &[("file", env_file)])
    );
    println!("  RUST_LOG=info");
    println!("  VAULT_ADDR=<vault-address>");
    println!("  VAULT_TOKEN=<vault-token>");
    println!("  SERVER_HOST=0.0.0.0");
    println!("  SERVER_PORT=5858");

    Ok(())
}

fn report_migrated(what_key: &str) {
    println!(
        "  * {}",
        t_args("cli-vault-migrated", &[("what", &t(what_key))])
    );
}
