//! Isolation for document exports (#2124 follow-up).
//!
//! `/api/docs/export/html` answered with `text/html` and interpolated
//! `doc.title` into `<title>` unescaped, so a stored title of
//! `</title><script>…` executed as the suite user. An export is a file, not a
//! page: every text export leaves as `Content-Disposition: attachment` with a
//! non-rendering content type, and the HTML one also carries the WebA
//! isolation policy (`botweba::WEBA_CSP`, #2124) so a proxy that drops the
//! disposition cannot turn it into a suite-origin document either.
//!
//! Two layers, exactly as in WebA:
//!
//! 1. [`escape_html`] — the title is text and is escaped, so it can never
//!    close `<title>` and inject markup. `content` stays raw on purpose (same
//!    reasoning as `botweba::render_html`): it is the document's own markup,
//!    and the policy below refuses to execute anything inside it.
//! 2. The policy header plus `attachment`/`octet-stream` — the document never
//!    renders on the suite origin at all.

use axum::http::{header, HeaderMap, HeaderValue};

use crate::storage_core::escape_html;

/// Policy served with the HTML export.
///
/// `sandbox allow-scripts` gives the document an opaque origin — no suite
/// `localStorage` (where the chat token lives), no suite cookies — and
/// `default-src 'none'` loads nothing else. Same shape as `botweba::WEBA_CSP`,
/// narrowed to what a static export needs.
pub const EXPORT_HTML_CSP: &str = "sandbox allow-scripts; default-src 'none'";

/// Content type used for the HTML export.
///
/// Deliberately not `text/html`: the browser downloads the file instead of
/// rendering user-authored markup on the suite origin.
pub const HTML_EXPORT_CONTENT_TYPE: &str = "application/octet-stream";

/// Name used when the title has no character a file name may keep.
const FALLBACK_NAME: &str = "document";

/// Upper bound for the generated file name stem.
const MAX_FILENAME_CHARS: usize = 80;

/// Builds the downloadable file name from the document title.
///
/// Only ASCII letters, digits, `-` and `_` survive; spaces become `_` and
/// everything else — quotes, slashes, newlines, dots — is dropped. The value
/// therefore cannot break out of the `Content-Disposition` header, cannot
/// traverse a path and cannot hide a second extension.
#[must_use]
pub fn export_filename(title: &str, extension: &str) -> String {
    let mut stem = String::with_capacity(title.len().min(MAX_FILENAME_CHARS));
    for character in title.chars() {
        if stem.len() >= MAX_FILENAME_CHARS {
            break;
        }
        match character {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' => stem.push(character),
            ' ' | '\t' => stem.push('_'),
            _ => {}
        }
    }
    let stem = stem.trim_matches('_');
    let stem = if stem.is_empty() { FALLBACK_NAME } else { stem };

    let extension: String = extension
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    if extension.is_empty() {
        stem.to_string()
    } else {
        format!("{stem}.{extension}")
    }
}

/// Headers for every exported file.
///
/// `nosniff` stops content-type guessing and `no-referrer` keeps the export
/// URL out of third-party logs. `csp` is set only for the HTML export.
#[must_use]
pub fn download_headers(
    content_type: &'static str,
    filename: &str,
    csp: Option<&'static str>,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(header::CONTENT_DISPOSITION, attachment_header(filename));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    if let Some(policy) = csp {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(policy),
        );
    }
    headers
}

/// Headers for the HTML export: attachment, opaque type, sandboxed policy.
#[must_use]
pub fn html_export_headers(title: &str) -> HeaderMap {
    download_headers(
        HTML_EXPORT_CONTENT_TYPE,
        &export_filename(title, "html"),
        Some(EXPORT_HTML_CSP),
    )
}

/// Headers for a plain-text export (`md`, `txt`).
#[must_use]
pub fn text_export_headers(
    title: &str,
    extension: &str,
    content_type: &'static str,
) -> HeaderMap {
    download_headers(content_type, &export_filename(title, extension), None)
}

/// Renders the standalone HTML document handed to the user.
///
/// The title is escaped; `content` is the document body and stays markup.
#[must_use]
pub fn render_html_export(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{}</title>
<style>
body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
</style>
</head>
<body>
{}
</body>
</html>"#,
        escape_html(title),
        content
    )
}

fn attachment_header(filename: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
        .unwrap_or_else(|error| {
            log::warn!("docs export: unsafe file name dropped: {error}");
            HeaderValue::from_static("attachment")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The payload from #2124: closing `<title>` is enough to run script.
    const TITLE_BREAKOUT: &str = "</title><script>alert(1)</script>";

    #[test]
    fn title_breakout_is_escaped() {
        let html = render_html_export(TITLE_BREAKOUT, "<p>body</p>");

        assert!(html.contains("&lt;/title&gt;&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        // Exactly one real `</title>`: the escaped one cannot close the head.
        assert_eq!(html.matches("</title>").count(), 1);
    }

    #[test]
    fn html_export_carries_the_sandbox_policy() {
        let headers = html_export_headers(TITLE_BREAKOUT);

        assert_eq!(
            headers.get(header::CONTENT_SECURITY_POLICY).and_then(|v| v.to_str().ok()),
            Some("sandbox allow-scripts; default-src 'none'")
        );
        assert_eq!(
            headers.get(header::X_CONTENT_TYPE_OPTIONS).and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            headers.get(header::REFERRER_POLICY).and_then(|v| v.to_str().ok()),
            Some("no-referrer")
        );
        assert_eq!(
            headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
            Some("application/octet-stream")
        );
        assert!(headers
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .starts_with("attachment;"));
    }

    #[test]
    fn content_payload_cannot_execute_without_the_policy() {
        // The body is the document's own markup, so it keeps its `<script>`
        // text; what stops it is the policy, asserted above. This test pins
        // that the title is never the place it lands.
        let html = render_html_export("Report", TITLE_BREAKOUT);
        assert!(html.contains("<body>\n</title><script>alert(1)</script>\n</body>"));
        assert_eq!(html.matches("<title>").count(), 1);
    }

    #[test]
    fn filename_drops_quotes_newlines_and_path_separators() {
        assert_eq!(
            export_filename("../../etc/pass\nevil\"quote", "html"),
            "etc_passevilquote.html"
        );
        assert_eq!(export_filename("  ", "txt"), "document.txt");
        assert_eq!(export_filename("Q3 Report", "md"), "Q3_Report.md");
    }

    #[test]
    fn text_export_has_no_policy_but_is_a_download() {
        let headers = text_export_headers("Report", "txt", "text/plain; charset=utf-8");

        assert!(headers.get(header::CONTENT_SECURITY_POLICY).is_none());
        assert_eq!(
            headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(
            headers
                .get(header::CONTENT_DISPOSITION)
                .and_then(|v| v.to_str().ok()),
            Some("attachment; filename=\"Report.txt\"")
        );
    }
}
