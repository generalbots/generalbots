//! Log-safe rendering for the browser agent (#2678).
//!
//! Form values and navigation URLs can carry credentials, tokens or PII. The
//! helpers here build strings that are safe to write to `botserver.log`;
//! callers must emit their return value and never the raw input.

/// Reduces a URL to `scheme://host[:port]` for logging (#2678).
///
/// Query strings, fragments and userinfo can carry tokens and identifiers, so
/// they never reach the log. A value that cannot be parsed is reported as an
/// opaque marker rather than echoed.
pub(crate) fn log_safe_url(url: &str) -> String {
    let without_scheme = match url.split_once("://") {
        Some((scheme, rest)) => (scheme, rest),
        None => return "<unparseable-url>".to_string(),
    };
    let (scheme, rest) = without_scheme;
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .rsplit('@')
        .next()
        .unwrap_or_default();
    if authority.is_empty() {
        return "<unparseable-url>".to_string();
    }
    format!("{scheme}://{authority}")
}

/// Build the log line for a form fill (#2678). The value is a secret or PII
/// (password, token, personal data), so only its byte length is emitted; the
/// selector keeps the diagnostic useful without leaking the field contents.
pub(crate) fn log_safe_fill(selector: &str, value: &str) -> String {
    format!("{selector} with {} bytes", value.len())
}

#[cfg(test)]
mod tests {
    use super::{log_safe_fill, log_safe_url};

    // #2678 — a synthetic sentinel stands in for a password/token/PII; each
    // assertion proves it is absent from the rendered log line.

    #[test]
    fn fill_value_never_appears_in_log_message() {
        let secret = "SENTINEL_FORM_SECRET_DO_NOT_LOG";
        let line = log_safe_fill("input#password", secret);
        assert!(
            line.contains("input#password"),
            "selector must stay diagnostic"
        );
        assert_eq!(line, format!("input#password with {} bytes", secret.len()));
        assert!(
            !line.contains(secret),
            "form secret leaked into log: {line}"
        );
    }

    #[test]
    fn secret_bearing_url_payloads_are_not_echoed() {
        let secret = "SENTINEL_QUERY_TOKEN_DO_NOT_LOG";
        let safe = log_safe_url(&format!(
            "https://{secret}@example.com/login?token={secret}&next=/admin#frag"
        ));
        assert_eq!(safe, "https://example.com");
        assert!(!safe.contains(secret), "URL secret leaked into log: {safe}");
    }

    #[test]
    fn query_strings_and_userinfo_are_stripped() {
        assert_eq!(
            log_safe_url("https://user:pass@example.com:8443/path?token=secret#frag"),
            "https://example.com:8443"
        );
        assert_eq!(
            log_safe_url("http://localhost:3000/suite/desktop.html?a=1"),
            "http://localhost:3000"
        );
    }

    #[test]
    fn unparseable_values_are_not_echoed() {
        assert_eq!(log_safe_url("not a url"), "<unparseable-url>");
    }
}
