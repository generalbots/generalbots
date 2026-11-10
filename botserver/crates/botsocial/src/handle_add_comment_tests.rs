//! Offline tests for the social post card fragment (issue #2075).
//! The renderer is pure: it formats a `PostWithAuthor` into HTMX markup.

use super::*;

fn instant() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("fixed instant")
}

fn post_with(content: &str, author: &str, avatar: Option<&str>) -> PostWithAuthor {
    PostWithAuthor {
        post: Post {
            id: Uuid::from_u128(1),
            organization_id: Uuid::from_u128(2),
            author_id: Uuid::from_u128(3),
            community_id: None,
            parent_id: None,
            content: content.to_string(),
            content_type: "text".to_string(),
            attachments: Vec::new(),
            mentions: Vec::new(),
            hashtags: Vec::new(),
            visibility: "organization".to_string(),
            is_announcement: false,
            is_pinned: false,
            poll_id: None,
            reaction_counts: HashMap::new(),
            comment_count: 4,
            share_count: 0,
            view_count: 0,
            created_at: instant(),
            updated_at: instant(),
            edited_at: None,
            deleted_at: None,
        },
        author: UserSummary {
            id: Uuid::from_u128(3),
            name: author.to_string(),
            avatar_url: avatar.map(str::to_string),
            title: None,
            is_leader: false,
        },
        community: None,
        user_reaction: None,
        is_bookmarked: false,
    }
}

#[test]
fn card_carries_the_post_identity_and_actions() {
    let html = render_post_card_html(&post_with("hello world", "Alice", None));

    assert!(html.contains(r#"data-post-id="00000000-0000-0000-0000-000000000001""#));
    assert!(html.contains(r#"hx-post="/api/social/posts/00000000-0000-0000-0000-000000000001/react""#));
    assert!(html.contains(r##"hx-target="#comments-00000000-0000-0000-0000-000000000001""##));
    assert!(html.contains("Comment 4"));
    assert!(html.contains("Nov 14, 2023"), "formatted date: {html}");
}

#[test]
fn card_falls_back_to_the_default_avatar_when_the_author_has_none() {
    let html = render_post_card_html(&post_with("hi", "Alice", None));
    assert!(html.contains(r#"src="/assets/default-avatar.svg""#));

    let with_avatar = render_post_card_html(&post_with(
        "hi",
        "Alice",
        Some("https://example.test/a.png"),
    ));
    assert!(with_avatar.contains(r#"src="https://example.test/a.png""#));
}

#[test]
fn card_renders_one_span_per_reaction() {
    let mut post = post_with("hi", "Alice", None);
    post.post
        .reaction_counts
        .insert("like".to_string(), 3);

    let html = render_post_card_html(&post);

    assert!(html.contains(r#"<span class="reaction">like 3</span>"#));
}

#[test]
fn card_escapes_markup_in_the_post_content() {
    let html = render_post_card_html(&post_with(
        "<script>alert(1)</script>",
        "Alice",
        None,
    ));

    assert!(!html.contains("<script>"), "script tag must not survive: {html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn card_escapes_markup_in_the_author_name_and_avatar_url() {
    let html = render_post_card_html(&post_with(
        "hi",
        "Alice <b>\"admin\"</b>",
        Some("x\" onerror=\"alert(1)"),
    ));

    assert!(!html.contains("<b>\"admin\"</b>"));
    assert!(html.contains("Alice &lt;b&gt;&quot;admin&quot;&lt;/b&gt;"));
    assert!(!html.contains("onerror=\"alert(1)\""));
    assert!(html.contains("&quot; onerror=&quot;alert(1)"));
}

#[test]
fn empty_content_still_produces_a_well_formed_card() {
    let html = render_post_card_html(&post_with("", "Alice", None));

    assert!(html.starts_with(r#"<article class="post-card""#));
    assert!(html.trim_end().ends_with("</article>"));
    assert!(html.contains(r#"<div class="post-content"></div>"#));
}
