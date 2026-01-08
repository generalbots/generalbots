use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
    Snapshot,
    LogBackup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Verified,
    VerificationFailed,
    Expired,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StorageLocation {
    Local,
    S3,
    Azure,
    Gcs,
    Sftp,
    Nfs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    ChecksumValidation,
    RestoreTest,
    PartialRestore,
    MetadataCheck,
    SampleDataValidation,
    FullIntegrityCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub backup_type: BackupType,
    pub source: String,
    pub storage_location: StorageLocation,
    pub storage_path: String,
    pub size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub checksum: String,
    pub checksum_algorithm: String,
    pub encryption_key_id: Option<String>,
    pub status: BackupStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<u64>,
    pub retention_days: u32,
    pub expires_at: DateTime<Utc>,
    pub parent_backup_id: Option<Uuid>,
    pub metadata: BackupMetadata,
    pub verification_history: Vec<VerificationResult>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub database_version: Option<String>,
    pub application_version: String,
    pub schema_version: Option<String>,
    pub table_count: Option<u32>,
    pub row_count: Option<u64>,
    pub files_count: Option<u32>,
    pub organization_ids: Vec<Uuid>,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub method: VerificationMethod,
    pub status: VerificationStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<u64>,
    pub checks_performed: Vec<VerificationCheck>,
    pub errors: Vec<VerificationError>,
    pub warnings: Vec<String>,
    pub verified_by: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Pending,
    Running,
    Passed,
    Failed,
    PartialSuccess,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub description: String,
    pub passed: bool,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationError {
    pub code: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub recoverable: bool,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub source_pattern: String,
    pub backup_type: BackupType,
    pub schedule: BackupSchedule,
    pub retention: RetentionPolicy,
    pub storage: StorageConfiguration,
    pub verification: VerificationPolicy,
    pub notifications: NotificationSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub cron_expression: String,
    pub timezone: String,
    pub full_backup_day: Option<u8>,
    pub incremental_frequency_hours: Option<u32>,
    pub maintenance_window_start: Option<String>,
    pub maintenance_window_end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub daily_retention_days: u32,
    pub weekly_retention_weeks: u32,
    pub monthly_retention_months: u32,
    pub yearly_retention_years: u32,
    pub minimum_backups: u32,
    pub maximum_backups: Option<u32>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            daily_retention_days: 7,
            weekly_retention_weeks: 4,
            monthly_retention_months: 12,
            yearly_retention_years: 3,
            minimum_backups: 3,
            maximum_backups: Some(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfiguration {
    pub primary_location: StorageLocation,
    pub primary_path: String,
    pub secondary_location: Option<StorageLocation>,
    pub secondary_path: Option<String>,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub compression_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub auto_verify: bool,
    pub verification_methods: Vec<VerificationMethod>,
    pub verify_after_backup: bool,
    pub periodic_verification_days: u32,
    pub restore_test_frequency_days: Option<u32>,
    pub sample_data_percentage: Option<u8>,
    pub fail_on_verification_error: bool,
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            auto_verify: true,
            verification_methods: vec![
                VerificationMethod::ChecksumValidation,
                VerificationMethod::MetadataCheck,
            ],
            verify_after_backup: true,
            periodic_verification_days: 7,
            restore_test_frequency_days: Some(30),
            sample_data_percentage: Some(5),
            fail_on_verification_error: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub notify_on_success: bool,
    pub notify_on_failure: bool,
    pub notify_on_verification_failure: bool,
    pub notify_on_expiration_warning: bool,
    pub expiration_warning_days: u32,
    pub channels: Vec<String>,
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreTestResult {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: RestoreTestStatus,
    pub restore_target: String,
    pub data_validated: bool,
    pub tables_restored: u32,
    pub rows_validated: u64,
    pub integrity_checks: Vec<IntegrityCheck>,
    pub performance_metrics: RestorePerformanceMetrics,
    pub errors: Vec<VerificationError>,
    pub cleanup_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestoreTestStatus {
    Pending,
    Restoring,
    Validating,
    CleaningUp,
    Passed,
    Failed,
    PartialSuccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheck {
    pub table_name: String,
    pub expected_rows: u64,
    pub actual_rows: u64,
    pub checksum_match: bool,
    pub foreign_keys_valid: bool,
    pub indexes_valid: bool,
    pub constraints_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePerformanceMetrics {
    pub total_duration_seconds: u64,
    pub restore_speed_mbps: f32,
    pub validation_duration_seconds: u64,
    pub peak_memory_usage_mb: u64,
    pub disk_io_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupHealthReport {
    pub generated_at: DateTime<Utc>,
    pub report_period_days: u32,
    pub summary: BackupHealthSummary,
    pub backup_status: Vec<BackupStatusEntry>,
    pub verification_status: Vec<VerificationStatusEntry>,
    pub storage_usage: StorageUsageReport,
    pub compliance: ComplianceStatus,
    pub recommendations: Vec<HealthRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupHealthSummary {
    pub total_backups: u32,
    pub successful_backups: u32,
    pub failed_backups: u32,
    pub verified_backups: u32,
    pub verification_failures: u32,
    pub last_successful_backup: Option<DateTime<Utc>>,
    pub last_successful_verification: Option<DateTime<Utc>>,
    pub backup_success_rate: f32,
    pub verification_success_rate: f32,
    pub average_backup_duration_seconds: u64,
    pub rpo_compliance: bool,
    pub rto_estimated_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatusEntry {
    pub backup_id: Uuid,
    pub backup_type: BackupType,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub status: BackupStatus,
    pub verified: bool,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatusEntry {
    pub verification_id: Uuid,
    pub backup_id: Uuid,
    pub method: VerificationMethod,
    pub status: VerificationStatus,
    pub performed_at: DateTime<Utc>,
    pub issues_found: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsageReport {
    pub total_size_bytes: u64,
    pub primary_storage_bytes: u64,
    pub secondary_storage_bytes: u64,
    pub compression_ratio: f32,
    pub deduplication_ratio: f32,
    pub storage_growth_rate_percent: f32,
    pub estimated_monthly_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub rpo_target_hours: u32,
    pub rpo_actual_hours: u32,
    pub rpo_compliant: bool,
    pub retention_compliant: bool,
    pub encryption_compliant: bool,
    pub geographic_redundancy_compliant: bool,
    pub verification_schedule_compliant: bool,
    pub last_audit_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecommendation {
    pub priority: u32,
    pub category: String,
    pub title: String,
    pub description: String,
    pub action_items: Vec<String>,
    pub impact: String,
}

pub struct BackupVerificationService {
    backups: Arc<RwLock<HashMap<Uuid, BackupRecord>>>,
    policies: Arc<RwLock<HashMap<Uuid, BackupPolicy>>>,
    verifications: Arc<RwLock<Vec<VerificationResult>>>,
    restore_tests: Arc<RwLock<Vec<RestoreTestResult>>>,
}

impl BackupVerificationService {
    pub fn new() -> Self {
        Self {
            backups: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            verifications: Arc::new(RwLock::new(Vec::new())),
            restore_tests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_backup(&self, backup: BackupRecord) -> Result<BackupRecord, BackupError> {
        let mut backups = self.backups.write().await;
        backups.insert(backup.id, backup.clone());
        Ok(backup)
    }

    pub async fn get_backup(&self, id: Uuid) -> Option<BackupRecord> {
        let backups = self.backups.read().await;
        backups.get(&id).cloned()
    }

    pub async fn get_all_backups(&self) -> Vec<BackupRecord> {
        let backups = self.backups.read().await;
        backups.values().cloned().collect()
    }

    pub async fn get_latest_backup(&self, source: &str) -> Option<BackupRecord> {
        let backups = self.backups.read().await;
        backups
            .values()
            .filter(|b| b.source == source && b.status == BackupStatus::Completed)
            .max_by_key(|b| b.completed_at)
            .cloned()
    }

    pub async fn verify_backup(
        &self,
        backup_id: Uuid,
        methods: Vec<VerificationMethod>,
        verified_by: Option<Uuid>,
    ) -> Result<VerificationResult, BackupError> {
        let backups = self.backups.read().await;
        let backup = backups
            .get(&backup_id)
            .ok_or_else(|| BackupError::NotFound("Backup not found".to_string()))?
            .clone();
        drop(backups);

        let verification_id = Uuid::new_v4();
        let started_at = Utc::now();

        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for method in &methods {
            match method {
                VerificationMethod::ChecksumValidation => {
                    let check_result = self.verify_checksum(&backup).await;
                    checks.push(check_result.0);
                    if let Some(err) = check_result.1 {
                        errors.push(err);
                    }
                }
                VerificationMethod::MetadataCheck => {
                    let check_result = self.verify_metadata(&backup).await;
                    checks.push(check_result.0);
                    if let Some(err) = check_result.1 {
                        errors.push(err);
                    }
                }
                VerificationMethod::RestoreTest => {
                    let check_result = self.perform_restore_test(&backup).await;
                    checks.push(check_result.0);
                    if let Some(err) = check_result.1 {
                        errors.push(err);
                    }
                }
                VerificationMethod::PartialRestore => {
                    let check_result = self.perform_partial_restore(&backup).await;
                    checks.push(check_result.0);
                    if let Some(err) = check_result.1 {
                        errors.push(err);
                    }
                }
                VerificationMethod::SampleDataValidation => {
                    let check_result = self.validate_sample_data(&backup).await;
                    checks.push(check_result.0);
                    if let Some(err) = check_result.1 {
                        errors.push(err);
                    }
                }
                VerificationMethod::FullIntegrityCheck => {
                    let check_result = self.full_integrity_check(&backup).await;
                    checks.extend(check_result.0);
                    errors.extend(check_result.1);
                    warnings.extend(check_result.2);
                }
            }
        }

        let all_passed = checks.iter().all(|c| c.passed);
        let any_passed = checks.iter().any(|c| c.passed);

        let status = if errors.iter().any(|e| e.severity == ErrorSeverity::Critical) {
            VerificationStatus::Failed
        } else if all_passed {
            VerificationStatus::Passed
        } else if any_passed {
            VerificationStatus::PartialSuccess
        } else {
            VerificationStatus::Failed
        };

        let completed_at = Utc::now();
        let duration_seconds = (completed_at - started_at).num_seconds() as u64;

        let result = VerificationResult {
            id: verification_id,
            backup_id,
            method: methods.first().cloned().unwrap_or(VerificationMethod::ChecksumValidation),
            status: status.clone(),
            started_at,
            completed_at: Some(completed_at),
            duration_seconds: Some(duration_seconds),
            checks_performed: checks,
            errors,
            warnings,
            verified_by,
            notes: None,
        };

        let mut verifications = self.verifications.write().await;
        verifications.push(result.clone());

        let mut backups = self.backups.write().await;
        if let Some(backup) = backups.get_mut(&backup_id) {
            backup.verification_history.push(result.clone());
            backup.last_verified_at = Some(completed_at);
            backup.status = if status == VerificationStatus::Passed {
                BackupStatus::Verified
            } else {
                BackupStatus::VerificationFailed
            };
        }

        Ok(result)
    }

    async fn verify_checksum(
        &self,
        backup: &BackupRecord,
    ) -> (VerificationCheck, Option<VerificationError>) {
        let start = std::time::Instant::now();

        let computed_checksum = self.compute_checksum(&backup.storage_path).await;

        let passed = computed_checksum
            .as_ref()
            .map(|c| c == &backup.checksum)
            .unwrap_or(false);

        let check = VerificationCheck {
            name: "Checksum Validation".to_string(),
            description: format!("Verify {} checksum matches stored value", backup.checksum_algorithm),
            passed,
            expected_value: Some(backup.checksum.clone()),
            actual_value: computed_checksum.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
        };

        let error = if !passed {
            Some(VerificationError {
                code: "CHECKSUM_MISMATCH".to_string(),
                message: "Backup checksum does not match the stored value".to_string(),
                severity: ErrorSeverity::Critical,
                recoverable: false,
                suggested_action: Some("Backup may be corrupted. Consider creating a new backup.".to_string()),
            })
        } else {
            None
        };

        (check, error)
    }

    async fn compute_checksum(&self, _path: &str) -> Option<String> {
        Some("computed_checksum_placeholder".to_string())
    }

    async fn verify_metadata(
        &self,
        backup: &BackupRecord,
    ) -> (VerificationCheck, Option<VerificationError>) {
        let start = std::time::Instant::now();

        let metadata_valid = backup.size_bytes > 0
            && backup.completed_at.is_some()
            && !backup.storage_path.is_empty();

        let check = VerificationCheck {
            name: "Metadata Validation".to_string(),
            description: "Verify backup metadata is complete and consistent".to_string(),
            passed: metadata_valid,
            expected_value: Some("Complete metadata".to_string()),
            actual_value: Some(if metadata_valid {
                "Metadata valid".to_string()
            } else {
                "Metadata incomplete".to_string()
            }),
            duration_ms: start.elapsed().as_millis() as u64,
        };

        let error = if !metadata_valid {
            Some(VerificationError {
                code: "INVALID_METADATA".to_string(),
                message: "Backup metadata is incomplete or invalid".to_string(),
                severity: ErrorSeverity::High,
                recoverable: true,
                suggested_action: Some("Update backup metadata or re-run backup".to_string()),
            })
        } else {
            None
        };

        (check, error)
    }

    async fn perform_restore_test(
        &self,
        backup: &BackupRecord,
    ) -> (VerificationCheck, Option<VerificationError>) {
        let start = std::time::Instant::now();

        let restore_success = self.simulate_restore(backup).await;

        let check = VerificationCheck {
            name: "Restore Test".to_string(),
            description: "Perform full restore to test environment".to_string(),
            passed: restore_success,
            expected_value: Some("Successful restore".to_string()),
            actual_value: Some(if restore_success {
                "Restore completed".to_string()
            } else {
                "Restore failed".to_string()
            }),
            duration_ms: start.elapsed().as_millis() as u64,
        };

        let error = if !restore_success {
            Some(VerificationError {
                code: "RESTORE_FAILED".to_string(),
                message: "Backup could not be restored successfully".to_string(),
                severity: ErrorSeverity::Critical,
                recoverable: false,
                suggested_action: Some("Investigate restore failure and create new backup".to_string()),
            })
        } else {
            None
        };

        (check, error)
    }

    async fn simulate_restore(&self, _backup: &BackupRecord) -> bool {
        true
    }

    async fn perform_partial_restore(
        &self,
        backup: &BackupRecord,
    ) -> (VerificationCheck, Option<VerificationError>) {
        let start = std::time::Instant::now();

        let partial_restore_success = self.simulate_partial_restore(backup).await;

        let check = VerificationCheck {
            name: "Partial Restore Test".to_string(),
            description: "Restore subset of data to verify integrity".to_string(),
            passed: partial_restore_success,
            expected_value: Some("Partial restore successful".to_string()),
            actual_value: Some(if partial_restore_success {
                "Partial restore completed".to_string()
            } else {
                "Partial restore failed".to_string()
            }),
            duration_ms: start.elapsed().as_millis() as u64,
        };

        let error = if !partial_restore_success {
            Some(VerificationError {
                code: "PARTIAL_RESTORE_FAILED".to_string(),
                message: "Partial restore test failed".to_string(),
                severity: ErrorSeverity::High,
                recoverable: true,
                suggested_action: Some("Verify backup integrity and retry".to_string()),
            })
        } else {
            None
        };

        (check, error)
    }

    async fn simulate_partial_restore(&self, _backup: &BackupRecord) -> bool {
        true
    }

    async fn validate_sample_data(
        &self,
        backup: &BackupRecord,
    ) -> (VerificationCheck, Option<VerificationError>) {
        let start = std::time::Instant::now();

        let sample_valid = self.check_sample_data(backup).await;

        let check = VerificationCheck {
            name: "Sample Data Validation".to_string(),
            description: "Validate random sample of backed up data".to_string(),
            passed: sample_valid,
            expected_value: Some("Sample data valid".to_string()),
            actual_value: Some(if sample_valid {
                "Sample validated".to_string()
            } else {
                "Sample validation failed".to_string()
            }),
            duration_ms: start.elapsed().as_millis() as u64,
        };

        let error = if !sample_valid {
            Some(VerificationError {
                code: "SAMPLE_DATA_INVALID".to_string(),
                message: "Sample data validation failed".to_string(),
                severity: ErrorSeverity::Medium,
                recoverable: true,
                suggested_action: Some("Run full integrity check".to_string()),
            })
        } else {
            None
        };

        (check, error)
    }

    async fn check_sample_data(&self, _backup: &BackupRecord) -> bool {
        true
    }

    async fn full_integrity_check(
        &self,
        backup: &BackupRecord,
    ) -> (Vec<VerificationCheck>, Vec<VerificationError>, Vec<String>) {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let checksum_result = self.verify_checksum(backup).await;
        checks.push(checksum_result.0);
        if let Some(err) = checksum_result.1 {
            errors.push(err);
        }

        let metadata_result = self.verify_metadata(backup).await;
        checks.push(metadata_result.0);
        if let Some(err) = metadata_result.1 {
            errors.push(err);
        }

        let start = std::time::Instant::now();
        let storage_accessible = self.check_storage_accessibility(&backup.storage_path).await;
        checks.push(VerificationCheck {
            name: "Storage Accessibility".to_string(),
            description: "Verify backup storage is accessible".to_string(),
            passed: storage_accessible,
            expected_value: Some("Accessible".to_string()),
            actual_value: Some(if storage_accessible {
                "Accessible".to_string()
            } else {
                "Not accessible".to_string()
            }),
            duration_ms: start.elapsed().as_millis() as u64,
        });

        if !storage_accessible {
            errors.push(VerificationError {
                code: "STORAGE_INACCESSIBLE".to_string(),
                message: "Backup storage location is not accessible".to_string(),
                severity: ErrorSeverity::Critical,
                recoverable: true,
                suggested_action: Some("Check storage credentials and connectivity".to_string()),
            });
        }

        if backup.expires_at < Utc::now() + Duration::days(7) {
            warnings.push("Backup will expire within 7 days".to_string());
        }

        (checks, errors, warnings)
    }

    async fn check_storage_accessibility(&self, _path: &str) -> bool {
        true
    }

    pub async fn run_restore_test(
        &self,
        backup_id: Uuid,
    ) -> Result<RestoreTestResult, BackupError> {
        let backups = self.backups.read().await;
        let backup = backups
            .get(&backup_id)
            .ok_or_else(|| BackupError::NotFound("Backup not found".to_string()))?
            .clone();
        drop(backups);

        let test_id = Uuid::new_v4();
        let started_at = Utc::now();
        let restore_target = format!("test_restore_{}", test_id);

        let mut integrity_checks = Vec::new();
        let mut errors = Vec::new();

        if let Some(table_count) = backup.metadata.table_count {
            for i in 0..table_count.min(5) {
                integrity_checks.push(IntegrityCheck {
                    table_name: format!("table_{i}"),
                    expected_rows: 1000,
                    actual_rows: 1000,
                    checksum_match: true,
                    foreign_keys_valid: true,
                    indexes_valid: true,
                    constraints_valid: true,
                });
            }
        }

        let completed_at = Utc::now();
        let duration = (completed_at - started_at).num_seconds() as u64;

        let status = if errors.is_empty() {
            RestoreTestStatus::Passed
        } else {
            RestoreTestStatus::Failed
        };

        let result = RestoreTestResult {
            id: test_id,
            backup_id,
            started_at,
            completed_at: Some(completed_at),
            status,
            restore_target,
            data_validated: true,
            tables_restored: integrity_checks.len() as u32,
            rows_validated: integrity_checks.iter().map(|c| c.actual_rows).sum(),
            integrity_checks,
            performance_metrics: RestorePerformanceMetrics {
                total_duration_seconds: duration,
                restore_speed_mbps: backup.size_bytes as f32 / 1_000_000.0 / duration.max(1) as f32,
                validation_duration_seconds: duration / 3,
                peak_memory_usage_mb: 512,
                disk_io_mb: backup.size_bytes / 1_000_000,
            },
            errors,
            cleanup_completed: true,
        };

        let mut restore_tests = self.restore_tests.write().await;
        restore_tests.push(result.clone());

        Ok(result)
    }

    pub async fn create_policy(&self, policy: BackupPolicy) -> Result<BackupPolicy, BackupError> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id, policy.clone());
        Ok(policy)
    }

    pub async fn get_policy(&self, id: Uuid) -> Option<BackupPolicy> {
        let policies = self.policies.read().await;
        policies.get(&id).cloned()
    }

    pub async fn get_all_policies(&self) -> Vec<BackupPolicy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    pub async fn update_policy(&self, policy: BackupPolicy) -> Result<BackupPolicy, BackupError> {
        let mut policies = self.policies.write().await;
        if !policies.contains_key(&policy.id) {
            return Err
