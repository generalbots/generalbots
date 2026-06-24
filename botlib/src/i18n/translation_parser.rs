#[cfg(feature = "i18n")]
use rust_embed::RustEmbed;
use std::collections::HashMap;

use super::Locale;
use crate::error::{BotError, BotResult};

#[cfg(feature = "i18n")]
#[derive(RustEmbed)]
#[folder = "locales"]
struct EmbeddedLocales;

#[cfg(feature = "i18n")]
pub fn list_embedded_files() -> Vec<String> {
    EmbeddedLocales::iter().map(|s| s.to_string()).collect()
}

pub type MessageArgs = HashMap<String, String>;

#[derive(Debug)]
pub struct TranslationFile {
    messages: HashMap<String, String>,
}

impl TranslationFile {
    pub fn parse(content: &str) -> Self {
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

    pub fn get(&self, key: &str) -> Option<&String> {
        let result = self.messages.get(key);
        if result.is_none() {
            log::warn!("Translation key not found in bundle: {} (available keys: {})", key, self.messages.len());
        }
        result
    }

    pub fn merge(&mut self, other: Self) {
        let before = self.messages.len();
        self.messages.extend(other.messages);
        let after = self.messages.len();
        log::debug!("Merged {} translations (total: {})", after - before, after);
    }
}

#[derive(Debug)]
pub struct LocaleBundle {
    pub locale: Locale,
    pub translations: TranslationFile,
}

impl LocaleBundle {
    #[cfg(not(feature = "i18n"))]
    pub fn load(locale_dir: &std::path::Path) -> BotResult<Self> {
        let dir_name = locale_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| BotError::config("invalid locale directory name"))?;

        let locale = Locale::new(dir_name)
            .ok_or_else(|| BotError::config(format!("invalid locale: {dir_name}")))?;

        let mut translations = TranslationFile {
            messages: HashMap::new(),
        };

        let entries = std::fs::read_dir(locale_dir)
            .map_err(|e| BotError::config(format!("failed to read locale directory: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| BotError::config(format!("failed to read directory entry: {e}")))?;

            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "ftl") {
                let content = std::fs::read_to_string(&path).map_err(|e| {
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
    pub fn load_embedded(locale_str: &str) -> BotResult<Self> {
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

    pub fn get_message(&self, key: &str) -> Option<&String> {
        self.translations.get(key)
    }
}
