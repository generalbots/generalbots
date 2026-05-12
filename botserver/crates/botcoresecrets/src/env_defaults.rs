use crate::manager::SecretsManager;
use anyhow::Result;
use std::collections::HashMap;

pub fn init_secrets_manager() -> Result<SecretsManager> {
    SecretsManager::from_env()
}

pub fn get_from_env(path: &str) -> Result<HashMap<String, String>> {
    let mut secrets = HashMap::new();

    let normalized = if path.starts_with("gbo/system/") {
        path.strip_prefix("gbo/system/").unwrap_or(path)
    } else {
        path
    };

    match normalized {
        "tables" | "gbo/tables" | "system/tables" => {
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "5432".into());
            secrets.insert("database".into(), "botserver".into());
            secrets.insert("username".into(), "gbuser".into());
            secrets.insert("password".into(), "changeme".into());
        }
        "directory" | "gbo/directory" | "system/directory" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "9000".into());
            secrets.insert("project_id".into(), String::new());
            secrets.insert("client_id".into(), String::new());
            secrets.insert("client_secret".into(), String::new());
        }
        "drive" | "gbo/drive" | "system/drive" => {
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "9000".into());
            secrets.insert("accesskey".into(), "minioadmin".into());
            secrets.insert("secret".into(), "minioadmin".into());
        }
        "cache" | "gbo/cache" | "system/cache" => {
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "6379".into());
            secrets.insert("password".into(), String::new());
        }
        "email" | "gbo/email" | "system/email" => {
            secrets.insert("smtp_host".into(), String::new());
            secrets.insert("smtp_port".into(), "587".into());
            secrets.insert("smtp_user".into(), String::new());
            secrets.insert("smtp_password".into(), String::new());
            secrets.insert("smtp_from".into(), String::new());
        }
        "llm" | "gbo/llm" | "system/llm" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("model".into(), "gpt-4".into());
            secrets.insert("openai_key".into(), String::new());
            secrets.insert("anthropic_key".into(), String::new());
            secrets.insert("ollama_url".into(), String::new());
        }
        "encryption" | "gbo/encryption" | "system/encryption" => {
            secrets.insert("master_key".into(), String::new());
        }
        "meet" | "gbo/meet" | "system/meet" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("app_id".into(), String::new());
            secrets.insert("app_secret".into(), String::new());
        }
        "vectordb" | "gbo/vectordb" | "system/vectordb" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "6333".into());
            secrets.insert("grpc_port".into(), "6334".into());
            secrets.insert("api_key".into(), String::new());
        }
        "observability" | "gbo/observability" | "system/observability" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("org".into(), "system".into());
            secrets.insert("bucket".into(), "metrics".into());
            secrets.insert("token".into(), String::new());
        }
        "alm" | "gbo/alm" | "system/alm" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("token".into(), String::new());
            secrets.insert("default_org".into(), String::new());
        }
        "security" | "gbo/security" | "system/security" => {
            secrets.insert("require_auth".into(), "true".into());
            secrets.insert("anonymous_paths".into(), String::new());
        }
        "cloud" | "gbo/cloud" | "system/cloud" => {
            secrets.insert("region".into(), "us-east-1".into());
            secrets.insert("access_key".into(), String::new());
            secrets.insert("secret_key".into(), String::new());
        }
        "app" | "gbo/app" | "system/app" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("environment".into(), "development".into());
        }
        "jwt" | "gbo/jwt" | "system/jwt" => {
            secrets.insert("secret".into(), String::new());
        }
        "models" | "gbo/models" | "system/models" => {
            secrets.insert("url".into(), String::new());
        }
        _ => {
            log::debug!("No default values for secret path: {}", path);
        }
    }

    Ok(secrets)
}
