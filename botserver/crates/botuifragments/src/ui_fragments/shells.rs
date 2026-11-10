/* =============================================================================
 * shells.rs — shared HTMX shell builders for the editor apps.
 *
 * These builders hold the markup botdocs, botsheet and botslides rendered by
 * hand: the modal, panel and sidebar skeletons, the empty state, the error
 * box, the labelled form field. App-specific markup (CSS prefix, body
 * attributes, controls, translations) stays in the app crate; only the shell
 * strings live here.
 *
 * Every builder escapes dynamic text through `htmx_escape`. `title`, `body`,
 * `attrs` and `content` are inserted verbatim because callers pass intentional
 * markup (localized titles with quoted names, hx-* attributes whose values are
 * already escaped). Never pass untrusted input to those parameters.
 * =============================================================================*/

use super::htmx_escape;

/// Empty-state box. `class` is the app CSS class (`dc-empty`, `sl-empty`,
/// `fragment-empty`); `message` is plain text and is escaped here.
pub fn empty_state(class: &str, message: &str) -> String {
    format!(
        r##"<div class="{class}" style="padding:24px;text-align:center;color:#94a3b8;font-style:italic;">
{message}</div>"##,
        class = htmx_escape(class),
        message = htmx_escape(message),
    )
}

/// Error box shared by the editor fragments. `label` is usually the translated
/// "Error:" word; both label and message are escaped.
pub fn error_box(class: &str, label: &str, message: &str) -> String {
    format!(
        r##"<div class="{class}" role="alert" style="padding:12px;border:1px solid #f87171;background:#7f1d1d;color:#fecaca;border-radius:6px;margin:8px 0;">
<strong>{label}</strong> {message}</div>"##,
        class = htmx_escape(class),
        label = htmx_escape(label),
        message = htmx_escape(message),
    )
}

/// `<div class="{prefix}-modal-body"...>` wrapper. `attrs` is trusted static
/// markup (hx-* attributes with already-escaped values); `content` is markup.
pub fn modal_body(prefix: &str, attrs: &str, content: &str) -> String {
    format!(
        r##"<div class="{prefix}-modal-body"{attrs}>{content}</div>"##,
        prefix = htmx_escape(prefix),
        attrs = attrs,
        content = content,
    )
}

/// Standard modal (480 px). `title` and `body` are trusted markup; `id` and
/// `prefix` are escaped.
pub fn modal_shell(prefix: &str, id: &str, title: &str, body: &str) -> String {
    modal_shell_with_content_style(prefix, id, title, 480, "", body)
}

/// Standard modal with a custom max width (560/640 for larger editors).
pub fn modal_shell_width(prefix: &str, id: &str, title: &str, width: u32, body: &str) -> String {
    modal_shell_with_content_style(prefix, id, title, width, "", body)
}

/// Modal whose content box needs extra CSS (e.g. the share modal scrolls).
pub fn modal_shell_with_content_style(
    prefix: &str,
    id: &str,
    title: &str,
    width: u32,
    content_style: &str,
    body: &str,
) -> String {
    format!(
        r##"<div class="{prefix}-modal" id="{id}" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="{prefix}-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:{width}px;width:90%;{content_style}">
<div class="{prefix}-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.{prefix}-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
{body}
</div>
</div>"##,
        prefix = htmx_escape(prefix),
        id = htmx_escape(id),
        width = width,
        content_style = content_style,
        title = title,
        body = body,
    )
}

/// Panel with the plain header and an 8 px body (the common editor panel).
pub fn panel_shell(prefix: &str, id: &str, title: &str, body: &str) -> String {
    panel_shell_with_body_style(prefix, id, title, "padding:8px;", body)
}

/// Panel variant with a custom body style (grids, scrolling lists, padding).
pub fn panel_shell_with_body_style(
    prefix: &str,
    id: &str,
    title: &str,
    body_style: &str,
    body: &str,
) -> String {
    format!(
        r##"<div class="{prefix}-panel" id="{id}">
<div class="{prefix}-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">{title}</h3>
</div>
<div class="{prefix}-panel-body" style="{body_style}">{body}</div></div>"##,
        prefix = htmx_escape(prefix),
        id = htmx_escape(id),
        title = title,
        body_style = body_style,
        body = body,
    )
}

/// Scrollable list sidebar: header with an action button, body column.
/// `title` and `action` are trusted markup; `id`/`prefix` are escaped.
pub fn sidebar_shell(prefix: &str, id: &str, title: &str, action: &str, body: &str) -> String {
    format!(
        r##"<div class="{prefix}-sidebar" id="{id}">
<div class="{prefix}-sidebar-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">{title}</h3>
{action}
</div>
<div class="{prefix}-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;max-height:calc(100vh - 200px);overflow-y:auto;">{body}</div></div>"##,
        prefix = htmx_escape(prefix),
        id = htmx_escape(id),
        title = title,
        action = action,
        body = body,
    )
}

/// Sidebar variant without an action button (search/recent results).
pub fn sidebar_shell_plain(prefix: &str, id: &str, title: &str, body: &str) -> String {
    format!(
        r##"<div class="{prefix}-sidebar" id="{id}">
<div class="{prefix}-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">{title}</h3>
</div>
<div class="{prefix}-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">{body}</div></div>"##,
        prefix = htmx_escape(prefix),
        id = htmx_escape(id),
        title = title,
        body = body,
    )
}

/// Labelled form field: the `<label>` wrapper every modal form repeats.
/// `control` is the input/select/textarea markup authored by the app.
pub fn form_field(label: &str, control: &str) -> String {
    format!(
        r##"<label style="color:#f8fafc;font-size:13px;">{label}
{control}
</label>"##,
        label = htmx_escape(label),
        control = control,
    )
}

#[cfg(test)]
#[path = "shells_tests.rs"]
mod shells_tests;
