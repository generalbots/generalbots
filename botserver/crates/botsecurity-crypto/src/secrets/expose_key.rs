use super::*;

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
        let chars: Vec<char> = key.chars().collect();
        if chars.len() <= 8 {
            return "*".repeat(chars.len());
        }

        let first: String = chars[..4].iter().collect();
        let last: String = chars[chars.len() - 4..].iter().collect();

        format!("{first}...{last}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_long_ascii_keys_at_the_edges() {
        let key = ApiKey::new("sk-1234567890abcdef1234567890abcdef", "openai");
        assert_eq!(key.masked(), "sk-1...cdef");
    }

    #[test]
    fn masks_short_keys_completely() {
        assert_eq!(ApiKey::new("short", "test").masked(), "*****");
        assert_eq!(ApiKey::new("", "test").masked(), "");
    }

    #[test]
    fn masks_multibyte_keys_without_splitting_characters() {
        let key = ApiKey::new("áááááááááá", "test");
        let masked = key.masked();

        assert_eq!(masked, "áááá...áááá");
        assert_eq!(masked.chars().count(), 11);
    }

    #[test]
    fn debug_and_display_never_expose_the_key() {
        let key = ApiKey::new("sk-super-secret-value", "openai");

        assert!(!format!("{key:?}").contains("super-secret"));
        assert!(!format!("{key}").contains("super-secret"));
        assert!(format!("{key}").contains("openai"));
    }
}
