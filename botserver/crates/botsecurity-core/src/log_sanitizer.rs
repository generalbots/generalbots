use std::sync::LazyLock;

static SANITIZATION_PATTERNS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    // Backslashes must be escaped FIRST. Running it after the control-character
    // rules re-escaped the backslash those rules had just inserted, so a
    // newline rendered as `\\n` (two backslashes) instead of `\n`.
    vec![
        ("\\", "\\\\"),
        ("\n", "\\n"),
        ("\r", "\\r"),
        ("\t", "\\t"),
        ("\"", "\\\""),
        ("'", "\\'"),
        ("\x00", "\\x00"),
        ("\x1B", "\\x1B"),
    ]
});

pub fn sanitize_for_log(input: &str) -> String {
    let mut result = input.to_string();

    for (pattern, replacement) in SANITIZATION_PATTERNS.iter() {
        result = result.replace(pattern, replacement);
    }

    if result.len() > 10000 {
        // Log messages carry arbitrary text; `String::truncate` panics when the
        // byte budget lands inside a multi-byte character.
        let end = botlib::text::char_boundary_at_or_before(&result, 10000);
        result.truncate(end);
        result.push_str("... [truncated]");
    }

    result
}

pub fn sanitize_log_value<T: std::fmt::Display>(value: T) -> String {
    sanitize_for_log(&value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 10 000-byte budget is a byte count over arbitrary text, so it can
    /// land inside a character: `String::truncate` then panics and takes the
    /// process down. 12 000 bytes of '€' puts byte 10 000 mid-character.
    #[test]
    fn truncation_lands_on_a_char_boundary() {
        let long = "€".repeat(4000);
        assert_eq!(long.len(), 12_000);

        let sanitized = sanitize_for_log(&long);

        assert!(sanitized.ends_with("... [truncated]"));
        // 9 999 bytes kept: the largest char boundary at or before the budget.
        let kept = sanitized.trim_end_matches("... [truncated]");
        assert_eq!(kept.len(), 9_999);
        assert!(kept.chars().all(|c| c == '€'));
    }

    #[test]
    fn short_input_is_returned_untouched() {
        assert_eq!(sanitize_for_log("ação"), "ação");
    }

    /// A crafted value must not be able to forge extra log lines or terminal
    /// control sequences.
    #[test]
    fn control_characters_cannot_forge_log_lines() {
        let sanitized = sanitize_for_log("ok\n2026-01-01 ERROR fake line");
        assert!(!sanitized.contains('\n'));
        assert!(sanitized.contains("\\n"));

        let with_cr = sanitize_for_log("a\rb");
        assert!(!with_cr.contains('\r'));

        let with_tab = sanitize_for_log("a\tb");
        assert!(!with_tab.contains('\t'));

        let with_escape = sanitize_for_log("a\u{1B}[31mred");
        assert!(!with_escape.contains('\u{1B}'));
        assert!(with_escape.contains("\\x1B"));
    }

    #[test]
    fn quotes_and_backslashes_are_escaped() {
        let sanitized = sanitize_for_log("he said \"hi\" and 'bye'");
        assert!(sanitized.contains("\\\""));
        assert!(sanitized.contains("\\'"));
        assert!(sanitize_for_log("C:\\temp").contains("\\\\"));
    }

    #[test]
    fn null_bytes_are_escaped() {
        let sanitized = sanitize_for_log("a\u{0}b");
        assert!(!sanitized.contains('\u{0}'));
        assert!(sanitized.contains("\\x00"));
    }

    #[test]
    fn sanitize_log_value_accepts_any_display_value() {
        assert_eq!(sanitize_log_value(42), "42");
        assert_eq!(sanitize_log_value(true), "true");
        // A real newline is escaped to a backslash followed by "n".
        assert_eq!(sanitize_log_value("a\nb"), format!("a{}nb", '\\'));
    }
}
