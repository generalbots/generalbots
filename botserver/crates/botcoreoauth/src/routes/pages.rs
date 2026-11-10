use super::*;

/// Minimal HTML error page for the OAuth endpoints.
///
/// Both `title` and `detail` are escaped here, so callers pass raw text (a
/// provider name, a query value) without having to remember to escape it.
pub(super) fn error_page(status: StatusCode, title: &str, detail: &str) -> Response {
    (
        status,
        Html(format!(
            r#"<!DOCTYPE html><html><head><title>{}</title></head><body>
            <h1>{}</h1><p>{}</p>
            <a href="/auth/login">Try Again</a></body></html>"#,
            html_escape(title),
            html_escape(title),
            html_escape(detail)
        )),
    )
        .into_response()
}
