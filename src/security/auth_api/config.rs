#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub require_auth: bool,
    pub jwt_secret: Option<String>,
    pub api_key_header: String,
    pub bearer_prefix: String,
    pub session_cookie_name: String,
    pub allow_anonymous_paths: Vec<String>,
    pub public_paths: Vec<String>,
    pub bot_id_header: String,
    pub org_id_header: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            jwt_secret: None,
            api_key_header: "X-API-Key".to_string(),
            bearer_prefix: "Bearer ".to_string(),
            session_cookie_name: "session_id".to_string(),
            allow_anonymous_paths: vec![
                "/health".to_string(),
                "/healthz".to_string(),
                "/api/health".to_string(),
                "/.well-known".to_string(),
                "/metrics".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/bootstrap".to_string(),
                "/api/auth/refresh".to_string(),
                "/oauth".to_string(),
                "/auth/callback".to_string(),
                "/webhook/whatsapp".to_string(),
            ],
            public_paths: vec![
                "/".to_string(),
                "/static".to_string(),
                "/favicon.ico".to_string(),
                "/robots.txt".to_string(),
            ],
            bot_id_header: "X-Bot-ID".to_string(),
            org_id_header: "X-Organization-ID".to_string(),
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(secret) = std::env::var("VAULT_TOKEN") {
            if !secret.is_empty() {
                let sm = crate::core::shared::utils::get_secrets_manager_sync();
                if let Some(sm) = sm {
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build();
                        let result = if let Ok(rt) = rt {
                            rt.block_on(async { sm.get_secret(crate::core::secrets::SecretPaths::JWT).await })
                        } else {
                            Err(anyhow::anyhow!("Failed to create runtime"))
                        };
                        let _ = tx.send(result);
                    });
                    if let Ok(Ok(secrets)) = rx.recv() {
                        if let Some(s) = secrets.get("secret") {
                            config.jwt_secret = Some(s.clone());
                        }
                        if let Some(r) = secrets.get("require_auth") {
                            config.require_auth = r == "true" || r == "1";
                        }
                        if let Some(p) = secrets.get("anonymous_paths") {
                            config.allow_anonymous_paths = p
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                    }
                }
            }
        }

        config
    }

    pub fn with_jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.jwt_secret = Some(secret.into());
        self
    }

    pub fn with_require_auth(mut self, require: bool) -> Self {
        self.require_auth = require;
        self
    }

    pub fn add_anonymous_path(mut self, path: impl Into<String>) -> Self {
        self.allow_anonymous_paths.push(path.into());
        self
    }

    pub fn add_public_path(mut self, path: impl Into<String>) -> Self {
        self.public_paths.push(path.into());
        self
    }

    pub fn is_public_path(&self, path: &str) -> bool {
        for public_path in &self.public_paths {
            if path == public_path || path.starts_with(&format!("{}/", public_path)) {
                return true;
            }
        }
        false
    }

    pub fn is_anonymous_allowed(&self, path: &str) -> bool {
        for allowed_path in &self.allow_anonymous_paths {
            if path == allowed_path || path.starts_with(&format!("{}/", allowed_path)) {
                return true;
            }
        }
        false
    }
}
