use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub author_id: Uuid,
    pub community_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub content_type: ContentType,
    pub attachments: Vec<Attachment>,
    pub mentions: Vec<Uuid>,
    pub hashtags: Vec<String>,
    pub visibility: PostVisibility,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    RichText,
    Markdown,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostVisibility {
    Public,
    Organization,
    Community,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub file_type: AttachmentType,
    pub url: String,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub thumbnail_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentType {
    Image,
    Video,
    Document,
    Link,
    File,
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
    pub visibility: CommunityVisibility,
    pub join_policy: JoinPolicy,
    pub owner_id: Uuid,
    pub admin_ids: Vec<Uuid>,
    pub moderator_ids: Vec<Uuid>,
    pub member_count: i32,
    pub post_count: i32,
    pub is_official: bool,
    pub is_featured: bool,
    pub settings: CommunitySettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommunityVisibility {
    Public,
    Private,
    Secret,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JoinPolicy {
    Open,
    Approval,
    InviteOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunitySettings {
    pub allow_member_posts: bool,
    pub require_post_approval: bool,
    pub allow_comments: bool,
    pub allow_reactions: bool,
    pub allow_polls: bool,
    pub allow_attachments: bool,
    pub allowed_attachment_types: Vec<AttachmentType>,
    pub max_attachment_size_mb: i32,
    pub enable_notifications: bool,
    pub custom_theme: Option<CommunityTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityTheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: Option<String>,
    pub header_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMember {
    pub community_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub notifications_enabled: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Admin,
    Moderator,
    Member,
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
    pub voters: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub priority: AnnouncementPriority,
    pub target_audience: TargetAudience,
    pub is_pinned: bool,
    pub requires_acknowledgment: bool,
    pub acknowledged_by: Vec<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetAudience {
    pub all_organization: bool,
    pub community_ids: Vec<Uuid>,
    pub role_ids: Vec<Uuid>,
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Praise {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub badge_type: PraiseBadge,
    pub message: String,
    pub is_public: bool,
    pub post_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PraiseBadge {
    ThankYou,
    GreatWork,
    TeamPlayer,
    Innovator,
    Leader,
    Helper,
    Mentor,
    RockStar,
    Custom(String),
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub community_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub hashtag: Option<String>,
    pub search: Option<String>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
    pub content_type: Option<ContentType>,
    pub community_id: Option<Uuid>,
    pub visibility: Option<PostVisibility>,
    pub mentions: Option<Vec<Uuid>>,
    pub hashtags: Option<Vec<String>>,
    pub attachments: Option<Vec<AttachmentRequest>>,
    pub poll: Option<CreatePollRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostForm {
    pub content: String,
    pub visibility: Option<String>,
    pub community_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttachmentRequest {
    pub file_type: AttachmentType,
    pub url: String,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub content: Option<String>,
    pub visibility: Option<PostVisibility>,
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommunityRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: String,
    pub visibility: Option<CommunityVisibility>,
    pub join_policy: Option<JoinPolicy>,
    pub settings: Option<CommunitySettings>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommunityRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub icon: Option<String>,
    pub visibility: Option<CommunityVisibility>,
    pub join_policy: Option<JoinPolicy>,
    pub settings: Option<CommunitySettings>,
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
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub content: String,
    pub priority: Option<AnnouncementPriority>,
    pub target_audience: Option<TargetAudience>,
    pub requires_acknowledgment: Option<bool>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePraiseRequest {
    pub to_user_id: Uuid,
    pub badge_type: PraiseBadge,
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

#[derive(Debug, Serialize)]
pub struct StaticSiteExport {
    pub community_html: String,
    pub posts_html: Vec<String>,
    pub assets: Vec<String>,
    pub metadata: StaticSiteMetadata,
}

#[derive(Debug, Serialize)]
pub struct StaticSiteMetadata {
    pub title: String,
    pub description: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SocialService {}

impl SocialService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_feed(
        &self,
        _organization_id: Uuid,
        _user_id: Uuid,
        _query: &FeedQuery,
    ) -> Result<FeedResponse, SocialError> {
        Ok(FeedResponse {
            posts: vec![],
            has_more: false,
            next_cursor: None,
        })
    }

    pub async fn create_post(
        &self,
        organization_id: Uuid,
        author_id: Uuid,
        req: CreatePostRequest,
    ) -> Result<Post, SocialError> {
        let now = Utc::now();
        Ok(Post {
            id: Uuid::new_v4(),
            organization_id,
            author_id,
            community_id: req.community_id,
            parent_id: None,
            content: req.content,
            content_type: req.content_type.unwrap_or(ContentType::Text),
            attachments: req
                .attachments
                .map(|a| {
                    a.into_iter()
                        .map(|att| Attachment {
                            id: Uuid::new_v4(),
                            file_type: att.file_type,
                            url: att.url,
                            name: att.name,
                            size: att.size,
                            mime_type: att.mime_type,
                            thumbnail_url: None,
                            metadata: None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            mentions: req.mentions.unwrap_or_default(),
            hashtags: req.hashtags.unwrap_or_default(),
            visibility: req.visibility.unwrap_or(PostVisibility::Organization),
            is_announcement: false,
            is_pinned: false,
            poll_id: None,
            reaction_counts: HashMap::new(),
            comment_count: 0,
            share_count: 0,
            view_count: 0,
            created_at: now,
            updated_at: now,
            edited_at: None,
            deleted_at: None,
        })
    }

    pub async fn get_post(
        &self,
        _organization_id: Uuid,
        _post_id: Uuid,
    ) -> Result<Option<Post>, SocialError> {
        Ok(None)
    }

    pub async fn update_post(
        &self,
        _organization_id: Uuid,
        _post_id: Uuid,
        _user_id: Uuid,
        _req: UpdatePostRequest,
    ) -> Result<Post, SocialError> {
        Err(SocialError::NotFound("Post not found".to_string()))
    }

    pub async fn delete_post(
        &self,
        _organization_id: Uuid,
        _post_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), SocialError> {
        Ok(())
    }

    pub async fn list_communities(
        &self,
        _organization_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<Community>, SocialError> {
        Ok(vec![])
    }

    pub async fn create_community(
        &self,
        organization_id: Uuid,
        owner_id: Uuid,
        req: CreateCommunityRequest,
    ) -> Result<Community, SocialError> {
        let now = Utc::now();
        let slug = req
            .slug
            .unwrap_or_else(|| req.name.to_lowercase().replace(' ', "-"));

        Ok(Community {
            id: Uuid::new_v4(),
            organization_id,
            name: req.name,
            slug,
            description: req.description,
            cover_image: None,
            icon: None,
            visibility: req.visibility.unwrap_or(CommunityVisibility::Private),
            join_policy: req.join_policy.unwrap_or(JoinPolicy::Open),
            owner_id,
            admin_ids: vec![owner_id],
            moderator_ids: vec![],
            member_count: 1,
            post_count: 0,
            is_official: false,
            is_featured: false,
            settings: req.settings.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            archived_at: None,
        })
    }

    pub async fn get_community(
        &self,
        _organization_id: Uuid,
        _community_id: Uuid,
    ) -> Result<Option<Community>, SocialError> {
        Ok(None)
    }

    pub async fn get_public_community_by_slug(
        &self,
        _slug: &str,
    ) -> Result<Option<Community>, SocialError> {
        Ok(None)
    }

    pub async fn update_community(
        &self,
        _organization_id: Uuid,
        _community_id: Uuid,
        _user_id: Uuid,
        _req: UpdateCommunityRequest,
    ) -> Result<Community, SocialError> {
        Err(SocialError::NotFound("Community not found".to_string()))
    }

    pub async fn join_community(
        &self,
        _organization_id: Uuid,
        community_id: Uuid,
        user_id: Uuid,
    ) -> Result<CommunityMember, SocialError> {
        Ok(CommunityMember {
            community_id,
            user_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            notifications_enabled: true,
            last_seen_at: None,
        })
    }

    pub async fn leave_community(
        &self,
        _organization_id: Uuid,
        _community_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), SocialError> {
        Ok(())
    }

    pub async fn add_reaction(
        &self,
        _organization_id: Uuid,
        post_id: Uuid,
        user_id: Uuid,
        reaction_type: &str,
    ) -> Result<Reaction, SocialError> {
        Ok(Reaction {
            id: Uuid::new_v4(),
            post_id,
            user_id,
            reaction_type: reaction_type.to_string(),
            created_at: Utc::now(),
        })
    }

    pub async fn remove_reaction(
        &self,
        _organization_id: Uuid,
        _post_id: Uuid,
        _user_id: Uuid,
        _reaction_type: &str,
    ) -> Result<(), SocialError> {
        Ok(())
    }

    pub async fn get_comments(
        &self,
        _organization_id: Uuid,
        _post_id: Uuid,
        _limit: Option<i32>,
        _offset: Option<i32>,
    ) -> Result<Vec<Comment>, SocialError> {
        Ok(vec![])
    }

    pub async fn add_comment(
        &self,
        _organization_id: Uuid,
        post_id: Uuid,
        user_id: Uuid,
        req: CreateCommentRequest,
    ) -> Result<Comment, SocialError> {
        let now = Utc::now();
        Ok(Comment {
            id: Uuid::new_v4(),
            post_id,
            parent_comment_id: req.parent_comment_id,
            author_id: user_id,
            content: req.content,
            mentions: req.mentions.unwrap_or_default(),
            reaction_counts: HashMap::new(),
            reply_count: 0,
            created_at: now,
            updated_at: now,
            edited_at: None,
            deleted_at: None,
        })
    }

    pub async fn create_poll(
        &self,
        _organization_id: Uuid,
        post_id: Uuid,
        req: CreatePollRequest,
    ) -> Result<Poll, SocialError> {
        Ok(Poll {
            id: Uuid::new_v4(),
            post_id,
            question: req.question,
            options: req
                .options
                .into_iter()
                .map(|text| PollOption {
                    id: Uuid::new_v4(),
                    text,
                    vote_count: 0,
                    voters: if req.anonymous.unwrap_or(false) {
                        None
                    } else {
                        Some(vec![])
                    },
                })
                .collect(),
            allow_multiple: req.allow_multiple.unwrap_or(false),
            allow_add_options: req.allow_add_options.unwrap_or(false),
            anonymous: req.anonymous.unwrap_or(false),
            ends_at: req.ends_at,
            total_votes: 0,
            created_at: Utc::now(),
        })
    }

    pub async fn vote_poll(
        &self,
        _organization_id: Uuid,
        _poll_id: Uuid,
        _user_id: Uuid,
        _option_ids: Vec<Uuid>,
    ) -> Result<Poll, SocialError> {
        Err(SocialError::NotFound("Poll not found".to_string()))
    }

    pub async fn get_announcements(
        &self,
        _organization_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<Announcement>, SocialError> {
        Ok(vec![])
    }

    pub async fn create_announcement(
        &self,
        organization_id: Uuid,
        author_id: Uuid,
        req: CreateAnnouncementRequest,
    ) -> Result<Announcement, SocialError> {
        let now = Utc::now();
        Ok(Announcement {
            id: Uuid::new_v4(),
            organization_id,
            author_id,
            title: req.title,
            content: req.content,
            priority: req.priority.unwrap_or(AnnouncementPriority::Normal),
            target_audience: req.target_audience.unwrap_or(TargetAudience {
                all_organization: true,
                community_ids: vec![],
                role_ids: vec![],
                user_ids: vec![],
            }),
            is_pinned: false,
            requires_acknowledgment: req.requires_acknowledgment.unwrap_or(false),
            acknowledged_by: vec![],
            starts_at: req.starts_at.unwrap_or(now),
            ends_at: req.ends_at,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn send_praise(
        &self,
        organization_id: Uuid,
        from_user_id: Uuid,
        req: CreatePraiseRequest,
    ) -> Result<Praise, SocialError> {
        Ok(Praise {
            id: Uuid::new_v4(),
            organization_id,
            from_user_id,
            to_user_id: req.to_user_id,
            badge_type: req.badge_type,
            message: req.message,
            is_public: req.is_public.unwrap_or(true),
            post_id: None,
            created_at: Utc::now(),
        })
    }

    pub async fn export_community_to_static(
        &self,
        _organization_id: Uuid,
        _community_id: Uuid,
    ) -> Result<StaticSiteExport, SocialError> {
        Ok(StaticSiteExport {
            community_html: String::new(),
            posts_html: vec![],
            assets: vec![],
            metadata: StaticSiteMetadata {
                title: String::new(),
                description: String::new(),
                generated_at: Utc::now(),
            },
        })
    }
}

impl Default for SocialService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SocialError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Validation error: {0}")]
    Validation(String),
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
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) | Self::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn handle_get_feed(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<FeedQuery>,
) -> Result<Json<FeedResponse>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let feed = service.get_feed(org_id, user_id, &query).await?;
    Ok(Json(feed))
}

pub async fn handle_get_feed_html(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<FeedQuery>,
) -> Result<Html<String>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let feed = service.get_feed(org_id, user_id, &query).await?;
    Ok(Html(render_feed_html(&feed.posts)))
}

pub async fn handle_create_post(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<CreatePostForm>,
) -> Result<Html<String>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();

    let visibility = form.visibility.as_deref().and_then(|v| match v {
        "public" => Some(PostVisibility::Public),
        "organization" => Some(PostVisibility::Organization),
        "community" => Some(PostVisibility::Community),
        _ => None,
    });

    let community_id = form
        .community_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok());

    let req = CreatePostRequest {
        content: form.content,
        content_type: None,
        community_id,
        visibility,
        mentions: None,
        hashtags: None,
        attachments: None,
        poll: None,
    };

    let post = service.create_post(org_id, user_id, req).await?;

    let post_with_author = PostWithAuthor {
        post,
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
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Option<Post>>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let post = service.get_post(org_id, post_id).await?;
    Ok(Json(post))
}

pub async fn handle_update_post(
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<Post>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let post = service.update_post(org_id, post_id, user_id, req).await?;
    Ok(Json(post))
}

pub async fn handle_delete_post(
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    service.delete_post(org_id, post_id, user_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_communities(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<Community>>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let communities = service.list_communities(org_id, user_id).await?;
    Ok(Json(communities))
}

pub async fn handle_create_community(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateCommunityRequest>,
) -> Result<Json<Community>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let community = service.create_community(org_id, user_id, req).await?;
    Ok(Json(community))
}

pub async fn handle_get_community(
    State(_state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<Option<Community>>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let community = service.get_community(org_id, community_id).await?;
    Ok(Json(community))
}

pub async fn handle_update_community(
    State(_state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
    Json(req): Json<UpdateCommunityRequest>,
) -> Result<Json<Community>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let community = service
        .update_community(org_id, community_id, user_id, req)
        .await?;
    Ok(Json(community))
}

pub async fn handle_join_community(
    State(_state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<CommunityMember>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let member = service
        .join_community(org_id, community_id, user_id)
        .await?;
    Ok(Json(member))
}

pub async fn handle_leave_community(
    State(_state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    service
        .leave_community(org_id, community_id, user_id)
        .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_add_reaction(
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateReactionRequest>,
) -> Result<Json<Reaction>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let reaction = service
        .add_reaction(org_id, post_id, user_id, &req.reaction_type)
        .await?;
    Ok(Json(reaction))
}

pub async fn handle_remove_reaction(
    State(_state): State<Arc<AppState>>,
    Path((post_id, reaction_type)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    service
        .remove_reaction(org_id, post_id, user_id, &reaction_type)
        .await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_get_comments(
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Query(query): Query<FeedQuery>,
) -> Result<Json<Vec<Comment>>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let comments = service
        .get_comments(org_id, post_id, query.limit, query.offset)
        .await?;
    Ok(Json(comments))
}

pub async fn handle_add_comment(
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let comment = service.add_comment(org_id, post_id, user_id, req).await?;
    Ok(Json(comment))
}

pub async fn handle_create_poll(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreatePollRequest>,
) -> Result<Json<Poll>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let post_id = Uuid::nil();
    let poll = service.create_poll(org_id, post_id, req).await?;
    Ok(Json(poll))
}

pub async fn handle_vote_poll(
    State(_state): State<Arc<AppState>>,
    Path(poll_id): Path<Uuid>,
    Json(req): Json<VotePollRequest>,
) -> Result<Json<Poll>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let poll = service
        .vote_poll(org_id, poll_id, user_id, req.option_ids)
        .await?;
    Ok(Json(poll))
}

pub async fn handle_get_announcements(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<Announcement>>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let announcements = service.get_announcements(org_id, user_id).await?;
    Ok(Json(announcements))
}

pub async fn handle_create_announcement(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateAnnouncementRequest>,
) -> Result<Json<Announcement>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let announcement = service.create_announcement(org_id, user_id, req).await?;
    Ok(Json(announcement))
}

pub async fn handle_get_public_community(
    State(_state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Option<Community>>, SocialError> {
    let service = SocialService::new();
    let community = service.get_public_community_by_slug(&slug).await?;
    Ok(Json(community))
}

pub async fn handle_get_public_community_html(
    State(_state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, SocialError> {
    let service = SocialService::new();
    let community = service.get_public_community_by_slug(&slug).await?;
    match community {
        Some(c) => Ok(Html(render_public_community_html(&c))),
        None => Err(SocialError::NotFound("Community not found".to_string())),
    }
}

pub async fn handle_export_community(
    State(_state): State<Arc<AppState>>,
    Path(community_id): Path<Uuid>,
) -> Result<Json<StaticSiteExport>, SocialError> {
    let service = SocialService::new();
    let org_id = Uuid::nil();
    let export = service
        .export_community_to_static(org_id, community_id)
        .await?;
    Ok(Json(export))
}

fn render_feed_html(posts: &[PostWithAuthor]) -> String {
    if posts.is_empty() {
        return r#"<div class="empty-feed"><p>No posts yet. Be the first to share something!</p></div>"#.to_string();
    }
    posts.iter().map(render_post_card_html).collect()
}

fn render_post_card_html(post: &PostWithAuthor) -> String {
    let reactions_html: String = post
        .post
        .reaction_counts
        .iter()
        .map(|(emoji, count)| format!("<span class=\"reaction\">{emoji} {count}</span>"))
        .collect();

    let avatar_url = post.author.avatar_url.as_deref().unwrap_or("/assets/default-avatar.svg");
    let post_time = post.post.created_at.format("%b %d, %Y");

    format!(
        "<article class=\"post-card\" data-post-id=\"{id}\">\
         <header class=\"post-header\">\
         <img class=\"avatar\" src=\"{avatar}\" alt=\"{name}\" />\
         <div class=\"post-meta\"><span class=\"author-name\">{name}</span><span class=\"post-time\">{time}</span></div>\
         </header>\
         <div class=\"post-content\">{content}</div>\
         <footer class=\"post-footer\">\
         <div class=\"reactions\">{reactions}</div>\
         <div class=\"post-actions\">\
         <button class=\"btn-react\" hx-post=\"/api/social/posts/{id}/react\" hx-swap=\"outerHTML\">Like</button>\
         <button class=\"btn-comment\" hx-get=\"/api/social/posts/{id}/comments\" hx-target=\"#comments-{id}\">Comment {comments}</button>\
         </div>\
         </footer>\
         <div id=\"comments-{id}\" class=\"comments-section\"></div>\
         </article>",
        id = post.post.id,
        avatar = avatar_url,
        name = post.author.name,
        time = post_time,
        content = post.post.content,
        reactions = reactions_html,
        comments = post.post.comment_count,
    )
}

fn render_public_community_html(community: &Community) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{} - Community</title>
<style>
:root {{ --primary: #6366f1; --bg: #0f172a; --surface: #1e293b; --text: #f8fafc; }}
body {{ font-family: system-ui, sans-serif; background: var(--bg); color: var(--text); margin: 0; }}
.community-header {{ background: var(--surface); padding: 2rem; text-align: center; }}
.posts-container {{ max-width: 800px; margin: 0 auto; padding: 1rem; }}
</style>
</head>
<body>
<header class="community-header">
<h1>{}</h1>
<p>{}</p>
<p>{} members</p>
</header>
<main class="posts-container">
<div id="posts" hx-get="/api/public/community/{}/posts" hx-trigger="load" hx-swap="innerHTML">Loading...</div>
</main>
<script src="https://unpkg.com/htmx.org@1.9.10"></script>
</body>
</html>"#,
        community.name,
        community.name,
        community.description,
        community.member_count,
        community.slug
    )
}

async fn handle_get_suggested_communities_html() -> Html<String> {
    Html(r#"
        <div class="community-suggestion">
            <div class="community-avatar">🌐</div>
            <div class="community-info">
                <span class="community-name">General Discussion</span>
                <span class="community-members">128 members</span>
            </div>
            <button class="btn-join" hx-post="/api/social/communities/general/join" hx-swap="outerHTML">Join</button>
        </div>
        <div class="community-suggestion">
            <div class="community-avatar">💡</div>
            <div class="community-info">
                <span class="community-name">Ideas & Feedback</span>
                <span class="community-members">64 members</span>
            </div>
            <button class="btn-join" hx-post="/api/social/communities/ideas/join" hx-swap="outerHTML">Join</button>
        </div>
        <div class="community-suggestion">
            <div class="community-avatar">🎉</div>
            <div class="community-info">
                <span class="community-name">Announcements</span>
                <span class="community-members">256 members</span>
            </div>
            <button class="btn-join" hx-post="/api/social/communities/announcements/join" hx-swap="outerHTML">Join</button>
        </div>
    "#.to_string())
}

pub fn configure_social_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/social/feed", get(handle_get_feed))
        .route("/api/ui/social/feed", get(handle_get_feed_html))
        .route("/api/ui/social/suggested", get(handle_get_suggested_communities_html))
        .route("/api/social/posts", post(handle_create_post))
        .route("/api/social/posts/:id", get(handle_get_post))
        .route("/api/social/posts/:id", put(handle_update_post))
        .route("/api/social/posts/:id", delete(handle_delete_post))
        .route("/api/social/posts/:id/react", post(handle_add_reaction))
        .route(
            "/api/social/posts/:id/react/:type",
            delete(handle_remove_reaction),
        )
        .route("/api/social/posts/:id/comments", get(handle_get_comments))
        .route("/api/social/posts/:id/comments", post(handle_add_comment))
        .route("/api/social/communities", get(handle_list_communities))
        .route("/api/social/communities", post(handle_create_community))
        .route("/api/social/communities/:id", get(handle_get_community))
        .route("/api/social/communities/:id", put(handle_update_community))
        .route(
            "/api/social/communities/:id/join",
            post(handle_join_community),
        )
        .route(
            "/api/social/communities/:id/leave",
            post(handle_leave_community),
        )
        .route(
            "/api/social/communities/:id/export",
            post(handle_export_community),
        )
        .route("/api/social/polls", post(handle_create_poll))
        .route("/api/social/polls/:id/vote", post(handle_vote_poll))
        .route("/api/social/announcements", get(handle_get_announcements))
        .route("/api/social/announcements", post(handle_create_announcement))
        .route(
            "/api/public/community/:slug",
            get(handle_get_public_community),
        )
        .route("/community/:slug", get(handle_get_public_community_html))
}
