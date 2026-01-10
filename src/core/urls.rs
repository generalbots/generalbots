#[derive(Debug)]
pub struct ApiUrls;

impl ApiUrls {
    // User management - JSON APIs
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

    // Groups - JSON APIs
    pub const GROUPS: &'static str = "/api/groups";
    pub const GROUP_BY_ID: &'static str = "/api/groups/:id";
    pub const GROUP_MEMBERS: &'static str = "/api/groups/:id/members";
    pub const GROUP_ADD_MEMBER: &'static str = "/api/groups/:id/members/:user_id";
    pub const GROUP_REMOVE_MEMBER: &'static str = "/api/groups/:id/members/:user_id";
    pub const GROUP_PERMISSIONS: &'static str = "/api/groups/:id/permissions";

    // Product - JSON APIs
    pub const PRODUCT: &'static str = "/api/product";

    // Auth - JSON APIs
    pub const AUTH: &'static str = "/api/auth";
    pub const AUTH_TOKEN: &'static str = "/api/auth/token";
    pub const AUTH_REFRESH: &'static str = "/api/auth/refresh";
    pub const AUTH_VERIFY: &'static str = "/api/auth/verify";
    pub const AUTH_OAUTH: &'static str = "/api/auth/oauth";
    pub const AUTH_OAUTH_CALLBACK: &'static str = "/api/auth/oauth/callback";

    // Sessions - JSON APIs
    pub const SESSIONS: &'static str = "/api/sessions";
    pub const SESSION_BY_ID: &'static str = "/api/sessions/:id";
    pub const SESSION_HISTORY: &'static str = "/api/sessions/:id/history";
    pub const SESSION_START: &'static str = "/api/sessions/:id/start";
    pub const SESSION_END: &'static str = "/api/sessions/:id/end";

    // Bots - JSON APIs
    pub const BOTS: &'static str = "/api/bots";
    pub const BOT_BY_ID: &'static str = "/api/bots/:id";
    pub const BOT_CONFIG: &'static str = "/api/bots/:id/config";
    pub const BOT_DEPLOY: &'static str = "/api/bots/:id/deploy";
    pub const BOT_LOGS: &'static str = "/api/bots/:id/logs";
    pub const BOT_METRICS: &'static str = "/api/bots/:id/metrics";

    // Drive - JSON APIs
    pub const DRIVE_LIST: &'static str = "/api/drive/list";
    pub const DRIVE_UPLOAD: &'static str = "/api/drive/upload";
    pub const DRIVE_DOWNLOAD: &'static str = "/api/drive/download/:path";
    pub const DRIVE_DELETE: &'static str = "/api/drive/delete/:path";
    pub const DRIVE_MKDIR: &'static str = "/api/drive/mkdir";
    pub const DRIVE_MOVE: &'static str = "/api/drive/move";
    pub const DRIVE_COPY: &'static str = "/api/drive/copy";
    pub const DRIVE_SHARE: &'static str = "/api/drive/share";
    pub const DRIVE_FILE: &'static str = "/api/drive/file/:path";

    // Email - JSON APIs
    pub const EMAIL_ACCOUNTS: &'static str = "/api/email/accounts";
    pub const EMAIL_ACCOUNT_BY_ID: &'static str = "/api/email/accounts/:id";
    pub const EMAIL_LIST: &'static str = "/api/email/list";
    pub const EMAIL_SEND: &'static str = "/api/email/send";
    pub const EMAIL_DRAFT: &'static str = "/api/email/draft";
    pub const EMAIL_FOLDERS: &'static str = "/api/email/folders/:account_id";
    pub const EMAIL_LATEST: &'static str = "/api/email/latest";
    pub const EMAIL_GET: &'static str = "/api/email/get/:campaign_id";
    pub const EMAIL_CLICK: &'static str = "/api/email/click/:campaign_id/:email";

    // Email - HTMX/HTML APIs
    pub const EMAIL_ACCOUNTS_HTMX: &'static str = "/api/ui/email/accounts";
    pub const EMAIL_LIST_HTMX: &'static str = "/api/ui/email/list";
    pub const EMAIL_FOLDERS_HTMX: &'static str = "/api/ui/email/folders/:account_id";
    pub const EMAIL_COMPOSE_HTMX: &'static str = "/api/ui/email/compose";
    pub const EMAIL_CONTENT_HTMX: &'static str = "/api/ui/email/content/:id";
    pub const EMAIL_LABELS_HTMX: &'static str = "/api/ui/email/labels";
    pub const EMAIL_TEMPLATES_HTMX: &'static str = "/api/ui/email/templates";
    pub const EMAIL_SIGNATURES_HTMX: &'static str = "/api/ui/email/signatures";
    pub const EMAIL_RULES_HTMX: &'static str = "/api/ui/email/rules";
    pub const EMAIL_SEARCH_HTMX: &'static str = "/api/ui/email/search";
    pub const EMAIL_AUTO_RESPONDER_HTMX: &'static str = "/api/ui/email/auto-responder";

    // Calendar - JSON APIs
    pub const CALENDAR_EVENTS: &'static str = "/api/calendar/events";
    pub const CALENDAR_EVENT_BY_ID: &'static str = "/api/calendar/events/:id";
    pub const CALENDAR_REMINDERS: &'static str = "/api/calendar/reminders";
    pub const CALENDAR_SHARE: &'static str = "/api/calendar/share";
    pub const CALENDAR_SYNC: &'static str = "/api/calendar/sync";
    pub const CALENDAR_EXPORT: &'static str = "/api/calendar/export.ics";
    pub const CALENDAR_IMPORT: &'static str = "/api/calendar/import";
    pub const CALENDAR_CALENDARS_JSON: &'static str = "/api/calendar/calendars";
    pub const CALENDAR_UPCOMING_JSON: &'static str = "/api/calendar/events/upcoming";

    // Calendar - HTMX/HTML APIs
    pub const CALENDAR_CALENDARS: &'static str = "/api/ui/calendar/calendars";
    pub const CALENDAR_UPCOMING: &'static str = "/api/ui/calendar/events/upcoming";
    pub const CALENDAR_NEW_EVENT_FORM: &'static str = "/api/ui/calendar/events/new";
    pub const CALENDAR_NEW_CALENDAR_FORM: &'static str = "/api/ui/calendar/calendars/new";

    // Tasks - JSON APIs
    pub const TASKS: &'static str = "/api/tasks";
    pub const TASK_BY_ID: &'static str = "/api/tasks/:id";
    pub const TASK_ASSIGN: &'static str = "/api/tasks/:id/assign";
    pub const TASK_STATUS: &'static str = "/api/tasks/:id/status";
    pub const TASK_PRIORITY: &'static str = "/api/tasks/:id/priority";
    pub const TASK_COMMENTS: &'static str = "/api/tasks/:id/comments";
    pub const TASKS_STATS_JSON: &'static str = "/api/tasks/stats/json";

    // Tasks - HTMX/HTML APIs
    pub const TASKS_LIST_HTMX: &'static str = "/api/ui/tasks";
    pub const TASKS_GET_HTMX: &'static str = "/api/ui/tasks/:id";
    pub const TASKS_STATS: &'static str = "/api/ui/tasks/stats";
    pub const TASKS_COMPLETED: &'static str = "/api/ui/tasks/completed";
    pub const TASKS_TIME_SAVED: &'static str = "/api/ui/tasks/time-saved";

    // Meet - JSON APIs
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

    // Voice - JSON APIs
    pub const VOICE_START: &'static str = "/api/voice/start";
    pub const VOICE_STOP: &'static str = "/api/voice/stop";
    pub const VOICE_STATUS: &'static str = "/api/voice/status";

    // DNS - JSON APIs
    pub const DNS_REGISTER: &'static str = "/api/dns/register";
    pub const DNS_REMOVE: &'static str = "/api/dns/remove";
    pub const DNS_LIST: &'static str = "/api/dns/list";
    pub const DNS_UPDATE: &'static str = "/api/dns/update";

    // Analytics - JSON APIs
    pub const ANALYTICS_DASHBOARD: &'static str = "/api/analytics/dashboard";
    pub const ANALYTICS_METRIC: &'static str = "/api/analytics/metric";
    pub const METRICS: &'static str = "/api/metrics";

    // Analytics - HTMX/HTML APIs
    pub const ANALYTICS_MESSAGES_COUNT: &'static str = "/api/ui/analytics/messages/count";
    pub const ANALYTICS_SESSIONS_ACTIVE: &'static str = "/api/ui/analytics/sessions/active";
    pub const ANALYTICS_RESPONSE_AVG: &'static str = "/api/ui/analytics/response/avg";
    pub const ANALYTICS_LLM_TOKENS: &'static str = "/api/ui/analytics/llm/tokens";
    pub const ANALYTICS_STORAGE_USAGE: &'static str = "/api/ui/analytics/storage/usage";
    pub const ANALYTICS_ERRORS_COUNT: &'static str = "/api/ui/analytics/errors/count";
    pub const ANALYTICS_TIMESERIES_MESSAGES: &'static str = "/api/ui/analytics/timeseries/messages";
    pub const ANALYTICS_TIMESERIES_RESPONSE: &'static str =
        "/api/ui/analytics/timeseries/response_time";
    pub const ANALYTICS_CHANNELS_DISTRIBUTION: &'static str =
        "/api/ui/analytics/channels/distribution";
    pub const ANALYTICS_BOTS_PERFORMANCE: &'static str = "/api/ui/analytics/bots/performance";
    pub const ANALYTICS_ACTIVITY_RECENT: &'static str = "/api/ui/analytics/activity/recent";
    pub const ANALYTICS_QUERIES_TOP: &'static str = "/api/ui/analytics/queries/top";
    pub const ANALYTICS_CHAT: &'static str = "/api/ui/analytics/chat";
    pub const ANALYTICS_LLM_STATS: &'static str = "/api/ui/analytics/llm/stats";
    pub const ANALYTICS_BUDGET_STATUS: &'static str = "/api/ui/analytics/budget/status";

    // Admin - JSON APIs
    pub const ADMIN_DASHBOARD: &'static str = "/api/admin/dashboard";
    pub const ADMIN_STATS: &'static str = "/api/admin/stats";
    pub const ADMIN_STATS_USERS: &'static str = "/api/admin/stats/users";
    pub const ADMIN_STATS_GROUPS: &'static str = "/api/admin/stats/groups";
    pub const ADMIN_STATS_BOTS: &'static str = "/api/admin/stats/bots";
    pub const ADMIN_STATS_STORAGE: &'static str = "/api/admin/stats/storage";
    pub const ADMIN_USERS: &'static str = "/api/admin/users";
    pub const ADMIN_GROUPS: &'static str = "/api/admin/groups";
    pub const ADMIN_BOTS: &'static str = "/api/admin/bots";
    pub const ADMIN_DNS: &'static str = "/api/admin/dns";
    pub const ADMIN_BILLING: &'static str = "/api/admin/billing";
    pub const ADMIN_SYSTEM: &'static str = "/api/admin/system";
    pub const ADMIN_LOGS: &'static str = "/api/admin/logs";
    pub const ADMIN_BACKUPS: &'static str = "/api/admin/backups";
    pub const ADMIN_SERVICES: &'static str = "/api/admin/services";
    pub const ADMIN_AUDIT: &'static str = "/api/admin/audit";

    // Health/Status - JSON APIs
    pub const HEALTH: &'static str = "/api/health";
    pub const STATUS: &'static str = "/api/status";
    pub const SERVICES_STATUS: &'static str = "/api/services/status";

    // i18n - JSON APIs
    pub const I18N_TRANSLATIONS: &'static str = "/api/i18n/:locale";
    pub const I18N_LOCALES: &'static str = "/api/i18n/locales";

    // Knowledge Base - JSON APIs
    pub const KB_SEARCH: &'static str = "/api/kb/search";
    pub const KB_UPLOAD: &'static str = "/api/kb/upload";
    pub const KB_DOCUMENTS: &'static str = "/api/kb/documents";
    pub const KB_DOCUMENT_BY_ID: &'static str = "/api/kb/documents/:id";
    pub const KB_INDEX: &'static str = "/api/kb/index";
    pub const KB_EMBEDDINGS: &'static str = "/api/kb/embeddings";

    // LLM - JSON APIs
    pub const LLM_CHAT: &'static str = "/api/llm/chat";
    pub const LLM_COMPLETIONS: &'static str = "/api/llm/completions";
    pub const LLM_EMBEDDINGS: &'static str = "/api/llm/embeddings";
    pub const LLM_MODELS: &'static str = "/api/llm/models";
    pub const LLM_GENERATE: &'static str = "/api/llm/generate";
    pub const LLM_IMAGE: &'static str = "/api/llm/image";

    // Attendance - JSON APIs
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

    // AutoTask - JSON APIs
    pub const AUTOTASK_CREATE: &'static str = "/api/autotask/create";
    pub const AUTOTASK_CLASSIFY: &'static str = "/api/autotask/classify";
    pub const AUTOTASK_COMPILE: &'static str = "/api/autotask/compile";
    pub const AUTOTASK_EXECUTE: &'static str = "/api/autotask/execute";
    pub const AUTOTASK_SIMULATE: &'static str = "/api/autotask/simulate/:plan_id";
    pub const AUTOTASK_GET: &'static str = "/api/autotask/tasks/:task_id";
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

    // AutoTask - HTMX/HTML APIs
    pub const AUTOTASK_LIST: &'static str = "/api/ui/autotask/list";

    // DB - JSON APIs
    pub const DB_TABLE: &'static str = "/api/db/:table";
    pub const DB_TABLE_RECORD: &'static str = "/api/db/:table/:id";
    pub const DB_TABLE_COUNT: &'static str = "/api/db/:table/count";
    pub const DB_TABLE_SEARCH: &'static str = "/api/db/:table/search";

    // Designer - HTMX/HTML APIs
    pub const DESIGNER_FILES: &'static str = "/api/ui/designer/files";
    pub const DESIGNER_LOAD: &'static str = "/api/ui/designer/load";
    pub const DESIGNER_SAVE: &'static str = "/api/ui/designer/save";
    pub const DESIGNER_VALIDATE: &'static str = "/api/ui/designer/validate";
    pub const DESIGNER_EXPORT: &'static str = "/api/ui/designer/export";
    pub const DESIGNER_MODIFY: &'static str = "/api/ui/designer/modify";
    pub const DESIGNER_DIALOGS: &'static str = "/api/ui/designer/dialogs";
    pub const DESIGNER_DIALOG_BY_ID: &'static str = "/api/ui/designer/dialogs/:id";

    // Mail/WhatsApp - JSON APIs
    pub const MAIL_SEND: &'static str = "/api/mail/send";
    pub const WHATSAPP_SEND: &'static str = "/api/whatsapp/send";

    // Files - JSON APIs
    pub const FILES_BY_ID: &'static str = "/api/files/:id";

    // Messages - JSON APIs
    pub const MESSAGES: &'static str = "/api/messages";

    // Email Tracking - JSON APIs
    pub const EMAIL_TRACKING_LIST: &'static str = "/api/email/tracking/list";
    pub const EMAIL_TRACKING_STATS: &'static str = "/api/email/tracking/stats";

    // Instagram - JSON APIs
    pub const INSTAGRAM_WEBHOOK: &'static str = "/api/instagram/webhook";
    pub const INSTAGRAM_SEND: &'static str = "/api/instagram/send";

    // Monitoring - HTMX/HTML APIs
    pub const MONITORING_DASHBOARD: &'static str = "/api/ui/monitoring/dashboard";
    pub const MONITORING_SERVICES: &'static str = "/api/ui/monitoring/services";
    pub const MONITORING_RESOURCES: &'static str = "/api/ui/monitoring/resources";
    pub const MONITORING_LOGS: &'static str = "/api/ui/monitoring/logs";
    pub const MONITORING_LLM: &'static str = "/api/ui/monitoring/llm";
    pub const MONITORING_HEALTH: &'static str = "/api/ui/monitoring/health";

    // MS Teams - JSON APIs
    pub const MSTEAMS_MESSAGES: &'static str = "/api/msteams/messages";
    pub const MSTEAMS_SEND: &'static str = "/api/msteams/send";

    // Docs (Word Processor) - HTMX/HTML APIs
    pub const DOCS_NEW: &'static str = "/api/ui/docs/new";
    pub const DOCS_LIST: &'static str = "/api/ui/docs/list";
    pub const DOCS_SEARCH: &'static str = "/api/ui/docs/search";
    pub const DOCS_SAVE: &'static str = "/api/ui/docs/save";
    pub const DOCS_AUTOSAVE: &'static str = "/api/ui/docs/autosave";
    pub const DOCS_BY_ID: &'static str = "/api/ui/docs/:id";
    pub const DOCS_DELETE: &'static str = "/api/ui/docs/:id/delete";
    pub const DOCS_TEMPLATE_BLANK: &'static str = "/api/ui/docs/template/blank";
    pub const DOCS_TEMPLATE_MEETING: &'static str = "/api/ui/docs/template/meeting";
    pub const DOCS_TEMPLATE_REPORT: &'static str = "/api/ui/docs/template/report";
    pub const DOCS_TEMPLATE_LETTER: &'static str = "/api/ui/docs/template/letter";
    pub const DOCS_AI_SUMMARIZE: &'static str = "/api/ui/docs/ai/summarize";
    pub const DOCS_AI_EXPAND: &'static str = "/api/ui/docs/ai/expand";
    pub const DOCS_AI_IMPROVE: &'static str = "/api/ui/docs/ai/improve";
    pub const DOCS_AI_SIMPLIFY: &'static str = "/api/ui/docs/ai/simplify";
    pub const DOCS_AI_TRANSLATE: &'static str = "/api/ui/docs/ai/translate";
    pub const DOCS_AI_CUSTOM: &'static str = "/api/ui/docs/ai/custom";
    pub const DOCS_EXPORT_PDF: &'static str = "/api/ui/docs/export/pdf";
    pub const DOCS_EXPORT_DOCX: &'static str = "/api/ui/docs/export/docx";
    pub const DOCS_EXPORT_MD: &'static str = "/api/ui/docs/export/md";
    pub const DOCS_EXPORT_HTML: &'static str = "/api/ui/docs/export/html";
    pub const DOCS_EXPORT_TXT: &'static str = "/api/ui/docs/export/txt";
    pub const DOCS_WS: &'static str = "/ws/docs/:doc_id";

    // Paper (Notes App) - HTMX/HTML APIs
    pub const PAPER_NEW: &'static str = "/api/ui/paper/new";
    pub const PAPER_LIST: &'static str = "/api/ui/paper/list";
    pub const PAPER_SEARCH: &'static str = "/api/ui/paper/search";
    pub const PAPER_SAVE: &'static str = "/api/ui/paper/save";
    pub const PAPER_AUTOSAVE: &'static str = "/api/ui/paper/autosave";
    pub const PAPER_BY_ID: &'static str = "/api/ui/paper/:id";
    pub const PAPER_DELETE: &'static str = "/api/ui/paper/:id/delete";
    pub const PAPER_TEMPLATE_BLANK: &'static str = "/api/ui/paper/template/blank";
    pub const PAPER_TEMPLATE_MEETING: &'static str = "/api/ui/paper/template/meeting";
    pub const PAPER_TEMPLATE_TODO: &'static str = "/api/ui/paper/template/todo";
    pub const PAPER_TEMPLATE_RESEARCH: &'static str = "/api/ui/paper/template/research";
    pub const PAPER_TEMPLATE_REPORT: &'static str = "/api/ui/paper/template/report";
    pub const PAPER_TEMPLATE_LETTER: &'static str = "/api/ui/paper/template/letter";
    pub const PAPER_AI_SUMMARIZE: &'static str = "/api/ui/paper/ai/summarize";
    pub const PAPER_AI_EXPAND: &'static str = "/api/ui/paper/ai/expand";
    pub const PAPER_AI_IMPROVE: &'static str = "/api/ui/paper/ai/improve";
    pub const PAPER_AI_SIMPLIFY: &'static str = "/api/ui/paper/ai/simplify";
    pub const PAPER_AI_TRANSLATE: &'static str = "/api/ui/paper/ai/translate";
    pub const PAPER_AI_CUSTOM: &'static str = "/api/ui/paper/ai/custom";
    pub const PAPER_EXPORT_PDF: &'static str = "/api/ui/paper/export/pdf";
    pub const PAPER_EXPORT_DOCX: &'static str = "/api/ui/paper/export/docx";
    pub const PAPER_EXPORT_MD: &'static str = "/api/ui/paper/export/md";
    pub const PAPER_EXPORT_HTML: &'static str = "/api/ui/paper/export/html";
    pub const PAPER_EXPORT_TXT: &'static str = "/api/ui/paper/export/txt";

    // Research - HTMX/HTML APIs
    pub const RESEARCH_COLLECTIONS: &'static str = "/api/ui/research/collections";
    pub const RESEARCH_COLLECTIONS_NEW: &'static str = "/api/ui/research/collections/new";
    pub const RESEARCH_COLLECTION_BY_ID: &'static str = "/api/ui/research/collections/:id";
    pub const RESEARCH_SEARCH: &'static str = "/api/ui/research/search";
    pub const RESEARCH_RECENT: &'static str = "/api/ui/research/recent";
    pub const RESEARCH_TRENDING: &'static str = "/api/ui/research/trending";
    pub const RESEARCH_PROMPTS: &'static str = "/api/ui/research/prompts";
    pub const RESEARCH_WEB_SEARCH: &'static str = "/api/ui/research/web/search";
    pub const RESEARCH_WEB_SUMMARIZE: &'static str = "/api/ui/research/web/summarize";
    pub const RESEARCH_WEB_DEEP: &'static str = "/api/ui/research/web/deep";
    pub const RESEARCH_WEB_HISTORY: &'static str = "/api/ui/research/web/history";
    pub const RESEARCH_WEB_INSTANT: &'static str = "/api/ui/research/web/instant";
    pub const RESEARCH_EXPORT_CITATIONS: &'static str = "/api/ui/research/export/citations";

    // Sources - HTMX/HTML APIs
    pub const SOURCES_PROMPTS: &'static str = "/api/ui/sources/prompts";
    pub const SOURCES_TEMPLATES: &'static str = "/api/ui/sources/templates";
    pub const SOURCES_NEWS: &'static str = "/api/ui/sources/news";
    pub const SOURCES_MCP_SERVERS: &'static str = "/api/ui/sources/mcp-servers";
    pub const SOURCES_LLM_TOOLS: &'static str = "/api/ui/sources/llm-tools";
    pub const SOURCES_MODELS: &'static str = "/api/ui/sources/models";
    pub const SOURCES_SEARCH: &'static str = "/api/ui/sources/search";
    pub const SOURCES_REPOSITORIES: &'static str = "/api/ui/sources/repositories";
    pub const SOURCES_REPOSITORIES_CONNECT: &'static str = "/api/ui/sources/repositories/connect";
    pub const SOURCES_REPOSITORIES_DISCONNECT: &'static str =
        "/api/ui/sources/repositories/disconnect";
    pub const SOURCES_APPS: &'static str = "/api/ui/sources/apps";
    pub const SOURCES_MCP: &'static str = "/api/ui/sources/mcp";
    pub const SOURCES_MCP_BY_NAME: &'static str = "/api/ui/sources/mcp/:name";
    pub const SOURCES_MCP_ENABLE: &'static str = "/api/ui/sources/mcp/:name/enable";
    pub const SOURCES_MCP_DISABLE: &'static str = "/api/ui/sources/mcp/:name/disable";
    pub const SOURCES_MCP_TOOLS: &'static str = "/api/ui/sources/mcp/:name/tools";
    pub const SOURCES_MCP_TEST: &'static str = "/api/ui/sources/mcp/:name/test";
    pub const SOURCES_MCP_SCAN: &'static str = "/api/ui/sources/mcp/scan";
    pub const SOURCES_MCP_EXAMPLES: &'static str = "/api/ui/sources/mcp/examples";
    pub const SOURCES_MENTIONS: &'static str = "/api/ui/sources/mentions";
    pub const SOURCES_TOOLS: &'static str = "/api/ui/sources/tools";

    // Sources Knowledge Base - HTMX/HTML APIs
    pub const SOURCES_KB_UPLOAD: &'static str = "/api/ui/sources/kb/upload";
    pub const SOURCES_KB_LIST: &'static str = "/api/ui/sources/kb/list";
    pub const SOURCES_KB_QUERY: &'static str = "/api/ui/sources/kb/query";
    pub const SOURCES_KB_BY_ID: &'static str = "/api/ui/sources/kb/:id";
    pub const SOURCES_KB_REINDEX: &'static str = "/api/ui/sources/kb/reindex";
    pub const SOURCES_KB_STATS: &'static str = "/api/ui/sources/kb/stats";

    // Workspaces - JSON APIs
    pub const WORKSPACES: &'static str = "/api/workspaces";
    pub const WORKSPACE_BY_ID: &'static str = "/api/workspaces/:workspace_id";
    pub const WORKSPACE_PAGES: &'static str = "/api/workspaces/:workspace_id/pages";
    pub const WORKSPACE_MEMBERS: &'static str = "/api/workspaces/:workspace_id/members";
    pub const WORKSPACE_MEMBER: &'static str = "/api/workspaces/:workspace_id/members/:user_id";
    pub const WORKSPACE_SEARCH: &'static str = "/api/workspaces/:workspace_id/search";
    pub const WORKSPACE_COMMANDS: &'static str = "/api/workspaces/commands";
    pub const PAGE_BY_ID: &'static str = "/api/pages/:page_id";

    // Project - JSON APIs
    pub const PROJECTS: &'static str = "/projects";
    pub const PROJECT_BY_ID: &'static str = "/projects/:project_id";
    pub const PROJECT_TASKS: &'static str = "/projects/:project_id/tasks";
    pub const PROJECT_GANTT: &'static str = "/projects/:project_id/gantt";
    pub const PROJECT_TIMELINE: &'static str = "/projects/:project_id/timeline";
    pub const PROJECT_CRITICAL_PATH: &'static str = "/projects/:project_id/critical-path";
    pub const PROJECT_TASK_PROGRESS: &'static str = "/tasks/:task_id/progress";
    pub const PROJECT_TASK_DEPENDENCIES: &'static str = "/tasks/:task_id/dependencies";
    pub const PROJECT_TASK: &'static str = "/tasks/:task_id";

    // Goals (OKR) - JSON APIs
    pub const GOALS_OBJECTIVES: &'static str = "/api/goals/objectives";
    pub const GOALS_OBJECTIVE_BY_ID: &'static str = "/api/goals/objectives/:id";
    pub const GOALS_KEY_RESULTS: &'static str = "/api/goals/objectives/:id/key-results";
    pub const GOALS_KEY_RESULT_BY_ID: &'static str = "/api/goals/key-results/:id";
    pub const GOALS_CHECK_IN: &'static str = "/api/goals/key-results/:id/check-in";
    pub const GOALS_HISTORY: &'static str = "/api/goals/key-results/:id/history";
    pub const GOALS_DASHBOARD: &'static str = "/api/goals/dashboard";
    pub const GOALS_ALIGNMENT: &'static str = "/api/goals/alignment";
    pub const GOALS_AI_SUGGEST: &'static str = "/api/goals/ai/suggest";

    // Security Admin - JSON APIs
    pub const SECURITY_OVERVIEW: &'static str = "/api/security/overview";
    pub const SECURITY_SCAN: &'static str = "/api/security/scan";
    pub const SECURITY_TLS: &'static str = "/api/security/tls";
    pub const SECURITY_RATE_LIMIT: &'static str = "/api/security/rate-limit";
    pub const SECURITY_CORS: &'static str = "/api/security/cors";
    pub const SECURITY_AUDIT: &'static str = "/api/security/audit";
    pub const SECURITY_API_KEYS: &'static str = "/api/security/api-keys";
    pub const SECURITY_API_KEY_BY_ID: &'static str = "/api/security/api-keys/:key_id";
    pub const SECURITY_MFA: &'static str = "/api/security/mfa";
    pub const SECURITY_SESSIONS: &'static str = "/api/security/sessions";
    pub const SECURITY_SESSION_BY_ID: &'static str = "/api/security/sessions/:session_id";
    pub const SECURITY_USER_SESSIONS: &'static str = "/api/security/users/:user_id/sessions";
    pub const SECURITY_PASSWORD_POLICY: &'static str = "/api/security/password-policy";

    // Player - JSON APIs
    pub const PLAYER_FILE: &'static str = "/api/player/:bot_id/file/*path";
    pub const PLAYER_STREAM: &'static str = "/api/player/:bot_id/stream/*path";
    pub const PLAYER_THUMBNAIL: &'static str = "/api/player/:bot_id/thumbnail/*path";

    // Canvas - JSON APIs
    pub const CANVAS_LIST: &'static str = "/api/canvas";
    pub const CANVAS_BY_ID: &'static str = "/api/canvas/:id";
    pub const CANVAS_ELEMENTS: &'static str = "/api/canvas/:id/elements";
    pub const CANVAS_ELEMENT_BY_ID: &'static str = "/api/canvas/:id/elements/:element_id";
    pub const CANVAS_EXPORT: &'static str = "/api/canvas/:id/export";
    pub const CANVAS_COLLABORATE: &'static str = "/api/canvas/:id/collaborate";

    // WebSocket endpoints
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
    pub const DRIVE: &'static str = "https://localhost:9000";
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
