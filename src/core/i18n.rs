use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{header::ACCEPT_LANGUAGE, request::Parts},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use botlib::i18n::{self, Locale as BotlibLocale, MessageArgs as BotlibMessageArgs};
use std::collections::HashMap;
use std::sync::Arc;

use crate::shared::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    language: String,
    region: Option<String>,
}

impl Locale {
    pub fn new(locale_str: &str) -> Option<Self> {
        if locale_str.is_empty() {
            return None;
        }

        let parts: Vec<&str> = locale_str.split(&['-', '_'][..]).collect();

        let language = parts.first()?.to_lowercase();
        if language.len() < 2 || language.len() > 3 {
            return None;
        }

        let region = parts.get(1).map(|r| r.to_uppercase());

        Some(Self { language, region })
    }

    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    #[must_use]
    pub fn to_bcp47(&self) -> String {
        match &self.region {
            Some(r) => format!("{}-{r}", self.language),
            None => self.language.clone(),
        }
    }

    fn to_botlib_locale(&self) -> BotlibLocale {
        BotlibLocale::new(&self.to_bcp47()).unwrap_or_default()
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            region: None,
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_bcp47())
    }
}

const AVAILABLE_LOCALES: &[&str] = &["en", "pt-BR", "es", "zh-CN"];

pub struct RequestLocale(pub Locale);

impl RequestLocale {
    #[must_use]
    pub fn locale(&self) -> &Locale {
        &self.0
    }

    #[must_use]
    pub fn language(&self) -> &str {
        self.0.language()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestLocale
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let locale = parts
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|h| h.to_str().ok())
            .map(parse_accept_language)
            .and_then(|langs| negotiate_locale(&langs))
            .unwrap_or_default();

        Ok(Self(locale))
    }
}

fn parse_accept_language(header: &str) -> Vec<(String, f32)> {
    let mut langs: Vec<(String, f32)> = header
        .split(',')
        .filter_map(|part| {
            let mut iter = part.trim().split(';');
            let lang = iter.next()?.trim().to_string();

            if lang.is_empty() || lang == "*" {
                return None;
            }

            let quality = iter
                .next()
                .and_then(|q| q.trim().strip_prefix("q="))
                .and_then(|q| q.parse().ok())
                .unwrap_or(1.0);

            Some((lang, quality))
        })
        .collect();

    langs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    langs
}

fn negotiate_locale(requested: &[(String, f32)]) -> Option<Locale> {
    for (lang, _) in requested {
        let requested_locale = Locale::new(lang)?;

        for available in AVAILABLE_LOCALES {
            let avail_locale = Locale::new(available)?;

            if requested_locale.language == avail_locale.language
                && requested_locale.region == avail_locale.region
            {
                return Some(avail_locale);
            }
        }

        for available in AVAILABLE_LOCALES {
            let avail_locale = Locale::new(available)?;
            if requested_locale.language == avail_locale.language {
                return Some(avail_locale);
            }
        }
    }

    Some(Locale::default())
}

pub type MessageArgs = HashMap<String, String>;

pub fn init_i18n(locales_path: &str) -> Result<(), String> {
    i18n::init(locales_path).map_err(|e| format!("Failed to initialize i18n: {e}"))
}

pub fn is_i18n_initialized() -> bool {
    i18n::is_initialized()
}

pub fn t(locale: &Locale, key: &str) -> String {
    t_with_args(locale, key, None)
}

pub fn t_with_args(locale: &Locale, key: &str, args: Option<&MessageArgs>) -> String {
    let botlib_locale = locale.to_botlib_locale();
    let botlib_args: Option<BotlibMessageArgs> = args.map(|a| {
        a.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    });
    i18n::get_with_args(&botlib_locale, key, botlib_args.as_ref())
}

pub fn available_locales() -> Vec<String> {
    if is_i18n_initialized() {
        i18n::available_locales()
    } else {
        AVAILABLE_LOCALES.iter().map(|s| (*s).to_string()).collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalizedError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl LocalizedError {
    pub fn new(locale: &Locale, code: &str) -> Self {
        Self {
            code: code.to_string(),
            message: t(locale, code),
            details: None,
        }
    }

    pub fn with_args(locale: &Locale, code: &str, args: &MessageArgs) -> Self {
        Self {
            code: code.to_string(),
            message: t_with_args(locale, code, Some(args)),
            details: None,
        }
    }

    pub fn not_found(locale: &Locale, entity: &str) -> Self {
        let mut args = MessageArgs::new();
        args.insert("entity".to_string(), entity.to_string());
        Self::with_args(locale, "error-http-404", &args)
    }

    pub fn validation(locale: &Locale, field: &str, error_key: &str) -> Self {
        let mut args = MessageArgs::new();
        args.insert("field".to_string(), field.to_string());
        Self::with_args(locale, error_key, &args)
    }

    pub fn internal(locale: &Locale) -> Self {
        Self::new(locale, "error-http-500")
    }

    pub fn unauthorized(locale: &Locale) -> Self {
        Self::new(locale, "error-http-401")
    }

    pub fn forbidden(locale: &Locale) -> Self {
        Self::new(locale, "error-http-403")
    }

    pub fn rate_limited(locale: &Locale, seconds: u64) -> Self {
        let mut args = MessageArgs::new();
        args.insert("seconds".to_string(), seconds.to_string());
        Self::with_args(locale, "error-http-429", &args)
    }

    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

const TRANSLATION_KEYS: &[&str] = &[
    "app-name",
    "app-tagline",
    "action-save",
    "action-cancel",
    "action-delete",
    "action-edit",
    "action-close",
    "action-confirm",
    "action-retry",
    "action-back",
    "action-next",
    "action-submit",
    "action-search",
    "action-refresh",
    "action-copy",
    "action-paste",
    "action-undo",
    "action-redo",
    "action-select",
    "action-select-all",
    "action-clear",
    "action-reset",
    "action-apply",
    "action-create",
    "action-update",
    "action-remove",
    "action-add",
    "action-upload",
    "action-download",
    "action-export",
    "action-import",
    "action-share",
    "action-send",
    "action-reply",
    "action-forward",
    "action-archive",
    "action-restore",
    "action-duplicate",
    "action-rename",
    "action-move",
    "action-filter",
    "action-sort",
    "action-view",
    "action-hide",
    "action-show",
    "action-expand",
    "action-collapse",
    "action-enable",
    "action-disable",
    "action-connect",
    "action-disconnect",
    "action-sync",
    "action-start",
    "action-stop",
    "action-pause",
    "action-resume",
    "action-continue",
    "action-finish",
    "action-complete",
    "action-approve",
    "action-reject",
    "action-accept",
    "action-decline",
    "action-login",
    "action-logout",
    "action-signup",
    "action-forgot-password",
    "label-loading",
    "label-saving",
    "label-processing",
    "label-searching",
    "label-uploading",
    "label-downloading",
    "label-no-results",
    "label-no-data",
    "label-empty",
    "label-none",
    "label-all",
    "label-selected",
    "label-required",
    "label-optional",
    "label-default",
    "label-custom",
    "label-new",
    "label-draft",
    "label-pending",
    "label-active",
    "label-inactive",
    "label-enabled",
    "label-disabled",
    "label-public",
    "label-private",
    "label-shared",
    "label-yes",
    "label-no",
    "label-on",
    "label-off",
    "label-true",
    "label-false",
    "label-unknown",
    "label-other",
    "label-more",
    "label-less",
    "label-details",
    "label-summary",
    "label-description",
    "label-name",
    "label-title",
    "label-type",
    "label-status",
    "label-priority",
    "label-date",
    "label-time",
    "label-size",
    "label-count",
    "label-total",
    "label-average",
    "label-minimum",
    "label-maximum",
    "label-version",
    "label-id",
    "label-created",
    "label-updated",
    "label-modified",
    "label-deleted",
    "label-by",
    "label-from",
    "label-to",
    "label-at",
    "label-in",
    "label-of",
    "status-success",
    "status-error",
    "status-warning",
    "status-info",
    "status-loading",
    "status-complete",
    "status-incomplete",
    "status-failed",
    "status-cancelled",
    "status-pending",
    "status-in-progress",
    "status-done",
    "status-ready",
    "status-not-ready",
    "status-connected",
    "status-disconnected",
    "status-online",
    "status-offline",
    "status-available",
    "status-unavailable",
    "status-busy",
    "status-away",
    "confirm-delete",
    "confirm-delete-item",
    "confirm-discard-changes",
    "confirm-logout",
    "confirm-cancel",
    "time-now",
    "time-today",
    "time-yesterday",
    "time-tomorrow",
    "time-this-week",
    "time-last-week",
    "time-next-week",
    "time-this-month",
    "time-last-month",
    "time-next-month",
    "time-this-year",
    "time-last-year",
    "time-next-year",
    "day-sunday",
    "day-monday",
    "day-tuesday",
    "day-wednesday",
    "day-thursday",
    "day-friday",
    "day-saturday",
    "day-sun",
    "day-mon",
    "day-tue",
    "day-wed",
    "day-thu",
    "day-fri",
    "day-sat",
    "month-january",
    "month-february",
    "month-march",
    "month-april",
    "month-may",
    "month-june",
    "month-july",
    "month-august",
    "month-september",
    "month-october",
    "month-november",
    "month-december",
    "month-jan",
    "month-feb",
    "month-mar",
    "month-apr",
    "month-may-short",
    "month-jun",
    "month-jul",
    "month-aug",
    "month-sep",
    "month-oct",
    "month-nov",
    "month-dec",
    "pagination-first",
    "pagination-previous",
    "pagination-next",
    "pagination-last",
    "pagination-items-per-page",
    "pagination-go-to-page",
    "validation-required",
    "validation-email-invalid",
    "validation-url-invalid",
    "validation-number-invalid",
    "validation-date-invalid",
    "validation-pattern-mismatch",
    "validation-passwords-mismatch",
    "a11y-skip-to-content",
    "a11y-loading",
    "a11y-menu-open",
    "a11y-menu-close",
    "a11y-expand",
    "a11y-collapse",
    "a11y-selected",
    "a11y-not-selected",
    "a11y-required",
    "a11y-error",
    "a11y-success",
    "a11y-warning",
    "a11y-info",
    "nav-home",
    "nav-chat",
    "nav-drive",
    "nav-tasks",
    "nav-mail",
    "nav-calendar",
    "nav-meet",
    "nav-paper",
    "nav-research",
    "nav-analytics",
    "nav-settings",
    "nav-admin",
    "nav-monitoring",
    "nav-sources",
    "nav-tools",
    "nav-attendant",
    "nav-learn",
    "nav-crm",
    "nav-billing",
    "nav-products",
    "nav-tickets",
    "nav-docs",
    "nav-sheet",
    "nav-slides",
    "nav-social",
    "nav-all-apps",
    "nav-people",
    "nav-editor",
    "nav-dashboards",
    "nav-security",
    "nav-designer",
    "nav-project",
    "nav-canvas",
    "nav-goals",
    "nav-player",
    "nav-workspace",
    "nav-video",
    "dashboard-title",
    "dashboard-welcome",
    "dashboard-quick-actions",
    "dashboard-recent-activity",
    "chat-title",
    "chat-placeholder",
    "chat-send",
    "chat-new-conversation",
    "chat-history",
    "chat-clear",
    "chat-typing",
    "chat-online",
    "chat-offline",
    "chat-connecting",
    "drive-title",
    "drive-upload",
    "drive-new-folder",
    "drive-download",
    "drive-delete",
    "drive-rename",
    "drive-move",
    "drive-copy",
    "drive-share",
    "drive-properties",
    "drive-empty-folder",
    "drive-search-placeholder",
    "drive-sort-name",
    "drive-sort-date",
    "drive-sort-size",
    "drive-sort-type",
    "tasks-title",
    "tasks-new",
    "tasks-all",
    "tasks-pending",
    "tasks-completed",
    "tasks-overdue",
    "tasks-today",
    "tasks-this-week",
    "tasks-no-tasks",
    "tasks-priority-low",
    "tasks-priority-medium",
    "tasks-priority-high",
    "tasks-priority-urgent",
    "tasks-assign",
    "tasks-due-date",
    "tasks-description",
    "calendar-title",
    "calendar-today",
    "calendar-day",
    "calendar-week",
    "calendar-month",
    "calendar-year",
    "calendar-new-event",
    "calendar-edit-event",
    "calendar-delete-event",
    "calendar-event-title",
    "calendar-event-location",
    "calendar-event-start",
    "calendar-event-end",
    "calendar-event-all-day",
    "calendar-event-repeat",
    "calendar-event-reminder",
    "calendar-no-events",
    "meet-title",
    "meet-join",
    "meet-leave",
    "meet-mute",
    "meet-unmute",
    "meet-video-on",
    "meet-video-off",
    "meet-share-screen",
    "meet-stop-sharing",
    "meet-participants",
    "meet-chat",
    "meet-settings",
    "meet-end-call",
    "meet-invite",
    "meet-copy-link",
    "email-title",
    "email-compose",
    "email-inbox",
    "email-sent",
    "email-drafts",
    "email-trash",
    "email-spam",
    "email-starred",
    "email-archive",
    "email-to",
    "email-cc",
    "email-bcc",
    "email-subject",
    "email-body",
    "email-attachments",
    "email-send",
    "email-save-draft",
    "email-discard",
    "email-reply",
    "email-reply-all",
    "email-forward",
    "email-mark-read",
    "email-mark-unread",
    "email-delete",
    "email-no-messages",
    "settings-title",
    "settings-general",
    "settings-account",
    "settings-notifications",
    "settings-privacy",
    "settings-security",
    "settings-appearance",
    "settings-language",
    "settings-timezone",
    "settings-theme",
    "settings-theme-light",
    "settings-theme-dark",
    "settings-theme-system",
    "settings-save",
    "settings-saved",
    "admin-title",
    "admin-users",
    "admin-bots",
    "admin-system",
    "admin-logs",
    "admin-backups",
    "admin-settings",
    "error-http-400",
    "error-http-401",
    "error-http-403",
    "error-http-404",
    "error-http-429",
    "error-http-500",
    "error-http-502",
    "error-http-503",
    "error-network",
    "error-timeout",
    "error-unknown",
    "paper-title",
    "paper-new-note",
    "paper-search-notes",
    "paper-quick-start",
    "paper-template-blank",
    "paper-template-meeting",
    "paper-template-todo",
    "paper-template-research",
    "paper-untitled",
    "paper-placeholder",
    "paper-commands",
    "paper-heading1",
    "paper-heading1-desc",
    "paper-heading2",
    "paper-heading2-desc",
    "paper-heading3",
    "paper-heading3-desc",
    "paper-paragraph",
    "paper-paragraph-desc",
    "paper-bullet-list",
    "paper-bullet-list-desc",
    "paper-numbered-list",
    "paper-numbered-list-desc",
    "paper-todo-list",
    "paper-todo-list-desc",
    "paper-quote",
    "paper-quote-desc",
    "paper-divider",
    "paper-divider-desc",
    "paper-code-block",
    "paper-code-block-desc",
    "paper-table",
    "paper-table-desc",
    "paper-image",
    "paper-image-desc",
    "paper-callout",
    "paper-callout-desc",
    "paper-ai-write",
    "paper-ai-write-desc",
    "paper-ai-summarize",
    "paper-ai-summarize-desc",
    "paper-ai-expand",
    "paper-ai-expand-desc",
    "paper-ai-improve",
    "paper-ai-improve-desc",
    "paper-ai-translate",
    "paper-ai-translate-desc",
    "paper-ai-assistant",
    "paper-ai-quick-actions",
    "paper-ai-rewrite",
    "paper-ai-make-shorter",
    "paper-ai-make-longer",
    "paper-ai-fix-grammar",
    "paper-ai-tone",
    "paper-ai-tone-professional",
    "paper-ai-tone-casual",
    "paper-ai-tone-friendly",
    "paper-ai-tone-formal",
    "paper-ai-translate-to",
    "paper-ai-custom-prompt",
    "paper-ai-custom-placeholder",
    "paper-ai-generate",
    "paper-ai-response",
    "paper-ai-apply",
    "paper-ai-regenerate",
    "paper-ai-copy",
    "paper-word-count",
    "paper-char-count",
    "paper-saved",
    "paper-saving",
    "paper-last-edited",
    "paper-last-edited-now",
    "paper-export",
    "paper-export-pdf",
    "paper-export-docx",
    "paper-export-markdown",
    "paper-export-html",
    "paper-export-txt",
    "chat-voice",
    "chat-message-placeholder",
    "drive-my-drive",
    "drive-shared",
    "drive-recent",
    "drive-starred",
    "drive-trash",
    "drive-loading-storage",
    "drive-storage-used",
    "drive-empty-folder",
    "drive-drop-files",
    "tasks-active",
    "tasks-awaiting",
    "tasks-paused",
    "tasks-blocked",
    "tasks-time-saved",
    "tasks-input-placeholder",
    "calendar-my-calendars",
    "email-scheduled",
    "email-tracking",
    "email-inbox",
    "email-starred",
    "email-sent",
    "email-drafts",
    "email-spam",
    "email-trash",
    "email-compose",
    "compliance-title",
    "compliance-subtitle",
    "compliance-export",
    "compliance-run-scan",
    "compliance-critical",
    "compliance-critical-desc",
    "compliance-high",
    "compliance-high-desc",
    "compliance-medium",
    "compliance-medium-desc",
    "compliance-low",
    "compliance-low-desc",
    "compliance-info",
    "compliance-info-desc",
    "compliance-filter-severity",
    "compliance-filter-type",
    "compliance-issues-found",
    "sources-title",
    "sources-subtitle",
    "sources-prompts",
    "sources-templates",
    "sources-news",
    "sources-mcp-servers",
    "sources-llm-tools",
    "sources-models",
    "sources-repositories",
    "sources-apps",
    "attendant-title",
    "attendant-subtitle",
    "attendant-queue",
    "attendant-active",
    "attendant-resolved",
    "attendant-assign",
    "attendant-transfer",
    "attendant-resolve",
    "attendant-no-items",
    "attendant-crm-disabled",
    "attendant-status-online",
    "attendant-select-conversation",
    "sources-search",
];

pub fn get_translations_json(locale: &Locale) -> serde_json::Value {
    let mut translations = serde_json::Map::new();

    for key in TRANSLATION_KEYS {
        translations.insert((*key).to_string(), serde_json::Value::String(t(locale, key)));
    }

    serde_json::Value::Object(translations)
}

pub fn configure_i18n_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/i18n/locales", get(handle_get_locales))
        .route("/api/i18n/:locale", get(handle_get_translations))
}

async fn handle_get_locales(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let locales = available_locales();
    Json(serde_json::json!({
        "locales": locales,
        "default": "en"
    }))
}

async fn handle_get_translations(
    State(_state): State<Arc<AppState>>,
    Path(locale_str): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::new(&locale_str).unwrap_or_default();

    let translations = get_translations_json(&locale);

    Json(serde_json::json!({
        "locale": locale.to_bcp47(),
        "translations": translations
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_accept_language_simple() {
        let result = parse_accept_language("en-US");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "en-US");
        assert!((result[0].1 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_accept_language_with_quality() {
        let result = parse_accept_language("pt-BR,pt;q=0.9,en;q=0.8");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "pt-BR");
        assert_eq!(result[1].0, "pt");
        assert_eq!(result[2].0, "en");
    }

    #[test]
    fn test_parse_accept_language_sorted_by_quality() {
        let result = parse_accept_language("en;q=0.5,pt-BR;q=0.9,es;q=0.7");
        assert_eq!(result[0].0, "pt-BR");
        assert_eq!(result[1].0, "es");
        assert_eq!(result[2].0, "en");
    }

    #[test]
    fn test_negotiate_locale_exact_match() {
        let requested = vec![("pt-BR".to_string(), 1.0)];
        let result = negotiate_locale(&requested);
        assert!(result.is_some());
        assert_eq!(
            result.as_ref().map(|l| l.to_bcp47()),
            Some("pt-BR".to_string())
        );
    }

    #[test]
    fn test_negotiate_locale_language_match() {
        let requested = vec![("pt-PT".to_string(), 1.0)];
        let result = negotiate_locale(&requested);
        assert!(result.is_some());
        assert_eq!(result.as_ref().map(|l| l.language()), Some("pt"));
    }

    #[test]
    fn test_negotiate_locale_fallback() {
        let requested = vec![("ja".to_string(), 1.0)];
        let result = negotiate_locale(&requested);
        assert!(result.is_some());
        assert_eq!(result.as_ref().map(|l| l.language()), Some("en"));
    }

    #[test]
    fn test_locale_default() {
        let locale = Locale::default();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), None);
    }

    #[test]
    fn test_locale_display() {
        let locale = Locale::new("pt-BR").unwrap();
        assert_eq!(locale.to_string(), "pt-BR");
    }

    #[test]
    fn test_localized_error_not_found() {
        let locale = Locale::default();
        let error = LocalizedError::not_found(&locale, "User");
        assert_eq!(error.code, "error-http-404");
    }

    #[test]
    fn test_localized_error_with_details() {
        let locale = Locale::default();
        let error =
            LocalizedError::internal(&locale).with_details(serde_json::json!({"trace_id": "abc123"}));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_available_locales_without_init() {
        let locales = available_locales();
        assert!(!locales.is_empty());
    }
}
