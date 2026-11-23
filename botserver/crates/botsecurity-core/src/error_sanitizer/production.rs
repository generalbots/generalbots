use super::*;
use std::sync::OnceLock;

impl Default for ErrorSanitizer {
    fn default() -> Self {
        Self {
            hide_internal_errors: true,
            log_internal_errors: true,
            include_request_id: true,
            sensitive_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "apikey".to_string(),
                "authorization".to_string(),
                "credential".to_string(),
                "private".to_string(),
                "key".to_string(),
                "database".to_string(),
                "connection".to_string(),
                "dsn".to_string(),
                "postgres".to_string(),
                "mysql".to_string(),
                "redis".to_string(),
                "mongodb".to_string(),
                "aws".to_string(),
                "azure".to_string(),
                "gcp".to_string(),
                "/home/".to_string(),
                "/root/".to_string(),
                "/etc/".to_string(),
                "/var/".to_string(),
                "/opt/".to_string(),
                "/usr/".to_string(),
                "/mnt/".to_string(),
                "/srv/".to_string(),
                "/Users/".to_string(),
                "c:\\".to_string(),
                "d:\\".to_string(),
                // Provider credentials in raw errors (#2753).
                "bearer ".to_string(),
                "authorization:".to_string(),
                "access_token".to_string(),
                "refresh_token".to_string(),
                "pwd=".to_string(),
                "uid=".to_string(),
            ],
        }
    }
}
impl ErrorSanitizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn production() -> Self {
        Self {
            hide_internal_errors: true,
            log_internal_errors: true,
            include_request_id: true,
            sensitive_patterns: Self::default().sensitive_patterns,
        }
    }

    pub fn development() -> Self {
        Self {
            hide_internal_errors: false,
            log_internal_errors: true,
            include_request_id: true,
            sensitive_patterns: Self::default().sensitive_patterns,
        }
    }

    pub fn with_hide_internal(mut self, hide: bool) -> Self {
        self.hide_internal_errors = hide;
        self
    }

    pub fn with_logging(mut self, log: bool) -> Self {
        self.log_internal_errors = log;
        self
    }

    pub fn with_request_id(mut self, include: bool) -> Self {
        self.include_request_id = include;
        self
    }

    pub fn add_sensitive_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.sensitive_patterns.push(pattern.into());
        self
    }

    pub fn sanitize_error<E: std::error::Error>(
        &self,
        error: &E,
        request_id: Option<&str>,
    ) -> SafeErrorResponse {
        let error_string = error.to_string();

        if self.log_internal_errors {
            error!(
                request_id = ?request_id,
                error = %error_string,
                "Internal error occurred"
            );
        }

        if self.hide_internal_errors || self.contains_sensitive(&error_string) {
            let mut response = SafeErrorResponse::internal_error();
            if self.include_request_id {
                if let Some(rid) = request_id {
                    response = response.with_request_id(rid);
                }
            }
            response
        } else {
            let sanitized = self.sanitize_message(&error_string);
            let mut response = SafeErrorResponse::new("error", sanitized);
            if self.include_request_id {
                if let Some(rid) = request_id {
                    response = response.with_request_id(rid);
                }
            }
            response
        }
    }

    pub fn sanitize_message(&self, message: &str) -> String {
        let mut result = message.to_string();

        for pattern in &self.sensitive_patterns {
            if result.to_lowercase().contains(&pattern.to_lowercase()) {
                result = redact_around_pattern(&result, pattern);
            }
        }

        result = redact_stack_traces(&result);
        result = redact_file_paths(&result);
        result = redact_ip_addresses(&result);
        result = redact_connection_strings(&result);
        result = redact_credentials(&result);

        result
    }

    pub fn contains_sensitive(&self, message: &str) -> bool {
        let lower = message.to_lowercase();
        for pattern in &self.sensitive_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }

        if looks_like_stack_trace(message) {
            return true;
        }

        if looks_like_connection_string(message) {
            return true;
        }

        if looks_like_internal_path(message) {
            return true;
        }

        if looks_like_credential(message) {
            return true;
        }

        false
    }

    pub fn safe_response_for_status(
        &self,
        status: StatusCode,
        request_id: Option<&str>,
    ) -> SafeErrorResponse {
        let mut response = match status {
            StatusCode::BAD_REQUEST => SafeErrorResponse::bad_request("Invalid request"),
            StatusCode::UNAUTHORIZED => SafeErrorResponse::unauthorized(),
            StatusCode::FORBIDDEN => SafeErrorResponse::forbidden(),
            StatusCode::NOT_FOUND => SafeErrorResponse::not_found("Resource"),
            StatusCode::METHOD_NOT_ALLOWED => SafeErrorResponse::method_not_allowed(&[]),
            StatusCode::CONFLICT => SafeErrorResponse::conflict("Resource conflict"),
            StatusCode::GONE => SafeErrorResponse::gone("Resource no longer available"),
            StatusCode::PAYLOAD_TOO_LARGE => SafeErrorResponse::payload_too_large(None),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => SafeErrorResponse::unsupported_media_type(&[]),
            StatusCode::TOO_MANY_REQUESTS => SafeErrorResponse::rate_limited(None),
            StatusCode::INTERNAL_SERVER_ERROR => SafeErrorResponse::internal_error(),
            StatusCode::SERVICE_UNAVAILABLE => SafeErrorResponse::service_unavailable(),
            StatusCode::GATEWAY_TIMEOUT => SafeErrorResponse::timeout(),
            _ => SafeErrorResponse::new(
                format!("error_{}", status.as_u16()),
                status.canonical_reason().unwrap_or("An error occurred"),
            ),
        };

        if self.include_request_id {
            if let Some(rid) = request_id {
                response = response.with_request_id(rid);
            }
        }

        response
    }
}
/// Redacts from the first occurrence of `pattern` through the end of the token
/// that follows it (`password=abc rest` becomes `[REDACTED] rest`).
///
/// The match is case-insensitive and the cuts are taken on char boundaries of
/// `text` itself. The search deliberately does not lower-case a copy and reuse
/// its offsets: `str::to_lowercase` can change the byte length (`İ` becomes two
/// code points), which made `&text[..start]` panic with "byte index N is not a
/// char boundary" — from inside the sanitizer every error path depends on.
///
/// An empty pattern is ignored; it used to redact the first token of the text.
pub(super) fn redact_around_pattern(text: &str, pattern: &str) -> String {
    if pattern.is_empty() {
        return text.to_string();
    }

    // Both offsets come from `text` and land on char boundaries, so the slices
    // below are always valid.
    let Some(found) = botlib::text::find_ignore_case(text, pattern) else {
        return text.to_string();
    };

    let (before, _) = text.split_at(found.start);
    let (_, after) = text.split_at(token_end(text, found.end));
    format!("{before}[REDACTED]{after}")
}

/// The byte offset just past the token starting at `from`: every character up
/// to the next whitespace, comma or semicolon, or the end of `text`.
fn token_end(text: &str, from: usize) -> usize {
    let Some(rest) = text.get(from..) else {
        return text.len();
    };

    for (offset, ch) in rest.char_indices() {
        if ch.is_whitespace() || ch == ',' || ch == ';' {
            return from + offset;
        }
    }

    text.len()
}
/// Redaction patterns. They are constants and the sanitizer runs on every
/// `log_and_sanitize` call, so compiling them per call (22 `Regex::new` calls
/// per sanitisation, on a path with 13 call sites) is pure waste: they are
/// compiled once per process instead.
const STACK_TRACE_PATTERNS: &[&str] = &[
    r"at .+:\d+:\d+",
    r"File .+, line \d+",
    r"\s+at .+\(.+\)",
    r"\.rs:\d+",
    r"\.go:\d+",
    r"\.py:\d+",
    r"\.java:\d+",
    r"\.js:\d+",
    r"\.ts:\d+",
];

const FILE_PATH_PATTERNS: &[&str] = &[
    r"/[a-zA-Z0-9_\-./]+\.(rs|go|py|java|js|ts|rb|php|c|cpp|h)",
    r"[A-Z]:\\[a-zA-Z0-9_\-\\]+\.\w+",
    r"/home/[a-zA-Z0-9_\-/]+",
    r"/root/[a-zA-Z0-9_\-/]+",
    r"/var/[a-zA-Z0-9_\-/]+",
    r"/etc/[a-zA-Z0-9_\-/]+",
    r"/opt/[a-zA-Z0-9_\-/]+",
    r"/usr/[a-zA-Z0-9_\-/]+",
    r"/mnt/[a-zA-Z0-9_\-/]+",
    r"/srv/[a-zA-Z0-9_\-/]+",
    r"/Users/[a-zA-Z0-9_\-/]+",
];

const IP_ADDRESS_PATTERN: &str = r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b";

// Kept in sync with CONNECTION_SCHEMES in looks_like_connection_string.rs.
const CONNECTION_STRING_PATTERNS: &[&str] = &[
    r"(?i)(postgres|postgresql|mysql|mariadb|mongodb\+srv|mongodb|redis|rediss|valkey|amqps?|kafka(\+ssl)?|ldaps?|ftps?|sftp|smb|https?)://[^\s]+",
    r"(?i)jdbc:[^\s]+",
    // Keyword DSNs (ODBC/ADO.NET/libpq) have no scheme; redact the whole
    // `Key=Value;...` chain once a host/server key is present (#2753).
    r"(?i)\b(server|data source|addr|host)\s*=[^;]+;(?:[^;]+;){1,}",
];

// Bearer tokens, auth headers and inline credentials returned by providers
// (`Graph 401: Bearer eyJ...`) are as sensitive as a DB password (#2753).
const CREDENTIAL_PATTERNS: &[&str] = &[
    r"(?i)\bbearer\s+[^\s,;]+",
    r"(?i)\bauthorization\s*:\s*[^\s,;]+",
    r"(?i)\b(password|passwd|pwd|secret|token|api[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret)\s*[=:]\s*[^\s,;]+",
    r"\beyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+",
];

/// Compiles `patterns` into `cache` on first use. A pattern that fails to
/// compile is logged and skipped, never panicking on a path that exists to
/// report errors.
fn compiled_patterns(
    cache: &'static OnceLock<Vec<regex::Regex>>,
    patterns: &[&str],
) -> &'static [regex::Regex] {
    cache.get_or_init(|| {
        patterns
            .iter()
            .filter_map(|pattern| match regex::Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(e) => {
                    log::error!("error_sanitizer pattern failed to compile ({pattern}): {e}");
                    None
                }
            })
            .collect()
    })
}

fn stack_trace_res() -> &'static [regex::Regex] {
    static CACHE: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    compiled_patterns(&CACHE, STACK_TRACE_PATTERNS)
}

fn file_path_res() -> &'static [regex::Regex] {
    static CACHE: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    compiled_patterns(&CACHE, FILE_PATH_PATTERNS)
}

fn ip_address_re() -> Option<&'static regex::Regex> {
    static CACHE: OnceLock<Option<regex::Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| match regex::Regex::new(IP_ADDRESS_PATTERN) {
            Ok(re) => Some(re),
            Err(e) => {
                log::error!("error_sanitizer IP pattern failed to compile: {e}");
                None
            }
        })
        .as_ref()
}

fn connection_string_res() -> &'static [regex::Regex] {
    static CACHE: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    compiled_patterns(&CACHE, CONNECTION_STRING_PATTERNS)
}

fn credential_res() -> &'static [regex::Regex] {
    static CACHE: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    compiled_patterns(&CACHE, CREDENTIAL_PATTERNS)
}

pub(super) fn redact_stack_traces(text: &str) -> String {
    let mut result = text.to_string();
    for re in stack_trace_res() {
        result = re
            .replace_all(&result, "[STACK_TRACE_REDACTED]")
            .to_string();
    }
    result
}
pub(super) fn redact_file_paths(text: &str) -> String {
    let mut result = text.to_string();
    for re in file_path_res() {
        result = re.replace_all(&result, "[PATH_REDACTED]").to_string();
    }
    result
}
pub(super) fn redact_ip_addresses(text: &str) -> String {
    match ip_address_re() {
        Some(re) => re.replace_all(text, "[IP_REDACTED]").to_string(),
        None => text.to_string(),
    }
}
pub(super) fn redact_connection_strings(text: &str) -> String {
    let mut result = text.to_string();
    for re in connection_string_res() {
        result = re
            .replace_all(&result, "[CONNECTION_STRING_REDACTED]")
            .to_string();
    }
    result
}

pub(super) fn redact_credentials(text: &str) -> String {
    let mut result = text.to_string();
    for re in credential_res() {
        result = re.replace_all(&result, "[CREDENTIAL_REDACTED]").to_string();
    }
    result
}
pub(super) fn looks_like_stack_trace(text: &str) -> bool {
    let indicators = [
        "at line",
        "stack trace",
        "backtrace",
        "panic:",
        "Traceback",
        "Exception in thread",
        "    at ",
        "Caused by:",
    ];

    let lower = text.to_lowercase();
    for indicator in &indicators {
        if lower.contains(&indicator.to_lowercase()) {
            return true;
        }
    }

    false
}
