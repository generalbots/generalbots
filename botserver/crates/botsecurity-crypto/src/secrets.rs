use std::fmt;

#[derive(Default)]
pub struct SecretString {
    inner: String,
}

impl SecretString {
    pub fn new(secret: String) -> Self {
        Self { inner: secret }
    }
}

impl std::str::FromStr for SecretString {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: s.to_string(),
        })
    }
}

impl SecretString {

    pub fn expose_secret(&self) -> &str {
        &self.inner
    }

    pub fn expose_secret_mut(&mut self) -> &mut String {
        &mut self.inner
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        zeroize_string(&mut self.inner);
    }
}

impl Clone for SecretString {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}


impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_default()
    }
}

#[derive(Default)]
pub struct SecretBytes {
    inner: Vec<u8>,
}

impl SecretBytes {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { inner: secret }
    }

    pub fn expose_secret(&self) -> &[u8] {
        &self.inner
    }

    pub fn expose_secret_mut(&mut self) -> &mut Vec<u8> {
        &mut self.inner
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Drop for SecretBytes {
    fn drop(&mut self) {
        zeroize_bytes(&mut self.inner);
    }
}

impl Clone for SecretBytes {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED BYTES]")
    }
}


impl From<Vec<u8>> for SecretBytes {
    fn from(v: Vec<u8>) -> Self {
        Self::new(v)
    }
}

impl From<&[u8]> for SecretBytes {
    fn from(s: &[u8]) -> Self {
        Self::new(s.to_vec())
    }
}

#[derive(Clone)]
pub struct ApiKey {
    key: SecretString,
    provider: String,
}

impl ApiKey {
    pub fn new(key: impl Into<SecretString>, provider: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            provider: provider.into(),
        }
    }

    pub fn expose_key(&self) -> &str {
        self.key.expose_secret()
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }

    pub fn masked(&self) -> String {
        let key = self.key.expose_secret();
        if key.len() <= 8 {
            return "*".repeat(key.len());
        }
        format!(
            "{}...{}",
            &key[..4],
            &key[key.len() - 4..]
        )
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKey")
            .field("key", &"[REDACTED]")
            .field("provider", &self.provider)
            .finish()
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey({}, {})", self.provider, self.masked())
    }
}

#[derive(Clone)]
pub struct DatabaseCredentials {
    username: String,
    password: SecretString,
    host: String,
    port: u16,
    database: String,
}

impl DatabaseCredentials {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<SecretString>,
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            host: host.into(),
            port,
            database: database.into(),
        }
    }

    pub fn from_url(url: &str) -> Option<Self> {
        let url = url.strip_prefix("postgres://")?;
        let (auth, rest) = url.split_once('@')?;
        let (username, password) = auth.split_once(':')?;
        let (host_port, database) = rest.split_once('/')?;

        let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
            (h.to_string(), p.parse().ok()?)
        } else {
            (host_port.to_string(), 5432)
        };

        let database = database.split('?').next()?.to_string();

        Some(Self {
            username: username.to_string(),
            password: password.parse().unwrap_or_default(),
            host,
            port,
            database,
        })
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn expose_password(&self) -> &str {
        self.password.expose_secret()
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub fn to_connection_string(&self) -> SecretString {
        SecretString::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        ))
    }

    pub fn to_safe_string(&self) -> String {
        format!(
            "postgres://{}:****@{}:{}/{}",
            self.username, self.host, self.port, self.database
        )
    }
}

impl fmt::Debug for DatabaseCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .finish()
    }
}

impl fmt::Display for DatabaseCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_safe_string())
    }
}

#[derive(Clone)]
pub struct JwtSecret {
    secret: SecretBytes,
    algorithm: String,
}

impl JwtSecret {
    pub fn new(secret: impl Into<SecretBytes>, algorithm: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            algorithm: algorithm.into(),
        }
    }

    pub fn expose_secret(&self) -> &[u8] {
        self.secret.expose_secret()
    }

    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }
}

impl fmt::Debug for JwtSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtSecret")
            .field("secret", &"[REDACTED]")
            .field("algorithm", &self.algorithm)
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct SecretsStore {
    api_keys: std::collections::HashMap<String, ApiKey>,
    database_credentials: Option<DatabaseCredentials>,
    jwt_secret: Option<JwtSecret>,
    custom_secrets: std::collections::HashMap<String, SecretString>,
}

impl SecretsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_api_key(&mut self, name: impl Into<String>, key: ApiKey) {
        self.api_keys.insert(name.into(), key);
    }

    pub fn get_api_key(&self, name: &str) -> Option<&ApiKey> {
        self.api_keys.get(name)
    }

    pub fn remove_api_key(&mut self, name: &str) -> Option<ApiKey> {
        self.api_keys.remove(name)
    }

    pub fn set_database_credentials(&mut self, creds: DatabaseCredentials) {
        self.database_credentials = Some(creds);
    }

    pub fn database_credentials(&self) -> Option<&DatabaseCredentials> {
        self.database_credentials.as_ref()
    }

    pub fn set_jwt_secret(&mut self, secret: JwtSecret) {
        self.jwt_secret = Some(secret);
    }

    pub fn jwt_secret(&self) -> Option<&JwtSecret> {
        self.jwt_secret.as_ref()
    }

    pub fn add_custom_secret(&mut self, name: impl Into<String>, secret: impl Into<SecretString>) {
        self.custom_secrets.insert(name.into(), secret.into());
    }

    pub fn get_custom_secret(&self, name: &str) -> Option<&SecretString> {
        self.custom_secrets.get(name)
    }

    pub fn remove_custom_secret(&mut self, name: &str) -> Option<SecretString> {
        self.custom_secrets.remove(name)
    }

    pub fn clear(&mut self) {
        self.api_keys.clear();
        self.database_credentials = None;
        self.jwt_secret = None;
        self.custom_secrets.clear();
    }
}

impl fmt::Debug for SecretsStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretsStore")
            .field("api_keys_count", &self.api_keys.len())
            .field("has_database_credentials", &self.database_credentials.is_some())
            .field("has_jwt_secret", &self.jwt_secret.is_some())
            .field("custom_secrets_count", &self.custom_secrets.len())
            .finish()
    }
}

#[inline(never)]
fn zeroize_string(s: &mut String) {
    // Overwrite with zeros using safe code
    let len = s.len();
    s.clear();
    s.reserve(len);
    for _ in 0..len {
        s.push('\0');
    }
    s.clear();
}

#[inline(never)]
fn zeroize_bytes(v: &mut Vec<u8>) {
    // Overwrite with zeros using safe code
    for byte in v.iter_mut() {
        *byte = 0;
    }
    v.clear();
}

pub fn redact_sensitive_data(text: &str) -> String {
    let patterns = [
        (r"password[=:]\s*\S+", "password=[REDACTED]"),
        (r"api[_-]?key[=:]\s*\S+", "api_key=[REDACTED]"),
        (r"token[=:]\s*\S+", "token=[REDACTED]"),
        (r"secret[=:]\s*\S+", "secret=[REDACTED]"),
        (r"Bearer\s+\S+", "Bearer [REDACTED]"),
        (r"Basic\s+\S+", "Basic [REDACTED]"),
    ];

    let mut result = text.to_string();
    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(&format!("(?i){}", pattern)) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }
    result
}

pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    let sensitive_keywords = [
        "password",
        "passwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "auth",
        "credential",
        "private",
        "key",
        "cert",
        "certificate",
    ];

    sensitive_keywords.iter().any(|kw| lower.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_redaction() -> Result<(), Box<dyn std::error::Error>> {
        let secret = SecretString::new("my-super-secret-password".to_string());
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
        assert_eq!(format!("{}", secret), "[REDACTED]");
    }

    #[test]
    fn test_secret_string_expose() {
        let secret = SecretString::new("my-password".to_string());
        assert_eq!(secret.expose_secret(), "my-password");
    }

    #[test]
    fn test_secret_bytes_redaction() {
        let secret = SecretBytes::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(format!("{:?}", secret), "[REDACTED BYTES]");
    }

    #[test]
    fn test_api_key_masked() {
        let key = ApiKey::new("sk-1234567890abcdef1234567890abcdef", "openai");
        let masked = key.masked();
        assert!(masked.starts_with("sk-1"));
        assert!(masked.ends_with("cdef"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn test_api_key_short() {
        let key = ApiKey::new("short", "test");
        let masked = key.masked();
        assert_eq!(masked, "*****");
    }

    #[test]
    fn test_database_credentials_from_url() -> Result<(), Box<dyn std::error::Error>> {
        let url = "postgres://user:pass@localhost:5432/mydb";
        let creds = DatabaseCredentials::from_url(url).ok_or("Failed to parse URL")?;
        assert_eq!(creds.username(), "user");
        assert_eq!(creds.expose_password(), "pass");
        assert_eq!(creds.host(), "localhost");
        assert_eq!(creds.port(), 5432);
        assert_eq!(creds.database(), "mydb");
        Ok(())
    }

    #[test]
    fn test_database_credentials_safe_string() {
        let creds = DatabaseCredentials::new("user", "secret", "localhost", 5432, "db");
        let safe = creds.to_safe_string();
        assert!(!safe.contains("secret"));
        assert!(safe.contains("****"));
    }

    #[test]
    fn test_redact_sensitive_data() {
        let text = "password=secret123 and api_key=abc123";
        let redacted = redact_sensitive_data(text);
        assert!(!redacted.contains("secret123"));
        assert!(!redacted.contains("abc123"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_is_sensitive_key() {
        assert!(is_sensitive_key("password"));
        assert!(is_sensitive_key("API_KEY"));
        assert!(is_sensitive_key("secret_token"));
        assert!(is_sensitive_key("db_password"));
        assert!(!is_sensitive_key("username"));
        assert!(!is_sensitive_key("email"));
    }

    #[test]
    fn test_secrets_store() {
        let mut store = SecretsStore::new();

        store.add_api_key("openai", ApiKey::new("sk-test", "openai"));
        assert!(store.get_api_key("openai").is_some());
        assert!(store.get_api_key("nonexistent").is_none());

        store.add_custom_secret("my_secret", "value");
        assert!(store.get_custom_secret("my_secret").is_some());

        store.clear();
        assert!(store.get_api_key("openai").is_none());
    }

    #[test]
    fn test_secret_string_default() {
        let secret: SecretString = Default::default();
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn test_secret_bytes_from() {
        let bytes: SecretBytes = vec![1, 2, 3].into();
        assert_eq!(bytes.len(), 3);

        let bytes2: SecretBytes = [4u8, 5, 6].as_slice().into();
        assert_eq!(bytes2.len(), 3);
    }
}
