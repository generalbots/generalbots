use std::sync::LazyLock;

static SANITIZATION_PATTERNS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    vec![
        ("\n", "\\n"),
        ("\r", "\\r"),
        ("\t", "\\t"),
        ("\\", "\\\\"),
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
        result.truncate(10000);
        result.push_str("... [truncated]");
    }

    result
}

pub fn sanitize_log_value<T: std::fmt::Display>(value: T) -> String {
    sanitize_for_log(&value.to_string())
}
