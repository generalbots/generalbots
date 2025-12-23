//! Centralized URL definitions for all API endpoints
//!
//! This module defines all API routes in a single place to avoid duplication
//! and ensure consistency across the application.

/// API endpoint paths
#[derive(Debug)]
pub struct ApiUrls;

impl ApiUrls {
    // ===== USER MANAGEMENT =====
    pub const USERS: &'static str = "/api/users";
    pub const USER_BY_ID: &'static str = "/api/users/:id";
    pub const USER_LOGIN: &'static str = "/api/users/login";
    pub const USER_LOGOUT: &'static str = "/api/users/logout";
    pub const USER_REGISTER: &'static str = "/api/users/register";
    pub const USER_PROFILE: &'static str = "/api/users/profile";
    pub const USER_PASSWORD: &'static str = "/api/users/password";
    pub const USER_SETTINGS: &'static str = "/api/users/settings";
    pub const USER_PROVISION: &'static str = "/api/users/provision";
    pub const USER_DEPROVISION: &'static str = "/api/users/:id/deprovision";

    // ===== GROUP MANAGEMENT =====
    pub const GROUPS: &'static str = "/api/groups";
    pub const GROUP_BY_ID: &'static str = "/api/groups/:id";
    pub const GROUP_MEMBERS: &'static str = "/api/groups/:id/members";
    pub const GROUP_ADD_MEMBER: &'static str = "/api/groups/:id/members/:user_id";
    pub const GROUP_REMOVE_MEMBER: &'static str = "/api/groups/:id/members/:user_id";
    pub const GROUP_PERMISSIONS: &'static str = "/api/groups/:id/permissions";

    // ===== AUTHENTICATION =====
    pub const AUTH: &'static str = "/api/auth";
    pub const AUTH_TOKEN: &'static str = "/api/auth/token";
    pub const AUTH_REFRESH: &'static str = "/api/auth/refresh";
    pub const AUTH_VERIFY: &'static str = "/api/auth/verify";
    pub const AUTH_OAUTH: &'static str = "/api/auth/oauth";
    pub const AUTH_OAUTH_CALLBACK: &'static str = "/api/auth/oauth/callback";

    // ===== SESSIONS =====
    pub const SESSIONS: &'static str = "/api/sessions";
    pub const SESSION_BY_ID: &'static str = "/api/sessions/:id";
    pub const SESSION_HISTORY: &'static str = "/api/sessions/:id/history";
    pub const SESSION_START: &'static str = "/api/sessions/:id/start";
    pub const SESSION_END: &'static str = "/api/sessions/:id/end";

    // ===== BOT MANAGEMENT =====
    pub const BOTS: &'static str = "/api/bots";
    pub const BOT_BY_ID: &'static str = "/api/bots/:id";
    pub const BOT_CONFIG: &'static str = "/api/bots/:id/config";
    pub const BOT_DEPLOY: &'static str = "/api/bots/:id/deploy";
    pub const BOT_LOGS: &'static str = "/api/bots/:id/logs";
    pub const BOT_METRICS: &'static str = "/api/bots/:id/metrics";

    // ===== DRIVE/STORAGE =====
    pub const DRIVE_LIST: &'static str = "/api/drive/list";
    pub const DRIVE_UPLOAD: &'static str = "/api/drive/upload";
    pub const DRIVE_DOWNLOAD: &'static str = "/api/drive/download/:path";
    pub const DRIVE_DELETE: &'static str = "/api/drive/delete/:path";
    pub const DRIVE_MKDIR: &'static str = "/api/drive/mkdir";
    pub const DRIVE_MOVE: &'static str = "/api/drive/move";
    pub const DRIVE_COPY: &'static str = "/api/drive/copy";
    pub const DRIVE_SHARE: &'static str = "/api/drive/share";

    // ===== EMAIL =====
    pub const EMAIL_ACCOUNTS: &'static str = "/api/email/accounts";
    pub const EMAIL_ACCOUNT_BY_ID: &'static str = "/api/email/accounts/:id";
    pub const EMAIL_LIST: &'static str = "/api/email/list";
    pub const EMAIL_SEND: &'static str = "/api/email/send";
    pub const EMAIL_DRAFT: &'static str = "/api/email/draft";
    pub const EMAIL_FOLDERS: &'static str = "/api/email/folders/:account_id";
    pub const EMAIL_LATEST: &'static str = "/api/email/latest";
    pub const EMAIL_GET: &'static str = "/api/email/get/:campaign_id";
    pub const EMAIL_CLICK: &'static str = "/api/email/click/:campaign_id/:email";

    // ===== CALENDAR =====
    pub const CALENDAR_EVENTS: &'static str = "/api/calendar/events";
    pub const CALENDAR_EVENT_BY_ID: &'static str = "/api/calendar/events/:id";
    pub const CALENDAR_REMINDERS: &'static str = "/api/calendar/reminders";
    pub const CALENDAR_SHARE: &'static str = "/api/calendar/share";
    pub const CALENDAR_SYNC: &'static str = "/api/calendar/sync";

    // ===== TASKS =====
    pub const TASKS: &'static str = "/api/tasks";
    pub const TASK_BY_ID: &'static str = "/api/tasks/:id";
    pub const TASK_ASSIGN: &'static str = "/api/tasks/:id/assign";
    pub const TASK_STATUS: &'static str = "/api/tasks/:id/status";
    pub const TASK_PRIORITY: &'static str = "/api/tasks/:id/priority";
    pub const TASK_COMMENTS: &'static str = "/api/tasks/:id/comments";

    // ===== MEETINGS =====
    pub const MEET_CREATE: &'static str = "/api/meet/create";
    pub const MEET_ROOMS: &'static str = "/api/meet/rooms";
    pub const MEET_ROOM_BY_ID: &'static str = "/api/meet/rooms/:id";
    pub const MEET_JOIN: &'static str = "/api/meet/rooms/:id/join";
    pub const MEET_LEAVE: &'static str = "/api/meet/rooms/:id/leave";
    pub const MEET_TOKEN: &'static str = "/api/meet/token";
    pub const MEET_INVITE: &'static str = "/api/meet/invite";
    pub const MEET_TRANSCRIPTION: &'static str = "/api/meet/rooms/:id/transcription";

    // ===== VOICE =====
    pub const VOICE_START: &'static str = "/api/voice/start";
    pub const VOICE_STOP: &'static str = "/api/voice/stop";
    pub const VOICE_STATUS: &'static str = "/api/voice/status";

    // ===== DNS =====
    pub const DNS_REGISTER: &'static str = "/api/dns/register";
    pub const DNS_REMOVE: &'static str = "/api/dns/remove";
    pub const DNS_LIST: &'static str = "/api/dns/list";
    pub const DNS_UPDATE: &'static str = "/api/dns/update";

    // ===== ANALYTICS =====
    pub const ANALYTICS_DASHBOARD: &'static str = "/api/analytics/dashboard";
    pub const ANALYTICS_METRIC: &'static str = "/api/analytics/metric";
    pub const METRICS: &'static str = "/api/metrics";

    // ===== ADMIN =====
    pub const ADMIN_STATS: &'static str = "/api/admin/stats";
    pub const ADMIN_USERS: &'static str = "/api/admin/users";
    pub const ADMIN_SYSTEM: &'static str = "/api/admin/system";
    pub const ADMIN_LOGS: &'static str = "/api/admin/logs";
    pub const ADMIN_BACKUPS: &'static str = "/api/admin/backups";
    pub const ADMIN_SERVICES: &'static str = "/api/admin/services";
    pub const ADMIN_AUDIT: &'static str = "/api/admin/audit";

    // ===== HEALTH & STATUS =====
    pub const HEALTH: &'static str = "/api/health";
    pub const STATUS: &'static str = "/api/status";
    pub const SERVICES_STATUS: &'static str = "/api/services/status";

    // ===== KNOWLEDGE BASE =====
    pub const KB_SEARCH: &'static str = "/api/kb/search";
    pub const KB_UPLOAD: &'static str = "/api/kb/upload";
    pub const KB_DOCUMENTS: &'static str = "/api/kb/documents";
    pub const KB_DOCUMENT_BY_ID: &'static str = "/api/kb/documents/:id";
    pub const KB_INDEX: &'static str = "/api/kb/index";
    pub const KB_EMBEDDINGS: &'static str = "/api/kb/embeddings";

    // ===== LLM =====
    pub const LLM_CHAT: &'static str = "/api/llm/chat";
    pub const LLM_COMPLETIONS: &'static str = "/api/llm/completions";
    pub const LLM_EMBEDDINGS: &'static str = "/api/llm/embeddings";
    pub const LLM_MODELS: &'static str = "/api/llm/models";

    // ===== WEBSOCKET =====
    pub const WS: &'static str = "/ws";
    pub const WS_MEET: &'static str = "/ws/meet";
    pub const WS_CHAT: &'static str = "/ws/chat";
    pub const WS_NOTIFICATIONS: &'static str = "/ws/notifications";
}

/// Internal service URLs
#[derive(Debug)]
pub struct InternalUrls;

impl InternalUrls {
    pub const DIRECTORY_BASE: &'static str = "http://localhost:8080";
    pub const DATABASE: &'static str = "postgres://localhost:5432";
    pub const CACHE: &'static str = "redis://localhost:6379";
    pub const DRIVE: &'static str = "http://localhost:9000";
    pub const EMAIL: &'static str = "http://localhost:8025";
    pub const LLM: &'static str = "http://localhost:8081";
    pub const EMBEDDING: &'static str = "http://localhost:8082";
    pub const QDRANT: &'static str = "http://localhost:6334";
    pub const FORGEJO: &'static str = "http://localhost:3000";
    pub const LIVEKIT: &'static str = "http://localhost:7880";
}

/// Helper functions for URL construction
impl ApiUrls {
    /// Replace path parameters in URL
    pub fn with_params(url: &str, params: &[(&str, &str)]) -> String {
        let mut result = url.to_string();
        for (key, value) in params {
            result = result.replace(&format!(":{}", key), value);
        }
        result
    }

    /// Build URL with query parameters
    pub fn with_query(url: &str, params: &[(&str, &str)]) -> String {
        if params.is_empty() {
            return url.to_string();
        }

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", url, query)
    }
}
