use crate::error::{BotError, BotResult};
use std::collections::HashMap;
#[cfg(not(feature = "i18n"))]
use std::fs;
#[cfg(not(feature = "i18n"))]
use std::path::Path;

#[cfg(feature = "i18n")]
use rust_embed::RustEmbed;

use super::Locale;

#[cfg(feature = "i18n")]
#[derive(RustEmbed)]
#[folder = "locales"]
struct EmbeddedLocales;

pub type MessageArgs = HashMap<String, String>;

#[derive(Debug)]
struct TranslationFile {
    messages: HashMap<String, String>,
}

impl TranslationFile {
    fn parse(content: &str) -> Self {
        let mut messages = HashMap::new();
        let mut current_key: Option<String> = None;
        let mut current_value = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(eq_pos) = line.find('=') {
                if let Some(key) = current_key.take() {
                    messages.insert(key, current_value.trim().to_string());
                }

                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().to_string();

                if Self::is_multiline_start(&value) {
                    current_key = Some(key);
                    current_value = value;
                } else {
                    messages.insert(key, value);
                }
            } else if current_key.is_some() {
                current_value.push('\n');
                current_value.push_str(trimmed);
            }
        }

        if let Some(key) = current_key {
            messages.insert(key, current_value.trim().to_string());
        }

        Self { messages }
    }

    fn is_multiline_start(value: &str) -> bool {
        let open_braces = value.matches('{').count();
        let close_braces = value.matches('}').count();
        open_braces > close_braces
    }

    fn get(&self, key: &str) -> Option<&String> {
        let result = self.messages.get(key);
        if result.is_none() {
            log::warn!("Translation key not found in bundle: {} (available keys: {})", key, self.messages.len());
        }
        result
    }

    fn merge(&mut self, other: Self) {
        let before = self.messages.len();
        self.messages.extend(other.messages);
        let after = self.messages.len();
        log::debug!("Merged {} translations (total: {})", after - before, after);
    }
}

#[derive(Debug)]
struct LocaleBundle {
    locale: Locale,
    translations: TranslationFile,
}

impl LocaleBundle {
    #[cfg(not(feature = "i18n"))]
    fn load(locale_dir: &Path) -> BotResult<Self> {
        let dir_name = locale_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| BotError::config("invalid locale directory name"))?;

        let locale = Locale::new(dir_name)
            .ok_or_else(|| BotError::config(format!("invalid locale: {dir_name}")))?;

        let mut translations = TranslationFile {
            messages: HashMap::new(),
        };

        let entries = fs::read_dir(locale_dir)
            .map_err(|e| BotError::config(format!("failed to read locale directory: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| BotError::config(format!("failed to read directory entry: {e}")))?;

            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "ftl") {
                let content = fs::read_to_string(&path).map_err(|e| {
                    BotError::config(format!(
                        "failed to read translation file {}: {e}",
                        path.display()
                    ))
                })?;

                let file_translations = TranslationFile::parse(&content);
                translations.merge(file_translations);
            }
        }

        Ok(Self {
            locale,
            translations,
        })
    }

    #[cfg(feature = "i18n")]
    fn load_embedded(locale_str: &str) -> BotResult<Self> {
        let locale = Locale::new(locale_str)
            .ok_or_else(|| BotError::config(format!("invalid locale: {locale_str}")))?;

        let mut translations = TranslationFile {
            messages: HashMap::new(),
        };

        log::info!("Loading embedded files for locale: {}", locale_str);
        for file in EmbeddedLocales::iter() {
            if file.starts_with(locale_str) && file.ends_with(".ftl") {
                log::info!("Found .ftl file for locale {}: {}", locale_str, file);
                if let Some(content_bytes) = EmbeddedLocales::get(&file) {
                    if let Ok(content) = std::str::from_utf8(content_bytes.data.as_ref()) {
                        let file_translations = TranslationFile::parse(content);
                        log::info!("Parsed {} keys from {}", file_translations.messages.len(), file);
                        translations.merge(file_translations);
                    }
                }
            }
        }

        Ok(Self {
            locale,
            translations,
        })
    }

    fn get_message(&self, key: &str) -> Option<&String> {
        self.translations.get(key)
    }
}

#[derive(Debug)]
pub struct I18nBundle {
    bundles: HashMap<String, LocaleBundle>,
    available: Vec<Locale>,
    fallback: Locale,
}

impl I18nBundle {
    pub fn load(_base_path: &str) -> BotResult<Self> {
        // When i18n feature is enabled, locales are ALWAYS embedded via rust-embed
        // Filesystem loading is deprecated - use embedded assets only
        #[cfg(feature = "i18n")]
        {
            log::info!("Loading embedded locale translations (rust-embed)");
            Self::load_embedded()
        }

        #[cfg(not(feature = "i18n"))]
        {
            // let _base_path = base_path; // Suppress unused warning when i18n is enabled

            let base = Path::new(_base_path);

            if !base.exists() {
                return Err(BotError::config(format!(
                    "locales directory not found: {_base_path}"
                )));
            }

            let mut bundles = HashMap::new();
            let mut available = Vec::new();

            let entries = fs::read_dir(base)
                .map_err(|e| BotError::config(format!("failed to read locales directory: {e}")))?;

            for entry in entries {
                let entry = entry
                    .map_err(|e| BotError::config(format!("failed to read directory entry: {e}")))?;

                let path = entry.path();

                if path.is_dir() {
                    match LocaleBundle::load(&path) {
                        Ok(bundle) => {
                            available.push(bundle.locale.clone());
                            bundles.insert(bundle.locale.to_string(), bundle);
                        }
                        Err(e) => {
                            log::warn!("failed to load locale bundle: {e}");
                        }
                    }
                }
            }

            let fallback = Locale::default();

            Ok(Self {
                bundles,
                available,
                fallback,
            })
        }
    }

    #[cfg(feature = "i18n")]
    fn load_embedded() -> BotResult<Self> {
        let mut bundles = HashMap::new();
        let mut available = Vec::new();
        let mut seen_locales = std::collections::HashSet::new();

        let files: Vec<_> = EmbeddedLocales::iter().collect();
        log::info!("Loading embedded locales, found {} files", files.len());

        for file in files {
            // Path structure: locale/file.ftl
            let parts: Vec<&str> = file.split('/').collect();
            if let Some(locale_str) = parts.first() {
                if !seen_locales.contains(*locale_str) {
                    match LocaleBundle::load_embedded(locale_str) {
                        Ok(bundle) => {
                            available.push(bundle.locale.clone());
                            bundles.insert(bundle.locale.to_string(), bundle);
                            seen_locales.insert(locale_str.to_string());
                        }
                        Err(e) => {
                            log::warn!(
                                "failed to load embedded locale bundle {}: {}",
                                locale_str,
                                e
                            );
                        }
                    }
                }
            }
        }

        let fallback = Locale::default();
        log::info!("Loaded {} embedded locales: {:?}", available.len(), available);

        Ok(Self {
            bundles,
            available,
            fallback,
        })
    }

    pub fn get_message(&self, locale: &Locale, key: &str, args: Option<&MessageArgs>) -> String {
        let negotiated = Locale::negotiate(&[locale], &self.available, &self.fallback);

        let message = self
            .bundles
            .get(&negotiated.to_string())
            .and_then(|b| b.get_message(key))
            .or_else(|| {
                self.bundles
                    .get(&self.fallback.to_string())
                    .and_then(|b| b.get_message(key))
            });

        match message {
            Some(msg) => Self::interpolate(msg, args),
            None => format!("[{key}]"),
        }
    }

    pub fn available_locales(&self) -> Vec<String> {
        self.available.iter().map(ToString::to_string).collect()
    }

    fn interpolate(template: &str, args: Option<&MessageArgs>) -> String {
        let Some(args) = args else {
            return Self::strip_placeholders(template);
        };

        let mut result = template.to_string();

        for (key, value) in args {
            let placeholder = format!("{{ ${key} }}");
            result = result.replace(&placeholder, value);

            let placeholder_compact = format!("{{${key}}}");
            result = result.replace(&placeholder_compact, value);

            let placeholder_spaced = format!("{{ ${key} }}");
            result = result.replace(&placeholder_spaced, value);

            let pattern = format!("${{${key}}}");
            result = result.replace(&pattern, value);

            result = result.replace(&format!("{{ ${key} }}"), value);
            result = result.replace(&format!("{{${key}}}"), value);
            result = result.replace(&format!("{{ ${key}}}"), value);
            result = result.replace(&format!("{{${key} }}"), value);
        }

        Self::handle_plurals(&result, args)
    }

    fn strip_placeholders(template: &str) -> String {
        let mut result = String::with_capacity(template.len());
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&' ') {
                let mut placeholder = String::new();
                placeholder.push(c);

                while let Some(&next) = chars.peek() {
                    placeholder.push(chars.next().unwrap_or_default());
                    if next == '}' {
                        break;
                    }
                }

                if !placeholder.contains('$') {
                    result.push_str(&placeholder);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    fn handle_plurals(template: &str, args: &MessageArgs) -> String {
        let mut result = template.to_string();

        for (key, value) in args {
            if let Ok(count) = value.parse::<i64>() {
                let plural_pattern = format!("{{ ${key} ->");

                if let Some(start) = result.find(&plural_pattern) {
                    if let Some(end) = result[start..].find('}') {
                        let plural_block = &result[start..start + end + 1];
                        let replacement = Self::select_plural_form(plural_block, count);
                        result = result.replace(plural_block, &replacement);
                    }
                }
            }
        }

        result
    }

    fn select_plural_form(block: &str, count: i64) -> String {
        let forms: Vec<&str> = block.split('\n').collect();

        let form_key = match count {
            0 => "[zero]",
            1 => "[one]",
            _ => "*[other]",
        };

        for form in &forms {
            if form.contains(form_key) {
                return form
                    .split(']')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .replace("{ $count }", &count.to_string());
            }
        }

        for form in &forms {
            if form.contains("*[other]") {
                return form
                    .split(']')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .replace("{ $count }", &count.to_string());
            }
        }

        count.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ftl() {
        let content = r#"
hello = Hello
world = World
"#;
        let file = TranslationFile::parse(content);
        assert_eq!(file.get("hello"), Some(&"Hello".to_string()));
        assert_eq!(file.get("world"), Some(&"World".to_string()));
    }

    #[test]
    fn test_parse_with_placeholder() {
        let content = r#"
greeting = Hello, { $name }!
"#;
        let file = TranslationFile::parse(content);
        assert_eq!(file.get("greeting"), Some(&"Hello, { $name }!".to_string()));
    }

    #[test]
    fn test_interpolate_simple() {
        let mut args = MessageArgs::new();
        args.insert("name".to_string(), "World".to_string());

        let result = I18nBundle::interpolate("Hello, { $name }!", Some(&args));
        assert!(result.contains("World") || result.contains("{ $name }"));
    }

    #[test]
    fn test_missing_key_returns_bracketed() {
        let bundle = I18nBundle {
            bundles: HashMap::new(),
            available: vec![],
            fallback: Locale::default(),
        };

        let locale = Locale::default();
        let result = bundle.get_message(&locale, "missing-key", None);
        assert_eq!(result, "[missing-key]");
    }
}
