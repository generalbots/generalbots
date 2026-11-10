//! Offline unit tests for the shared HTMX shell builders (#2067).
//!
//! `shells.rs` is the single place the editor apps build their modal, panel and
//! sidebar skeletons. Every parameter that is documented as escaped must stay
//! escaped: a caller that forwards a document title or a user name into
//! `prefix`/`id`/`message` must not be able to inject markup. Parameters that
//! are documented as trusted markup (`title`, `body`, `attrs`, `control`) are
//! inserted verbatim and are asserted to be verbatim here, so the contract is
//! visible instead of implied.

use super::{
    empty_state, error_box, form_field, htmx_escape, modal_body, modal_shell,
    modal_shell_width, modal_shell_with_content_style, panel_shell,
    panel_shell_with_body_style, sidebar_shell, sidebar_shell_plain,
};

#[test]
fn htmx_escape_covers_the_five_dangerous_characters() {
    assert_eq!(htmx_escape("&"), "&amp;");
    assert_eq!(htmx_escape("<"), "&lt;");
    assert_eq!(htmx_escape(">"), "&gt;");
    assert_eq!(htmx_escape("\""), "&quot;");
    assert_eq!(htmx_escape("'"), "&#39;");
    assert_eq!(htmx_escape("plain text 123"), "plain text 123");
    assert_eq!(htmx_escape(""), "");
    // Ampersand first, so an existing entity is not re-interpreted.
    assert_eq!(htmx_escape("&lt;"), "&amp;lt;");
    assert_eq!(
        htmx_escape(r#"<img src="x" onerror='alert(1)'>"#),
        "&lt;img src=&quot;x&quot; onerror=&#39;alert(1)&#39;&gt;"
    );
}

#[test]
fn empty_state_escapes_class_and_message() {
    let html = empty_state("dc-empty", "No <b>documents</b> yet");
    assert!(html.contains("class=\"dc-empty\""));
    assert!(html.contains("No &lt;b&gt;documents&lt;/b&gt; yet"));
    assert!(!html.contains("<b>documents</b>"));
    assert!(html.contains("font-style:italic"));

    // The class is escaped too: it is interpolated into an attribute.
    let hostile = empty_state("a\"onload=\"x", "m");
    assert!(!hostile.contains("a\"onload=\"x"));
    assert!(hostile.contains("&quot;onload=&quot;x"));
}

#[test]
fn error_box_marks_the_alert_role_and_escapes_both_labels() {
    let html = error_box("sl-error", "Erro:", "campo <obrigatório>");
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("class=\"sl-error\""));
    assert!(html.contains("<strong>Erro:</strong>"));
    assert!(html.contains("campo &lt;obrigatório&gt;"));
    assert!(!html.contains("<obrigatório>"));
}

#[test]
fn modal_body_escapes_the_prefix_and_keeps_attrs_verbatim() {
    let html = modal_body("dc", " hx-post=\"/save\"", "<p>body</p>");
    assert_eq!(
        html,
        "<div class=\"dc-modal-body\" hx-post=\"/save\"><p>body</p></div>"
    );
}

#[test]
fn modal_shell_wraps_the_body_with_its_prefix_and_id() {
    let html = modal_shell("sl", "share-modal", "Compartilhar", "<p>x</p>");
    assert!(html.contains("<div class=\"sl-modal\" id=\"share-modal\""));
    assert!(html.contains("<div class=\"sl-modal-content\""));
    assert!(html.contains("max-width:480px"));
    assert!(html.contains("<h3 style=\"margin:0;color:#f8fafc;\">Compartilhar</h3>"));
    assert!(html.contains("<p>x</p>"));
    // The close button targets the prefix-scoped modal.
    assert!(html.contains("this.closest('.sl-modal').remove()"));
    assert!(html.ends_with("</div>"));
}

#[test]
fn modal_shell_width_changes_only_the_max_width() {
    let standard = modal_shell("dc", "id", "T", "B");
    let wide = modal_shell_width("dc", "id", "T", 640, "B");
    assert!(standard.contains("max-width:480px"));
    assert!(wide.contains("max-width:640px"));
    assert!(!wide.contains("max-width:480px"));
}

#[test]
fn modal_shell_with_content_style_appends_the_extra_css() {
    let html = modal_shell_with_content_style("dc", "id", "T", 560, "overflow-y:auto;", "B");
    assert!(html.contains("max-width:560px"));
    assert!(html.contains("overflow-y:auto;"));
}

#[test]
fn modal_shell_escapes_prefix_and_id_but_not_the_title() {
    let html = modal_shell("p\"x", "i\"d", "<em>Title</em>", "B");
    assert!(html.contains("class=\"p&quot;x-modal\""));
    assert!(html.contains("id=\"i&quot;d\""));
    // Documented as trusted markup: the caller is responsible for the title.
    assert!(html.contains("<em>Title</em>"));
}

#[test]
fn panel_shell_uses_the_default_eight_pixel_body() {
    let html = panel_shell("dc", "p1", "Outline", "<ul></ul>");
    assert!(html.contains("<div class=\"dc-panel\" id=\"p1\">"));
    assert!(html.contains("<div class=\"dc-panel-body\" style=\"padding:8px;\">"));
    assert!(html.contains("<ul></ul>"));
    assert!(html.contains("<h3 style=\"margin:0;color:#f8fafc;font-size:16px;\">Outline</h3>"));
}

#[test]
fn panel_shell_with_body_style_keeps_the_custom_style() {
    let html = panel_shell_with_body_style("sl", "p2", "Slides", "padding:0;overflow:auto;", "B");
    assert!(html.contains("style=\"padding:0;overflow:auto;\""));
    assert!(!html.contains("padding:8px;"));
}

#[test]
fn sidebar_shell_renders_the_action_slot() {
    let html = sidebar_shell("dc", "s1", "Documents", "<button>New</button>", "<p>list</p>");
    assert!(html.contains("<div class=\"dc-sidebar\" id=\"s1\">"));
    assert!(html.contains("<button>New</button>"));
    assert!(html.contains("<p>list</p>"));
    assert!(html.contains("overflow-y:auto"));
    assert!(html.contains("max-height:calc(100vh - 200px)"));
}

#[test]
fn sidebar_shell_plain_has_no_action_slot() {
    let html = sidebar_shell_plain("dc", "s2", "Recent", "<p>r</p>");
    assert!(html.contains("<div class=\"dc-sidebar\" id=\"s2\">"));
    assert!(html.contains("<p>r</p>"));
    assert!(!html.contains("overflow-y:auto"));
    assert!(!html.contains("<button"));
}

#[test]
fn sidebar_escapes_prefix_and_id() {
    let html = sidebar_shell("a<b", "i>d", "T", "A", "B");
    assert!(html.contains("class=\"a&lt;b-sidebar\""));
    assert!(html.contains("id=\"i&gt;d\""));
}

#[test]
fn form_field_escapes_the_label_and_keeps_the_control() {
    let html = form_field("Título", "<input name=\"title\">");
    assert!(html.contains("<label style=\"color:#f8fafc;font-size:13px;\">Título"));
    assert!(html.contains("<input name=\"title\">"));
    assert!(html.ends_with("</label>"));

    let hostile = form_field("<script>x</script>", "<input>");
    assert!(!hostile.contains("<script>x</script>"));
    assert!(hostile.contains("&lt;script&gt;x&lt;/script&gt;"));
}
