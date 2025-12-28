#[derive(Debug)]
pub struct ApiUrls;

impl ApiUrls {
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

    pub const GROUPS: &'static str = "/api/groups";
    pub const GROUP_BY_ID: &'static str = "/api/groups/:id";
    pub const GROUP_MEMBERS: &'static str = "/api/groups/:id/members";
    pub const GROUP_ADD_MEMBER: &'static str = "/api/groups/:id/members/:user_id";
    pub const GROUP_REMOVE_MEMBER: &'static str = "/api/groups/:id/members/:user_id";
    pub const GROUP_PERMISSIONS: &'static str = "/api/groups/:id/permissions";

    pub const AUTH: &'static str = "/api/auth";
    pub const AUTH_TOKEN: &'static str = "/api/auth/token";
    pub const AUTH_REFRESH: &'static str = "/api/auth/refresh";
    pub const AUTH_VERIFY: &'static str = "/api/auth/verify";
    pub const AUTH_OAUTH: &'static str = "/api/auth/oauth";
    pub const AUTH_OAUTH_CALLBACK: &'static str = "/api/auth/oauth/callback";

    pub const SESSIONS: &'static str = "/api/sessions";
    pub const SESSION_BY_ID: &'static str = "/api/sessions/:id";
    pub const SESSION_HISTORY: &'static str = "/api/sessions/:id/history";
    pub const SESSION_START: &'static str = "/api/sessions/:id/start";
    pub const SESSION_END: &'static str = "/api/sessions/:id/end";

    pub const BOTS: &'static str = "/api/bots";
    pub const BOT_BY_ID: &'static str = "/api/bots/:id";
    pub const BOT_CONFIG: &'static str = "/api/bots/:id/config";
    pub const BOT_DEPLOY: &'static str = "/api/bots/:id/deploy";
    pub const BOT_LOGS: &'static str = "/api/bots/:id/logs";
    pub const BOT_METRICS: &'static str = "/api/bots/:id/metrics";

    pub const DRIVE_LIST: &'static str = "/api/drive/list";
    pub const DRIVE_UPLOAD: &'static str = "/api/drive/upload";
    pub const DRIVE_DOWNLOAD: &'static str = "/api/drive/download/:path";
    pub const DRIVE_DELETE: &'static str = "/api/drive/delete/:path";
    pub const DRIVE_MKDIR: &'static str = "/api/drive/mkdir";
    pub const DRIVE_MOVE: &'static str = "/api/drive/move";
    pub const DRIVE_COPY: &'static str = "/api/drive/copy";
    pub const DRIVE_SHARE: &'static str = "/api/drive/share";
    pub const DRIVE_FILE: &'static str = "/api/drive/file/:path";

    pub const EMAIL_ACCOUNTS: &'static str = "/api/email/accounts";
    pub const EMAIL_ACCOUNT_BY_ID: &'static str = "/api/email/accounts/:id";
    pub const EMAIL_LIST: &'static str = "/api/email/list";
    pub const EMAIL_SEND: &'static str = "/api/email/send";
    pub const EMAIL_DRAFT: &'static str = "/api/email/draft";
    pub const EMAIL_FOLDERS: &'static str = "/api/email/folders/:account_id";
    pub const EMAIL_LATEST: &'static str = "/api/email/latest";
    pub const EMAIL_GET: &'static str = "/api/email/get/:campaign_id";
    pub const EMAIL_CLICK: &'static str = "/api/email/click/:campaign_id/:email";

    pub const CALENDAR_EVENTS: &'static str = "/api/calendar/events";
    pub const CALENDAR_EVENT_BY_ID: &'static str = "/api/calendar/events/:id";
    pub const CALENDAR_REMINDERS: &'static str = "/api/calendar/reminders";
    pub const CALENDAR_SHARE: &'static str = "/api/calendar/share";
    pub const CALENDAR_SYNC: &'static str = "/api/calendar/sync";
    pub const CALENDAR_EXPORT: &'static str = "/api/calendar/export.ics";
    pub const CALENDAR_IMPORT: &'static str = "/api/calendar/import";
    pub const CALENDAR_CALENDARS: &'static str = "/api/calendar/calendars";
    pub const CALENDAR_UPCOMING: &'static str = "/api/calendar/events/upcoming";

    pub const TASKS: &'static str = "/api/tasks";
    pub const TASK_BY_ID: &'static str = "/api/tasks/:id";
    pub const TASK_ASSIGN: &'static str = "/api/tasks/:id/assign";
    pub const TASK_STATUS: &'static str = "/api/tasks/:id/status";
    pub const TASK_PRIORITY: &'static str = "/api/tasks/:id/priority";
    pub const TASK_COMMENTS: &'static str = "/api/tasks/:id/comments";

    pub const MEET_CREATE: &'static str = "/api/meet/create";
    pub const MEET_ROOMS: &'static str = "/api/meet/rooms";
    pub const MEET_ROOM_BY_ID: &'static str = "/api/meet/rooms/:id";
    pub const MEET_JOIN: &'static str = "/api/meet/rooms/:id/join";
    pub const MEET_LEAVE: &'static str = "/api/meet/rooms/:id/leave";
    pub const MEET_TOKEN: &'static str = "/api/meet/token";
    pub const MEET_INVITE: &'static str = "/api/meet/invite";
    pub const MEET_TRANSCRIPTION: &'static str = "/api/meet/rooms/:id/transcription";
    pub const MEET_PARTICIPANTS: &'static str = "/api/meet/participants";
    pub const MEET_RECENT: &'static str = "/api/meet/recent";
    pub const MEET_SCHEDULED: &'static str = "/api/meet/scheduled";

    pub const VOICE_START: &'static str = "/api/voice/start";
    pub const VOICE_STOP: &'static str = "/api/voice/stop";
    pub const VOICE_STATUS: &'static str = "/api/voice/status";

    pub const DNS_REGISTER: &'static str = "/api/dns/register";
    pub const DNS_REMOVE: &'static str = "/api/dns/remove";
    pub const DNS_LIST: &'static str = "/api/dns/list";
    pub const DNS_UPDATE: &'static str = "/api/dns/update";

    pub const ANALYTICS_DASHBOARD: &'static str = "/api/analytics/dashboard";
    pub const ANALYTICS_METRIC: &'static str = "/api/analytics/metric";
    pub const ANALYTICS_MESSAGES_COUNT: &'static str = "/api/analytics/messages/count";
    pub const ANALYTICS_SESSIONS_ACTIVE: &'static str = "/api/analytics/sessions/active";
    pub const ANALYTICS_RESPONSE_AVG: &'static str = "/api/analytics/response/avg";
    pub const ANALYTICS_LLM_TOKENS: &'static str = "/api/analytics/llm/tokens";
    pub const ANALYTICS_STORAGE_USAGE: &'static str = "/api/analytics/storage/usage";
    pub const ANALYTICS_ERRORS_COUNT: &'static str = "/api/analytics/errors/count";
    pub const ANALYTICS_TIMESERIES_MESSAGES: &'static str = "/api/analytics/timeseries/messages";
    pub const ANALYTICS_TIMESERIES_RESPONSE: &'static str =
        "/api/analytics/timeseries/response_time";
    pub const ANALYTICS_CHANNELS_DISTRIBUTION: &'static str =
        "/api/analytics/channels/distribution";
    pub const ANALYTICS_BOTS_PERFORMANCE: &'static str = "/api/analytics/bots/performance";
    pub const ANALYTICS_ACTIVITY_RECENT: &'static str = "/api/analytics/activity/recent";
    pub const ANALYTICS_QUERIES_TOP: &'static str = "/api/analytics/queries/top";
    pub const ANALYTICS_CHAT: &'static str = "/api/analytics/chat";
    pub const ANALYTICS_LLM_STATS: &'static str = "/api/analytics/llm/stats";
    pub const ANALYTICS_BUDGET_STATUS: &'static str = "/api/analytics/budget/status";
    pub const METRICS: &'static str = "/api/metrics";

    pub const ADMIN_STATS: &'static str = "/api/admin/stats";
    pub const ADMIN_USERS: &'static str = "/api/admin/users";
    pub const ADMIN_SYSTEM: &'static str = "/api/admin/system";
    pub const ADMIN_LOGS: &'static str = "/api/admin/logs";
    pub const ADMIN_BACKUPS: &'static str = "/api/admin/backups";
    pub const ADMIN_SERVICES: &'static str = "/api/admin/services";
    pub const ADMIN_AUDIT: &'static str = "/api/admin/audit";

    pub const HEALTH: &'static str = "/api/health";
    pub const STATUS: &'static str = "/api/status";
    pub const SERVICES_STATUS: &'static str = "/api/services/status";

    pub const KB_SEARCH: &'static str = "/api/kb/search";
    pub const KB_UPLOAD: &'static str = "/api/kb/upload";
    pub const KB_DOCUMENTS: &'static str = "/api/kb/documents";
    pub const KB_DOCUMENT_BY_ID: &'static str = "/api/kb/documents/:id";
    pub const KB_INDEX: &'static str = "/api/kb/index";
    pub const KB_EMBEDDINGS: &'static str = "/api/kb/embeddings";

    pub const LLM_CHAT: &'static str = "/api/llm/chat";
    pub const LLM_COMPLETIONS: &'static str = "/api/llm/completions";
    pub const LLM_EMBEDDINGS: &'static str = "/api/llm/embeddings";
    pub const LLM_MODELS: &'static str = "/api/llm/models";
    pub const LLM_GENERATE: &'static str = "/api/llm/generate";
    pub const LLM_IMAGE: &'static str = "/api/llm/image";

    pub const ATTENDANCE_QUEUE: &'static str = "/api/attendance/queue";
    pub const ATTENDANCE_ATTENDANTS: &'static str = "/api/attendance/attendants";
    pub const ATTENDANCE_ASSIGN: &'static str = "/api/attendance/assign";
    pub const ATTENDANCE_TRANSFER: &'static str = "/api/attendance/transfer";
    pub const ATTENDANCE_RESOLVE: &'static str = "/api/attendance/resolve/:session_id";
    pub const ATTENDANCE_INSIGHTS: &'static str = "/api/attendance/insights";
    pub const ATTENDANCE_RESPOND: &'static str = "/api/attendance/respond";
    pub const ATTENDANCE_LLM_TIPS: &'static str = "/api/attendance/llm/tips";
    pub const ATTENDANCE_LLM_POLISH: &'static str = "/api/attendance/llm/polish";
    pub const ATTENDANCE_LLM_SMART_REPLIES: &'static str = "/api/attendance/llm/smart-replies";
    pub const ATTENDANCE_LLM_SUMMARY: &'static str = "/api/attendance/llm/summary/:session_id";
    pub const ATTENDANCE_LLM_SENTIMENT: &'static str = "/api/attendance/llm/sentiment";
    pub const ATTENDANCE_LLM_CONFIG: &'static str = "/api/attendance/llm/config/:bot_id";

    pub const AUTOTASK_CREATE: &'static str = "/api/autotask/create";
    pub const AUTOTASK_CLASSIFY: &'static str = "/api/autotask/classify";
    pub const AUTOTASK_COMPILE: &'static str = "/api/autotask/compile";
    pub const AUTOTASK_EXECUTE: &'static str = "/api/autotask/execute";
    pub const AUTOTASK_SIMULATE: &'static str = "/api/autotask/simulate/:plan_id";
    pub const AUTOTASK_LIST: &'static str = "/api/autotask/list";
    pub const AUTOTASK_STATS: &'static str = "/api/autotask/stats";
    pub const AUTOTASK_PAUSE: &'static str = "/api/autotask/:task_id/pause";
    pub const AUTOTASK_RESUME: &'static str = "/api/autotask/:task_id/resume";
    pub const AUTOTASK_CANCEL: &'static str = "/api/autotask/:task_id/cancel";
    pub const AUTOTASK_TASK_SIMULATE: &'static str = "/api/autotask/:task_id/simulate";
    pub const AUTOTASK_DECISIONS: &'static str = "/api/autotask/:task_id/decisions";
    pub const AUTOTASK_DECIDE: &'static str = "/api/autotask/:task_id/decide";
    pub const AUTOTASK_APPROVALS: &'static str = "/api/autotask/:task_id/approvals";
    pub const AUTOTASK_APPROVE: &'static str = "/api/autotask/:task_id/approve";
    pub const AUTOTASK_TASK_EXECUTE: &'static str = "/api/autotask/:task_id/execute";
    pub const AUTOTASK_LOGS: &'static str = "/api/autotask/:task_id/logs";
    pub const AUTOTASK_RECOMMENDATIONS_APPLY: &'static str =
        "/api/autotask/recommendations/:rec_id/apply";
    pub const AUTOTASK_PENDING: &'static str = "/api/autotask/pending";
    pub const AUTOTASK_PENDING_ITEM: &'static str = "/api/autotask/pending/:item_id";

    pub const DB_TABLE: &'static str = "/api/db/:table";
    pub const DB_TABLE_RECORD: &'static str = "/api/db/:table/:id";
    pub const DB_TABLE_COUNT: &'static str = "/api/db/:table/count";
    pub const DB_TABLE_SEARCH: &'static str = "/api/db/:table/search";

    pub const DESIGNER_FILES: &'static str = "/api/v1/designer/files";
    pub const DESIGNER_LOAD: &'static str = "/api/v1/designer/load";
    pub const DESIGNER_SAVE: &'static str = "/api/v1/designer/save";
    pub const DESIGNER_VALIDATE: &'static str = "/api/v1/designer/validate";
    pub const DESIGNER_EXPORT: &'static str = "/api/v1/designer/export";
    pub const DESIGNER_MODIFY: &'static str = "/api/designer/modify";

    pub const MAIL_SEND: &'static str = "/api/mail/send";
    pub const WHATSAPP_SEND: &'static str = "/api/whatsapp/send";

    pub const FILES_BY_ID: &'static str = "/api/files/:id";

    pub const MESSAGES: &'static str = "/api/messages";

    pub const DESIGNER_DIALOGS: &'static str = "/api/designer/dialogs";
    pub const DESIGNER_DIALOG_BY_ID: &'static str = "/api/designer/dialogs/:id";

    pub const EMAIL_TRACKING_LIST: &'static str = "/api/email/tracking/list";
    pub const EMAIL_TRACKING_STATS: &'static str = "/api/email/tracking/stats";

    pub const INSTAGRAM_WEBHOOK: &'static str = "/api/instagram/webhook";
    pub const INSTAGRAM_SEND: &'static str = "/api/instagram/send";

    pub const MONITORING_DASHBOARD: &'static str = "/api/monitoring/dashboard";
    pub const MONITORING_SERVICES: &'static str = "/api/monitoring/services";
    pub const MONITORING_RESOURCES: &'static str = "/api/monitoring/resources";
    pub const MONITORING_LOGS: &'static str = "/api/monitoring/logs";
    pub const MONITORING_LLM: &'static str = "/api/monitoring/llm";
    pub const MONITORING_HEALTH: &'static str = "/api/monitoring/health";

    pub const MSTEAMS_MESSAGES: &'static str = "/api/msteams/messages";
    pub const MSTEAMS_SEND: &'static str = "/api/msteams/send";

    pub const PAPER_NEW: &'static str = "/api/paper/new";
    pub const PAPER_LIST: &'static str = "/api/paper/list";
    pub const PAPER_SEARCH: &'static str = "/api/paper/search";
    pub const PAPER_SAVE: &'static str = "/api/paper/save";
    pub const PAPER_AUTOSAVE: &'static str = "/api/paper/autosave";
    pub const PAPER_BY_ID: &'static str = "/api/paper/:id";
    pub const PAPER_DELETE: &'static str = "/api/paper/:id/delete";
    pub const PAPER_TEMPLATE_BLANK: &'static str = "/api/paper/template/blank";
    pub const PAPER_TEMPLATE_MEETING: &'static str = "/api/paper/template/meeting";
    pub const PAPER_TEMPLATE_TODO: &'static str = "/api/paper/template/todo";
    pub const PAPER_TEMPLATE_RESEARCH: &'static str = "/api/paper/template/research";
    pub const PAPER_AI_SUMMARIZE: &'static str = "/api/paper/ai/summarize";
    pub const PAPER_AI_EXPAND: &'static str = "/api/paper/ai/expand";
    pub const PAPER_AI_IMPROVE: &'static str = "/api/paper/ai/improve";
    pub const PAPER_AI_SIMPLIFY: &'static str = "/api/paper/ai/simplify";
    pub const PAPER_AI_TRANSLATE: &'static str = "/api/paper/ai/translate";
    pub const PAPER_AI_CUSTOM: &'static str = "/api/paper/ai/custom";
    pub const PAPER_EXPORT_PDF: &'static str = "/api/paper/export/pdf";
    pub const PAPER_EXPORT_DOCX: &'static str = "/api/paper/export/docx";
    pub const PAPER_EXPORT_MD: &'static str = "/api/paper/export/md";
    pub const PAPER_EXPORT_HTML: &'static str = "/api/paper/export/html";
    pub const PAPER_EXPORT_TXT: &'static str = "/api/paper/export/txt";

    pub const RESEARCH_COLLECTIONS: &'static str = "/api/research/collections";
    pub const RESEARCH_COLLECTIONS_NEW: &'static str = "/api/research/collections/new";
    pub const RESEARCH_COLLECTION_BY_ID: &'static str = "/api/research/collections/:id";
    pub const RESEARCH_SEARCH: &'static str = "/api/research/search";
    pub const RESEARCH_RECENT: &'static str = "/api/research/recent";
    pub const RESEARCH_TRENDING: &'static str = "/api/research/trending";
    pub const RESEARCH_PROMPTS: &'static str = "/api/research/prompts";

    pub const SOURCES_PROMPTS: &'static str = "/api/sources/prompts";
    pub const SOURCES_TEMPLATES: &'static str = "/api/sources/templates";
    pub const SOURCES_NEWS: &'static str = "/api/sources/news";
    pub const SOURCES_MCP_SERVERS: &'static str = "/api/sources/mcp-servers";
    pub const SOURCES_LLM_TOOLS: &'static str = "/api/sources/llm-tools";
    pub const SOURCES_MODELS: &'static str = "/api/sources/models";
    pub const SOURCES_SEARCH: &'static str = "/api/sources/search";
    pub const SOURCES_REPOSITORIES: &'static str = "/api/sources/repositories";
    pub const SOURCES_REPOSITORIES_CONNECT: &'static str = "/api/sources/repositories/connect";
    pub const SOURCES_REPOSITORIES_DISCONNECT: &'static str =
        "/api/sources/repositories/disconnect";
    pub const SOURCES_APPS: &'static str = "/api/sources/apps";
    pub const SOURCES_MCP: &'static str = "/api/sources/mcp";
    pub const SOURCES_MCP_BY_NAME: &'static str = "/api/sources/mcp/:name";
    pub const SOURCES_MCP_ENABLE: &'static str = "/api/sources/mcp/:name/enable";
    pub const SOURCES_MCP_DISABLE: &'static str = "/api/sources/mcp/:name/disable";
    pub const SOURCES_MCP_TOOLS: &'static str = "/api/sources/mcp/:name/tools";
    pub const SOURCES_MCP_TEST: &'static str = "/api/sources/mcp/:name/test";
    pub const SOURCES_MCP_SCAN: &'static str = "/api/sources/mcp/scan";
    pub const SOURCES_MCP_EXAMPLES: &'static str = "/api/sources/mcp/examples";
    pub const SOURCES_MENTIONS: &'static str = "/api/sources/mentions";
    pub const SOURCES_TOOLS: &'static str = "/api/sources/tools";

    pub const TASKS_STATS: &'static str = "/api/tasks/stats";
    pub const TASKS_STATS_JSON: &'static str = "/api/tasks/stats/json";
    pub const TASKS_COMPLETED: &'static str = "/api/tasks/completed";

    pub const WS: &'static str = "/ws";
    pub const WS_MEET: &'static str = "/ws/meet";
    pub const WS_CHAT: &'static str = "/ws/chat";
    pub const WS_NOTIFICATIONS: &'static str = "/ws/notifications";
    pub const WS_ATTENDANT: &'static str = "/ws/attendant";
}

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
    pub const BOTMODELS_VISION_QRCODE: &'static str = "/api/v1/vision/qrcode";
    pub const BOTMODELS_SPEECH_TO_TEXT: &'static str = "/api/v1/speech/to-text";
    pub const BOTMODELS_VISION_DESCRIBE_VIDEO: &'static str = "/api/v1/vision/describe-video";
}

impl ApiUrls {
    pub fn with_params(url: &str, params: &[(&str, &str)]) -> String {
        let mut result = url.to_string();
        for (key, value) in params {
            result = result.replace(&format!(":{key}"), value);
        }
        result
    }

    pub fn with_query(url: &str, params: &[(&str, &str)]) -> String {
        if params.is_empty() {
            return url.to_string();
        }

        let query = params
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{url}?{query}")
    }
}
