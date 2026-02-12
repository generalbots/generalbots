pub mod ui;

use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::{
    social_comments, social_communities, social_community_members, social_posts, social_praises,
    social_reactions,
};
use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = social_posts)]
pub struct DbPost {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub author_id: Uuid,
    pub community_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub content_type: String,
    pub attachments: serde_json::Value,
    pub mentions: serde_json::Value,
    pub hashtags: Vec<Option<String>>,
    pub visibility: String,
    pub is_announcement: bool,
    pub is_pinned: bool,
    pub poll_id: Option<Uuid>,
    pub reaction_counts: serde_json::Value,
    pub comment_count: i32,
    pub share_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = social_communities)]
pub struct DbCommunity {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub icon: Option<String>,
    pub visibility: String,
    pub join_policy: String,
    pub owner_id: Uuid,
    pub member_count: i32,
    pub post_count: i32,
    pub is_official: bool,
    pub is_featured: bool,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = social_community_members)]
pub struct DbCommunityMember {
    pub id: Uuid,
    pub community_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub notifications_enabled: bool,
    pub joined_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = social_comments)]
pub struct DbComment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub mentions: serde_json::Value,
    pub reaction_counts: serde_json::Value,
    pub reply_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = social_reactions)]
pub struct DbReaction {
    pub id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub user_id: Uuid,
    pub reaction_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = social_praises)]
pub struct DbPraise {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub badge_type: String,
    pub message: Option<String>,
    pub is_public: bool,
    pub post_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub author_id: Uuid,
    pub community_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub content_type: String,
    pub attachments: Vec<Attachment>,
    pub mentions: Vec<Uuid>,
    pub hashtags: Vec<String>,
    pub visibility: String,
    pub is_announcement: bool,
    pub is_pinned: bool,
    pub poll_id: Option<Uuid>,
    pub reaction_counts: HashMap<String, i32>,
    pub comment_count: i32,
    pub share_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub file_type: String,
    pub url: String,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub cover_image: Option<String>,
    pub icon: Option<String>,
    pub visibility: String,
    pub join_policy: String,
    pub owner_id: Uuid,
    pub member_count: i32,
    pub post_count: i32,
    pub is_official: bool,
    pub is_featured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMember {
    pub community_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub notifications_enabled: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub reaction_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub mentions: Vec<Uuid>,
    pub reaction_counts: HashMap<String, i32>,
    pub reply_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub id: Uuid,
    pub post_id: Uuid,
    pub question: String,
    pub options: Vec<PollOption>,
    pub allow_multiple: bool,
    pub allow_add_options: bool,
    pub anonymous: bool,
    pub ends_at: Option<DateTime<Utc>>,
    pub total_votes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub id: Uuid,
    pub text: String,
    pub vote_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Praise {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub badge_type: String,
    pub message: String,
    pub is_public: bool,
    pub post_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub community_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
    pub content_type: Option<String>,
    pub community_id: Option<Uuid>,
    pub visibility: Option<String>,
    pub mentions: Option<Vec<Uuid>>,
    pub hashtags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostForm {
    pub content: String,
    pub visibility: Option<String>,
    pub community_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub content: Option<String>,
    pub visibility: Option<String>,
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommunityRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub join_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommunityRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub icon: Option<String>,
    pub visibility: Option<String>,
    pub join_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
    pub mentions: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReactionRequest {
    pub reaction_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub question: String,
    pub options: Vec<String>,
    pub allow_multiple: Option<bool>,
    pub allow_add_options: Option<bool>,
    pub anonymous: Option<bool>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct VotePollRequest {
    pub option_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePraiseRequest {
    pub to_user_id: Uuid,
    pub badge_type: String,
    pub message: String,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct FeedResponse {
    pub posts: Vec<PostWithAuthor>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostWithAuthor {
    #[serde(flatten)]
    pub post: Post,
    pub author: UserSummary,
    pub community: Option<CommunitySummary>,
    pub user_reaction: Option<String>,
    pub is_bookmarked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub title: Option<String>,
    pub is_leader: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommunitySummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub icon: Option<String>,
}

fn db_post_to_post(db: DbPost) -> Post {
    let attachments: Vec<Attachment> = serde_json::from_value(db.attachments).unwrap_or_default();
    let mentions: Vec<Uuid> = serde_json::from_value(db.mentions).unwrap_or_default();
    let reaction_counts: HashMap<String, i32> =
        serde_json::from_value(db.reaction_counts).unwrap_or_default();
    let hashtags: Vec<String> = db.hashtags.into_iter().flatten().collect();

    Post {
        id: db.id,
        organization_id: db.org_id,
        author_id: db.author_id,
        community_id: db.community_id,
        parent_id: db.parent_id,
        content: db.content,
        content_type: db.content_type,
        attachments,
        mentions,
        hashtags,
        visibility: db.visibility,
        is_announcement: db.is_announcement,
        is_pinned: db.is_pinned,
        poll_id: db.poll_id,
        reaction_counts,
        comment_count: db.comment_count,
        share_count: db.share_count,
        view_count: db.view_count,
        created_at: db.created_at,
        updated_at: db.updated_at,
        edited_at: db.edited_at,
        deleted_at: db.deleted_at,
    }
}

fn db_community_to_community(db: DbCommunity) -> Community {
    Community {
        id: db.id,
        organization_id: db.org_id,
        name: db.name,
        slug: db.slug,
        description: db.description.unwrap_or_default(),
        cover_image: db.cover_image,
        icon: db.icon,
        visibility: db.visibility,
        join_policy: db.join_policy,
        owner_id: db.owner_id,
        member_count: db.member_count,
        post_count: db.post_count,
        is_official: db.is_official,
        is_featured: db.is_featured,
        created_at: db.created_at,
        updated_at: db.updated_at,
        archived_at: db.archived_at,
    }
}

fn db_comment_to_comment(db: DbComment) -> Comment {
    let mentions: Vec<Uuid> = serde_json::from_value(db.mentions).unwrap_or_default();
    let reaction_counts: HashMap<String, i32> =
        serde_json::from_value(db.reaction_counts).unwrap_or_default();

    Comment {
        id: db.id,
        post_id: db.post_id,
        parent_comment_id: db.parent_comment_id,
        author_id: db.author_id,
        content: db.content,
        mentions,
        reaction_counts,
        reply_count: db.reply_count,
        created_at: db.created_at,
        updated_at: db.updated_at,
        edited_at: db.edited_at,
        deleted_at: db.deleted_at,
    }
}

fn db_member_to_member(db: DbCommunityMember) -> CommunityMember {
    CommunityMember {
        community_id: db.community_id,
        user_id: db.user_id,
        role: db.role,
        joined_at: db.joined_at,
        notifications_enabled: db.notifications_enabled,
        last_seen_at: db.last_seen_at,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SocialError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for SocialError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Database(msg) | Self::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn handle_get_feed(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FeedQuery>,
) -> Result<Json<FeedResponse>, SocialError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let mut db_query = social_posts::table
            .filter(social_posts::bot_id.eq(bot_id))
            .filter(social_posts::deleted_at.is_null())
            .into_boxed();

        if let Some(community_id) = query.community_id {
            db_query = db_query.filter(social_posts::community_id.eq(community_id));
        }

        if let Some(author_id) = query.author_id {
            db_query = db_query.filter(social_posts::author_id.eq(author_id));
        }

        if let Some(ref search) = query.search {
            let term = format!("%{search}%");
            db_query = db_query.filter(social_posts::content.ilike(term));
        }

        let db_posts: Vec<DbPost> = db_query
            .order(social_posts::created_at.desc())
            .offset(offset)
            .limit(limit + 1)
            .load(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        let has_more = db_posts.len() > limit as usize;
        let posts: Vec<DbPost> = db_posts.into_iter().take(limit as usize).collect();

        let posts_with_author: Vec<PostWithAuthor> = posts
            .into_iter()
            .map(|db_post| {
                let author_id = db_post.author_id;
                let post = db_post_to_post(db_post);
                PostWithAuthor {
                    post,
                    author: UserSummary {
                        id: author_id,
                        name: "User".to_string(),
                        avatar_url: None,
                        title: None,
                        is_leader: false,
                    },
                    community: None,
                    user_reaction: None,
                    is_bookmarked: false,
                }
            })
            .collect();

        Ok::<_, SocialError>(FeedResponse {
            posts: posts_with_author,
            has_more,
            next_cursor: None,
        })
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_feed_html(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FeedQuery>,
) -> Result<Html<String>, SocialError> {
    let feed = handle_get_feed(State(state), Query(query)).await?;
    Ok(Html(render_feed_html(&feed.posts)))
}

pub async fn handle_create_post(
    State(state): State<Arc<AppState>>,
    Form(form): Form<CreatePostForm>,
) -> Result<Html<String>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    let visibility = form.visibility.unwrap_or_else(|| "organization".to_string());
    let community_id = form
        .community_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok());

    let content = form.content;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot(&mut conn);
        let org_id = Uuid::nil();
        let now = Utc::now();

        let db_post = DbPost {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            author_id: user_id,
            community_id,
            parent_id: None,
            content,
            content_type: "text".to_string(),
            attachments: serde_json::json!([]),
            mentions: serde_json::json!([]),
            hashtags: vec![],
            visibility,
            is_announcement: false,
            is_pinned: false,
            poll_id: None,
            reaction_counts: serde_json::json!({}),
            comment_count: 0,
            share_count: 0,
            view_count: 0,
            created_at: now,
            updated_at: now,
            edited_at: None,
            deleted_at: None,
        };

        diesel::insert_into(social_posts::table)
            .values(&db_post)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        if let Some(cid) = community_id {
            let _ = diesel::update(social_communities::table.filter(social_communities::id.eq(cid)))
                .set(social_communities::post_count.eq(social_communities::post_count + 1))
                .execute(&mut conn);
        }

        Ok::<_, SocialError>(db_post_to_post(db_post))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    let post_with_author = PostWithAuthor {
        post: result,
        author: UserSummary {
            id: user_id,
            name: "You".to_string(),
            avatar_url: None,
            title: None,
            is_leader: false,
        },
        community: None,
        user_reaction: None,
        is_bookmarked: false,
    };

    Ok(Html(render_post_card_html(&post_with_author)))
}

pub async fn handle_get_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Post>, SocialError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;

        let db_post: DbPost = social_posts::table
            .filter(social_posts::id.eq(post_id))
            .filter(social_posts::deleted_at.is_null())
            .first(&mut conn)
            .map_err(|_| SocialError::NotFound("Post not found".to_string()))?;

        Ok::<_, SocialError>(db_post_to_post(db_post))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_update_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<Post>, SocialError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut db_post: DbPost = social_posts::table
            .filter(social_posts::id.eq(post_id))
            .filter(social_posts::deleted_at.is_null())
            .first(&mut conn)
            .map_err(|_| SocialError::NotFound("Post not found".to_string()))?;

        if let Some(content) = req.content {
            db_post.content = content;
            db_post.edited_at = Some(now);
        }
        if let Some(visibility) = req.visibility {
            db_post.visibility = visibility;
        }
        if let Some(is_pinned) = req.is_pinned {
            db_post.is_pinned = is_pinned;
        }
        db_post.updated_at = now;

        diesel::update(social_posts::table.filter(social_posts::id.eq(post_id)))
            .set((
                social_posts::content.eq(&db_post.content),
                social_posts::visibility.eq(&db_post.visibility),
                social_posts::is_pinned.eq(db_post.is_pinned),
                social_posts::edited_at.eq(db_post.edited_at),
                social_posts::updated_at.eq(db_post.updated_at),
            ))
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(db_post_to_post(db_post))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_delete_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, SocialError> {
    let pool = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let now = Utc::now();

        diesel::update(social_posts::table.filter(social_posts::id.eq(post_id)))
            .set(social_posts::deleted_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(())
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_communities(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Community>>, SocialError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let db_communities: Vec<DbCommunity> = social_communities::table
            .filter(social_communities::bot_id.eq(bot_id))
            .filter(social_communities::archived_at.is_null())
            .order(social_communities::member_count.desc())
            .limit(50)
            .load(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(db_communities.into_iter().map(db_community_to_community).collect())
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_community(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCommunityRequest>,
) -> Result<Json<Community>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot(&mut conn);
        let org_id = Uuid::nil();
        let now = Utc::now();

        let slug = req
            .slug
            .unwrap_or_else(|| req.name.to_lowercase().replace(' ', "-"));

        let db_community = DbCommunity {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            name: req.name,
            slug,
            description: req.description,
            cover_image: None,
            icon: None,
            visibility: req.visibility.unwrap_or_else(|| "private".to_string()),
            join_policy: req.join_policy.unwrap_or_else(|| "open".to_string()),
            owner_id: user_id,
            member_count: 1,
            post_count: 0,
            is_official: false,
            is_featured: false,
            settings: serde_json::json!({}),
            created_at: now,
            updated_at: now,
            archived_at: None,
        };

        diesel::insert_into(social_communities::table)
            .values(&db_community)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        let member = DbCommunityMember {
            id: Uuid::new_v4(),
            community_id: db_community.id,
            user_id,
            role: "owner".to_string(),
            notifications_enabled: true,
            joined_at: now,
            last_seen_at: Some(now),
        };

        diesel::insert_into(social_community_members::table)
            .values(&member)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(db_community_to_community(db_community))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_community(
    State(state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<Community>, SocialError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;

        let db_community: DbCommunity = social_communities::table
            .filter(social_communities::id.eq(community_id))
            .filter(social_communities::archived_at.is_null())
            .first(&mut conn)
            .map_err(|_| SocialError::NotFound("Community not found".to_string()))?;

        Ok::<_, SocialError>(db_community_to_community(db_community))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_join_community(
    State(state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<CommunityMember>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let now = Utc::now();

        let member = DbCommunityMember {
            id: Uuid::new_v4(),
            community_id,
            user_id,
            role: "member".to_string(),
            notifications_enabled: true,
            joined_at: now,
            last_seen_at: Some(now),
        };

        diesel::insert_into(social_community_members::table)
            .values(&member)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        diesel::update(social_communities::table.filter(social_communities::id.eq(community_id)))
            .set(social_communities::member_count.eq(social_communities::member_count + 1))
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(db_member_to_member(member))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_leave_community(
    State(state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;

        diesel::delete(
            social_community_members::table
                .filter(social_community_members::community_id.eq(community_id))
                .filter(social_community_members::user_id.eq(user_id)),
        )
        .execute(&mut conn)
        .map_err(|e| SocialError::Database(e.to_string()))?;

        diesel::update(social_communities::table.filter(social_communities::id.eq(community_id)))
            .set(social_communities::member_count.eq(social_communities::member_count - 1))
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(())
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_add_reaction(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateReactionRequest>,
) -> Result<Json<Reaction>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let now = Utc::now();

        let db_reaction = DbReaction {
            id: Uuid::new_v4(),
            post_id: Some(post_id),
            comment_id: None,
            user_id,
            reaction_type: req.reaction_type.clone(),
            created_at: now,
        };

        diesel::insert_into(social_reactions::table)
            .values(&db_reaction)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(Reaction {
            id: db_reaction.id,
            post_id,
            user_id,
            reaction_type: req.reaction_type,
            created_at: now,
        })
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_remove_reaction(
    State(state): State<Arc<AppState>>,
    Path((post_id, reaction_type)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;

        diesel::delete(
            social_reactions::table
                .filter(social_reactions::post_id.eq(post_id))
                .filter(social_reactions::user_id.eq(user_id))
                .filter(social_reactions::reaction_type.eq(reaction_type)),
        )
        .execute(&mut conn)
        .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(())
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_get_comments(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Query(query): Query<FeedQuery>,
) -> Result<Json<Vec<Comment>>, SocialError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;

        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let db_comments: Vec<DbComment> = social_comments::table
            .filter(social_comments::post_id.eq(post_id))
            .filter(social_comments::deleted_at.is_null())
            .order(social_comments::created_at.asc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(db_comments.into_iter().map(db_comment_to_comment).collect())
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_add_comment(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, SocialError> {
    let pool = state.conn.clone();
    let user_id = Uuid::nil();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let now = Utc::now();

        let db_comment = DbComment {
            id: Uuid::new_v4(),
            post_id,
            parent_comment_id: req.parent_comment_id,
            author_id: user_id,
            content: req.content,
            mentions: serde_json::to_value(&req.mentions.unwrap_or_default()).unwrap_or_default(),
            reaction_counts: serde_json::json!({}),
            reply_count: 0,
            created_at: now,
            updated_at: now,
            edited_at: None,
            deleted_at: None,
        };

        diesel::insert_into(social_comments::table)
            .values(&db_comment)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        diesel::update(social_posts::table.filter(social_posts::id.eq(post_id)))
            .set(social_posts::comment_count.eq(social_posts::comment_count + 1))
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(db_comment_to_comment(db_comment))
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_send_praise(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePraiseRequest>,
) -> Result<Json<Praise>, SocialError> {
    let pool = state.conn.clone();
    let from_user_id = Uuid::nil();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| SocialError::Database(e.to_string()))?;
        let (bot_id, _bot_name) = get_default_bot(&mut conn);
        let org_id = Uuid::nil();
        let now = Utc::now();

        let db_praise = DbPraise {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            from_user_id,
            to_user_id: req.to_user_id,
            badge_type: req.badge_type.clone(),
            message: Some(req.message.clone()),
            is_public: req.is_public.unwrap_or(true),
            post_id: None,
            created_at: now,
        };

        diesel::insert_into(social_praises::table)
            .values(&db_praise)
            .execute(&mut conn)
            .map_err(|e| SocialError::Database(e.to_string()))?;

        Ok::<_, SocialError>(Praise {
            id: db_praise.id,
            organization_id: org_id,
            from_user_id,
            to_user_id: req.to_user_id,
            badge_type: req.badge_type,
            message: req.message,
            is_public: db_praise.is_public,
            post_id: None,
            created_at: now,
        })
    })
    .await
    .map_err(|e| SocialError::Internal(e.to_string()))??;

    Ok(Json(result))
}

fn render_feed_html(posts: &[PostWithAuthor]) -> String {
    if posts.is_empty() {
        return r##"<div class="empty-feed"><p>No posts yet. Be the first to share something!</p></div>"##.to_string();
    }
    posts.iter().map(render_post_card_html).collect()
}

fn render_post_card_html(post: &PostWithAuthor) -> String {
    let reactions_html: String = post
        .post
        .reaction_counts
        .iter()
        .map(|(emoji, count)| format!(r##"<span class="reaction">{emoji} {count}</span>"##))
        .collect();

    let avatar_url = post.author.avatar_url.as_deref().unwrap_or("/assets/default-avatar.svg");
    let post_time = post.post.created_at.format("%b %d, %Y");

    format!(
        r##"<article class="post-card" data-post-id="{id}">
<header class="post-header">
<img class="avatar" src="{avatar}" alt="{name}" />
<div class="post-meta"><span class="author-name">{name}</span><span class="post-time">{time}</span></div>
</header>
<div class="post-content">{content}</div>
<footer class="post-footer">
<div class="reactions">{reactions}</div>
<div class="post-actions">
<button class="btn-react" hx-post="/api/social/posts/{id}/react" hx-swap="outerHTML">Like</button>
<button class="btn-comment" hx-get="/api/social/posts/{id}/comments" hx-target="#comments-{id}">Comment {comments}</button>
</div>
</footer>
<div id="comments-{id}" class="comments-section"></div>
</article>"##,
        id = post.post.id,
        avatar = avatar_url,
        name = post.author.name,
        time = post_time,
        content = post.post.content,
        reactions = reactions_html,
        comments = post.post.comment_count,
    )
}

async fn handle_get_suggested_communities_html(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let pool = state.conn.clone();

    let communities = tokio::task::spawn_blocking(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let (bot_id, _) = get_default_bot(&mut conn);

        social_communities::table
            .filter(social_communities::bot_id.eq(bot_id))
            .filter(social_communities::archived_at.is_null())
            .filter(social_communities::visibility.eq("public"))
            .order(social_communities::member_count.desc())
            .limit(5)
            .load::<DbCommunity>(&mut conn)
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    if communities.is_empty() {
        return Html(r##"<div class="empty-suggestions"><p>No communities available</p></div>"##.to_string());
    }

    let html: String = communities
        .into_iter()
        .map(|c| {
            format!(
                r##"<div class="community-suggestion">
<div class="community-avatar">{}</div>
<div class="community-info">
<span class="community-name">{}</span>
<span class="community-members">{} members</span>
</div>
<button class="btn-join" hx-post="/api/social/communities/{}/join" hx-swap="outerHTML">Join</button>
</div>"##,
                c.icon.as_deref().unwrap_or("🌐"),
                c.name,
                c.member_count,
                c.id
            )
        })
        .collect();

    Html(html)
}

pub fn configure_social_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/social/feed", get(handle_get_feed))
        .route("/api/ui/social/feed", get(handle_get_feed_html))
        .route("/api/ui/social/suggested", get(handle_get_suggested_communities_html))
        .route("/api/social/posts", post(handle_create_post))
        .route("/api/social/posts/{id}", get(handle_get_post).put(handle_update_post).delete(handle_delete_post))
        .route("/api/social/posts/{id}/react", post(handle_add_reaction))
        .route("/api/social/posts/{id}/react/{type}", delete(handle_remove_reaction))
        .route("/api/social/posts/{id}/comments", get(handle_get_comments).post(handle_add_comment))
        .route("/api/social/communities", get(handle_list_communities).post(handle_create_community))
        .route("/api/social/communities/{id}", get(handle_get_community))
        .route("/api/social/communities/{id}/join", post(handle_join_community))
        .route("/api/social/communities/{id}/leave", post(handle_leave_community))
        .route("/api/social/praise", post(handle_send_praise))
}
