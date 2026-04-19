use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text, Timestamptz};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

const DEFAULT_RETENTION_DAYS: i64 = 180;
const MIN_RETENTION_DAYS: i64 = 7;
const MAX_RETENTION_DAYS: i64 = 3650;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupCategory {
    BotHistory,
    ChatSessions,
    SystemLogs,
    AccessLogs,
    DebugLogs,
    TempFiles,
    CacheData,
    AnalyticsRaw,
    OldCalendarEvents,
    Attachments,
}

impl std::fmt::Display for CleanupCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BotHistory => write!(f, "Bot History"),
            Self::ChatSessions => write!(f, "Chat Sessions"),
            Self::SystemLogs => write!(f, "System Logs"),
            Self::AccessLogs => write!(f, "Access Logs"),
            Self::DebugLogs => write!(f, "Debug Logs"),
            Self::TempFiles => write!(f, "Temp Files"),
            Self::CacheData => write!(f, "Cache Data"),
            Self::AnalyticsRaw => write!(f, "Analytics Raw Data"),
            Self::OldCalendarEvents => write!(f, "Old Calendar Events"),
            Self::Attachments => write!(f, "Old Attachments"),
        }
    }
}

impl CleanupCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::BotHistory,
            Self::ChatSessions,
            Self::SystemLogs,
            Self::AccessLogs,
            Self::DebugLogs,
            Self::TempFiles,
            Self::CacheData,
            Self::AnalyticsRaw,
        ]
    }

    pub fn default_enabled() -> Vec<Self> {
        vec![
            Self::BotHistory,
            Self::ChatSessions,
            Self::SystemLogs,
            Self::DebugLogs,
            Self::TempFiles,
            Self::CacheData,
        ]
    }

    fn table_name(&self) -> &'static str {
        match self {
            Self::BotHistory => "bot_conversation_history",
            Self::ChatSessions => "chat_sessions",
            Self::SystemLogs => "system_logs",
            Self::AccessLogs => "access_logs",
            Self::DebugLogs => "debug_logs",
            Self::TempFiles => "temp_files",
            Self::CacheData => "cache_entries",
            Self::AnalyticsRaw => "analytics_events",
            Self::OldCalendarEvents => "calendar_events",
            Self::Attachments => "attachments",
        }
    }

    fn timestamp_column(&self) -> &'static str {
        match self {
            Self::BotHistory => "created_at",
            Self::ChatSessions => "last_activity_at",
            Self::SystemLogs => "logged_at",
            Self::AccessLogs => "accessed_at",
            Self::DebugLogs => "logged_at",
            Self::TempFiles => "created_at",
            Self::CacheData => "created_at",
            Self::AnalyticsRaw => "event_time",
            Self::OldCalendarEvents => "end_time",
            Self::Attachments => "created_at",
        }
    }

    fn is_essential(&self) -> bool {
        matches!(self, Self::OldCalendarEvents | Self::Attachments)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    pub organization_id: Uuid,
    pub retention_days: i64,
    pub enabled_categories: Vec<CleanupCategory>,
    pub auto_cleanup_enabled: bool,
    pub auto_cleanup_schedule: Option<String>,
    pub storage_alert_threshold_percent: u8,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            organization_id: Uuid::nil(),
            retention_days: DEFAULT_RETENTION_DAYS,
            enabled_categories: CleanupCategory::default_enabled(),
            auto_cleanup_enabled: false,
            auto_cleanup_schedule: None,
            storage_alert_threshold_percent: 80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPreview {
    pub categories: Vec<CategoryPreview>,
    pub total_records: i64,
    pub estimated_bytes_freed: i64,
    pub estimated_bytes_freed_formatted: String,
    pub cutoff_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPreview {
    pub category: CleanupCategory,
    pub display_name: String,
    pub record_count: i64,
    pub estimated_bytes: i64,
    pub estimated_bytes_formatted: String,
    pub oldest_record: Option<DateTime<Utc>>,
    pub is_essential: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub success: bool,
    pub categories_cleaned: Vec<CategoryResult>,
    pub total_records_deleted: i64,
    pub bytes_freed: i64,
    pub bytes_freed_formatted: String,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub category: CleanupCategory,
    pub display_name: String,
    pub records_deleted: i64,
    pub bytes_freed: i64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub organization_id: Uuid,
    pub categories: Vec<CategoryStorage>,
    pub total_bytes: i64,
    pub total_bytes_formatted: String,
    pub quota_bytes: Option<i64>,
    pub quota_percent_used: Option<f32>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStorage {
    pub category: CleanupCategory,
    pub display_name: String,
    pub bytes_used: i64,
    pub bytes_formatted: String,
    pub record_count: i64,
    pub percent_of_total: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupHistory {
    pub id: String,
    pub organization_id: Uuid,
    pub performed_at: DateTime<Utc>,
    pub performed_by: Option<Uuid>,
    pub trigger: CleanupTrigger,
    pub categories_cleaned: Vec<CleanupCategory>,
    pub records_deleted: i64,
    pub bytes_freed: i64,
    pub retention_days_used: i64,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupTrigger {
    Manual,
    Scheduled,
    StorageAlert,
}

impl std::fmt::Display for CleanupTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "Manual"),
            Self::Scheduled => write!(f, "Scheduled"),
            Self::StorageAlert => write!(f, "Storage Alert"),
        }
    }
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct SizeRow {
    #[diesel(sql_type = BigInt)]
    size_bytes: i64,
}

#[derive(QueryableByName)]
struct OldestRow {
    #[diesel(sql_type = Timestamptz)]
    oldest: DateTime<Utc>,
}

pub struct CleanupService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
}

impl CleanupService {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    ) -> Self {
        Self { pool }
    }

    pub async fn preview_cleanup(
        &self,
        organization_id: Uuid,
        retention_days: Option<i64>,
        categories: Option<Vec<CleanupCategory>>,
    ) -> Result<CleanupPreview, CleanupError> {
        let retention = self.validate_retention_days(retention_days.unwrap_or(DEFAULT_RETENTION_DAYS))?;
        let cutoff_date = Utc::now() - Duration::days(retention);
        let cats = categories.unwrap_or_else(CleanupCategory::default_enabled);

        let mut category_previews = Vec::new();
        let mut total_records = 0i64;
        let mut total_bytes = 0i64;

        for category in cats {
            let preview = self.preview_category(organization_id, category, cutoff_date).await?;
            total_records += preview.record_count;
            total_bytes += preview.estimated_bytes;
            category_previews.push(preview);
        }

        Ok(CleanupPreview {
            categories: category_previews,
            total_records,
            estimated_bytes_freed: total_bytes,
            estimated_bytes_freed_formatted: format_bytes(total_bytes),
            cutoff_date,
        })
    }

    async fn preview_category(
        &self,
        organization_id: Uuid,
        category: CleanupCategory,
        cutoff_date: DateTime<Utc>,
    ) -> Result<CategoryPreview, CleanupError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            CleanupError::DatabaseConnection
        })?;

        let table = category.table_name();
        let ts_col = category.timestamp_column();

        let count_sql = format!(
            "SELECT COUNT(*) as count FROM {table} WHERE organization_id = $1 AND {ts_col} < $2"
        );

        let count_result: Vec<CountRow> = diesel::sql_query(&count_sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .bind::<Timestamptz, _>(cutoff_date)
            .load(&mut conn)
            .unwrap_or_default();

        let record_count = count_result.first().map(|r| r.count).unwrap_or(0);

        let size_sql = format!(
            "SELECT COALESCE(pg_total_relation_size('{table}') *
             (SELECT COUNT(*) FROM {table} WHERE organization_id = $1 AND {ts_col} < $2)::float /
             NULLIF((SELECT COUNT(*) FROM {table} WHERE organization_id = $1), 0), 0)::bigint as size_bytes"
        );

        let size_result: Vec<SizeRow> = diesel::sql_query(&size_sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .bind::<Timestamptz, _>(cutoff_date)
            .load(&mut conn)
            .unwrap_or_default();

        let estimated_bytes = size_result.first().map(|r| r.size_bytes).unwrap_or(0);

        let oldest_sql = format!(
            "SELECT MIN({ts_col}) as oldest FROM {table} WHERE organization_id = $1 AND {ts_col} < $2"
        );

        let oldest_result: Result<Vec<OldestRow>, _> = diesel::sql_query(&oldest_sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .bind::<Timestamptz, _>(cutoff_date)
            .load(&mut conn);

        let oldest_record = oldest_result.ok().and_then(|r| r.first().map(|o| o.oldest));

        Ok(CategoryPreview {
            category,
            display_name: category.to_string(),
            record_count,
            estimated_bytes,
            estimated_bytes_formatted: format_bytes(estimated_bytes),
            oldest_record,
            is_essential: category.is_essential(),
        })
    }

    pub async fn execute_cleanup(
        &self,
        organization_id: Uuid,
        retention_days: Option<i64>,
        categories: Option<Vec<CleanupCategory>>,
        performed_by: Option<Uuid>,
        trigger: CleanupTrigger,
    ) -> Result<CleanupResult, CleanupError> {
        let start_time = std::time::Instant::now();
        let retention = self.validate_retention_days(retention_days.unwrap_or(DEFAULT_RETENTION_DAYS))?;
        let cutoff_date = Utc::now() - Duration::days(retention);
        let cats = categories.unwrap_or_else(CleanupCategory::default_enabled);

        info!(
            "Starting cleanup for org {} with {} day retention ({} categories)",
            organization_id,
            retention,
            cats.len()
        );

        let mut category_results = Vec::new();
        let mut total_deleted = 0i64;
        let mut total_bytes = 0i64;
        let mut errors = Vec::new();

        for category in &cats {
            match self.cleanup_category(organization_id, *category, cutoff_date).await {
                Ok(result) => {
                    total_deleted += result.records_deleted;
                    total_bytes += result.bytes_freed;
                    category_results.push(result);
                }
                Err(e) => {
                    let error_msg = format!("{}: {e}", category);
                    errors.push(error_msg.clone());
                    category_results.push(CategoryResult {
                        category: *category,
                        display_name: category.to_string(),
                        records_deleted: 0,
                        bytes_freed: 0,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let success = errors.is_empty();

        let history = CleanupHistory {
            id: Uuid::new_v4().to_string(),
            organization_id,
            performed_at: Utc::now(),
            performed_by,
            trigger,
            categories_cleaned: cats.clone(),
            records_deleted: total_deleted,
            bytes_freed: total_bytes,
            retention_days_used: retention,
            duration_ms,
            success,
            error_message: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        };

        if let Err(e) = self.save_cleanup_history(&history).await {
            warn!("Failed to save cleanup history: {e}");
        }

        info!(
            "Cleanup completed for org {}: deleted {} records, freed {} in {}ms",
            organization_id,
            total_deleted,
            format_bytes(total_bytes),
            duration_ms
        );

        Ok(CleanupResult {
            success,
            categories_cleaned: category_results,
            total_records_deleted: total_deleted,
            bytes_freed: total_bytes,
            bytes_freed_formatted: format_bytes(total_bytes),
            duration_ms,
            errors,
        })
    }

    async fn cleanup_category(
        &self,
        organization_id: Uuid,
        category: CleanupCategory,
        cutoff_date: DateTime<Utc>,
    ) -> Result<CategoryResult, CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        let table = category.table_name();
        let ts_col = category.timestamp_column();

        let size_before: i64 = diesel::sql_query(format!(
            "SELECT pg_total_relation_size('{table}') as size_bytes"
        ))
        .load::<SizeRow>(&mut conn)
        .ok()
        .and_then(|r| r.first().map(|s| s.size_bytes))
        .unwrap_or(0);

        let delete_sql = format!(
            "DELETE FROM {table} WHERE organization_id = $1 AND {ts_col} < $2"
        );

        let deleted = diesel::sql_query(&delete_sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .bind::<Timestamptz, _>(cutoff_date)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to cleanup {}: {e}", category);
                CleanupError::CleanupFailed(category.to_string())
            })?;

        let size_after: i64 = diesel::sql_query(format!(
            "SELECT pg_total_relation_size('{table}') as size_bytes"
        ))
        .load::<SizeRow>(&mut conn)
        .ok()
        .and_then(|r| r.first().map(|s| s.size_bytes))
        .unwrap_or(0);

        let bytes_freed = (size_before - size_after).max(0);

        debug!(
            "Cleaned {} from {}: {} records, {} freed",
            category,
            table,
            deleted,
            format_bytes(bytes_freed)
        );

        Ok(CategoryResult {
            category,
            display_name: category.to_string(),
            records_deleted: deleted as i64,
            bytes_freed,
            success: true,
            error: None,
        })
    }

    pub async fn get_storage_usage(&self, organization_id: Uuid) -> Result<StorageUsage, CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        let mut categories = Vec::new();
        let mut total_bytes = 0i64;

        for category in CleanupCategory::all() {
            let table = category.table_name();
            let _ts_col = category.timestamp_column();

            let count_sql = format!(
                "SELECT COUNT(*) as count FROM {table} WHERE organization_id = $1"
            );

            let count: i64 = diesel::sql_query(&count_sql)
                .bind::<diesel::sql_types::Uuid, _>(organization_id)
                .load::<CountRow>(&mut conn)
                .ok()
                .and_then(|r| r.first().map(|c| c.count))
                .unwrap_or(0);

            let size_sql = format!(
                "SELECT COALESCE(pg_total_relation_size('{table}') *
                 (SELECT COUNT(*) FROM {table} WHERE organization_id = $1)::float /
                 NULLIF((SELECT COUNT(*) FROM {table}), 0), 0)::bigint as size_bytes"
            );

            let bytes: i64 = diesel::sql_query(&size_sql)
                .bind::<diesel::sql_types::Uuid, _>(organization_id)
                .load::<SizeRow>(&mut conn)
                .ok()
                .and_then(|r| r.first().map(|s| s.size_bytes))
                .unwrap_or(0);

            total_bytes += bytes;

            categories.push(CategoryStorage {
                category,
                display_name: category.to_string(),
                bytes_used: bytes,
                bytes_formatted: format_bytes(bytes),
                record_count: count,
                percent_of_total: 0.0,
            });
        }

        for cat in &mut categories {
            cat.percent_of_total = if total_bytes > 0 {
                (cat.bytes_used as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };
        }

        categories.sort_by(|a, b| b.bytes_used.cmp(&a.bytes_used));

        let quota = self.get_organization_quota(organization_id).await.ok();
        let quota_percent = quota.map(|q| (total_bytes as f32 / q as f32) * 100.0);

        Ok(StorageUsage {
            organization_id,
            categories,
            total_bytes,
            total_bytes_formatted: format_bytes(total_bytes),
            quota_bytes: quota,
            quota_percent_used: quota_percent,
            last_updated: Utc::now(),
        })
    }

    pub async fn get_cleanup_history(
        &self,
        organization_id: Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<CleanupHistory>, CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        let limit = limit.unwrap_or(50).min(100);

        #[derive(QueryableByName)]
        struct HistoryRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            organization_id: Uuid,
            #[diesel(sql_type = Timestamptz)]
            performed_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            performed_by: Option<Uuid>,
            #[diesel(sql_type = Text)]
            trigger: String,
            #[diesel(sql_type = Text)]
            categories_cleaned: String,
            #[diesel(sql_type = BigInt)]
            records_deleted: i64,
            #[diesel(sql_type = BigInt)]
            bytes_freed: i64,
            #[diesel(sql_type = BigInt)]
            retention_days_used: i64,
            #[diesel(sql_type = BigInt)]
            duration_ms: i64,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            success: bool,
            #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
            error_message: Option<String>,
        }

        let sql = r#"
            SELECT id, organization_id, performed_at, performed_by, trigger,
                   categories_cleaned, records_deleted, bytes_freed,
                   retention_days_used, duration_ms, success, error_message
            FROM cleanup_history
            WHERE organization_id = $1
            ORDER BY performed_at DESC
            LIMIT $2
        "#;

        let rows: Vec<HistoryRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .bind::<diesel::sql_types::Integer, _>(limit)
            .load(&mut conn)
            .unwrap_or_default();

        let history = rows
            .into_iter()
            .map(|row| {
                let trigger = match row.trigger.as_str() {
                    "scheduled" => CleanupTrigger::Scheduled,
                    "storage_alert" => CleanupTrigger::StorageAlert,
                    _ => CleanupTrigger::Manual,
                };

                let categories: Vec<CleanupCategory> = row
                    .categories_cleaned
                    .split(',')
                    .filter_map(|s| match s.trim() {
                        "bot_history" => Some(CleanupCategory::BotHistory),
                        "chat_sessions" => Some(CleanupCategory::ChatSessions),
                        "system_logs" => Some(CleanupCategory::SystemLogs),
                        "access_logs" => Some(CleanupCategory::AccessLogs),
                        "debug_logs" => Some(CleanupCategory::DebugLogs),
                        "temp_files" => Some(CleanupCategory::TempFiles),
                        "cache_data" => Some(CleanupCategory::CacheData),
                        "analytics_raw" => Some(CleanupCategory::AnalyticsRaw),
                        _ => None,
                    })
                    .collect();

                CleanupHistory {
                    id: row.id,
                    organization_id: row.organization_id,
                    performed_at: row.performed_at,
                    performed_by: row.performed_by,
                    trigger,
                    categories_cleaned: categories,
                    records_deleted: row.records_deleted,
                    bytes_freed: row.bytes_freed,
                    retention_days_used: row.retention_days_used,
                    duration_ms: row.duration_ms as u64,
                    success: row.success,
                    error_message: row.error_message,
                }
            })
            .collect();

        Ok(history)
    }

    pub async fn get_cleanup_config(&self, organization_id: Uuid) -> Result<CleanupConfig, CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = BigInt)]
            retention_days: i64,
            #[diesel(sql_type = Text)]
            enabled_categories: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            auto_cleanup_enabled: bool,
            #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
            auto_cleanup_schedule: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            storage_alert_threshold: i32,
        }

        let sql = r#"
            SELECT retention_days, enabled_categories, auto_cleanup_enabled,
                   auto_cleanup_schedule, storage_alert_threshold
            FROM cleanup_config
            WHERE organization_id = $1
        "#;

        let result: Option<ConfigRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .ok()
            .and_then(|r: Vec<ConfigRow>| r.into_iter().next());

        match result {
            Some(row) => {
                let categories: Vec<CleanupCategory> = row
                    .enabled_categories
                    .split(',')
                    .filter_map(|s| match s.trim() {
                        "bot_history" => Some(CleanupCategory::BotHistory),
                        "chat_sessions" => Some(CleanupCategory::ChatSessions),
                        "system_logs" => Some(CleanupCategory::SystemLogs),
                        "access_logs" => Some(CleanupCategory::AccessLogs),
                        "debug_logs" => Some(CleanupCategory::DebugLogs),
                        "temp_files" => Some(CleanupCategory::TempFiles),
                        "cache_data" => Some(CleanupCategory::CacheData),
                        "analytics_raw" => Some(CleanupCategory::AnalyticsRaw),
                        _ => None,
                    })
                    .collect();

                Ok(CleanupConfig {
                    organization_id,
                    retention_days: row.retention_days,
                    enabled_categories: categories,
                    auto_cleanup_enabled: row.auto_cleanup_enabled,
                    auto_cleanup_schedule: row.auto_cleanup_schedule,
                    storage_alert_threshold_percent: row.storage_alert_threshold as u8,
                })
            }
            None => Ok(CleanupConfig {
                organization_id,
                ..Default::default()
            }),
        }
    }

    pub async fn save_cleanup_config(&self, config: &CleanupConfig) -> Result<(), CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        let categories_str: String = config
            .enabled_categories
            .iter()
            .map(|c| match c {
                CleanupCategory::BotHistory => "bot_history",
                CleanupCategory::ChatSessions => "chat_sessions",
                CleanupCategory::SystemLogs => "system_logs",
                CleanupCategory::AccessLogs => "access_logs",
                CleanupCategory::DebugLogs => "debug_logs",
                CleanupCategory::TempFiles => "temp_files",
                CleanupCategory::CacheData => "cache_data",
                CleanupCategory::AnalyticsRaw => "analytics_raw",
                CleanupCategory::OldCalendarEvents => "old_calendar_events",
                CleanupCategory::Attachments => "attachments",
            })
            .collect::<Vec<_>>()
            .join(",");

        let sql = r#"
            INSERT INTO cleanup_config (organization_id, retention_days, enabled_categories,
                                        auto_cleanup_enabled, auto_cleanup_schedule,
                                        storage_alert_threshold, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (organization_id) DO UPDATE SET
                retention_days = EXCLUDED.retention_days,
                enabled_categories = EXCLUDED.enabled_categories,
                auto_cleanup_enabled = EXCLUDED.auto_cleanup_enabled,
                auto_cleanup_schedule = EXCLUDED.auto_cleanup_schedule,
                storage_alert_threshold = EXCLUDED.storage_alert_threshold,
                updated_at = NOW()
        "#;

        diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(config.organization_id)
            .bind::<BigInt, _>(config.retention_days)
            .bind::<Text, _>(&categories_str)
            .bind::<diesel::sql_types::Bool, _>(config.auto_cleanup_enabled)
            .bind::<diesel::sql_types::Nullable<Text>, _>(config.auto_cleanup_schedule.as_deref())
            .bind::<diesel::sql_types::Integer, _>(config.storage_alert_threshold_percent as i32)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to save cleanup config: {e}");
                CleanupError::ConfigSaveFailed
            })?;

        Ok(())
    }

    async fn save_cleanup_history(&self, history: &CleanupHistory) -> Result<(), CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        let trigger_str = match history.trigger {
            CleanupTrigger::Manual => "manual",
            CleanupTrigger::Scheduled => "scheduled",
            CleanupTrigger::StorageAlert => "storage_alert",
        };

        let categories_str: String = history
            .categories_cleaned
            .iter()
            .map(|c| match c {
                CleanupCategory::BotHistory => "bot_history",
                CleanupCategory::ChatSessions => "chat_sessions",
                CleanupCategory::SystemLogs => "system_logs",
                CleanupCategory::AccessLogs => "access_logs",
                CleanupCategory::DebugLogs => "debug_logs",
                CleanupCategory::TempFiles => "temp_files",
                CleanupCategory::CacheData => "cache_data",
                CleanupCategory::AnalyticsRaw => "analytics_raw",
                CleanupCategory::OldCalendarEvents => "old_calendar_events",
                CleanupCategory::Attachments => "attachments",
            })
            .collect::<Vec<_>>()
            .join(",");

        let sql = r#"
            INSERT INTO cleanup_history (id, organization_id, performed_at, performed_by,
                                         trigger, categories_cleaned, records_deleted,
                                         bytes_freed, retention_days_used, duration_ms,
                                         success, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;

        diesel::sql_query(sql)
            .bind::<Text, _>(&history.id)
            .bind::<diesel::sql_types::Uuid, _>(history.organization_id)
            .bind::<Timestamptz, _>(history.performed_at)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(history.performed_by)
            .bind::<Text, _>(trigger_str)
            .bind::<Text, _>(&categories_str)
            .bind::<BigInt, _>(history.records_deleted)
            .bind::<BigInt, _>(history.bytes_freed)
            .bind::<BigInt, _>(history.retention_days_used)
            .bind::<BigInt, _>(history.duration_ms as i64)
            .bind::<diesel::sql_types::Bool, _>(history.success)
            .bind::<diesel::sql_types::Nullable<Text>, _>(history.error_message.as_deref())
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to save cleanup history: {e}");
                CleanupError::HistorySaveFailed
            })?;

        Ok(())
    }

    async fn get_organization_quota(&self, organization_id: Uuid) -> Result<i64, CleanupError> {
        let mut conn = self.pool.get().map_err(|_| CleanupError::DatabaseConnection)?;

        #[derive(QueryableByName)]
        struct QuotaRow {
            #[diesel(sql_type = BigInt)]
            storage_quota_bytes: i64,
        }

        let sql = "SELECT storage_quota_bytes FROM organizations WHERE id = $1";

        let result: Option<QuotaRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .ok()
            .and_then(|r: Vec<QuotaRow>| r.into_iter().next());

        result
            .map(|r| r.storage_quota_bytes)
            .ok_or(CleanupError::OrganizationNotFound)
    }

    fn validate_retention_days(&self, days: i64) -> Result<i64, CleanupError> {
        if days < MIN_RETENTION_DAYS {
            return Err(CleanupError::RetentionTooShort(MIN_RETENTION_DAYS));
        }
        if days > MAX_RETENTION_DAYS {
            return Err(CleanupError::RetentionTooLong(MAX_RETENTION_DAYS));
        }
        Ok(days)
    }
}

fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    const TB: i64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[derive(Debug, Clone)]
pub enum CleanupError {
    DatabaseConnection,
    CleanupFailed(String),
    ConfigSaveFailed,
    HistorySaveFailed,
    OrganizationNotFound,
    RetentionTooShort(i64),
    RetentionTooLong(i64),
    InvalidCategory(String),
}

impl std::fmt::Display for CleanupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::CleanupFailed(cat) => write!(f, "Cleanup failed for {cat}"),
            Self::ConfigSaveFailed => write!(f, "Failed to save configuration"),
            Self::HistorySaveFailed => write!(f, "Failed to save history"),
            Self::OrganizationNotFound => write!(f, "Organization not found"),
            Self::RetentionTooShort(min) => write!(f, "Retention must be at least {min} days"),
            Self::RetentionTooLong(max) => write!(f, "Retention cannot exceed {max} days"),
            Self::InvalidCategory(cat) => write!(f, "Invalid category: {cat}"),
        }
    }
}

impl std::error::Error for CleanupError {}

impl IntoResponse for CleanupError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::OrganizationNotFound => StatusCode::NOT_FOUND,
            Self::RetentionTooShort(_) | Self::RetentionTooLong(_) | Self::InvalidCategory(_) => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn create_cleanup_tables_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS cleanup_config (
        organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
        retention_days BIGINT NOT NULL DEFAULT 180,
        enabled_categories TEXT NOT NULL DEFAULT 'bot_history,chat_sessions,system_logs,debug_logs,temp_files,cache_data',
        auto_cleanup_enabled BOOLEAN NOT NULL DEFAULT FALSE,
        auto_cleanup_schedule TEXT,
        storage_alert_threshold INTEGER NOT NULL DEFAULT 80,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS cleanup_history (
        id TEXT PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
        performed_at TIMESTAMPTZ NOT NULL,
        performed_by UUID REFERENCES users(id),
        trigger TEXT NOT NULL,
        categories_cleaned TEXT NOT NULL,
        records_deleted BIGINT NOT NULL,
        bytes_freed BIGINT NOT NULL,
        retention_days_used BIGINT NOT NULL,
        duration_ms BIGINT NOT NULL,
        success BOOLEAN NOT NULL,
        error_message TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_cleanup_history_org ON cleanup_history(organization_id);
    CREATE INDEX IF NOT EXISTS idx_cleanup_history_performed ON cleanup_history(performed_at DESC);
    "#
}

pub fn cleanup_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/preview", get(preview_cleanup_handler))
        .route("/execute", post(execute_cleanup_handler))
        .route("/storage", get(storage_usage_handler))
        .route("/history", get(cleanup_history_handler))
        .route("/config", get(get_config_handler))
        .route("/config", post(save_config_handler))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct PreviewQuery {
    organization_id: Uuid,
    retention_days: Option<i64>,
    categories: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExecuteRequest {
    organization_id: Uuid,
    retention_days: Option<i64>,
    categories: Option<Vec<String>>,
    performed_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct StorageQuery {
    organization_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    organization_id: Uuid,
    limit: Option<i32>,
}

async fn preview_cleanup_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PreviewQuery>,
) -> Result<Json<CleanupPreview>, CleanupError> {
    let service = CleanupService::new(Arc::new(state.conn.clone()));

    let categories = query.categories.map(|s| {
        s.split(',')
            .filter_map(|c| match c.trim() {
                "bot_history" => Some(CleanupCategory::BotHistory),
                "chat_sessions" => Some(CleanupCategory::ChatSessions),
                "system_logs" => Some(CleanupCategory::SystemLogs),
                "access_logs" => Some(CleanupCategory::AccessLogs),
                "debug_logs" => Some(CleanupCategory::DebugLogs),
                "temp_files" => Some(CleanupCategory::TempFiles),
                "cache_data" => Some(CleanupCategory::CacheData),
                "analytics_raw" => Some(CleanupCategory::AnalyticsRaw),
                _ => None,
            })
            .collect()
    });

    let preview = service
        .preview_cleanup(query.organization_id, query.retention_days, categories)
        .await?;

    Ok(Json(preview))
}

async fn execute_cleanup_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecuteRequest>,
) -> Result<Json<CleanupResult>, CleanupError> {
    let service = CleanupService::new(Arc::new(state.conn.clone()));

    let categories = request.categories.map(|cats| {
        cats.iter()
            .filter_map(|c| match c.as_str() {
                "bot_history" => Some(CleanupCategory::BotHistory),
                "chat_sessions" => Some(CleanupCategory::ChatSessions),
                "system_logs" => Some(CleanupCategory::SystemLogs),
                "access_logs" => Some(CleanupCategory::AccessLogs),
                "debug_logs" => Some(CleanupCategory::DebugLogs),
                "temp_files" => Some(CleanupCategory::TempFiles),
                "cache_data" => Some(CleanupCategory::CacheData),
                "analytics_raw" => Some(CleanupCategory::AnalyticsRaw),
                _ => None,
            })
            .collect()
    });

    let result = service
        .execute_cleanup(
            request.organization_id,
            request.retention_days,
            categories,
            request.performed_by,
            CleanupTrigger::Manual,
        )
        .await?;

    Ok(Json(result))
}

async fn storage_usage_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StorageQuery>,
) -> Result<Json<StorageUsage>, CleanupError> {
    let service = CleanupService::new(Arc::new(state.conn.clone()));
    let usage = service.get_storage_usage(query.organization_id).await?;
    Ok(Json(usage))
}

async fn cleanup_history_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<CleanupHistory>>, CleanupError> {
    let service = CleanupService::new(Arc::new(state.conn.clone()));
    let history = service
        .get_cleanup_history(query.organization_id, query.limit)
        .await?;
    Ok(Json(history))
}

async fn get_config_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StorageQuery>,
) -> Result<Json<CleanupConfig>, CleanupError> {
    let service = CleanupService::new(Arc::new(state.conn.clone()));
    let config = service.get_cleanup_config(query.organization_id).await?;
    Ok(Json(config))
}

async fn save_config_handler(
    State(state): State<Arc<AppState>>,
    Json(config): Json<CleanupConfig>,
) -> Result<StatusCode, CleanupError> {
    let service = CleanupService::new(Arc::new(state.conn.clone()));
    service.save_cleanup_config(&config).await?;
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_category_display() {
        assert_eq!(CleanupCategory::BotHistory.to_string(), "Bot History");
        assert_eq!(CleanupCategory::ChatSessions.to_string(), "Chat Sessions");
        assert_eq!(CleanupCategory::SystemLogs.to_string(), "System Logs");
    }

    #[test]
    fn test_cleanup_category_all() {
        let all = CleanupCategory::all();
        assert!(all.len() >= 8);
        assert!(all.contains(&CleanupCategory::BotHistory));
        assert!(all.contains(&CleanupCategory::CacheData));
    }

    #[test]
    fn test_cleanup_category_default_enabled() {
        let defaults = CleanupCategory::default_enabled();
        assert!(defaults.contains(&CleanupCategory::BotHistory));
        assert!(defaults.contains(&CleanupCategory::TempFiles));
        assert!(!defaults.contains(&CleanupCategory::AnalyticsRaw));
    }

    #[test]
    fn test_cleanup_config_default() {
        let config = CleanupConfig::default();
        assert_eq!(config.retention_days, DEFAULT_RETENTION_DAYS);
        assert!(!config.auto_cleanup_enabled);
        assert_eq!(config.storage_alert_threshold_percent, 80);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn test_cleanup_error_display() {
        let err = CleanupError::DatabaseConnection;
        assert_eq!(err.to_string(), "Database connection failed");

        let err = CleanupError::RetentionTooShort(7);
        assert_eq!(err.to_string(), "Retention must be at least 7 days");
    }

    #[test]
    fn test_cleanup_trigger_display() {
        assert_eq!(CleanupTrigger::Manual.to_string(), "Manual");
        assert_eq!(CleanupTrigger::Scheduled.to_string(), "Scheduled");
        assert_eq!(CleanupTrigger::StorageAlert.to_string(), "Storage Alert");
    }

    #[test]
    fn test_category_is_essential() {
        assert!(!CleanupCategory::BotHistory.is_essential());
        assert!(!CleanupCategory::TempFiles.is_essential());
        assert!(CleanupCategory::OldCalendarEvents.is_essential());
        assert!(CleanupCategory::Attachments.is_essential());
    }

    #[test]
    fn test_category_preview_creation() {
        let preview = CategoryPreview {
            category: CleanupCategory::BotHistory,
            display_name: "Bot History".to_string(),
            record_count: 1000,
            estimated_bytes: 1024 * 1024,
            estimated_bytes_formatted: "1.00 MB".to_string(),
            oldest_record: Some(Utc::now()),
            is_essential: false,
        };
        assert_eq!(preview.record_count, 1000);
        assert!(!preview.is_essential);
    }
}
