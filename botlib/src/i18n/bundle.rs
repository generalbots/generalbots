use std::collections::HashMap;

use super::translation_parser::LocaleBundle;
use super::{Locale, MessageArgs};
use crate::error::BotResult;

#[derive(Debug)]
pub struct I18nBundle {
    bundles: HashMap<String, LocaleBundle>,
    available: Vec<Locale>,
    fallback: Locale,
}

impl I18nBundle {
    pub fn load(_base_path: &str) -> BotResult<Self> {
        #[cfg(feature = "i18n")]
        {
            log::info!("Loading embedded locale translations (rust-embed)");
            Self::load_embedded()
        }

        #[cfg(not(feature = "i18n"))]
        {
            let base = std::path::Path::new(_base_path);

            if !base.exists() {
                return Err(crate::error::BotError::config(format!(
                    "locales directory not found: {_base_path}"
                )));
            }

            let mut bundles = HashMap::new();
            let mut available = Vec::new();

            let entries = std::fs::read_dir(base)
                .map_err(|e| crate::error::BotError::config(format!("failed to read locales directory: {e}")))?;

            for entry in entries {
                let entry = entry
                    .map_err(|e| crate::error::BotError::config(format!("failed to read directory entry: {e}")))?;

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

        let files: Vec<_> = super::translation_parser::list_embedded_files();
        log::info!("Loading embedded locales, found {} files", files.len());

        for file in files {
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
            result = result.replace(&format!("{{ ${key} }}"), value);
            result = result.replace(&format!("{{${key}}}"), value);
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
    use super::translation_parser::TranslationFile;
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
