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
    pub internal_token: Option<String>,
    pub internal_token_header: String,
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
                "/api/cloud/auth/login".to_string(),
                "/api/cloud/auth/signup".to_string(),
                "/api/cloud/auth".to_string(),
                "/oauth".to_string(),
                "/auth/callback".to_string(),
                "/webhook/whatsapp".to_string(),
                // Host→bot lookup used by the UI server (botui) when rendering
                // the suite for a domain/subdomain before any user auth exists.
                "/api/domains/resolve".to_string(),
            ],
            public_paths: vec![
                "/static".to_string(),
                "/favicon.ico".to_string(),
                "/robots.txt".to_string(),
            ],
            bot_id_header: "X-Bot-ID".to_string(),
            org_id_header: "X-Organization-ID".to_string(),
            internal_token: None,
            internal_token_header: "X-Internal-Token".to_string(),
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
                if let Some(provider) = botsecurity_core::get_vault_provider() {
                    if let Ok(jwt_secret) = provider.get_jwt_secret() {
                        config.jwt_secret = Some(jwt_secret);
                    }
                }
            }
        }

        if let Ok(token) = std::env::var("INTERNAL_API_TOKEN") {
            if !token.is_empty() {
                config.internal_token = Some(token);
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
        self.public_paths.iter().any(|p| path_matches(p, path))
    }

    pub fn is_anonymous_allowed(&self, path: &str) -> bool {
        self.allow_anonymous_paths
            .iter()
            .any(|p| path_matches(p, path))
    }
}

/// Match `path` against a configured `pattern`.
///
/// * A `pattern` of `"/"` matches everything.
/// * Without a `*`, the legacy prefix semantics apply: exact equality or a
///   `pattern/` prefix (so `/api/bots` matches `/api/bots/list` but not
///   `/api/bots-config`).
/// * A `pattern` containing `*` is a simple glob where `*` matches any
///   sequence of characters (including none). This scopes anonymous access to
///   a single route shape — e.g. `/api/bots/*/access` matches
///   `/api/bots/foo/access` but not `/api/bots/list` or
///   `/api/bots/foo/config`.
fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == "/" {
        return true;
    }
    if !pattern.contains('*') {
        return path == pattern || path.starts_with(&format!("{}/", pattern));
    }

    let segments: Vec<&str> = pattern.split('*').collect();
    let first = segments.first().copied().unwrap_or("");
    let last = segments.last().copied().unwrap_or("");

    if !path.starts_with(first) {
        return false;
    }
    if !last.is_empty() && !path.ends_with(last) {
        return false;
    }

    // Middle literals must appear in order between the prefix and suffix.
    let mut cursor = first.len();
    let middle_count = segments.len().saturating_sub(1);
    for seg in segments.iter().take(middle_count).skip(1) {
        if seg.is_empty() {
            continue;
        }
        match path[cursor..].find(seg) {
            Some(rel) => cursor += rel + seg.len(),
            None => return false,
        }
    }
    true
}
