use crate::manager::SecretsManager;
use crate::paths::SecretPaths;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

pub struct TenantSecrets;

impl SecretsManager {
    pub fn get_directory_config_sync(&self) -> (String, String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::DIRECTORY).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_default(),
                secrets.get("project_id").cloned().unwrap_or_default(),
                secrets.get("client_id").cloned().unwrap_or_default(),
                secrets.get("client_secret").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), String::new(), String::new(), String::new())
    }

    pub fn get_email_config(&self) -> (String, u16, String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::EMAIL).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("smtp_host").cloned().unwrap_or_default(),
                secrets.get("smtp_port").and_then(|p| p.parse().ok()).unwrap_or(587),
                secrets.get("smtp_user").cloned().unwrap_or_default(),
                secrets.get("smtp_password").cloned().unwrap_or_default(),
                secrets.get("smtp_from").cloned().unwrap_or_default(),
            );
        }
        (String::new(), 587, String::new(), String::new(), String::new())
    }

    pub fn get_llm_config(&self) -> (String, String, Option<String>, Option<String>, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::LLM).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_default(),
                secrets.get("model").cloned().unwrap_or_else(|| "gpt-4".into()),
                secrets.get("openai_key").cloned(),
                secrets.get("anthropic_key").cloned(),
                secrets.get("ollama_url").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), "gpt-4".to_string(), None, None, "".to_string())
    }

    pub fn get_meet_config(&self) -> (String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::MEET).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_default(),
                secrets.get("app_id").cloned().unwrap_or_default(),
                secrets.get("app_secret").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), String::new(), String::new())
    }

    pub fn get_vectordb_config_sync(&self) -> (String, Option<String>) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::VECTORDB).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (secrets.get("url").cloned().unwrap_or_default(), secrets.get("api_key").cloned());
        }
        ("".to_string(), None)
    }

    pub fn get_observability_config_sync(&self) -> (String, String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::OBSERVABILITY).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_default(),
                secrets.get("org").cloned().unwrap_or_else(|| "system".into()),
                secrets.get("bucket").cloned().unwrap_or_else(|| "metrics".into()),
                secrets.get("token").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), "system".to_string(), "metrics".to_string(), String::new())
    }

    pub fn get_alm_config(&self) -> (String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::ALM).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_default(),
                secrets.get("token").cloned().unwrap_or_default(),
                secrets.get("default_org").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), String::new(), String::new())
    }

    pub fn get_jwt_secret_sync(&self) -> String {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { self_owned.get_secret(SecretPaths::JWT).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return secrets.get("secret").cloned().unwrap_or_default();
        }
        String::new()
    }

    pub fn get_email_config_for_bot_sync(&self, bot_id: &Uuid) -> (String, u16, String, String, String) {
        let bot_path = format!("gbo/bots/{}/email", bot_id);
        let default_path = "gbo/bots/default/email".to_string();
        let paths = vec![bot_path, default_path, SecretPaths::EMAIL.to_string()];

        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    for path in paths {
                        if let Ok(secrets) = self_owned.get_secret(&path).await {
                            if !secrets.is_empty() && secrets.contains_key("smtp_from") {
                                return Some((
                                    secrets.get("smtp_host").cloned().unwrap_or_default(),
                                    secrets.get("smtp_port").and_then(|p| p.parse().ok()).unwrap_or(587),
                                    secrets.get("smtp_user").cloned().unwrap_or_default(),
                                    secrets.get("smtp_password").cloned().unwrap_or_default(),
                                    secrets.get("smtp_from").cloned().unwrap_or_default(),
                                ));
                            }
                        }
                    }
                    None
                })
            } else { None };
            let _ = tx.send(result);
        });

        if let Ok(Some(config)) = rx.recv() {
            return config;
        }
        (String::new(), 587, String::new(), String::new(), String::new())
    }

    // ============ TENANT INFRASTRUCTURE ============

    pub async fn get_database_config_for_tenant(&self, tenant: &str) -> Result<(String, u16, String, String, String)> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/tables", tenant_path)).await {
            return Ok((
                s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
                s.get("port").and_then(|p| p.parse().ok()).unwrap_or(5432),
                s.get("database").cloned().unwrap_or_else(|| "botserver".into()),
                s.get("username").cloned().unwrap_or_else(|| "gbuser".into()),
                s.get("password").cloned().unwrap_or_default(),
            ));
        }
        self.get_database_config().await
    }

    pub async fn get_drive_config_for_tenant(&self, tenant: &str) -> Result<(String, String, String, String)> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/drive", tenant_path)).await {
            return Ok((
                s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
                s.get("port").cloned().unwrap_or_else(|| "9100".into()),
                s.get("accesskey").cloned().unwrap_or_default(),
                s.get("secret").cloned().unwrap_or_default(),
            ));
        }
        let s = self.get_secret(SecretPaths::DRIVE).await?;
        Ok((
            s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
            s.get("port").cloned().unwrap_or_else(|| "9100".into()),
            s.get("accesskey").cloned().unwrap_or_default(),
            s.get("secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_cache_config_for_tenant(&self, tenant: &str) -> Result<(String, u16, Option<String>)> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/cache", tenant_path)).await {
            return Ok((
                s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
                s.get("port").and_then(|p| p.parse().ok()).unwrap_or(6379),
                s.get("password").cloned(),
            ));
        }
        let url_val = self.get_secret(SecretPaths::CACHE).await?.get("url").cloned();
        let host = url_val
            .as_ref()
            .map(|u| u.split("://").nth(1).unwrap_or("localhost").split(':').next().unwrap_or("localhost"))
            .unwrap_or("localhost")
            .to_string();
        let port = url_val.as_ref().and_then(|u| u.split(':').nth(1)).and_then(|p| p.parse().ok()).unwrap_or(6379);
        Ok((host, port, None))
    }

    pub async fn get_smtp_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/email", tenant_path)).await { return Ok(s); }
        self.get_secret(SecretPaths::EMAIL).await
    }

    pub async fn get_llm_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/llm", tenant_path)).await { return Ok(s); }
        self.get_secret(SecretPaths::LLM).await
    }

    pub async fn get_directory_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/directory", tenant_path)).await { return Ok(s); }
        self.get_secret(SecretPaths::DIRECTORY).await
    }

    pub async fn get_models_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/models", tenant_path)).await { return Ok(s); }
        self.get_secret(SecretPaths::MODELS).await
    }

    // ============ ORG BOT/USER SECRETS ============

    pub async fn get_bot_email_config(&self, org_id: &str, bot_id: &str) -> Result<HashMap<String, String>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        if let Ok(s) = self.get_secret(&format!("{}/email", path)).await { return Ok(s); }
        self.get_secret(SecretPaths::EMAIL).await
    }

    pub async fn get_bot_whatsapp_config(&self, org_id: &str, bot_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        Ok(self.get_secret(&format!("{}/whatsapp", path)).await.ok())
    }

    pub async fn get_bot_llm_config(&self, org_id: &str, bot_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        Ok(self.get_secret(&format!("{}/llm", path)).await.ok())
    }

    pub async fn get_bot_api_keys_config(&self, org_id: &str, bot_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        Ok(self.get_secret(&format!("{}/api-keys", path)).await.ok())
    }

    pub async fn get_user_email_config(&self, org_id: &str, user_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_user(org_id, user_id);
        Ok(self.get_secret(&format!("{}/email", path)).await.ok())
    }

    pub async fn get_user_oauth_config(&self, org_id: &str, user_id: &str, provider: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_user(org_id, user_id);
        Ok(self.get_secret(&format!("{}/oauth/{}", path, provider)).await.ok())
    }

    pub fn get_tenant_id_for_org(&self, org_id: Uuid) -> Result<String> {
        let _ = org_id;
        Ok("default".to_string())
    }
}
