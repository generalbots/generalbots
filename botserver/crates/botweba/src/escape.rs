//! Escaping and URL vetting for the WebA render shell (#2124).
//!
//! Isolation — the sandbox CSP and the opaque-origin iframe — is the boundary
//! that keeps a published app away from the suite origin. Escaping is the
//! second layer: no user value ever reaches the document as markup, and every
//! `src`/`href` is vetted before it is written.

/// Longest resource URL accepted in `config.scripts` / `config.styles`.
const MAX_URL_LEN: usize = 2048;

/// Escapes text for HTML text and quoted attribute positions.
///
/// Applied to every non-content field of the shell: the app name, the
/// description, meta-tag names and values, and the vetted script/style URLs.
#[must_use]
pub fn escape_html(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for character in raw.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Returns the URL when it is safe to place in a `src`/`href`, else `None`.
///
/// Only absolute `http`/`https` URLs and same-origin relative paths are
/// accepted. `javascript:`, `data:`, `vbscript:`, `blob:` and protocol-relative
/// URLs (`//host`) are dropped, so a script or stylesheet entry can never
/// become executable markup.
#[must_use]
pub fn safe_resource_url(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() || value.len() > MAX_URL_LEN {
        return None;
    }
    // Whitespace and control characters are how a `javascript:` scheme is
    // smuggled past a naive prefix check; quotes/angle brackets/backticks are
    // how an attribute is escaped.
    if value.chars().any(|character| {
        character.is_control() || matches!(character, ' ' | '"' | '\'' | '<' | '>' | '`' | '\\')
    }) {
        return None;
    }
    // A protocol-relative URL points at another host and is never needed here.
    if value.starts_with("//") {
        return None;
    }

    match value.find(|character: char| matches!(character, ':' | '/' | '?' | '#')) {
        Some(index) if value.as_bytes()[index] == b':' => {
            let scheme = value[..index].to_ascii_lowercase();
            if scheme != "http" && scheme != "https" {
                return None;
            }
            Some(value.to_string())
        }
        // Relative path, query or fragment: resolved against this document.
        _ => Some(value.to_string()),
    }
}
