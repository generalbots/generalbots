//! Offline tests for the social row conversions and the error mapping
//! (issue #2075). No database is opened: the helpers take owned rows.

use super::*;

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("fixed instant")
}

fn db_post() -> DbPost {
    DbPost {
        id: Uuid::from_u128(1),
        org_id: Uuid::from_u128(2),
        bot_id: Uuid::from_u128(3),
        author_id: Uuid::from_u128(4),
        community_id: Some(Uuid::from_u128(5)),
        parent_id: None,
        content: "hello".to_string(),
        content_type: "text".to_string(),
        attachments: serde_json::json!([{
            "id": Uuid::from_u128(6).to_string(),
            "file_type": "image",
            "url": "https://example.test/a.png",
            "name": "a.png",
            "size": 12,
            "mime_type": "image/png",
            "thumbnail_url": null
        }]),
        mentions: serde_json::json!([Uuid::from_u128(7).to_string()]),
        hashtags: vec![Some("rust".to_string()), None, Some("tests".to_string())],
        visibility: "organization".to_string(),
        is_announcement: true,
        is_pinned: false,
        poll_id: None,
        reaction_counts: serde_json::json!({ "like": 3 }),
        comment_count: 2,
        share_count: 1,
        view_count: 40,
        created_at: now(),
        updated_at: now(),
        edited_at: None,
        deleted_at: None,
        branch_id: None,
    }
}

#[test]
fn post_conversion_flattens_hashtags_and_parses_json_columns() {
    let post = db_post_to_post(db_post());

    assert_eq!(post.hashtags, vec!["rust".to_string(), "tests".to_string()]);
    assert_eq!(post.attachments.len(), 1);
    assert_eq!(post.attachments[0].mime_type, "image/png");
    assert_eq!(post.mentions, vec![Uuid::from_u128(7)]);
    assert_eq!(post.reaction_counts.get("like"), Some(&3));
    assert_eq!(post.visibility, "organization");
    assert!(post.is_announcement);
    assert_eq!(post.comment_count, 2);
}

#[test]
fn post_conversion_falls_back_to_empty_collections_for_bad_json() {
    let mut row = db_post();
    row.attachments = serde_json::json!("not-an-array");
    row.mentions = serde_json::json!(7);
    row.reaction_counts = serde_json::json!({ "like": "many" });

    let post = db_post_to_post(row);

    assert!(post.attachments.is_empty());
    assert!(post.mentions.is_empty());
    assert!(post.reaction_counts.is_empty());
}

#[test]
fn community_conversion_defaults_a_missing_description_to_empty() {
    let row = DbCommunity {
        id: Uuid::from_u128(10),
        org_id: Uuid::from_u128(2),
        bot_id: Uuid::from_u128(3),
        name: "Engineering".to_string(),
        slug: "engineering".to_string(),
        description: None,
        cover_image: None,
        icon: None,
        visibility: "public".to_string(),
        join_policy: "open".to_string(),
        owner_id: Uuid::from_u128(4),
        member_count: 8,
        post_count: 3,
        is_official: true,
        is_featured: false,
        settings: serde_json::json!({}),
        created_at: now(),
        updated_at: now(),
        archived_at: None,
        branch_id: None,
    };

    let community = db_community_to_community(row);

    assert_eq!(community.description, "");
    assert_eq!(community.slug, "engineering");
    assert_eq!(community.member_count, 8);
    assert!(community.is_official);
}

#[test]
fn comment_conversion_parses_mentions_and_reactions() {
    let row = DbComment {
        id: Uuid::from_u128(11),
        post_id: Uuid::from_u128(1),
        parent_comment_id: Some(Uuid::from_u128(12)),
        author_id: Uuid::from_u128(4),
        content: "nice".to_string(),
        mentions: serde_json::json!([Uuid::from_u128(7).to_string()]),
        reaction_counts: serde_json::json!({ "heart": 2 }),
        reply_count: 1,
        created_at: now(),
        updated_at: now(),
        edited_at: None,
        deleted_at: None,
    };

    let comment = db_comment_to_comment(row);

    assert_eq!(comment.parent_comment_id, Some(Uuid::from_u128(12)));
    assert_eq!(comment.mentions, vec![Uuid::from_u128(7)]);
    assert_eq!(comment.reaction_counts.get("heart"), Some(&2));
    assert_eq!(comment.reply_count, 1);
}

#[test]
fn member_conversion_keeps_role_and_notification_flags() {
    let row = DbCommunityMember {
        id: Uuid::from_u128(13),
        community_id: Uuid::from_u128(10),
        user_id: Uuid::from_u128(4),
        role: "moderator".to_string(),
        notifications_enabled: false,
        joined_at: now(),
        last_seen_at: Some(now()),
    };

    let member = db_member_to_member(row);

    assert_eq!(member.community_id, Uuid::from_u128(10));
    assert_eq!(member.role, "moderator");
    assert!(!member.notifications_enabled);
    assert_eq!(member.last_seen_at, Some(now()));
}

fn status_of(error: SocialError) -> u16 {
    use axum::response::IntoResponse;
    error.into_response().status().as_u16()
}

#[test]
fn social_errors_map_to_the_published_http_status() {
    let cases: Vec<(SocialError, u16)> = vec![
        (SocialError::NotFound("post".to_string()), 404),
        (SocialError::Unauthorized("user".to_string()), 401),
        (SocialError::Database("boom".to_string()), 500),
        (SocialError::Internal("boom".to_string()), 500),
    ];

    for (error, expected) in cases {
        assert_eq!(status_of(error), expected);
    }
}
