//! The SaaS JWT signing secret — one implementation for every crate (#2140).
//!
//! The cloud login flow mints HS256 tokens and several crates verify them
//! (`botserver` routes, `botsettings`, `botvibe`, `botcloud`, `botemail`,
//! `botbrowserpolicy`). When each crate re-implemented the lookup, a crate that
//! kept a public fallback accepted tokens forged with a repository-visible
//! constant, and a crate that resolved a different value rejected tokens the
//! login path had just minted. This module is the only resolver:
//!
//! 1. the persisted `saas_jwt_secret` in `directory_config.json`;
//! 2. env `SAAS_JWT_SECRET`;
//! 3. env `JWT_SECRET`;
//! 4. nothing usable — generate a random secret and persist it.
//!
//! A *known placeholder* (the sample config values shipped in this repository)
//! is treated exactly like "nothing configured". The public development
//! constant is returned only by a debug build that explicitly opted in with
//! `GB_ALLOW_DEV_JWT_SECRET=1`; every other build generates a random secret.
//!
//! The generated secret is persisted atomically so the minter and every
//! verifier agree across restarts. When persistence is impossible the
//! resolution **fails closed** in production (`BOTSERVER_ENV=production`), so
//! the server refuses to start instead of serving tokens nobody can verify.

use std::path::PathBuf;
use std::sync::OnceLock;

/// Public, repository-visible development secret.
///
/// Returned only by a debug build with `GB_ALLOW_DEV_JWT_SECRET` set; never by
/// a release build, and never when a real secret is configured.
pub const DEV_FALLBACK_SECRET: &str = "dev-secret-key-change-in-production-minimum-32-chars";

/// Values shipped in sample configs. Both are public knowledge, so they are
/// treated as "no secret configured" rather than as a credential.
const KNOWN_PLACEHOLDERS: [&str; 2] = [DEV_FALLBACK_SECRET, "embedded-change-me-in-production"];

/// Opt-in that allows [`DEV_FALLBACK_SECRET`] in a debug build.
pub const ALLOW_DEV_FALLBACK_ENV: &str = "GB_ALLOW_DEV_JWT_SECRET";
/// Deployment environment flag. `production`/`prod` make resolution fail closed.
pub const ENVIRONMENT_ENV: &str = "BOTSERVER_ENV";
/// Env var holding the secret, preferred over [`ENV_JWT_SECRET`].
pub const ENV_SAAS_JWT_SECRET: &str = "SAAS_JWT_SECRET";
/// Legacy env var, still honoured so existing deployments keep working.
pub const ENV_JWT_SECRET: &str = "JWT_SECRET";
/// Key of the persisted secret inside `directory_config.json`.
pub const CONFIG_KEY: &str = "saas_jwt_secret";
/// Config file (relative to the stack path) that persists the secret.
pub const CONFIG_FILE: &str = "conf/system/directory_config.json";
/// Length of a generated secret (characters, not bytes).
const GENERATED_SECRET_LEN: usize = 48;

/// Where the resolved secret came from. Logged once at boot so an operator can
/// see whether tokens survive a restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtSecretSource {
    /// `saas_jwt_secret` in `directory_config.json`.
    ConfigFile,
    /// Env `SAAS_JWT_SECRET`.
    EnvSaasJwtSecret,
    /// Env `JWT_SECRET`.
    EnvJwtSecret,
    /// [`DEV_FALLBACK_SECRET`], debug build with the explicit opt-in.
    DevFallback,
    /// Randomly generated because nothing usable was configured.
    Generated,
}

impl JwtSecretSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConfigFile => "directory_config.json",
            Self::EnvSaasJwtSecret => ENV_SAAS_JWT_SECRET,
            Self::EnvJwtSecret => ENV_JWT_SECRET,
            Self::DevFallback => "development fallback",
            Self::Generated => "generated",
        }
    }

    /// True when the value did not come from configuration and therefore has
    /// to be persisted before the next process can verify these tokens.
    fn must_persist(self) -> bool {
        matches!(
            self,
            Self::EnvSaasJwtSecret | Self::EnvJwtSecret | Self::Generated
        )
    }
}

/// A resolved secret plus its provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtSecret {
    pub secret: String,
    pub source: JwtSecretSource,
}

impl JwtSecret {
    /// True when the value is one of the repository-visible placeholders.
    pub fn is_public_placeholder(&self) -> bool {
        is_placeholder(&self.secret)
    }
}

/// Why resolution failed. A failure means "do not serve": there is no fallback
/// value, because any fallback an attacker can read is a forgeable key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwtSecretError {
    /// A production deployment could not persist the generated secret, so the
    /// minter and the next process' verifier would disagree.
    NotPersisted { reason: String },
}

impl std::fmt::Display for JwtSecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPersisted { reason } => write!(
                f,
                "no SaaS JWT secret configured and the generated one could not be persisted \
                 ({reason}); set {ENV_SAAS_JWT_SECRET} or {ENV_JWT_SECRET}, or make {} writable",
                CONFIG_FILE
            ),
        }
    }
}

impl std::error::Error for JwtSecretError {}

/// Build-time and environment facts that decide which branch applies.
///
/// Explicit instead of read inside the resolver so tests can drive a release
/// build (`debug_build: false`) on a debug binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JwtSecretPolicy {
    pub debug_build: bool,
    pub allow_dev_fallback: bool,
    pub production: bool,
}

impl JwtSecretPolicy {
    /// The policy of the running process.
    pub fn from_env() -> Self {
        Self {
            debug_build: cfg!(debug_assertions),
            allow_dev_fallback: env_flag(ALLOW_DEV_FALLBACK_ENV),
            production: is_production_env(),
        }
    }
}

/// The three places a configured secret can come from, in priority order.
#[derive(Debug, Clone, Copy, Default)]
pub struct JwtSecretInputs<'a> {
    pub config_file_value: Option<&'a str>,
    pub env_saas_jwt_secret: Option<&'a str>,
    pub env_jwt_secret: Option<&'a str>,
}

/// True for `1`/`true`/`yes`/`on` (case-insensitive), the flag shape used
/// across the stack.
fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// `BOTSERVER_ENV=production` (or `prod`).
pub fn is_production_env() -> bool {
    std::env::var(ENVIRONMENT_ENV)
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "production" | "prod"))
        .unwrap_or(false)
}

/// A configured value, or `None` when it is absent, blank or a placeholder.
fn usable(value: Option<&str>) -> Option<&str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| !is_placeholder(value))
}

fn is_placeholder(value: &str) -> bool {
    KNOWN_PLACEHOLDERS.contains(&value.trim())
}

/// A 48-character random secret from an alphabet that is safe in a JSON string
/// and in an HTTP header.
fn generate_secret() -> String {
    use rand::Rng;
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut rng = rand::rng();
    (0..GENERATED_SECRET_LEN)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

/// Decides the secret from explicit inputs.
///
/// This is the injected resolver the tests drive: the only impurity is the
/// random generation, and the result never depends on the process environment.
pub fn resolve_secret(inputs: JwtSecretInputs<'_>, policy: JwtSecretPolicy) -> JwtSecret {
    if let Some(secret) = usable(inputs.config_file_value) {
        return JwtSecret {
            secret: secret.to_string(),
            source: JwtSecretSource::ConfigFile,
        };
    }
    if let Some(secret) = usable(inputs.env_saas_jwt_secret) {
        return JwtSecret {
            secret: secret.to_string(),
            source: JwtSecretSource::EnvSaasJwtSecret,
        };
    }
    if let Some(secret) = usable(inputs.env_jwt_secret) {
        return JwtSecret {
            secret: secret.to_string(),
            source: JwtSecretSource::EnvJwtSecret,
        };
    }
    // Nothing usable. The public constant is reachable only when the build and
    // the operator both said "this is a development box".
    if policy.debug_build && policy.allow_dev_fallback {
        return JwtSecret {
            secret: DEV_FALLBACK_SECRET.to_string(),
            source: JwtSecretSource::DevFallback,
        };
    }
    JwtSecret {
        secret: generate_secret(),
        source: JwtSecretSource::Generated,
    }
}

/// Resolves the secret and persists it when it did not come from the file, so
/// the next process (and therefore every verifier) sees the same value.
///
/// `persist` is injected so a test can simulate a read-only filesystem: a
/// production policy then fails closed instead of continuing with a secret
/// that will not exist after a restart.
pub fn resolve_with<F>(
    inputs: JwtSecretInputs<'_>,
    policy: JwtSecretPolicy,
    persist: F,
) -> Result<JwtSecret, JwtSecretError>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    let resolved = resolve_secret(inputs, policy);
    if !resolved.source.must_persist() {
        return Ok(resolved);
    }
    match persist(&resolved.secret) {
        Ok(()) => Ok(resolved),
        Err(reason) => {
            if policy.production {
                log::error!("SaaS JWT secret could not be persisted: {reason}");
                return Err(JwtSecretError::NotPersisted { reason });
            }
            // Development: the secret stays in memory for this process only,
            // so tokens minted here are unverifiable after a restart.
            log::error!(
                "SaaS JWT secret could not be persisted ({reason}); this process uses an \
                 ephemeral secret and tokens will not survive a restart"
            );
            Ok(resolved)
        }
    }
}

/// Absolute path of the config file that persists the secret.
pub fn config_file_path() -> PathBuf {
    PathBuf::from(botlib::security::get_stack_path()).join(CONFIG_FILE)
}

/// The persisted secret, or `None` when the file is missing/unreadable or has
/// no `saas_jwt_secret`.
pub fn persisted_secret() -> Option<String> {
    let content = std::fs::read_to_string(config_file_path()).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get(CONFIG_KEY)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

/// Writes the secret into the config file atomically: a temporary file in the
/// same directory is renamed over the target, so a crash never leaves a
/// half-written config behind.
fn persist_secret(secret: &str) -> Result<(), String> {
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let mut json = match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("parse {}: {e}", path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => return Err(format!("read {}: {error}", path.display())),
    };
    if !json.is_object() {
        json = serde_json::json!({});
    }
    json[CONFIG_KEY] = serde_json::Value::String(secret.to_string());
    let pretty = serde_json::to_string_pretty(&json).map_err(|e| format!("serialize: {e}"))?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, format!("{pretty}\n"))
        .map_err(|e| format!("write {}: {e}", temp.display()))?;
    std::fs::rename(&temp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("rename {}: {e}", path.display())
    })
}

/// Resolves the secret from the real environment and config file, without
/// caching. Prefer [`resolve_saas_jwt_secret`]: this one may generate a new
/// secret on each call when nothing is configured.
pub fn resolve_saas_jwt_secret_uncached() -> Result<JwtSecret, JwtSecretError> {
    let file_value = persisted_secret();
    let env_saas = std::env::var(ENV_SAAS_JWT_SECRET).ok();
    let env_jwt = std::env::var(ENV_JWT_SECRET).ok();
    let inputs = JwtSecretInputs {
        config_file_value: file_value.as_deref(),
        env_saas_jwt_secret: env_saas.as_deref(),
        env_jwt_secret: env_jwt.as_deref(),
    };
    let resolved = resolve_with(inputs, JwtSecretPolicy::from_env(), persist_secret)?;
    match resolved.source {
        JwtSecretSource::DevFallback => log::warn!(
            "Using the public development JWT secret ({ALLOW_DEV_FALLBACK_ENV} is set); \
             tokens signed with it are forgeable — set {ENV_SAAS_JWT_SECRET}"
        ),
        JwtSecretSource::Generated => log::warn!(
            "No SaaS JWT secret configured; generated one and persisted it to {}",
            config_file_path().display()
        ),
        source => log::info!("SaaS JWT secret loaded from {}", source.as_str()),
    }
    Ok(resolved)
}

/// The process-wide secret, resolved exactly once.
///
/// Caching is what keeps the minter and every verifier on the same value: a
/// generated secret must not be regenerated per call, or tokens would be
/// signed with one key and verified with another.
pub fn process_saas_jwt_secret() -> Result<&'static str, JwtSecretError> {
    static SECRET: OnceLock<Result<JwtSecret, JwtSecretError>> = OnceLock::new();
    SECRET
        .get_or_init(resolve_saas_jwt_secret_uncached)
        .as_ref()
        .map(|resolved| resolved.secret.as_str())
        .map_err(Clone::clone)
}

/// The SaaS JWT signing secret. The one function every crate calls.
pub fn resolve_saas_jwt_secret() -> Result<String, JwtSecretError> {
    process_saas_jwt_secret().map(str::to_string)
}

#[cfg(test)]
#[path = "jwt_secret_tests.rs"]
mod tests;
