use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    language: String,
    region: Option<String>,
}

impl Locale {
    pub fn new(locale_str: &str) -> Option<Self> {
        if locale_str.is_empty() {
            return None;
        }

        let parts: Vec<&str> = locale_str.split(&['-', '_'][..]).collect();

        let language = parts.first()?.to_lowercase();
        if language.len() < 2 || language.len() > 3 {
            return None;
        }

        let region = parts.get(1).map(|r| r.to_uppercase());

        Some(Self { language, region })
    }

    pub fn from_parts(language: &str, region: Option<&str>) -> Option<Self> {
        if language.is_empty() || language.len() < 2 || language.len() > 3 {
            return None;
        }

        Some(Self {
            language: language.to_lowercase(),
            region: region.map(|r| r.to_uppercase()),
        })
    }

    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    #[must_use]
    pub fn to_string_with_separator(&self, separator: char) -> String {
        match &self.region {
            Some(r) => format!("{}{separator}{r}", self.language),
            None => self.language.clone(),
        }
    }

    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        if self.language != other.language {
            return false;
        }

        match (&self.region, &other.region) {
            (Some(a), Some(b)) => a == b,
            (None, _) | (_, None) => true,
        }
    }

    pub fn negotiate<'a>(
        requested: &[&'a Locale],
        available: &'a [Locale],
        fallback: &'a Locale,
    ) -> &'a Locale {
        for req in requested {
            for avail in available {
                if req.language == avail.language && req.region == avail.region {
                    return avail;
                }
            }
        }

        for req in requested {
            for avail in available {
                if req.language == avail.language {
                    return avail;
                }
            }
        }

        for avail in available {
            if avail == fallback {
                return avail;
            }
        }

        available.first().unwrap_or(fallback)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            region: None,
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.region {
            Some(r) => write!(f, "{}-{r}", self.language),
            None => write!(f, "{}", self.language),
        }
    }
}

impl TryFrom<&str> for Locale {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("invalid locale string")
    }
}

impl TryFrom<String> for Locale {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value).ok_or("invalid locale string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_locale() {
        let locale = Locale::new("en").unwrap();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), None);
    }

    #[test]
    fn test_parse_locale_with_region() {
        let locale = Locale::new("pt-BR").unwrap();
        assert_eq!(locale.language(), "pt");
        assert_eq!(locale.region(), Some("BR"));
    }

    #[test]
    fn test_parse_locale_with_underscore() {
        let locale = Locale::new("zh_CN").unwrap();
        assert_eq!(locale.language(), "zh");
        assert_eq!(locale.region(), Some("CN"));
    }

    #[test]
    fn test_locale_display() {
        let locale = Locale::new("pt-BR").unwrap();
        assert_eq!(locale.to_string(), "pt-BR");

        let locale_simple = Locale::new("en").unwrap();
        assert_eq!(locale_simple.to_string(), "en");
    }

    #[test]
    fn test_locale_matches() {
        let en = Locale::new("en").unwrap();
        let en_us = Locale::new("en-US").unwrap();
        let en_gb = Locale::new("en-GB").unwrap();
        let pt_br = Locale::new("pt-BR").unwrap();

        assert!(en.matches(&en_us));
        assert!(en.matches(&en_gb));
        assert!(!en_us.matches(&en_gb));
        assert!(!en.matches(&pt_br));
    }

    #[test]
    fn test_default_locale() {
        let locale = Locale::default();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), None);
    }

    #[test]
    fn test_invalid_locale() {
        assert!(Locale::new("").is_none());
        assert!(Locale::new("x").is_none());
    }

    #[test]
    fn test_negotiate_exact_match() {
        let requested = Locale::new("pt-BR").unwrap();
        let available = vec![
            Locale::new("en").unwrap(),
            Locale::new("pt-BR").unwrap(),
            Locale::new("es").unwrap(),
        ];
        let fallback = Locale::default();

        let result = Locale::negotiate(&[&requested], &available, &fallback);
        assert_eq!(result.to_string(), "pt-BR");
    }

    #[test]
    fn test_negotiate_language_match() {
        let requested = Locale::new("pt-PT").unwrap();
        let available = vec![
            Locale::new("en").unwrap(),
            Locale::new("pt-BR").unwrap(),
            Locale::new("es").unwrap(),
        ];
        let fallback = Locale::default();

        let result = Locale::negotiate(&[&requested], &available, &fallback);
        assert_eq!(result.language(), "pt");
    }

    #[test]
    fn test_negotiate_fallback() {
        let requested = Locale::new("ja").unwrap();
        let available = vec![
            Locale::new("en").unwrap(),
            Locale::new("pt-BR").unwrap(),
        ];
        let fallback = Locale::new("en").unwrap();

        let result = Locale::negotiate(&[&requested], &available, &fallback);
        assert_eq!(result.language(), "en");
    }
}
