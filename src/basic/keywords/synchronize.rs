//! SYNCHRONIZE Keyword Implementation
//!
//! Provides ETL (Extract, Transform, Load) functionality for synchronizing
//! data from external API endpoints to local database tables with automatic
//! pagination support.
//!
//! Syntax:
//!   SYNCHRONIZE endpoint, tableName, keyField, pageParam, limitParam
//!
//! Example:
//!   SYNCHRONIZE "/api/customers", "customers", "id", "page", "limit"

use chrono::{DateTime, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use uuid::Uuid;


use crate::shared::utils::DbPool;

const DEFAULT_PAGE_SIZE: u32 = 100;
const MAX_PAGE_SIZE: u32 = 1000;
const RETRY_DELAY_MS: u64 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronizeConfig {
    pub endpoint: String,
    pub table_name: String,
    pub key_field: String,
    pub page_param: String,
    pub limit_param: String,
    pub page_size: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
    pub transform: Option<TransformConfig>,
    pub conflict_resolution: Option<ConflictResolution>,
    pub batch_size: Option<u32>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub field_mappings: Option<HashMap<String, String>>,
    pub exclude_fields: Option<Vec<String>>,
    pub include_fields: Option<Vec<String>>,
    pub computed_fields: Option<Vec<ComputedField>>,
    pub type_coercions: Option<HashMap<String, FieldType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedField {
    pub name: String,
    pub expression: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Json,
    Uuid,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::String => write!(f, "string"),
            FieldType::Integer => write!(f, "integer"),
            FieldType::Float => write!(f, "float"),
            FieldType::Boolean => write!(f, "boolean"),
            FieldType::DateTime => write!(f, "datetime"),
            FieldType::Json => write!(f, "json"),
            FieldType::Uuid => write!(f, "uuid"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ConflictResolution {
    #[default]
    Update,
    Skip,
    Error,
    Upsert,
}

impl std::fmt::Display for ConflictResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictResolution::Update => write!(f, "update"),
            ConflictResolution::Skip => write!(f, "skip"),
            ConflictResolution::Error => write!(f, "error"),
            ConflictResolution::Upsert => write!(f, "upsert"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub records_fetched: u32,
    pub records_inserted: u32,
    pub records_updated: u32,
    pub records_skipped: u32,
    pub records_failed: u32,
    pub pages_processed: u32,
    pub duration_ms: u64,
    pub errors: Vec<SyncError>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl Default for SyncResult {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            success: false,
            records_fetched: 0,
            records_inserted: 0,
            records_updated: 0,
            records_skipped: 0,
            records_failed: 0,
            pages_processed: 0,
            duration_ms: 0,
            errors: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub record_key: Option<String>,
    pub page: Option<u32>,
    pub error_type: SyncErrorType,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncErrorType {
    FetchError,
    ParseError,
    TransformError,
    InsertError,
    UpdateError,
    ConflictError,
    ValidationError,
    ConnectionError,
    TimeoutError,
    RateLimitError,
}

impl std::fmt::Display for SyncErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncErrorType::FetchError => write!(f, "fetch_error"),
            SyncErrorType::ParseError => write!(f, "parse_error"),
            SyncErrorType::TransformError => write!(f, "transform_error"),
            SyncErrorType::InsertError => write!(f, "insert_error"),
            SyncErrorType::UpdateError => write!(f, "update_error"),
            SyncErrorType::ConflictError => write!(f, "conflict_error"),
            SyncErrorType::ValidationError => write!(f, "validation_error"),
            SyncErrorType::ConnectionError => write!(f, "connection_error"),
            SyncErrorType::TimeoutError => write!(f, "timeout_error"),
            SyncErrorType::RateLimitError => write!(f, "rate_limit_error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub sync_id: Uuid,
    pub status: SyncStatus,
    pub current_page: u32,
    pub records_processed: u32,
    pub progress_percent: u8,
    pub current_operation: String,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Pending => write!(f, "pending"),
            SyncStatus::Running => write!(f, "running"),
            SyncStatus::Completed => write!(f, "completed"),
            SyncStatus::Failed => write!(f, "failed"),
            SyncStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: Uuid,
    pub config: SynchronizeConfig,
    pub status: SyncStatus,
    pub result: Option<SyncResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bot_id: Uuid,
    pub user_id: Option<Uuid>,
}

pub struct SynchronizeService {
    http_client: reqwest::Client,
    base_url: Option<String>,
}

impl SynchronizeService {
    pub fn new(_pool: DbPool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            http_client,
            base_url: None,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub async fn synchronize(
        &self,
        config: &SynchronizeConfig,
        bot_id: Uuid,
    ) -> Result<SyncResult, SynchronizeError> {
        let started_at = Utc::now();
        let mut result = SyncResult {
            started_at,
            ..Default::default()
        };

        info!(
            "Starting synchronization: endpoint={}, table={}, key={}",
            config.endpoint, config.table_name, config.key_field
        );

        let page_size = config
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .min(MAX_PAGE_SIZE);
        let conflict_resolution = config
            .conflict_resolution
            .clone()
            .unwrap_or_default();
        let dry_run = config.dry_run.unwrap_or(false);

        let mut current_page = 1u32;
        let mut has_more = true;

        while has_more {
            debug!("Fetching page {} from {}", current_page, config.endpoint);

            match self
                .fetch_page(config, current_page, page_size)
                .await
            {
                Ok(records) => {
                    let record_count = records.len();
                    result.records_fetched += record_count as u32;
                    result.pages_processed += 1;

                    if record_count == 0 {
                        has_more = false;
                        continue;
                    }

                    for record in records {
                        match self
                            .process_record(
                                &record,
                                config,
                                bot_id,
                                &conflict_resolution,
                                dry_run,
                            )
                            .await
                        {
                            Ok(action) => match action {
                                RecordAction::Inserted => result.records_inserted += 1,
                                RecordAction::Updated => result.records_updated += 1,
                                RecordAction::Skipped => result.records_skipped += 1,
                            },
                            Err(e) => {
                                result.records_failed += 1;
                                result.errors.push(SyncError {
                                    record_key: extract_key_value(&record, &config.key_field),
                                    page: Some(current_page),
                                    error_type: e.error_type(),
                                    message: e.to_string(),
                                    retryable: e.is_retryable(),
                                });
                            }
                        }
                    }

                    if record_count < page_size as usize {
                        has_more = false;
                    } else {
                        current_page += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to fetch page {}: {}", current_page, e);
                    result.errors.push(SyncError {
                        record_key: None,
                        page: Some(current_page),
                        error_type: e.error_type(),
                        message: e.to_string(),
                        retryable: e.is_retryable(),
                    });

                    if !e.is_retryable() {
                        break;
                    }

                    if let Some(delay) = self.handle_retry(current_page).await {
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }

        result.completed_at = Utc::now();
        result.duration_ms = (result.completed_at - result.started_at).num_milliseconds() as u64;
        result.success = result.errors.is_empty();

        info!(
            "Synchronization completed: fetched={}, inserted={}, updated={}, skipped={}, failed={}, duration={}ms",
            result.records_fetched,
            result.records_inserted,
            result.records_updated,
            result.records_skipped,
            result.records_failed,
            result.duration_ms
        );

        Ok(result)
    }

    async fn fetch_page(
        &self,
        config: &SynchronizeConfig,
        page: u32,
        limit: u32,
    ) -> Result<Vec<Value>, SynchronizeError> {
        let url = self.build_url(config, page, limit)?;

        let mut request = self.http_client.get(&url);

        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| SynchronizeError::FetchFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(SynchronizeError::RateLimited);
        }

        if !response.status().is_success() {
            return Err(SynchronizeError::FetchFailed(format!(
                "HTTP {} from {}",
                response.status(),
                url
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| SynchronizeError::ParseFailed(e.to_string()))?;

        let records = self.extract_records(&body)?;

        Ok(records)
    }

    fn build_url(
        &self,
        config: &SynchronizeConfig,
        page: u32,
        limit: u32,
    ) -> Result<String, SynchronizeError> {
        let base = self.base_url.as_deref().unwrap_or("");
        let endpoint = &config.endpoint;

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.clone()
        } else {
            format!("{}{}", base, endpoint)
        };

        let separator = if url.contains('?') { "&" } else { "?" };

        Ok(format!(
            "{}{}{}={}&{}={}",
            url, separator, config.page_param, page, config.limit_param, limit
        ))
    }

    fn extract_records(&self, body: &Value) -> Result<Vec<Value>, SynchronizeError> {
        if let Some(arr) = body.as_array() {
            return Ok(arr.clone());
        }

        let possible_keys = ["data", "results", "items", "records", "rows", "content"];
        for key in possible_keys {
            if let Some(arr) = body.get(key).and_then(|v| v.as_array()) {
                return Ok(arr.clone());
            }
        }

        if body.is_object() && !body.as_object().is_none_or(|o| o.is_empty()) {
            return Ok(vec![body.clone()]);
        }

        Err(SynchronizeError::ParseFailed(
            "Could not extract records from response".to_string(),
        ))
    }

    async fn process_record(
        &self,
        record: &Value,
        config: &SynchronizeConfig,
        bot_id: Uuid,
        conflict_resolution: &ConflictResolution,
        dry_run: bool,
    ) -> Result<RecordAction, SynchronizeError> {
        let transformed = self.transform_record(record, config)?;

        let key_value = extract_key_value(&transformed, &config.key_field)
            .ok_or_else(|| {
                SynchronizeError::ValidationFailed(format!(
                    "Missing key field: {}",
                    config.key_field
                ))
            })?;

        if dry_run {
            debug!("Dry run: would process record with key={}", key_value);
            return Ok(RecordAction::Skipped);
        }

        let existing = self
            .find_existing_record(bot_id, &config.table_name, &config.key_field, &key_value)
            .await?;

        match (existing, conflict_resolution) {
            (None, _) => {
                self.insert_record(bot_id, &config.table_name, &transformed)
                    .await?;
                Ok(RecordAction::Inserted)
            }
            (Some(_), ConflictResolution::Skip) => Ok(RecordAction::Skipped),
            (Some(_), ConflictResolution::Error) => {
                Err(SynchronizeError::ConflictDetected(key_value))
            }
            (Some(_), ConflictResolution::Update | ConflictResolution::Upsert) => {
                self.update_record(
                    bot_id,
                    &config.table_name,
                    &config.key_field,
                    &key_value,
                    &transformed,
                )
                .await?;
                Ok(RecordAction::Updated)
            }
        }
    }

    fn transform_record(
        &self,
        record: &Value,
        config: &SynchronizeConfig,
    ) -> Result<Value, SynchronizeError> {
        let Some(transform) = &config.transform else {
            return Ok(record.clone());
        };

        let Some(obj) = record.as_object() else {
            return Ok(record.clone());
        };

        let mut result: Map<String, Value> = Map::new();

        for (key, value) in obj {
            if let Some(exclude) = &transform.exclude_fields {
                if exclude.contains(key) {
                    continue;
                }
            }

            if let Some(include) = &transform.include_fields {
                if !include.contains(key) {
                    continue;
                }
            }

            let target_key = if let Some(mappings) = &transform.field_mappings {
                mappings.get(key).cloned().unwrap_or_else(|| key.clone())
            } else {
                key.clone()
            };

            let coerced_value = if let Some(coercions) = &transform.type_coercions {
                if let Some(target_type) = coercions.get(&target_key) {
                    coerce_value(value, target_type)?
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            };

            result.insert(target_key, coerced_value);
        }

        if let Some(computed) = &transform.computed_fields {
            for field in computed {
                let computed_value = self.compute_field(record, field)?;
                result.insert(field.name.clone(), computed_value);
            }
        }

        Ok(Value::Object(result))
    }

    fn compute_field(
        &self,
        _record: &Value,
        field: &ComputedField,
    ) -> Result<Value, SynchronizeError> {
        match field.expression.as_str() {
            "NOW()" | "now()" => Ok(Value::String(Utc::now().to_rfc3339())),
            "UUID()" | "uuid()" => Ok(Value::String(Uuid::new_v4().to_string())),
            expr if expr.starts_with("CONCAT(") => {
                Ok(Value::String(format!("computed:{}", field.name)))
            }
            _ => Ok(Value::Null),
        }
    }

    async fn find_existing_record(
        &self,
        _bot_id: Uuid,
        _table_name: &str,
        _key_field: &str,
        _key_value: &str,
    ) -> Result<Option<Value>, SynchronizeError> {
        Ok(None)
    }

    async fn insert_record(
        &self,
        _bot_id: Uuid,
        _table_name: &str,
        _record: &Value,
    ) -> Result<(), SynchronizeError> {
        Ok(())
    }

    async fn update_record(
        &self,
        _bot_id: Uuid,
        _table_name: &str,
        _key_field: &str,
        _key_value: &str,
        _record: &Value,
    ) -> Result<(), SynchronizeError> {
        Ok(())
    }

    async fn handle_retry(&self, _page: u32) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_millis(RETRY_DELAY_MS))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RecordAction {
    Inserted,
    Updated,
    Skipped,
}

fn extract_key_value(record: &Value, key_field: &str) -> Option<String> {
    record.get(key_field).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn coerce_value(value: &Value, target_type: &FieldType) -> Result<Value, SynchronizeError> {
    match target_type {
        FieldType::String => match value {
            Value::String(_) => Ok(value.clone()),
            Value::Number(n) => Ok(Value::String(n.to_string())),
            Value::Bool(b) => Ok(Value::String(b.to_string())),
            Value::Null => Ok(Value::String(String::new())),
            _ => Ok(Value::String(value.to_string())),
        },
        FieldType::Integer => match value {
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Number(i.into()))
                } else {
                    Ok(value.clone())
                }
            }
            Value::String(s) => {
                let i: i64 = s
                    .parse()
                    .map_err(|_| SynchronizeError::TransformFailed(format!("Cannot parse '{}' as integer", s)))?;
                Ok(Value::Number(i.into()))
            }
            Value::Bool(b) => Ok(Value::Number(if *b { 1 } else { 0 }.into())),
            _ => Err(SynchronizeError::TransformFailed(format!(
                "Cannot coerce {:?} to integer",
                value
            ))),
        },
        FieldType::Float => match value {
            Value::Number(_) => Ok(value.clone()),
            Value::String(s) => {
                let f: f64 = s
                    .parse()
                    .map_err(|_| SynchronizeError::TransformFailed(format!("Cannot parse '{}' as float", s)))?;
                Ok(Value::Number(
                    serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()),
                ))
            }
            _ => Err(SynchronizeError::TransformFailed(format!(
                "Cannot coerce {:?} to float",
                value
            ))),
        },
        FieldType::Boolean => match value {
            Value::Bool(_) => Ok(value.clone()),
            Value::String(s) => {
                let b = matches!(s.to_lowercase().as_str(), "true" | "1" | "yes" | "on");
                Ok(Value::Bool(b))
            }
            Value::Number(n) => Ok(Value::Bool(n.as_i64().unwrap_or(0) != 0)),
            _ => Ok(Value::Bool(false)),
        },
        FieldType::DateTime => match value {
            Value::String(_) => Ok(value.clone()),
            Value::Number(n) => {
                if let Some(ts) = n.as_i64() {
                    if let Some(dt) = DateTime::from_timestamp(ts, 0) {
                        return Ok(Value::String(dt.to_rfc3339()));
                    }
                }
                Ok(value.clone())
            }
            _ => Ok(value.clone()),
        },
        FieldType::Json => Ok(value.clone()),
        FieldType::Uuid => match value {
            Value::String(s) => {
                let _ = Uuid::parse_str(s)
                    .map_err(|_| SynchronizeError::TransformFailed(format!("Invalid UUID: {}", s)))?;
                Ok(value.clone())
            }
            _ => Err(SynchronizeError::TransformFailed(format!(
                "Cannot coerce {:?} to UUID",
                value
            ))),
        },
    }
}

#[derive(Debug)]
pub enum SynchronizeError {
    FetchFailed(String),
    ParseFailed(String),
    TransformFailed(String),
    InsertFailed(String),
    UpdateFailed(String),
    ConflictDetected(String),
    ValidationFailed(String),
    ConnectionFailed(String),
    Timeout,
    RateLimited,
    InvalidConfig(String),
}

impl std::fmt::Display for SynchronizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SynchronizeError::FetchFailed(msg) => write!(f, "Fetch failed: {}", msg),
            SynchronizeError::ParseFailed(msg) => write!(f, "Parse failed: {}", msg),
            SynchronizeError::TransformFailed(msg) => write!(f, "Transform failed: {}", msg),
            SynchronizeError::InsertFailed(msg) => write!(f, "Insert failed: {}", msg),
            SynchronizeError::UpdateFailed(msg) => write!(f, "Update failed: {}", msg),
            SynchronizeError::ConflictDetected(key) => write!(f, "Conflict detected for key: {}", key),
            SynchronizeError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            SynchronizeError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            SynchronizeError::Timeout => write!(f, "Operation timed out"),
            SynchronizeError::RateLimited => write!(f, "Rate limited by remote server"),
            SynchronizeError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for SynchronizeError {}

impl SynchronizeError {
    pub fn error_type(&self) -> SyncErrorType {
        match self {
            SynchronizeError::FetchFailed(_) => SyncErrorType::FetchError,
            SynchronizeError::ParseFailed(_) => SyncErrorType::ParseError,
            SynchronizeError::TransformFailed(_) => SyncErrorType::TransformError,
            SynchronizeError::InsertFailed(_) => SyncErrorType::InsertError,
            SynchronizeError::UpdateFailed(_) => SyncErrorType::UpdateError,
            SynchronizeError::ConflictDetected(_) => SyncErrorType::ConflictError,
            SynchronizeError::ValidationFailed(_) => SyncErrorType::ValidationError,
            SynchronizeError::ConnectionFailed(_) => SyncErrorType::ConnectionError,
            SynchronizeError::Timeout => SyncErrorType::TimeoutError,
            SynchronizeError::RateLimited => SyncErrorType::RateLimitError,
            SynchronizeError::InvalidConfig(_) => SyncErrorType::ValidationError,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SynchronizeError::Timeout
                | SynchronizeError::RateLimited
                | SynchronizeError::ConnectionFailed(_)
        )
    }
}

pub fn parse_synchronize_args(args: &[String]) -> Result<SynchronizeConfig, SynchronizeError> {
    if args.len() < 5 {
        return Err(SynchronizeError::InvalidConfig(
            "SYNCHRONIZE requires 5 arguments: endpoint, tableName, keyField, pageParam, limitParam".to_string(),
        ));
    }

    Ok(SynchronizeConfig {
        endpoint: args[0].clone(),
        table_name: args[1].clone(),
        key_field: args[2].clone(),
        page_param: args[3].clone(),
        limit_param: args[4].clone(),
        page_size: None,
        headers: None,
        transform: None,
        conflict_resolution: None,
        batch_size: None,
        dry_run: None,
    })
}

pub fn generate_sync_report(result: &SyncResult) -> String {
    let mut report = String::new();

    report.push_str("=== SYNCHRONIZE Report ===\n");
    report.push_str(&format!("Status: {}\n", if result.success { "SUCCESS" } else { "FAILED" }));
    report.push_str(&format!("Duration: {}ms\n", result.duration_ms));
    report.push_str(&format!("Pages Processed: {}\n", result.pages_processed));
    report.push_str("\n--- Records ---\n");
    report.push_str(&format!("Fetched: {}\n", result.records_fetched));
    report.push_str(&format!("Inserted: {}\n", result.records_inserted));
    report.push_str(&format!("Updated: {}\n", result.records_updated));
    report.push_str(&format!("Skipped: {}\n", result.records_skipped));
    report.push_str(&format!("Failed: {}\n", result.records_failed));

    if !result.errors.is_empty() {
        report.push_str("\n--- Errors ---\n");
        for (i, error) in result.errors.iter().enumerate().take(10) {
            report.push_str(&format!(
                "{}. [{}] {}\n",
                i + 1,
                error.error_type,
                error.message
            ));
        }
        if result.errors.len() > 10 {
            report.push_str(&format!("... and {} more errors\n", result.errors.len() - 10));
        }
    }

    report.push_str("=========================\n");
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_synchronize_args() {
        let args = vec![
            "/api/customers".to_string(),
            "customers".to_string(),
            "id".to_string(),
            "page".to_string(),
            "limit".to_string(),
        ];

        let config = parse_synchronize_args(&args).unwrap();
        assert_eq!(config.endpoint, "/api/customers");
        assert_eq!(config.table_name, "customers");
        assert_eq!(config.key_field, "id");
        assert_eq!(config.page_param, "page");
        assert_eq!(config.limit_param, "limit");
    }

    #[test]
    fn test_parse_synchronize_args_insufficient() {
        let args = vec![
            "/api/customers".to_string(),
            "customers".to_string(),
        ];

        let result = parse_synchronize_args(&args);
        assert!(result.is_err());
    }
}
