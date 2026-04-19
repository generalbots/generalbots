use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub const MAX_LOOP_ITERATIONS: u32 = 100_000;
pub const MAX_RECURSION_DEPTH: u32 = 100;
pub const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_UPLOAD_SIZE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_REQUEST_BODY_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_STRING_LENGTH: usize = 10 * 1024 * 1024;
pub const MAX_ARRAY_LENGTH: usize = 1_000_000;
pub const MAX_CONCURRENT_REQUESTS_PER_USER: u32 = 100;
pub const MAX_CONCURRENT_REQUESTS_GLOBAL: u32 = 10_000;
pub const MAX_WEBSOCKET_CONNECTIONS_PER_USER: u32 = 10;
pub const MAX_WEBSOCKET_CONNECTIONS_GLOBAL: u32 = 50_000;
pub const MAX_DB_QUERY_RESULTS: u32 = 10_000;
pub const MAX_DB_CONNECTIONS_PER_TENANT: u32 = 20;
pub const MAX_LLM_TOKENS_PER_REQUEST: u32 = 128_000;
pub const MAX_LLM_REQUESTS_PER_MINUTE: u32 = 60;
pub const MAX_KB_DOCUMENTS_PER_BOT: u32 = 100_000;
pub const MAX_KB_DOCUMENT_SIZE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_SCRIPT_EXECUTION_SECONDS: u64 = 300;
pub const MAX_API_CALLS_PER_MINUTE: u32 = 1000;
pub const MAX_API_CALLS_PER_HOUR: u32 = 10_000;
pub const MAX_DRIVE_STORAGE_BYTES: u64 = 10 * 1024 * 1024 * 1024;
pub const MAX_SESSION_IDLE_SECONDS: u64 = 3600;
pub const MAX_SESSIONS_PER_USER: u32 = 10;
pub const MAX_BOTS_PER_TENANT: u32 = 100;
pub const MAX_TOOLS_PER_BOT: u32 = 500;
pub const MAX_PENDING_TASKS: u32 = 1000;
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
pub const RATE_LIMIT_BURST_MULTIPLIER: f64 = 1.5;

#[derive(Debug, Clone)]
pub struct SystemLimits {
    pub max_loop_iterations: u32,
    pub max_recursion_depth: u32,
    pub max_file_size_bytes: u64,
    pub max_upload_size_bytes: u64,
    pub max_request_body_bytes: u64,
    pub max_string_length: usize,
    pub max_array_length: usize,
    pub max_concurrent_requests_per_user: u32,
    pub max_concurrent_requests_global: u32,
    pub max_websocket_connections_per_user: u32,
    pub max_websocket_connections_global: u32,
    pub max_db_query_results: u32,
    pub max_db_connections_per_tenant: u32,
    pub max_llm_tokens_per_request: u32,
    pub max_llm_requests_per_minute: u32,
    pub max_kb_documents_per_bot: u32,
    pub max_kb_document_size_bytes: u64,
    pub max_script_execution_seconds: u64,
    pub max_api_calls_per_minute: u32,
    pub max_api_calls_per_hour: u32,
    pub max_drive_storage_bytes: u64,
    pub max_session_idle_seconds: u64,
    pub max_sessions_per_user: u32,
    pub max_bots_per_tenant: u32,
    pub max_tools_per_bot: u32,
    pub max_pending_tasks: u32,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_burst_multiplier: f64,
}

impl Default for SystemLimits {
    fn default() -> Self {
        Self {
            max_loop_iterations: MAX_LOOP_ITERATIONS,
            max_recursion_depth: MAX_RECURSION_DEPTH,
            max_file_size_bytes: MAX_FILE_SIZE_BYTES,
            max_upload_size_bytes: MAX_UPLOAD_SIZE_BYTES,
            max_request_body_bytes: MAX_REQUEST_BODY_BYTES,
            max_string_length: MAX_STRING_LENGTH,
            max_array_length: MAX_ARRAY_LENGTH,
            max_concurrent_requests_per_user: MAX_CONCURRENT_REQUESTS_PER_USER,
            max_concurrent_requests_global: MAX_CONCURRENT_REQUESTS_GLOBAL,
            max_websocket_connections_per_user: MAX_WEBSOCKET_CONNECTIONS_PER_USER,
            max_websocket_connections_global: MAX_WEBSOCKET_CONNECTIONS_GLOBAL,
            max_db_query_results: MAX_DB_QUERY_RESULTS,
            max_db_connections_per_tenant: MAX_DB_CONNECTIONS_PER_TENANT,
            max_llm_tokens_per_request: MAX_LLM_TOKENS_PER_REQUEST,
            max_llm_requests_per_minute: MAX_LLM_REQUESTS_PER_MINUTE,
            max_kb_documents_per_bot: MAX_KB_DOCUMENTS_PER_BOT,
            max_kb_document_size_bytes: MAX_KB_DOCUMENT_SIZE_BYTES,
            max_script_execution_seconds: MAX_SCRIPT_EXECUTION_SECONDS,
            max_api_calls_per_minute: MAX_API_CALLS_PER_MINUTE,
            max_api_calls_per_hour: MAX_API_CALLS_PER_HOUR,
            max_drive_storage_bytes: MAX_DRIVE_STORAGE_BYTES,
            max_session_idle_seconds: MAX_SESSION_IDLE_SECONDS,
            max_sessions_per_user: MAX_SESSIONS_PER_USER,
            max_bots_per_tenant: MAX_BOTS_PER_TENANT,
            max_tools_per_bot: MAX_TOOLS_PER_BOT,
            max_pending_tasks: MAX_PENDING_TASKS,
            rate_limit_window_seconds: RATE_LIMIT_WINDOW_SECONDS,
            rate_limit_burst_multiplier: RATE_LIMIT_BURST_MULTIPLIER,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitType {
    LoopIterations,
    RecursionDepth,
    FileSize,
    UploadSize,
    RequestBody,
    StringLength,
    ArrayLength,
    ConcurrentRequests,
    WebsocketConnections,
    DbQueryResults,
    DbConnections,
    LlmTokens,
    LlmRequests,
    KbDocuments,
    KbDocumentSize,
    ScriptExecution,
    ApiCallsMinute,
    ApiCallsHour,
    DriveStorage,
    SessionIdle,
    SessionsPerUser,
    BotsPerTenant,
    ToolsPerBot,
    PendingTasks,
}

impl std::fmt::Display for LimitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoopIterations => write!(f, "loop_iterations"),
            Self::RecursionDepth => write!(f, "recursion_depth"),
            Self::FileSize => write!(f, "file_size"),
            Self::UploadSize => write!(f, "upload_size"),
            Self::RequestBody => write!(f, "request_body"),
            Self::StringLength => write!(f, "string_length"),
            Self::ArrayLength => write!(f, "array_length"),
            Self::ConcurrentRequests => write!(f, "concurrent_requests"),
            Self::WebsocketConnections => write!(f, "websocket_connections"),
            Self::DbQueryResults => write!(f, "db_query_results"),
            Self::DbConnections => write!(f, "db_connections"),
            Self::LlmTokens => write!(f, "llm_tokens"),
            Self::LlmRequests => write!(f, "llm_requests"),
            Self::KbDocuments => write!(f, "kb_documents"),
            Self::KbDocumentSize => write!(f, "kb_document_size"),
            Self::ScriptExecution => write!(f, "script_execution"),
            Self::ApiCallsMinute => write!(f, "api_calls_minute"),
            Self::ApiCallsHour => write!(f, "api_calls_hour"),
            Self::DriveStorage => write!(f, "drive_storage"),
            Self::SessionIdle => write!(f, "session_idle"),
            Self::SessionsPerUser => write!(f, "sessions_per_user"),
            Self::BotsPerTenant => write!(f, "bots_per_tenant"),
            Self::ToolsPerBot => write!(f, "tools_per_bot"),
            Self::PendingTasks => write!(f, "pending_tasks"),
        }
    }
}

#[derive(Debug)]
pub struct LimitExceeded {
    pub limit_type: LimitType,
    pub current: u64,
    pub maximum: u64,
    pub retry_after_secs: Option<u64>,
}

impl std::fmt::Display for LimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Limit exceeded for {}: {} > {} (max)",
            self.limit_type, self.current, self.maximum
        )
    }
}

impl std::error::Error for LimitExceeded {}

#[derive(Debug)]
struct RateLimitEntry {
    count: AtomicU64,
    window_start: RwLock<Instant>,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            window_start: RwLock::new(Instant::now()),
        }
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    limits: SystemLimits,
    per_user_minute: RwLock<HashMap<String, Arc<RateLimitEntry>>>,
    per_user_hour: RwLock<HashMap<String, Arc<RateLimitEntry>>>,
    global_minute: Arc<RateLimitEntry>,
    global_hour: Arc<RateLimitEntry>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(SystemLimits::default())
    }
}

impl RateLimiter {
    pub fn new(limits: SystemLimits) -> Self {
        Self {
            limits,
            per_user_minute: RwLock::new(HashMap::new()),
            per_user_hour: RwLock::new(HashMap::new()),
            global_minute: Arc::new(RateLimitEntry::new()),
            global_hour: Arc::new(RateLimitEntry::new()),
        }
    }

    pub async fn check_rate_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        self.check_global_limits().await?;
        self.check_user_limits(user_id).await
    }

    async fn check_global_limits(&self) -> Result<(), LimitExceeded> {
        let now = Instant::now();

        {
            let window_start = self.global_minute.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(60) {
                drop(window_start);
                let mut window_start = self.global_minute.window_start.write().await;
                *window_start = now;
                self.global_minute.count.store(0, Ordering::SeqCst);
            }
        }

        let count = self.global_minute.count.fetch_add(1, Ordering::SeqCst) + 1;
        let max = u64::from(self.limits.max_api_calls_per_minute) * 100;

        if count > max {
            self.global_minute.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsMinute,
                current: count,
                maximum: max,
                retry_after_secs: Some(60),
            });
        }

        {
            let window_start = self.global_hour.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(3600) {
                drop(window_start);
                let mut window_start = self.global_hour.window_start.write().await;
                *window_start = now;
                self.global_hour.count.store(0, Ordering::SeqCst);
            }
        }

        let hour_count = self.global_hour.count.fetch_add(1, Ordering::SeqCst) + 1;
        let hour_max = u64::from(self.limits.max_api_calls_per_hour) * 100;

        if hour_count > hour_max {
            self.global_hour.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsHour,
                current: hour_count,
                maximum: hour_max,
                retry_after_secs: Some(3600),
            });
        }

        Ok(())
    }

    async fn check_user_limits(&self, user_id: &str) -> Result<(), LimitExceeded> {
        self.check_user_minute_limit(user_id).await?;
        self.check_user_hour_limit(user_id).await
    }

    async fn check_user_minute_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        let entry = {
            let map = self.per_user_minute.read().await;
            map.get(user_id).cloned()
        };

        let entry = match entry {
            Some(e) => e,
            None => {
                let new_entry = Arc::new(RateLimitEntry::new());
                let mut map = self.per_user_minute.write().await;
                map.insert(user_id.to_string(), Arc::clone(&new_entry));
                new_entry
            }
        };

        let now = Instant::now();
        {
            let window_start = entry.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(60) {
                drop(window_start);
                let mut window_start = entry.window_start.write().await;
                *window_start = now;
                entry.count.store(0, Ordering::SeqCst);
            }
        }

        let count = entry.count.fetch_add(1, Ordering::SeqCst) + 1;
        let max = u64::from(self.limits.max_api_calls_per_minute);

        if count > max {
            entry.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsMinute,
                current: count,
                maximum: max,
                retry_after_secs: Some(60),
            });
        }

        Ok(())
    }

    async fn check_user_hour_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        let entry = {
            let map = self.per_user_hour.read().await;
            map.get(user_id).cloned()
        };

        let entry = match entry {
            Some(e) => e,
            None => {
                let new_entry = Arc::new(RateLimitEntry::new());
                let mut map = self.per_user_hour.write().await;
                map.insert(user_id.to_string(), Arc::clone(&new_entry));
                new_entry
            }
        };

        let now = Instant::now();
        {
            let window_start = entry.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(3600) {
                drop(window_start);
                let mut window_start = entry.window_start.write().await;
                *window_start = now;
                entry.count.store(0, Ordering::SeqCst);
            }
        }

        let count = entry.count.fetch_add(1, Ordering::SeqCst) + 1;
        let max = u64::from(self.limits.max_api_calls_per_hour);

        if count > max {
            entry.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsHour,
                current: count,
                maximum: max,
                retry_after_secs: Some(3600),
            });
        }

        Ok(())
    }

    pub async fn cleanup_stale_entries(&self) {
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(7200);

        {
            let mut map = self.per_user_minute.write().await;
            let mut to_remove = Vec::new();
            for (user_id, entry) in map.iter() {
                let window_start = entry.window_start.read().await;
                if now.duration_since(*window_start) > stale_threshold {
                    to_remove.push(user_id.clone());
                }
            }
            for user_id in to_remove {
                map.remove(&user_id);
            }
        }

        {
            let mut map = self.per_user_hour.write().await;
            let mut to_remove = Vec::new();
            for (user_id, entry) in map.iter() {
                let window_start = entry.window_start.read().await;
                if now.duration_since(*window_start) > stale_threshold {
                    to_remove.push(user_id.clone());
                }
            }
            for user_id in to_remove {
                map.remove(&user_id);
            }
        }
    }
}

pub fn check_loop_limit(iterations: u32, max: u32) -> Result<(), LimitExceeded> {
    if iterations >= max {
        return Err(LimitExceeded {
            limit_type: LimitType::LoopIterations,
            current: u64::from(iterations),
            maximum: u64::from(max),
            retry_after_secs: None,
        });
    }
    Ok(())
}

pub fn check_recursion_limit(depth: u32, max: u32) -> Result<(), LimitExceeded> {
    if depth >= max {
        return Err(LimitExceeded {
            limit_type: LimitType::RecursionDepth,
            current: u64::from(depth),
            maximum: u64::from(max),
            retry_after_secs: None,
        });
    }
    Ok(())
}

pub fn check_file_size_limit(size: u64, max: u64) -> Result<(), LimitExceeded> {
    if size > max {
        return Err(LimitExceeded {
            limit_type: LimitType::FileSize,
            current: size,
            maximum: max,
            retry_after_secs: None,
        });
    }
    Ok(())
}

pub fn check_string_length_limit(length: usize, max: usize) -> Result<(), LimitExceeded> {
    if length > max {
        return Err(LimitExceeded {
            limit_type: LimitType::StringLength,
            current: length as u64,
            maximum: max as u64,
            retry_after_secs: None,
        });
    }
    Ok(())
}

pub fn check_array_length_limit(length: usize, max: usize) -> Result<(), LimitExceeded> {
    if length > max {
        return Err(LimitExceeded {
            limit_type: LimitType::ArrayLength,
            current: length as u64,
            maximum: max as u64,
            retry_after_secs: None,
        });
    }
    Ok(())
}

pub fn format_limit_error_response(error: &LimitExceeded) -> (u16, String) {
    let status = 429;
    let body = serde_json::json!({
        "error": "rate_limit_exceeded",
        "message": error.to_string(),
        "limit_type": error.limit_type.to_string(),
        "current": error.current,
        "maximum": error.maximum,
        "retry_after_secs": error.retry_after_secs,
    });
    (status, body.to_string())
}
