//! Web module with Askama templates for HTMX and authentication

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    middleware,
    response::{Html, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_cookies::CookieManagerLayer;
use uuid::Uuid;

use crate::shared::state::AppState;

// Authentication modules
pub mod auth;
pub mod auth_handlers;
pub mod chat_handlers;

// Module stubs - to be implemented with full HTMX
pub mod drive {
    use super::*;
    use crate::web::auth::AuthenticatedUser;

    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/api/files/list", get(list_files))
            .route("/api/files/read", post(read_file))
            .route("/api/files/write", post(write_file))
            .route("/api/files/delete", post(delete_file))
            .route("/api/files/create-folder", post(create_folder))
            .route("/api/files/download", get(download_file))
            .route("/api/files/share", get(share_file))
    }

    pub async fn drive_page(AuthenticatedUser { claims }: AuthenticatedUser) -> impl IntoResponse {
        DriveTemplate {
            user_name: claims.name,
            user_email: claims.email,
        }
    }

    #[derive(Template)]
    #[template(path = "drive.html")]
    struct DriveTemplate {
        user_name: String,
        user_email: String,
    }

    async fn list_files(
        Query(params): Query<HashMap<String, String>>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        // Implementation will connect to actual S3/MinIO backend
        Json(serde_json::json!([]))
    }

    async fn read_file(
        Json(payload): Json<FileRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "content": ""
        }))
    }

    async fn write_file(
        Json(payload): Json<WriteFileRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true
        }))
    }

    async fn delete_file(
        Json(payload): Json<FileRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true
        }))
    }

    async fn create_folder(
        Json(payload): Json<CreateFolderRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true
        }))
    }

    async fn download_file(
        Query(params): Query<HashMap<String, String>>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        StatusCode::NOT_IMPLEMENTED
    }

    async fn share_file(
        Query(params): Query<HashMap<String, String>>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "share_url": ""
        }))
    }

    #[derive(Deserialize)]
    struct FileRequest {
        bucket: Option<String>,
        path: String,
    }

    #[derive(Deserialize)]
    struct WriteFileRequest {
        bucket: Option<String>,
        path: String,
        content: String,
    }

    #[derive(Deserialize)]
    struct CreateFolderRequest {
        bucket: Option<String>,
        path: String,
        name: String,
    }
}

pub mod mail {
    use super::*;
    use crate::web::auth::AuthenticatedUser;

    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/api/email/accounts", get(get_accounts))
            .route("/api/email/list", post(list_emails))
            .route("/api/email/send", post(send_email))
            .route("/api/email/delete", post(delete_email))
            .route("/api/email/mark", post(mark_email))
            .route("/api/email/draft", post(save_draft))
    }

    pub async fn mail_page(AuthenticatedUser { claims }: AuthenticatedUser) -> impl IntoResponse {
        MailTemplate {
            user_name: claims.name,
            user_email: claims.email,
        }
    }

    #[derive(Template)]
    #[template(path = "mail.html")]
    struct MailTemplate {
        user_name: String,
        user_email: String,
    }

    async fn get_accounts(AuthenticatedUser { claims }: AuthenticatedUser) -> impl IntoResponse {
        // Will integrate with actual email service
        Json(serde_json::json!({
            "success": true,
            "data": [{
                "id": "1",
                "email": claims.email,
                "display_name": claims.name,
                "is_primary": true
            }]
        }))
    }

    async fn list_emails(
        Json(payload): Json<ListEmailsRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true,
            "data": []
        }))
    }

    async fn send_email(
        Json(payload): Json<SendEmailRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true,
            "message_id": Uuid::new_v4().to_string()
        }))
    }

    async fn delete_email(
        Json(payload): Json<EmailActionRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true
        }))
    }

    async fn mark_email(
        Json(payload): Json<MarkEmailRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true
        }))
    }

    async fn save_draft(
        Json(payload): Json<SendEmailRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true,
            "draft_id": Uuid::new_v4().to_string()
        }))
    }

    #[derive(Deserialize)]
    struct ListEmailsRequest {
        account_id: String,
        folder: String,
        limit: usize,
        offset: usize,
    }

    #[derive(Deserialize)]
    struct SendEmailRequest {
        account_id: String,
        to: String,
        cc: Option<String>,
        bcc: Option<String>,
        subject: String,
        body: String,
        is_html: bool,
    }

    #[derive(Deserialize)]
    struct EmailActionRequest {
        account_id: String,
        email_id: String,
    }

    #[derive(Deserialize)]
    struct MarkEmailRequest {
        account_id: String,
        email_id: String,
        read: bool,
    }
}

pub mod meet {
    use super::*;
    use crate::web::auth::AuthenticatedUser;

    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/api/meet/create", post(create_meeting))
            .route("/api/meet/token", post(get_meeting_token))
            .route("/api/meet/invite", post(send_invites))
    }

    pub async fn meet_page(AuthenticatedUser { claims }: AuthenticatedUser) -> impl IntoResponse {
        MeetTemplate {
            user_name: claims.name,
            user_email: claims.email,
        }
    }

    #[derive(Template)]
    #[template(path = "meet.html")]
    struct MeetTemplate {
        user_name: String,
        user_email: String,
    }

    pub async fn websocket_handler(
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| handle_meet_socket(socket, state))
    }

    async fn handle_meet_socket(socket: axum::extract::ws::WebSocket, _state: AppState) {
        // WebRTC signaling implementation
    }

    async fn create_meeting(
        Json(payload): Json<CreateMeetingRequest>,
        AuthenticatedUser { claims }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "name": payload.name,
            "host": claims.email
        }))
    }

    async fn get_meeting_token(
        Json(payload): Json<TokenRequest>,
        AuthenticatedUser { claims }: AuthenticatedUser,
    ) -> impl IntoResponse {
        // Will integrate with LiveKit for actual tokens
        Json(serde_json::json!({
            "token": base64::encode(format!("{}:{}", payload.room_id, claims.sub))
        }))
    }

    async fn send_invites(
        Json(payload): Json<InviteRequest>,
        AuthenticatedUser { .. }: AuthenticatedUser,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "success": true,
            "sent": payload.emails.len()
        }))
    }

    #[derive(Deserialize)]
    struct CreateMeetingRequest {
        name: String,
        description: Option<String>,
        settings: Option<MeetingSettings>,
    }

    #[derive(Deserialize)]
    struct MeetingSettings {
        enable_transcription: bool,
        enable_recording: bool,
        enable_bot: bool,
        waiting_room: bool,
    }

    #[derive(Deserialize)]
    struct TokenRequest {
        room_id: String,
        user_name: String,
    }

    #[derive(Deserialize)]
    struct InviteRequest {
        meeting_id: String,
        emails: Vec<String>,
    }
}

pub mod tasks {
    use super::*;
    use crate::web::auth::AuthenticatedUser;

    pub fn routes() -> Router<AppState> {
        Router::new()
    }

    pub async fn tasks_page(AuthenticatedUser { claims }: AuthenticatedUser) -> impl IntoResponse {
        TasksTemplate {
            user_name: claims.name,
            user_email: claims.email,
        }
    }

    #[derive(Template)]
    #[template(path = "tasks.html")]
    struct TasksTemplate {
        user_name: String,
        user_email: String,
    }
}

/// Base template data
#[derive(Default)]
pub struct BaseContext {
    pub user_name: String,
    pub user_email: String,
    pub user_initial: String,
}

/// Home page template
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    base: BaseContext,
    apps: Vec<AppCard>,
}

/// App card for home page
#[derive(Serialize)]
struct AppCard {
    name: String,
    icon: String,
    description: String,
    url: String,
}

/// Apps menu template
#[derive(Template)]
#[template(path = "partials/apps_menu.html")]
struct AppsMenuTemplate {
    apps: Vec<AppMenuItem>,
}

/// App menu item
#[derive(Serialize)]
struct AppMenuItem {
    name: String,
    icon: String,
    url: String,
    active: bool,
}

/// User menu template
#[derive(Template)]
#[template(path = "partials/user_menu.html")]
struct UserMenuTemplate {
    user_name: String,
    user_email: String,
    user_initial: String,
}

/// Create the main web router
pub fn create_router(app_state: AppState) -> Router {
    // Initialize authentication
    let auth_config = auth::AuthConfig::from_env();

    // Create session storage
    let sessions: Arc<RwLock<HashMap<String, auth::UserSession>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // Add to app state extensions
    let mut app_state = app_state;
    app_state.extensions.insert(auth_config.clone());
    app_state.extensions.insert(sessions);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/login", get(auth_handlers::login_page))
        .route("/auth/login", post(auth_handlers::login_submit))
        .route("/auth/callback", get(auth_handlers::oauth_callback))
        .route("/api/auth/mode", get(get_auth_mode))
        .route("/health", get(health_check));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Pages
        .route("/", get(home_handler))
        .route("/chat", get(chat_handlers::chat_page))
        .route("/drive", get(drive::drive_page))
        .route("/mail", get(mail::mail_page))
        .route("/meet", get(meet::meet_page))
        .route("/tasks", get(tasks::tasks_page))
        // Auth endpoints
        .route("/logout", post(auth_handlers::logout))
        .route("/api/auth/user", get(auth_handlers::get_user_info))
        .route("/api/auth/refresh", post(auth_handlers::refresh_token))
        .route("/api/auth/check", get(auth_handlers::check_session))
        // API endpoints
        .merge(chat_handlers::routes())
        .merge(drive::routes())
        .merge(mail::routes())
        .merge(meet::routes())
        .merge(tasks::routes())
        // Partials
        .route("/api/apps/menu", get(apps_menu_handler))
        .route("/api/user/menu", get(user_menu_handler))
        .route("/api/theme/toggle", post(toggle_theme_handler))
        // WebSocket endpoints
        .route("/ws", get(websocket_handler))
        .route("/ws/chat", get(chat_handlers::websocket_handler))
        .route("/ws/meet", get(meet::websocket_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CookieManagerLayer::new())
        .with_state(app_state)
}

/// Home page handler
async fn home_handler(
    State(_state): State<AppState>,
    auth::AuthenticatedUser { claims }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    let template = HomeTemplate {
        base: BaseContext {
            user_name: claims.name.clone(),
            user_email: claims.email.clone(),
            user_initial: claims
                .name
                .chars()
                .next()
                .unwrap_or('U')
                .to_uppercase()
                .to_string(),
        },
        apps: vec![
            AppCard {
                name: "Chat".to_string(),
                icon: "💬".to_string(),
                description: "AI-powered conversations".to_string(),
                url: "/chat".to_string(),
            },
            AppCard {
                name: "Drive".to_string(),
                icon: "📁".to_string(),
                description: "Secure file storage".to_string(),
                url: "/drive".to_string(),
            },
            AppCard {
                name: "Mail".to_string(),
                icon: "✉️".to_string(),
                description: "Email management".to_string(),
                url: "/mail".to_string(),
            },
            AppCard {
                name: "Meet".to_string(),
                icon: "🎥".to_string(),
                description: "Video conferencing".to_string(),
                url: "/meet".to_string(),
            },
            AppCard {
                name: "Tasks".to_string(),
                icon: "✓".to_string(),
                description: "Task management".to_string(),
                url: "/tasks".to_string(),
            },
        ],
    };

    template
}

/// Apps menu handler
async fn apps_menu_handler(
    State(_state): State<AppState>,
    auth::AuthenticatedUser { .. }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    let template = AppsMenuTemplate {
        apps: vec![
            AppMenuItem {
                name: "Chat".to_string(),
                icon: "💬".to_string(),
                url: "/chat".to_string(),
                active: false,
            },
            AppMenuItem {
                name: "Drive".to_string(),
                icon: "📁".to_string(),
                url: "/drive".to_string(),
                active: false,
            },
            AppMenuItem {
                name: "Mail".to_string(),
                icon: "✉️".to_string(),
                url: "/mail".to_string(),
                active: false,
            },
            AppMenuItem {
                name: "Meet".to_string(),
                icon: "🎥".to_string(),
                url: "/meet".to_string(),
                active: false,
            },
            AppMenuItem {
                name: "Tasks".to_string(),
                icon: "✓".to_string(),
                url: "/tasks".to_string(),
                active: false,
            },
        ],
    };

    template
}

/// User menu handler
async fn user_menu_handler(
    State(_state): State<AppState>,
    auth::AuthenticatedUser { claims }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    let template = UserMenuTemplate {
        user_name: claims.name.clone(),
        user_email: claims.email.clone(),
        user_initial: claims
            .name
            .chars()
            .next()
            .unwrap_or('U')
            .to_uppercase()
            .to_string(),
    };

    template
}

/// Theme toggle handler
async fn toggle_theme_handler(
    State(_state): State<AppState>,
    auth::AuthenticatedUser { .. }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    Response::builder()
        .header("HX-Trigger", "theme-changed")
        .body("".to_string())
        .unwrap()
}

/// Main WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    auth::AuthenticatedUser { claims }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    _state: AppState,
    claims: auth::Claims,
) {
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "user": claims.name,
        "session": claims.session_id
    });
    let _ = sender
        .send(axum::extract::ws::Message::Text(welcome.to_string()))
        .await;

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    // Echo back for now with user info
                    let response = serde_json::json!({
                        "type": "message",
                        "from": claims.name,
                        "content": text,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    let _ = sender
                        .send(axum::extract::ws::Message::Text(response.to_string()))
                        .await;
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get authentication mode (for login page)
async fn get_auth_mode(State(state): State<AppState>) -> impl IntoResponse {
    let auth_config = state.extensions.get::<auth::AuthConfig>();
    let mode = if auth_config.is_some() && !auth_config.unwrap().zitadel_client_secret.is_empty() {
        "production"
    } else {
        "development"
    };

    Json(serde_json::json!({
        "mode": mode
    }))
}

/// Common types for HTMX responses
#[derive(Serialize)]
pub struct HtmxResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
}

/// Notification for HTMX
#[derive(Serialize, Template)]
#[template(path = "partials/notification.html")]
pub struct NotificationTemplate {
    pub message: String,
    pub severity: String, // info, success, warning, error
}

/// Message template for chat/notifications
#[derive(Serialize, Template)]
#[template(path = "partials/message.html")]
pub struct MessageTemplate {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub is_user: bool,
}
