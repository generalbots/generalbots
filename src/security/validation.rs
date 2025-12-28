use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    Required(String),
    TooShort { field: String, min: usize, actual: usize },
    TooLong { field: String, max: usize, actual: usize },
    InvalidFormat { field: String, expected: String },
    InvalidRange { field: String, min: String, max: String },
    InvalidValue { field: String, message: String },
    InvalidEmail(String),
    InvalidUrl(String),
    InvalidUuid(String),
    InvalidPhone(String),
    Forbidden { field: String, reason: String },
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required(field) => write!(f, "Field '{}' is required", field),
            Self::TooShort { field, min, actual } => {
                write!(f, "Field '{}' is too short: {} < {} chars", field, actual, min)
            }
            Self::TooLong { field, max, actual } => {
                write!(f, "Field '{}' is too long: {} > {} chars", field, actual, max)
            }
            Self::InvalidFormat { field, expected } => {
                write!(f, "Field '{}' has invalid format, expected: {}", field, expected)
            }
            Self::InvalidRange { field, min, max } => {
                write!(f, "Field '{}' must be between {} and {}", field, min, max)
            }
            Self::InvalidValue { field, message } => {
                write!(f, "Field '{}' has invalid value: {}", field, message)
            }
            Self::InvalidEmail(email) => write!(f, "Invalid email address: {}", email),
            Self::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            Self::InvalidUuid(uuid) => write!(f, "Invalid UUID: {}", uuid),
            Self::InvalidPhone(phone) => write!(f, "Invalid phone number: {}", phone),
            Self::Forbidden { field, reason } => {
                write!(f, "Field '{}' is forbidden: {}", field, reason)
            }
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Default)]
pub struct ValidationResult {
    errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn into_errors(self) -> Vec<ValidationError> {
        self.errors
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
    }

    pub fn to_error_messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.to_string()).collect()
    }
}

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).expect("Invalid email regex")
});

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(/[-a-zA-Z0-9()@:%_\+.~#?&/=]*)?$"
    ).expect("Invalid URL regex")
});

static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    ).expect("Invalid UUID regex")
});

static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\+?[1-9]\d{6,14}$").expect("Invalid phone regex")
});

static ALPHANUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9]+$").expect("Invalid alphanumeric regex")
});

static SLUG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("Invalid slug regex")
});

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]{2,31}$").expect("Invalid username regex")
});

pub fn validate_required<'a>(value: Option<&'a str>, field_name: &str) -> Result<&'a str, ValidationError> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(ValidationError::Required(field_name.to_string())),
    }
}

pub fn validate_string_required(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::Required(field_name.to_string()))
    } else {
        Ok(())
    }
}

pub fn validate_length(
    value: &str,
    field_name: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> Result<(), ValidationError> {
    let len = value.len();

    if let Some(min_len) = min {
        if len < min_len {
            return Err(ValidationError::TooShort {
                field: field_name.to_string(),
                min: min_len,
                actual: len,
            });
        }
    }

    if let Some(max_len) = max {
        if len > max_len {
            return Err(ValidationError::TooLong {
                field: field_name.to_string(),
                max: max_len,
                actual: len,
            });
        }
    }

    Ok(())
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.len() > 254 {
        return Err(ValidationError::InvalidEmail(email.to_string()));
    }

    if EMAIL_REGEX.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail(email.to_string()))
    }
}

pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.len() > 2048 {
        return Err(ValidationError::InvalidUrl(url.to_string()));
    }

    if URL_REGEX.is_match(url) {
        Ok(())
    } else {
        Err(ValidationError::InvalidUrl(url.to_string()))
    }
}

pub fn validate_uuid(uuid: &str) -> Result<(), ValidationError> {
    if UUID_REGEX.is_match(uuid) {
        Ok(())
    } else {
        Err(ValidationError::InvalidUuid(uuid.to_string()))
    }
}

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();

    if PHONE_REGEX.is_match(&digits) {
        Ok(())
    } else {
        Err(ValidationError::InvalidPhone(phone.to_string()))
    }
}

pub fn validate_alphanumeric(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if ALPHANUMERIC_REGEX.is_match(value) {
        Ok(())
    } else {
        Err(ValidationError::InvalidFormat {
            field: field_name.to_string(),
            expected: "alphanumeric characters only".to_string(),
        })
    }
}

pub fn validate_slug(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if SLUG_REGEX.is_match(value) {
        Ok(())
    } else {
        Err(ValidationError::InvalidFormat {
            field: field_name.to_string(),
            expected: "lowercase alphanumeric with hyphens".to_string(),
        })
    }
}

pub fn validate_username(value: &str) -> Result<(), ValidationError> {
    if USERNAME_REGEX.is_match(value) {
        Ok(())
    } else {
        Err(ValidationError::InvalidFormat {
            field: "username".to_string(),
            expected: "3-32 chars, starting with letter, alphanumeric with _ and -".to_string(),
        })
    }
}

pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    field_name: &str,
    min: Option<T>,
    max: Option<T>,
) -> Result<(), ValidationError> {
    let min_str = min.as_ref().map(|m| m.to_string()).unwrap_or_else(|| "-∞".to_string());
    let max_str = max.as_ref().map(|m| m.to_string()).unwrap_or_else(|| "∞".to_string());

    if let Some(ref min_val) = min {
        if value < *min_val {
            return Err(ValidationError::InvalidRange {
                field: field_name.to_string(),
                min: min_str,
                max: max_str,
            });
        }
    }

    if let Some(ref max_val) = max {
        if value > *max_val {
            return Err(ValidationError::InvalidRange {
                field: field_name.to_string(),
                min: min_str,
                max: max_str,
            });
        }
    }

    Ok(())
}

pub fn validate_one_of<T: PartialEq + std::fmt::Debug>(
    value: &T,
    field_name: &str,
    allowed: &[T],
) -> Result<(), ValidationError> {
    if allowed.contains(value) {
        Ok(())
    } else {
        Err(ValidationError::InvalidValue {
            field: field_name.to_string(),
            message: format!("must be one of {:?}", allowed),
        })
    }
}

pub fn validate_not_in<T: PartialEq + std::fmt::Debug>(
    value: &T,
    field_name: &str,
    forbidden: &[T],
    reason: &str,
) -> Result<(), ValidationError> {
    if forbidden.contains(value) {
        Err(ValidationError::Forbidden {
            field: field_name.to_string(),
            reason: reason.to_string(),
        })
    } else {
        Ok(())
    }
}

pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::TooShort {
            field: "password".to_string(),
            min: 8,
            actual: password.len(),
        });
    }

    if password.len() > 128 {
        return Err(ValidationError::TooLong {
            field: "password".to_string(),
            max: 128,
            actual: password.len(),
        });
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let strength_score = [has_uppercase, has_lowercase, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if strength_score < 3 {
        return Err(ValidationError::InvalidValue {
            field: "password".to_string(),
            message: "must contain at least 3 of: uppercase, lowercase, digit, special char".to_string(),
        });
    }

    Ok(())
}

pub fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn strip_html_tags(input: &str) -> String {
    static HTML_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"<[^>]*>").expect("Invalid HTML tag regex")
    });

    HTML_TAG_REGEX.replace_all(input, "").to_string()
}

pub fn validate_no_html(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if value.contains('<') || value.contains('>') {
        Err(ValidationError::Forbidden {
            field: field_name.to_string(),
            reason: "HTML tags are not allowed".to_string(),
        })
    } else {
        Ok(())
    }
}

pub fn validate_no_script_injection(value: &str, field_name: &str) -> Result<(), ValidationError> {
    let lower = value.to_lowercase();

    let dangerous_patterns = [
        "javascript:",
        "vbscript:",
        "data:",
        "onclick",
        "onerror",
        "onload",
        "onmouseover",
        "onfocus",
        "onblur",
        "<script",
        "</script",
        "eval(",
        "expression(",
    ];

    for pattern in dangerous_patterns {
        if lower.contains(pattern) {
            return Err(ValidationError::Forbidden {
                field: field_name.to_string(),
                reason: format!("Potentially dangerous pattern '{}' detected", pattern),
            });
        }
    }

    Ok(())
}

pub struct Validator {
    result: ValidationResult,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            result: ValidationResult::new(),
        }
    }

    pub fn required(mut self, value: Option<&str>, field_name: &str) -> Self {
        if let Err(e) = validate_required(value, field_name) {
            self.result.add_error(e);
        }
        self
    }

    pub fn string_required(mut self, value: &str, field_name: &str) -> Self {
        if let Err(e) = validate_string_required(value, field_name) {
            self.result.add_error(e);
        }
        self
    }

    pub fn length(mut self, value: &str, field_name: &str, min: Option<usize>, max: Option<usize>) -> Self {
        if let Err(e) = validate_length(value, field_name, min, max) {
            self.result.add_error(e);
        }
        self
    }

    pub fn email(mut self, value: &str) -> Self {
        if let Err(e) = validate_email(value) {
            self.result.add_error(e);
        }
        self
    }

    pub fn url(mut self, value: &str) -> Self {
        if let Err(e) = validate_url(value) {
            self.result.add_error(e);
        }
        self
    }

    pub fn uuid(mut self, value: &str) -> Self {
        if let Err(e) = validate_uuid(value) {
            self.result.add_error(e);
        }
        self
    }

    pub fn phone(mut self, value: &str) -> Self {
        if let Err(e) = validate_phone(value) {
            self.result.add_error(e);
        }
        self
    }

    pub fn alphanumeric(mut self, value: &str, field_name: &str) -> Self {
        if let Err(e) = validate_alphanumeric(value, field_name) {
            self.result.add_error(e);
        }
        self
    }

    pub fn slug(mut self, value: &str, field_name: &str) -> Self {
        if let Err(e) = validate_slug(value, field_name) {
            self.result.add_error(e);
        }
        self
    }

    pub fn username(mut self, value: &str) -> Self {
        if let Err(e) = validate_username(value) {
            self.result.add_error(e);
        }
        self
    }

    pub fn password(mut self, value: &str) -> Self {
        if let Err(e) = validate_password_strength(value) {
            self.result.add_error(e);
        }
        self
    }

    pub fn no_html(mut self, value: &str, field_name: &str) -> Self {
        if let Err(e) = validate_no_html(value, field_name) {
            self.result.add_error(e);
        }
        self
    }

    pub fn no_script(mut self, value: &str, field_name: &str) -> Self {
        if let Err(e) = validate_no_script_injection(value, field_name) {
            self.result.add_error(e);
        }
        self
    }

    pub fn custom<F>(mut self, validation_fn: F) -> Self
    where
        F: FnOnce() -> Option<ValidationError>,
    {
        if let Some(error) = validation_fn() {
            self.result.add_error(error);
        }
        self
    }

    pub fn validate(self) -> Result<(), ValidationResult> {
        if self.result.is_valid() {
            Ok(())
        } else {
            Err(self.result)
        }
    }

    pub fn result(self) -> ValidationResult {
        self.result
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required() {
        assert!(validate_required(Some("value"), "field").is_ok());
        assert!(validate_required(Some(""), "field").is_err());
        assert!(validate_required(None, "field").is_err());
        assert!(validate_required(Some("  "), "field").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", "field", Some(1), Some(10)).is_ok());
        assert!(validate_length("hi", "field", Some(3), None).is_err());
        assert!(validate_length("hello world", "field", None, Some(5)).is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://sub.domain.com/path?query=1").is_ok());
        assert!(validate_url("ftp://invalid.com").is_err());
        assert!(validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_validate_uuid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("550e8400-e29b-41d4-a716").is_err());
    }

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("+1234567890").is_ok());
        assert!(validate_phone("1234567890").is_ok());
        assert!(validate_phone("123").is_err());
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("123user").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(validate_password_strength("SecurePass1!").is_ok());
        assert!(validate_password_strength("Weak1!").is_err());
        assert!(validate_password_strength("alllowercase").is_err());
        assert!(validate_password_strength("ALLUPPERCASE").is_err());
    }

    #[test]
    fn test_sanitize_html() {
        assert_eq!(sanitize_html("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(sanitize_html("Hello & World"), "Hello &amp; World");
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html_tags("<script>bad</script>text"), "badtext");
    }

    #[test]
    fn test_validate_no_script_injection() {
        assert!(validate_no_script_injection("normal text", "field").is_ok());
        assert!(validate_no_script_injection("javascript:alert(1)", "field").is_err());
        assert!(validate_no_script_injection("<script>bad</script>", "field").is_err());
        assert!(validate_no_script_injection("onclick=hack", "field").is_err());
    }

    #[test]
    fn test_validator_chain() {
        let result = Validator::new()
            .string_required("test", "name")
            .length("test", "name", Some(1), Some(100))
            .no_html("test", "name")
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_validator_with_errors() {
        let result = Validator::new()
            .string_required("", "name")
            .email("invalid-email")
            .validate();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.errors().len(), 2);
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::Required("username".to_string());
        assert!(err.to_string().contains("username"));
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("my-slug", "field").is_ok());
        assert!(validate_slug("slug123", "field").is_ok());
        assert!(validate_slug("My-Slug", "field").is_err());
        assert!(validate_slug("slug_bad", "field").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, "count", Some(1), Some(10)).is_ok());
        assert!(validate_range(0, "count", Some(1), None).is_err());
        assert!(validate_range(100, "count", None, Some(50)).is_err());
    }

    #[test]
    fn test_validate_one_of() {
        assert!(validate_one_of(&"admin", "role", &["admin", "user", "guest"]).is_ok());
        assert!(validate_one_of(&"hacker", "role", &["admin", "user", "guest"]).is_err());
    }
}
