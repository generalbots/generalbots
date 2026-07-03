use std::collections::HashMap;
use std::sync::RwLock;

/// Registro central de feature flags para funcionalidades SOON (planejadas
/// mas ainda não implementadas). Controlado via config.csv ou variáveis de
/// ambiente. Permite ativar/desativar funcionalidades em desenvolvimento
/// sem precisar recompilar o binário.
///
/// # Exemplo de uso
/// ```rust,ignore
/// let flags = FeatureFlags::new();
/// flags.set_enabled("huge_file_streaming", true);
/// assert!(flags.is_enabled("huge_file_streaming"));
/// ```
pub struct FeatureFlags {
    flags: RwLock<HashMap<String, bool>>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        let mut flags = HashMap::new();
        flags.insert("instagram_campaigns".to_string(), false);
        flags.insert("whatsapp_business_api".to_string(), false);
        flags.insert("desktop1_container".to_string(), false);
        flags.insert("ext4_luks".to_string(), false);
        flags.insert("skills_library".to_string(), false);
        flags.insert("setup_wizard".to_string(), true);
        flags.insert("version_dashboard".to_string(), false);
        flags.insert("huge_file_streaming".to_string(), false);
        flags.insert("log_rotation".to_string(), false);
        flags.insert("bot_access_restriction".to_string(), false);
        flags.insert("global_css".to_string(), false);
        flags.insert("preview_keyword".to_string(), false);
        flags.insert("session_pool".to_string(), false);
        Self {
            flags: RwLock::new(flags),
        }
    }

    pub fn is_enabled(&self, feature: &str) -> bool {
        self.flags
            .read()
            .ok()
            .and_then(|f| f.get(feature).copied())
            .unwrap_or(false)
    }

    pub fn set_enabled(&self, feature: &str, enabled: bool) {
        if let Ok(mut flags) = self.flags.write() {
            flags.insert(feature.to_string(), enabled);
        }
    }

    pub fn list_all(&self) -> Vec<(String, bool)> {
        self.flags
            .read()
            .ok()
            .map(|f| {
                let mut list: Vec<_> = f.iter().map(|(k, v)| (k.clone(), *v)).collect();
                list.sort_by(|a, b| a.0.cmp(&b.0));
                list
            })
            .unwrap_or_default()
    }

    pub fn enabled_features(&self) -> Vec<String> {
        self.list_all()
            .into_iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn count_enabled(&self) -> usize {
        self.enabled_features().len()
    }

    pub fn load_from_env(&self, prefix: &str) {
        for (feature, _) in self.list_all() {
            let env_key = format!("{}{}", prefix, feature.to_uppercase());
            if let Ok(val) = std::env::var(&env_key) {
                let enabled = val == "1" || val.to_lowercase() == "true";
                self.set_enabled(&feature, enabled);
            }
        }
    }

    pub fn load_from_csv(&self, config_str: &str) {
        for line in config_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("key,") {
                continue;
            }
            if let Some((key, value)) = line.split_once(',') {
                let key = key.trim();
                let value = value.trim();
                if key.starts_with("feature_") {
                    let feature_name = key.trim_start_matches("feature_");
                    let enabled = value == "1" || value.to_lowercase() == "true";
                    self.set_enabled(feature_name, enabled);
                }
            }
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        let flags = FeatureFlags::new();
        assert!(!flags.is_enabled("instagram_campaigns"));
    }

    #[test]
    fn test_enable_disable() {
        let flags = FeatureFlags::new();
        assert!(!flags.is_enabled("huge_file_streaming"));
        flags.set_enabled("huge_file_streaming", true);
        assert!(flags.is_enabled("huge_file_streaming"));
        flags.set_enabled("huge_file_streaming", false);
        assert!(!flags.is_enabled("huge_file_streaming"));
    }

    #[test]
    fn test_list_all() {
        let flags = FeatureFlags::new();
        let list = flags.list_all();
        assert!(!list.is_empty());
        assert!(list.iter().any(|(k, _)| k == "setup_wizard"));
    }

    #[test]
    fn test_unknown_feature() {
        let flags = FeatureFlags::new();
        assert!(!flags.is_enabled("nonexistent_feature"));
    }

    #[test]
    fn test_load_from_env() {
        std::env::set_var("GB_FEATURE_SETUP_WIZARD", "true");
        let flags = FeatureFlags::new();
        flags.load_from_env("GB_FEATURE_");
        assert!(flags.is_enabled("setup_wizard"));
        std::env::remove_var("GB_FEATURE_SETUP_WIZARD");
    }

    #[test]
    fn test_load_from_csv() {
        let flags = FeatureFlags::new();
        let csv = "feature_huge_file_streaming,true\nfeature_log_rotation,false\n";
        flags.load_from_csv(csv);
        assert!(flags.is_enabled("huge_file_streaming"));
        assert!(!flags.is_enabled("log_rotation"));
    }
}
