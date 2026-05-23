/// Configuration for the `mention-class` property in config.csv.
///
/// Controls which entity types the LLM can `@mention` when generating
/// responses. Parsed from a comma-separated string, e.g.:
///   `crms,kbs,websites`
///
/// When a class is absent from the string the corresponding mentions
/// are **disabled**.
#[derive(Debug, Clone)]
pub struct MentionConfig {
    /// Allow `@crm` / `@contact` mentions.
    pub crms: bool,
    /// Allow `@kb` / `@knowledge` mentions.
    pub kbs: bool,
    /// Allow `@site` / `@url` mentions.
    pub websites: bool,
}

impl Default for MentionConfig {
    fn default() -> Self {
        Self { crms: true, kbs: true, websites: true }
    }
}

impl MentionConfig {
    /// Parse from a config.csv field value.
    ///
    /// Special values:
    /// - Empty string or `"all"` → enable everything (default).
    /// - Any other value → only the listed classes are enabled.
    pub fn from_string(s: &str) -> Self {
        let s = s.trim().to_lowercase();
        if s.is_empty() || s == "all" {
            return Self::default();
        }
        let classes: Vec<&str> = s.split(',').map(|c| c.trim()).collect();
        Self {
            crms: classes.contains(&"crms"),
            kbs: classes.contains(&"kbs"),
            websites: classes.contains(&"websites"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_all_enabled() {
        let cfg = MentionConfig::default();
        assert!(cfg.crms);
        assert!(cfg.kbs);
        assert!(cfg.websites);
    }

    #[test]
    fn test_empty_string_returns_default() {
        let cfg = MentionConfig::from_string("");
        assert!(cfg.crms);
        assert!(cfg.kbs);
        assert!(cfg.websites);
    }

    #[test]
    fn test_all_string_returns_default() {
        let cfg = MentionConfig::from_string("all");
        assert!(cfg.crms);
        assert!(cfg.kbs);
        assert!(cfg.websites);
    }

    #[test]
    fn test_single_class() {
        let cfg = MentionConfig::from_string("crms");
        assert!(cfg.crms);
        assert!(!cfg.kbs);
        assert!(!cfg.websites);
    }

    #[test]
    fn test_multiple_classes() {
        let cfg = MentionConfig::from_string("crms, kbs");
        assert!(cfg.crms);
        assert!(cfg.kbs);
        assert!(!cfg.websites);
    }

    #[test]
    fn test_case_insensitive() {
        let cfg = MentionConfig::from_string("CRMS, KBS");
        assert!(cfg.crms);
        assert!(cfg.kbs);
        assert!(!cfg.websites);
    }

    #[test]
    fn test_all_disabled() {
        let cfg = MentionConfig::from_string("nonexistent");
        assert!(!cfg.crms);
        assert!(!cfg.kbs);
        assert!(!cfg.websites);
    }
}
