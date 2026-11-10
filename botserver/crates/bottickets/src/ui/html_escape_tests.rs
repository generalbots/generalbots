use super::*;

#[test]
fn html_escape_neutralizes_every_dangerous_character() {
    let cases = [
        ("<script>", "&lt;script&gt;"),
        ("a & b", "a &amp; b"),
        ("\"quoted\"", "&quot;quoted&quot;"),
        ("it's", "it&#x27;s"),
        ("", ""),
        ("ação 日本語", "ação 日本語"),
    ];
    for (input, expected) in cases {
        assert_eq!(html_escape(input), expected, "input={input}");
    }
}

#[test]
fn html_escape_escapes_ampersands_first_so_markup_is_not_double_decoded() {
    // `&` is replaced first, so the `&lt;` produced by `<` is not escaped again.
    assert_eq!(html_escape("<&"), "&lt;&amp;");
    assert_eq!(html_escape("&lt;"), "&amp;lt;");
}

#[test]
fn priority_badges_map_each_known_priority_and_fall_back_to_unknown() {
    let cases = [
        ("urgent", "Urgent", "badge-danger"),
        ("high", "High", "badge-warning"),
        ("medium", "Medium", "badge-info"),
        ("low", "Low", "badge-secondary"),
    ];
    for (priority, label, class) in cases {
        let badge = priority_badge(priority);
        assert!(badge.contains(label), "priority={priority} badge={badge}");
        assert!(badge.contains(class), "priority={priority} badge={badge}");
    }

    // Matching is exact and case-sensitive: "Urgent" is not "urgent".
    for unknown in ["Urgent", "critical", "", "blocker"] {
        assert_eq!(
            priority_badge(unknown),
            "<span class=\"badge\">Unknown</span>",
            "priority={unknown}"
        );
    }
}

#[test]
fn status_badges_map_each_known_status_and_fall_back_to_unknown() {
    let cases = [
        ("open", "Open", "badge-primary"),
        ("pending", "Pending", "badge-warning"),
        ("resolved", "Resolved", "badge-success"),
        ("closed", "Closed", "badge-secondary"),
    ];
    for (status, label, class) in cases {
        let badge = status_badge(status);
        assert!(badge.contains(label), "status={status} badge={badge}");
        assert!(badge.contains(class), "status={status} badge={badge}");
    }

    for unknown in ["Open", "in_progress", ""] {
        assert_eq!(
            status_badge(unknown),
            "<span class=\"badge\">Unknown</span>",
            "status={unknown}"
        );
    }
}

#[test]
fn the_empty_state_renders_the_icon_title_and_description() {
    let html = render_empty_state("🎫", "No tickets", "Create your first ticket");
    assert!(html.contains("empty-state"));
    assert!(html.contains("🎫"));
    assert!(html.contains("No tickets"));
    assert!(html.contains("Create your first ticket"));
}
