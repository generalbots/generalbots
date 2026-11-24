use super::production::looks_like_stack_trace;
use super::*;

pub(super) fn looks_like_connection_string(text: &str) -> bool {
    let indicators = [
        "://",
        "host=",
        "dbname=",
        "user=",
        "password=",
        "port=",
        "sslmode=",
        // Keyword/DSN forms (ODBC, ADO.NET, libpq) carry the same secrets as a
        // URL but use `Key=Value;` pairs instead of a scheme (#2753).
        "server=",
        "data source=",
        "initial catalog=",
        "database=",
        "user id=",
        "uid=",
        "pwd=",
        "trusted_connection",
        "connect_timeout=",
    ];

    let lower = text.to_lowercase();

    // A scheme-qualified URL is a connection string on its own: it carries the
    // host, and frequently the credentials (`postgres://user:pass@host/db`).
    // Counting indicators alone missed it, because such a string usually
    // contains only `://`.
    // Keep this list in sync with CONNECTION_STRING_PATTERNS in production.rs:
    // a scheme we detect here but do not redact there is a leak, and vice versa.
    const CONNECTION_SCHEMES: &[&str] = &[
        "postgres://",
        "postgresql://",
        "mysql://",
        "mariadb://",
        "mongodb://",
        "mongodb+srv://",
        "redis://",
        "rediss://",
        "valkey://",
        "amqp://",
        "amqps://",
        "kafka://",
        "kafka+ssl://",
        "ldap://",
        "ldaps://",
        "jdbc:",
        "ftp://",
        "ftps://",
        "sftp://",
        "smb://",
        "http://",
        "https://",
    ];
    if CONNECTION_SCHEMES
        .iter()
        .any(|scheme| lower.contains(scheme))
    {
        return true;
    }

    let count = indicators.iter().filter(|i| lower.contains(*i)).count();
    count >= 2
}

/// True when the payload embeds a filesystem path that is internal to the
/// deployment. Error strings like `at /opt/gbo/bin/db.rs:42` or
/// `C:\srv\app\config.rs` expose layout and usernames and must never reach a
/// client (#2753).
pub(super) fn looks_like_internal_path(text: &str) -> bool {
    const PATH_MARKERS: &[&str] = &[
        "/home/", "/root/", "/etc/", "/var/", "/opt/", "/usr/", "/mnt/", "/srv/", "/Users/",
        "c:\\", "d:\\", "\\\\",
    ];
    let lower = text.to_lowercase();
    PATH_MARKERS.iter().any(|marker| lower.contains(marker)) || looks_like_stack_trace(text)
}

/// True when the payload embeds an auth credential: a bearer token, an
/// `Authorization` header or a JSON Web Token. These routinely appear in
/// provider errors (`Graph API returned 401: Bearer eyJ...`) and are as
/// sensitive as a database password (#2753).
pub(super) fn looks_like_credential(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("bearer ")
        || lower.contains("authorization:")
        || lower.contains("authorization =")
        || lower.contains("api-key:")
        || lower.contains("api_key=")
        || lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower.contains("eyj")
}
pub fn sanitize_for_log(message: &str) -> String {
    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_message(message)
}
pub fn safe_error<E: std::error::Error>(error: E) -> SafeErrorResponse {
    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_error(&error, None)
}
pub fn safe_error_with_request_id<E: std::error::Error>(
    error: E,
    request_id: &str,
) -> SafeErrorResponse {
    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_error(&error, Some(request_id))
}
pub fn log_and_sanitize<E: std::error::Error>(
    error: &E,
    context: &str,
    request_id: Option<&str>,
) -> SafeErrorResponse {
    warn!(
        context = %context,
        request_id = ?request_id,
        error = %error,
        "Error occurred"
    );

    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_error(error, request_id)
}
/// [`log_and_sanitize`] as a JSON body, ready to be wrapped in `Json(...)`.
///
/// Handler arms that must keep the concrete type of a sibling success arm (a
/// `Json<serde_json::Value>` tuple, typically) cannot return
/// `Json<SafeErrorResponse>`. This is the shared shape those call sites use, so
/// the mandated sanitizer stays a one-liner and no new handler needs to decide
/// what to leak (#1503).
///
/// The parameter is deliberately `Display` rather than `std::error::Error`: the
/// handlers in this tree pass `diesel::result::Error`, `r2d2::Error`,
/// `FromUtf8Error`, a bare `String`, `anyhow::Error`, `SafeCommand`'s
/// `GuardError` and boxed `dyn Error` values, and of those only some implement
/// `Error`. A narrower bound turns the shared helper into a compile error at
/// one call site per crate, which is exactly the churn this helper exists to
/// remove.
pub fn log_and_sanitize_json(
    error: &(impl std::fmt::Display + ?Sized),
    context: &str,
    request_id: Option<&str>,
) -> serde_json::Value {
    serde_json::json!(log_and_sanitize_str(
        &error.to_string(),
        context,
        request_id
    ))
}

pub fn log_and_sanitize_str(
    error_msg: &str,
    context: &str,
    request_id: Option<&str>,
) -> SafeErrorResponse {
    warn!(
        context = %context,
        request_id = ?request_id,
        error = %error_msg,
        "Error occurred"
    );

    SafeErrorResponse::internal_error().with_request_id(request_id.unwrap_or("unknown").to_string())
}

/// The axum status/body error tuple used by the majority of handlers, with the
/// body replaced by the generic, non-leaking sanitizer message.
///
/// The cause is still logged server-side by [`log_and_sanitize_str`]; the
/// client only ever sees `error.message` from [`SafeErrorResponse`]. This is
/// the one-line migration for the 200+ `format!("DB error: {e}")` sites named
/// in #2753 — those handlers keep their return type and drop the leak.
pub fn log_and_sanitize_tuple(
    error: &(impl std::fmt::Display + ?Sized),
    context: &str,
    status: StatusCode,
) -> (StatusCode, String) {
    (
        status,
        log_and_sanitize_str(&error.to_string(), context, None).message,
    )
}

#[cfg(test)]
mod hostile_payload_tests {
    use super::*;

    #[derive(Debug)]
    struct Hostile(String);

    impl std::fmt::Display for Hostile {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for Hostile {}

    /// A DSN with no URL scheme: keyword pairs still give away host, database
    /// and credentials, so detection must not depend on `://` being present.
    #[test]
    fn keyword_dsn_is_detected_without_a_scheme() {
        let dsn = "Server=tcp:db.internal,1433;Initial Catalog=AppDb;User ID=app;Password=s3cr3t;";
        assert!(looks_like_connection_string(dsn));
        assert!(looks_like_internal_path("at /opt/gbo/bin/handler.rs:42"));
        assert!(looks_like_internal_path("error at C:\\srv\\app\\db.rs"));
        assert!(!looks_like_internal_path("table does not exist"));
    }

    #[test]
    fn every_redacted_scheme_is_detected() {
        for scheme in [
            "rediss://",
            "valkey://",
            "mariadb://",
            "amqps://",
            "kafka+ssl://",
            "ldaps://",
            "ftps://",
            "smb://",
        ] {
            let uri = format!("{scheme}user:pw@host:5432/db");
            assert!(looks_like_connection_string(&uri), "{uri} must be detected");
        }
    }

    #[test]
    fn provider_tokens_and_headers_are_detected() {
        assert!(looks_like_credential(
            "Graph 401: Bearer eyJhbGciOiJIUzI1NiJ9.x.y"
        ));
        assert!(looks_like_credential("Authorization: Basic dXNlcjpwYXNz"));
        assert!(looks_like_credential("api_key=sk_live_51H8xyz"));
        assert!(!looks_like_credential("HTTP 500 from provider"));
    }

    /// The sanitizer is the last line of defence: even a hostile payload that
    /// reaches it directly must come back without passwords, tokens, DSNs or
    /// absolute paths.
    #[test]
    fn hostile_payloads_are_redacted_by_the_sanitizer() {
        let sanitizer = ErrorSanitizer::production();
        let payloads = [
            "connect failed: postgres://app:s3cr3t@10.1.2.3:5432/app",
            "odbc error Server=db.internal;Database=AppDb;Uid=app;Pwd=s3cr3t;",
            "Graph 401 Unauthorized: Bearer eyJhbGciOiJIUzI1NiJ9.abc.def",
            "panic at /opt/gbo/bin/handler.rs:42:17",
            "error at C:\\srv\\app\\config.rs:7",
        ];
        for payload in payloads {
            let response = sanitizer.sanitize_error(&Hostile(payload.to_string()), None);
            let body = match serde_json::to_string(&response) {
                Ok(body) => body,
                Err(e) => {
                    log::warn!("could not serialize sanitized response for assertion: {e}");
                    String::new()
                }
            };
            assert!(!body.contains("s3cr3t"), "{payload} leaked a password");
            assert!(!body.contains("eyJ"), "{payload} leaked a token");
            assert!(!body.contains("postgres://"), "{payload} leaked a DSN");
            assert!(!body.contains("/opt/gbo"), "{payload} leaked a path");
            assert!(!body.contains("C:\\srv"), "{payload} leaked a path");
        }
    }

    #[test]
    fn development_mode_redacts_every_sensitive_material() {
        let sanitizer = ErrorSanitizer::development();
        let redacted = sanitizer.sanitize_message(
            "connect postgres://app:s3cr3t@db/app Bearer eyJhbGciOiJIUzI1NiJ9.x.y at /opt/gbo/db.rs",
        );
        assert!(!redacted.contains("s3cr3t"), "password leaked: {redacted}");
        assert!(!redacted.contains("eyJ"), "token leaked: {redacted}");
        assert!(!redacted.contains("/opt/gbo"), "path leaked: {redacted}");
        assert!(!redacted.contains("postgres://"), "DSN leaked: {redacted}");
    }

    #[test]
    fn contains_sensitive_covers_dsns_credentials_and_paths() {
        let sanitizer = ErrorSanitizer::production();
        for payload in [
            "Server=db.internal;Initial Catalog=App;Uid=app;Pwd=s3cr3t;",
            "Graph 401: Bearer eyJhbGciOiJIUzI1NiJ9.a.b",
            "failed at /opt/gbo/bin/handler.rs:42",
        ] {
            assert!(sanitizer.contains_sensitive(payload), "missed: {payload}");
        }
    }

    #[test]
    fn tuple_helper_returns_generic_body_and_preserves_status() {
        let (status, body) = log_and_sanitize_tuple(
            "DB error: password=s3cr3t at /opt/gbo/db.rs",
            "hostile_test",
            StatusCode::SERVICE_UNAVAILABLE,
        );
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(!body.contains("s3cr3t"));
        assert!(!body.contains("/opt/gbo"));
        assert!(!body.is_empty());
    }
}
