//! Shared helpers for building safe `Content-Disposition` header values.
//!
//! Stored object names (Drive keys, mirrored provider names, attachment names)
//! are untrusted input. Placing one verbatim inside the quoted `filename`
//! parameter lets a `"` close the string early and inject extra parameters,
//! while CR/LF can split the header. Every download handler builds its header
//! through this module so the same filter is applied everywhere (#2843).

/// Strip characters that could break out of a quoted `filename` parameter or
/// inject header lines: quotes, backslashes, path separators and every C0/C1
/// control character (CR, LF and NUL included).
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control() && !matches!(c, '"' | '\\' | '/'))
        .collect()
}

/// `attachment; filename="…"` with a sanitized name.
pub fn attachment_disposition(name: &str) -> String {
    format!("attachment; filename=\"{}\"", sanitize_filename(name))
}

/// `inline; filename="…"` with a sanitized name.
pub fn inline_disposition(name: &str) -> String {
    format!("inline; filename=\"{}\"", sanitize_filename(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_quote_backslash_and_control_payloads() {
        assert_eq!(sanitize_filename("a\"b\\c.txt"), "abc.txt");
        assert_eq!(sanitize_filename("evil\r\nX-Injected: 1.txt"), "evilX-Injected: 1.txt");
        assert_eq!(sanitize_filename("a\0b\x01c\x7fd.txt"), "abcd.txt");
        assert_eq!(sanitize_filename("..\\..\\etc/passwd"), "....etcpasswd");
    }

    #[test]
    fn disposition_has_exactly_one_filename_parameter() {
        // A classic breakout: a quote closes `filename`, a second opens a new
        // one, CR/LF tries to add a header. All dangerous chars are stripped,
        // leaving a single quoted parameter (exactly two `"` delimiters).
        let hostile = "evil\" filename=\"attacker\r\nX-Injected: 1\x00.txt";
        let value = attachment_disposition(hostile);
        assert_eq!(
            value,
            "attachment; filename=\"evil filename=attackerX-Injected: 1.txt\""
        );
        assert!(value.starts_with("attachment; filename=\""));
        assert_eq!(value.matches('"').count(), 2);
        assert!(value.chars().all(|c| !matches!(c, '\r' | '\n' | '\0' | '\\')));
        assert!(axum::http::HeaderValue::from_str(&value).is_ok());

        let inline = inline_disposition(hostile);
        assert!(inline.starts_with("inline; filename=\""));
        assert_eq!(inline.matches('"').count(), 2);
        assert!(axum::http::HeaderValue::from_str(&inline).is_ok());
    }
}
