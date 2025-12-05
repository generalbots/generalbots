//! Drive integration module for attendance system
//! Handles file storage and synchronization for attendance records

use anyhow::{anyhow, Result};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use chrono::TimeZone;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Drive configuration for attendance storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceDriveConfig {
    pub bucket_name: String,
    pub prefix: String,
    pub sync_enabled: bool,
    pub region: Option<String>,
}

impl Default for AttendanceDriveConfig {
    fn default() -> Self {
        Self {
            bucket_name: "attendance".to_string(),
            prefix: "records/".to_string(),
            sync_enabled: true,
            region: None,
        }
    }
}

/// Drive service for attendance data
#[derive(Debug, Clone)]
pub struct AttendanceDriveService {
    config: AttendanceDriveConfig,
    client: Client,
}

impl AttendanceDriveService {
    /// Create new attendance drive service
    pub async fn new(config: AttendanceDriveConfig) -> Result<Self> {
        let sdk_config = if let Some(region) = &config.region {
            aws_config::from_env()
                .region(aws_config::Region::new(region.clone()))
                .load()
                .await
        } else {
            aws_config::from_env().load().await
        };

        let client = Client::new(&sdk_config);

        Ok(Self { config, client })
    }

    /// Create new service with existing S3 client
    pub fn with_client(config: AttendanceDriveConfig, client: Client) -> Self {
        Self { config, client }
    }

    /// Get the full S3 key for a record
    fn get_record_key(&self, record_id: &str) -> String {
        format!("{}{}", self.config.prefix, record_id)
    }

    /// Upload attendance record to drive
    pub async fn upload_record(&self, record_id: &str, data: Vec<u8>) -> Result<()> {
        let key = self.get_record_key(record_id);

        log::info!(
            "Uploading attendance record {} to s3://{}/{}",
            record_id,
            self.config.bucket_name,
            key
        );

        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .body(body)
            .content_type("application/octet-stream")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to upload attendance record: {}", e))?;

        log::debug!("Successfully uploaded attendance record {}", record_id);
        Ok(())
    }

    /// Download attendance record from drive
    pub async fn download_record(&self, record_id: &str) -> Result<Vec<u8>> {
        let key = self.get_record_key(record_id);

        log::info!(
            "Downloading attendance record {} from s3://{}/{}",
            record_id,
            self.config.bucket_name,
            key
        );

        let result = self
            .client
            .get_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to download attendance record: {}", e))?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| anyhow!("Failed to read attendance record body: {}", e))?;

        log::debug!("Successfully downloaded attendance record {}", record_id);
        Ok(data.into_bytes().to_vec())
    }

    /// List attendance records in drive
    pub async fn list_records(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let list_prefix = if let Some(p) = prefix {
            format!("{}{}", self.config.prefix, p)
        } else {
            self.config.prefix.clone()
        };

        log::info!(
            "Listing attendance records in s3://{}/{}",
            self.config.bucket_name,
            list_prefix
        );

        let mut records = Vec::new();
        let mut continuation_token = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.config.bucket_name)
                .prefix(&list_prefix)
                .max_keys(1000);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let result = request
                .send()
                .await
                .map_err(|e| anyhow!("Failed to list attendance records: {}", e))?;

            if let Some(contents) = result.contents {
                for obj in contents {
                    if let Some(key) = obj.key {
                        // Remove prefix to get record ID
                        if let Some(record_id) = key.strip_prefix(&self.config.prefix) {
                            records.push(record_id.to_string());
                        }
                    }
                }
            }

            if result.is_truncated.unwrap_or(false) {
                continuation_token = result.next_continuation_token;
            } else {
                break;
            }
        }

        log::debug!("Found {} attendance records", records.len());
        Ok(records)
    }

    /// Delete attendance record from drive
    pub async fn delete_record(&self, record_id: &str) -> Result<()> {
        let key = self.get_record_key(record_id);

        log::info!(
            "Deleting attendance record {} from s3://{}/{}",
            record_id,
            self.config.bucket_name,
            key
        );

        self.client
            .delete_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to delete attendance record: {}", e))?;

        log::debug!("Successfully deleted attendance record {}", record_id);
        Ok(())
    }

    /// Batch delete multiple attendance records
    pub async fn delete_records(&self, record_ids: &[String]) -> Result<()> {
        if record_ids.is_empty() {
            return Ok(());
        }

        log::info!(
            "Batch deleting {} attendance records from bucket {}",
            record_ids.len(),
            self.config.bucket_name
        );

        // S3 batch delete is limited to 1000 objects per request
        for chunk in record_ids.chunks(1000) {
            let objects: Vec<_> = chunk
                .iter()
                .map(|id| {
                    aws_sdk_s3::types::ObjectIdentifier::builder()
                        .key(self.get_record_key(id))
                        .build()
                        .unwrap()
                })
                .collect();

            let delete = aws_sdk_s3::types::Delete::builder()
                .set_objects(Some(objects))
                .build()
                .map_err(|e| anyhow!("Failed to build delete request: {}", e))?;

            self.client
                .delete_objects()
                .bucket(&self.config.bucket_name)
                .delete(delete)
                .send()
                .await
                .map_err(|e| anyhow!("Failed to batch delete attendance records: {}", e))?;
        }

        log::debug!(
            "Successfully batch deleted {} attendance records",
            record_ids.len()
        );
        Ok(())
    }

    /// Check if an attendance record exists
    pub async fn record_exists(&self, record_id: &str) -> Result<bool> {
        let key = self.get_record_key(record_id);

        match self
            .client
            .head_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(sdk_err) => {
                if sdk_err.to_string().contains("404") || sdk_err.to_string().contains("NotFound") {
                    Ok(false)
                } else {
                    Err(anyhow!(
                        "Failed to check attendance record existence: {}",
                        sdk_err
                    ))
                }
            }
        }
    }

    /// Sync local attendance records with drive
    pub async fn sync_records(&self, local_path: PathBuf) -> Result<SyncResult> {
        if !self.config.sync_enabled {
            log::debug!("Attendance drive sync is disabled");
            return Ok(SyncResult::default());
        }

        log::info!(
            "Syncing attendance records from {:?} to s3://{}/{}",
            local_path,
            self.config.bucket_name,
            self.config.prefix
        );

        if !local_path.exists() {
            return Err(anyhow!("Local path does not exist: {:?}", local_path));
        }

        let mut uploaded = 0;
        let mut failed = 0;
        let mut skipped = 0;

        let mut entries = fs::read_dir(&local_path)
            .await
            .map_err(|e| anyhow!("Failed to read local directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| anyhow!("Failed to read directory entry: {}", e))?
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => {
                    log::warn!("Skipping file with invalid name: {:?}", path);
                    skipped += 1;
                    continue;
                }
            };

            // Check if record already exists in S3
            if self.record_exists(&file_name).await? {
                log::debug!("Record {} already exists in drive, skipping", file_name);
                skipped += 1;
                continue;
            }

            // Read file and upload
            match fs::read(&path).await {
                Ok(data) => match self.upload_record(&file_name, data).await {
                    Ok(_) => {
                        log::debug!("Uploaded attendance record: {}", file_name);
                        uploaded += 1;
                    }
                    Err(e) => {
                        log::error!("Failed to upload {}: {}", file_name, e);
                        failed += 1;
                    }
                },
                Err(e) => {
                    log::error!("Failed to read file {:?}: {}", path, e);
                    failed += 1;
                }
            }
        }

        let result = SyncResult {
            uploaded,
            failed,
            skipped,
        };

        log::info!(
            "Sync completed: {} uploaded, {} failed, {} skipped",
            result.uploaded,
            result.failed,
            result.skipped
        );

        Ok(result)
    }

    /// Get metadata for an attendance record
    pub async fn get_record_metadata(&self, record_id: &str) -> Result<RecordMetadata> {
        let key = self.get_record_key(record_id);

        let result = self
            .client
            .head_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get attendance record metadata: {}", e))?;

        Ok(RecordMetadata {
            size: result.content_length.unwrap_or(0) as usize,
            last_modified: result
                .last_modified
                .and_then(|t| t.to_millis().ok())
                .map(|ms| chrono::Utc.timestamp_millis_opt(ms as i64).unwrap()),
            content_type: result.content_type,
            etag: result.e_tag,
        })
    }
}

/// Result of sync operation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub uploaded: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Metadata for an attendance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub size: usize,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}
