use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::Write;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

const MAX_LOGS_PER_APP: usize = 500;
const MAX_LOGS_FOR_DESIGNER: usize = 50;
const LOG_RETENTION_DAYS: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub source: LogSource,
    pub app_name: String,
    pub bot_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub message: String,
    pub details: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "debug"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    Server,
    Client,
    Generator,
    Designer,
    Validation,
    Runtime,
}

impl std::fmt::Display for LogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Server => write!(f, "server"),
            Self::Client => write!(f, "client"),
            Self::Generator => write!(f, "generator"),
            Self::Designer => write!(f, "designer"),
            Self::Validation => write!(f, "validation"),
            Self::Runtime => write!(f, "runtime"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientLogRequest {
    pub app_name: String,
    pub level: String,
    pub message: String,
    pub details: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub stack_trace: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryParams {
    pub app_name: Option<String>,
    pub level: Option<String>,
    pub source: Option<String>,
    pub limit: Option<usize>,
    pub since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_logs: usize,
    pub errors: usize,
    pub warnings: usize,
    pub by_app: HashMap<String, usize>,
}

pub struct AppLogStore {
    logs: RwLock<HashMap<String, VecDeque<AppLogEntry>>>,
    global_logs: RwLock<VecDeque<AppLogEntry>>,
}

impl AppLogStore {
    pub fn new() -> Self {
        Self {
            logs: RwLock::new(HashMap::new()),
            global_logs: RwLock::new(VecDeque::with_capacity(MAX_LOGS_PER_APP)),
        }
    }

    pub fn log(
        &self,
        app_name: &str,
        level: LogLevel,
        source: LogSource,
        message: &str,
        details: Option<String>,
        bot_id: Option<Uuid>,
        user_id: Option<Uuid>,
    ) {
        let entry = AppLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            source,
            app_name: app_name.to_string(),
            bot_id,
            user_id,
            message: message.to_string(),
            details,
            file_path: None,
            line_number: None,
            stack_trace: None,
        };

        self.add_entry(entry);

        match level {
            LogLevel::Debug => debug!("[{}] {}: {}", app_name, source, message),
            LogLevel::Info => info!("[{}] {}: {}", app_name, source, message),
            LogLevel::Warn => warn!("[{}] {}: {}", app_name, source, message),
            LogLevel::Error | LogLevel::Critical => {
                error!("[{}] {}: {}", app_name, source, message);
            }
        }
    }

    pub fn log_error(
        &self,
        app_name: &str,
        source: LogSource,
        message: &str,
        error: &str,
        file_path: Option<&str>,
        line_number: Option<u32>,
        stack_trace: Option<&str>,
    ) {
        let entry = AppLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: LogLevel::Error,
            source,
            app_name: app_name.to_string(),
            bot_id: None,
            user_id: None,
            message: message.to_string(),
            details: Some(error.to_string()),
            file_path: file_path.map(String::from),
            line_number,
            stack_trace: stack_trace.map(String::from),
        };

        self.add_entry(entry);

        error!(
            "[{}] {}: {} - {} ({}:{})",
            app_name,
            source,
            message,
            error,
            file_path.unwrap_or("unknown"),
            line_number.unwrap_or(0)
        );
    }

    pub fn log_client(
        &self,
        request: ClientLogRequest,
        bot_id: Option<Uuid>,
        user_id: Option<Uuid>,
    ) {
        let level = match request.level.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "critical" => LogLevel::Critical,
            _ => LogLevel::Info,
        };

        let entry = AppLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            source: LogSource::Client,
            app_name: request.app_name,
            bot_id,
            user_id,
            message: request.message,
            details: request.details,
            file_path: request.file_path,
            line_number: request.line_number,
            stack_trace: request.stack_trace,
        };

        self.add_entry(entry);
    }

    fn add_entry(&self, entry: AppLogEntry) {
        if let Ok(mut logs) = self.logs.write() {
            let app_logs = logs
                .entry(entry.app_name.clone())
                .or_insert_with(|| VecDeque::with_capacity(MAX_LOGS_PER_APP));

            if app_logs.len() >= MAX_LOGS_PER_APP {
                app_logs.pop_front();
            }
            app_logs.push_back(entry.clone());
        }

        if let Ok(mut global) = self.global_logs.write() {
            if global.len() >= MAX_LOGS_PER_APP {
                global.pop_front();
            }
            global.push_back(entry);
        }
    }

    pub fn get_logs(&self, params: &LogQueryParams) -> Vec<AppLogEntry> {
        let limit = params.limit.unwrap_or(100).min(500);
        let cutoff = params
            .since
            .unwrap_or_else(|| Utc::now() - Duration::days(LOG_RETENTION_DAYS));

        let level_filter: Option<LogLevel> =
            params
                .level
                .as_ref()
                .and_then(|l| match l.to_lowercase().as_str() {
                    "debug" => Some(LogLevel::Debug),
                    "info" => Some(LogLevel::Info),
                    "warn" => Some(LogLevel::Warn),
                    "error" => Some(LogLevel::Error),
                    "critical" => Some(LogLevel::Critical),
                    _ => None,
                });

        let source_filter: Option<LogSource> =
            params
                .source
                .as_ref()
                .and_then(|s| match s.to_lowercase().as_str() {
                    "server" => Some(LogSource::Server),
                    "client" => Some(LogSource::Client),
                    "generator" => Some(LogSource::Generator),
                    "designer" => Some(LogSource::Designer),
                    "validation" => Some(LogSource::Validation),
                    "runtime" => Some(LogSource::Runtime),
                    _ => None,
                });

        if let Some(ref app_name) = params.app_name {
            if let Ok(logs) = self.logs.read() {
                if let Some(app_logs) = logs.get(app_name) {
                    return app_logs
                        .iter()
                        .rev()
                        .filter(|e| e.timestamp >= cutoff)
                        .filter(|e| level_filter.is_none_or(|l| e.level == l))
                        .filter(|e| source_filter.is_none_or(|s| e.source == s))
                        .take(limit)
                        .cloned()
                        .collect();
                }
            }
            return Vec::new();
        }

        if let Ok(global) = self.global_logs.read() {
            return global
                .iter()
                .rev()
                .filter(|e| e.timestamp >= cutoff)
                .filter(|e| level_filter.is_none_or(|l| e.level == l))
                .filter(|e| source_filter.is_none_or(|s| e.source == s))
                .take(limit)
                .cloned()
                .collect();
        }

        Vec::new()
    }

    pub fn get_errors_for_designer(&self, app_name: &str) -> Vec<AppLogEntry> {
        if let Ok(logs) = self.logs.read() {
            if let Some(app_logs) = logs.get(app_name) {
                let cutoff = Utc::now() - Duration::hours(1);
                return app_logs
                    .iter()
                    .rev()
                    .filter(|e| e.timestamp >= cutoff)
                    .filter(|e| {
                        matches!(
                            e.level,
                            LogLevel::Error | LogLevel::Critical | LogLevel::Warn
                        )
                    })
                    .take(MAX_LOGS_FOR_DESIGNER)
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }

    pub fn format_errors_for_prompt(&self, app_name: &str) -> Option<String> {
        let errors = self.get_errors_for_designer(app_name);

        if errors.is_empty() {
            return None;
        }

        let mut output = String::new();
        output.push_str("\n\n=== RECENT ERRORS AND WARNINGS ===\n");
        output.push_str("The following issues were detected. Please fix them:\n\n");

        for (idx, entry) in errors.iter().enumerate() {
            let _ = writeln!(
                output,
                "{}. [{}] [{}] {}",
                idx + 1,
                entry.level,
                entry.source,
                entry.message
            );

            if let Some(ref details) = entry.details {
                let _ = writeln!(output, "   Details: {details}");
            }

            if let Some(ref file) = entry.file_path {
                let _ = writeln!(
                    output,
                    "   Location: {}:{}",
                    file,
                    entry.line_number.unwrap_or(0)
                );
            }

            if let Some(ref stack) = entry.stack_trace {
                let short_stack: String = stack.lines().take(3).collect::<Vec<_>>().join("\n   ");
                let _ = writeln!(output, "   Stack: {short_stack}");
            }

            output.push('\n');
        }

        output.push_str("=== END OF ERRORS ===\n");
        Some(output)
    }

    pub fn get_stats(&self) -> LogStats {
        let mut stats = LogStats {
            total_logs: 0,
            errors: 0,
            warnings: 0,
            by_app: HashMap::new(),
        };

        if let Ok(logs) = self.logs.read() {
            for (app_name, app_logs) in logs.iter() {
                let count = app_logs.len();
                stats.total_logs += count;
                stats.by_app.insert(app_name.clone(), count);

                for entry in app_logs {
                    match entry.level {
                        LogLevel::Error | LogLevel::Critical => stats.errors += 1,
                        LogLevel::Warn => stats.warnings += 1,
                        _ => {}
                    }
                }
            }
        }

        stats
    }

    pub fn cleanup_old_logs(&self) {
        let cutoff = Utc::now() - Duration::days(LOG_RETENTION_DAYS);

        if let Ok(mut logs) = self.logs.write() {
            for app_logs in logs.values_mut() {
                while let Some(front) = app_logs.front() {
                    if front.timestamp < cutoff {
                        app_logs.pop_front();
                    } else {
                        break;
                    }
                }
            }

            logs.retain(|_, v| !v.is_empty());
        }

        if let Ok(mut global) = self.global_logs.write() {
            while let Some(front) = global.front() {
                if front.timestamp < cutoff {
                    global.pop_front();
                } else {
                    break;
                }
            }
        }

        info!("Log cleanup completed");
    }

    pub fn clear_app_logs(&self, app_name: &str) {
        if let Ok(mut logs) = self.logs.write() {
            logs.remove(app_name);
        }
        info!("Cleared logs for app: {}", app_name);
    }
}

impl Default for AppLogStore {
    fn default() -> Self {
        Self::new()
    }
}

pub static APP_LOGS: Lazy<Arc<AppLogStore>> = Lazy::new(|| Arc::new(AppLogStore::new()));

pub fn log_generator_info(app_name: &str, message: &str) {
    APP_LOGS.log(
        app_name,
        LogLevel::Info,
        LogSource::Generator,
        message,
        None,
        None,
        None,
    );
}

pub fn log_generator_error(app_name: &str, message: &str, error: &str) {
    APP_LOGS.log_error(
        app_name,
        LogSource::Generator,
        message,
        error,
        None,
        None,
        None,
    );
}

pub fn log_validation_error(
    app_name: &str,
    message: &str,
    file_path: Option<&str>,
    line_number: Option<u32>,
) {
    APP_LOGS.log_error(
        app_name,
        LogSource::Validation,
        message,
        "Validation failed",
        file_path,
        line_number,
        None,
    );
}

pub fn log_runtime_error(app_name: &str, message: &str, error: &str, stack_trace: Option<&str>) {
    APP_LOGS.log_error(
        app_name,
        LogSource::Runtime,
        message,
        error,
        None,
        None,
        stack_trace,
    );
}

pub fn get_designer_error_context(app_name: &str) -> Option<String> {
    APP_LOGS.format_errors_for_prompt(app_name)
}

pub fn start_log_cleanup_scheduler() {
    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
        APP_LOGS.cleanup_old_logs();
    });
    info!("Log cleanup scheduler started (runs hourly)");
}

pub fn generate_client_logger_js() -> &'static str {
    r"
(function() {
    const APP_NAME = document.body.dataset.appName || window.location.pathname.split('/')[1] || 'unknown';
    const LOG_ENDPOINT = '/api/app-logs/client';
    const LOG_BUFFER = [];
    const FLUSH_INTERVAL = 5000;
    const MAX_BUFFER_SIZE = 50;

    function sendLogs() {
        if (LOG_BUFFER.length === 0) return;

        const logs = LOG_BUFFER.splice(0, LOG_BUFFER.length);

        fetch(LOG_ENDPOINT, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ logs: logs })
        }).catch(function(e) {
            console.warn('Failed to send logs:', e);
        });
    }

    function addLog(level, message, details) {
        const entry = {
            app_name: APP_NAME,
            level: level,
            message: message,
            details: details || null,
            file_path: null,
            line_number: null,
            stack_trace: null,
            user_agent: navigator.userAgent
        };

        LOG_BUFFER.push(entry);

        if (LOG_BUFFER.length >= MAX_BUFFER_SIZE) {
            sendLogs();
        }
    }

    window.onerror = function(message, source, lineno, colno, error) {
        addLog('error', message, JSON.stringify({
            source: source,
            line: lineno,
            column: colno,
            stack: error ? error.stack : null
        }));
        return false;
    };

    window.onunhandledrejection = function(event) {
        addLog('error', 'Unhandled Promise Rejection: ' + event.reason,
            event.reason && event.reason.stack ? event.reason.stack : null);
    };

    const originalConsoleError = console.error;
    console.error = function() {
        addLog('error', Array.from(arguments).join(' '));
        originalConsoleError.apply(console, arguments);
    };

    const originalConsoleWarn = console.warn;
    console.warn = function() {
        addLog('warn', Array.from(arguments).join(' '));
        originalConsoleWarn.apply(console, arguments);
    };

    document.body.addEventListener('htmx:responseError', function(evt) {
        addLog('error', 'HTMX Request Failed', JSON.stringify({
            url: evt.detail.xhr.responseURL,
            status: evt.detail.xhr.status,
            response: evt.detail.xhr.responseText.substring(0, 500)
        }));
    });

    document.body.addEventListener('htmx:sendError', function(evt) {
        addLog('error', 'HTMX Send Error', JSON.stringify({
            url: evt.detail.requestConfig.path
        }));
    });

    setInterval(sendLogs, FLUSH_INTERVAL);
    window.addEventListener('beforeunload', sendLogs);

    window.AppLogger = {
        debug: function(msg, details) { addLog('debug', msg, details); },
        info: function(msg, details) { addLog('info', msg, details); },
        warn: function(msg, details) { addLog('warn', msg, details); },
        error: function(msg, details) { addLog('error', msg, details); },
        flush: sendLogs
    };

    console.log('[AppLogger] Initialized for app:', APP_NAME);
})();
"
}
