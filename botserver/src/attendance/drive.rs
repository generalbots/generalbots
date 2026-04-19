use anyhow::{anyhow, Result};
use crate::drive::s3_repository::S3Repository;
use chrono::TimeZone;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceDriveConfig {
    pub bucket_name: String,
    pub prefix: String,
    pub sync_enabled: bool,
    pub endpoint: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

impl Default for AttendanceDriveConfig {
    fn default() -> Self {
        Self {
            bucket_name: "attendance".to_string(),
            prefix: "records/".to_string(),
            sync_enabled: true,
            endpoint: None,
            access_key: None,
            secret_key: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttendanceDriveService {
    config: AttendanceDriveConfig,
    client: S3Repository,
}

impl AttendanceDriveService {
    pub async fn new(config: AttendanceDriveConfig) -> Result<Self> {
        let endpoint = config.endpoint.as_deref().unwrap_or("http://localhost:9100");
        let access_key = config.access_key.as_deref().unwrap_or("minioadmin");
        let secret_key = config.secret_key.as_deref().unwrap_or("minioadmin");

        let client = S3Repository::new(endpoint, access_key, secret_key, &config.bucket_name)
            .map_err(|e| anyhow!("Failed to create S3 repository: {}", e))?;

        Ok(Self { config, client })
    }

    pub fn with_client(config: AttendanceDriveConfig, client: S3Repository) -> Self {
        Self { config, client }
    }

    fn get_record_key(&self, record_id: &str) -> String {
        format!("{}{}", self.config.prefix, record_id)
    }

    pub async fn upload_record(&self, record_id: &str, data: Vec<u8>) -> Result<()> {
        let key = self.get_record_key(record_id);

        log::info!(
            "Uploading attendance record {} to s3://{}/{}",
            record_id, self.config.bucket_name, key
        );

        self.client
            .put_object(&self.config.bucket_name, &key, data, Some("application/octet-stream"))
            .await
            .map_err(|e| anyhow!("Failed to upload attendance record: {}", e))?;

        log::debug!("Successfully uploaded attendance record {}", record_id);
        Ok(())
    }

    pub async fn download_record(&self, record_id: &str) -> Result<Vec<u8>> {
        let key = self.get_record_key(record_id);

        log::info!(
            "Downloading attendance record {} from s3://{}/{}",
            record_id, self.config.bucket_name, key
        );

        let data = self.client
            .get_object(&self.config.bucket_name, &key)
            .await
            .map_err(|e| anyhow!("Failed to download attendance record: {}", e))?;

        log::debug!("Successfully downloaded attendance record {}", record_id);
        Ok(data)
    }

    pub async fn list_records(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let list_prefix = if let Some(p) = prefix {
            format!("{}{}", self.config.prefix, p)
        } else {
            self.config.prefix.clone()
        };

        log::info!(
            "Listing attendance records in s3://{}/{}",
            self.config.bucket_name, list_prefix
        );

        let keys = self.client
            .list_objects(&self.config.bucket_name, Some(&list_prefix))
            .await
            .map_err(|e| anyhow!("Failed to list attendance records: {}", e))?;

        let records: Vec<String> = keys
            .iter()
            .filter_map(|key| key.strip_prefix(&self.config.prefix).map(|s| s.to_string()))
            .collect();

        log::debug!("Found {} attendance records", records.len());
        Ok(records)
    }

    pub async fn delete_record(&self, record_id: &str) -> Result<()> {
        let key = self.get_record_key(record_id);

        log::info!(
            "Deleting attendance record {} from s3://{}/{}",
            record_id, self.config.bucket_name, key
        );

        self.client
            .delete_object(&self.config.bucket_name, &key)
            .await
            .map_err(|e| anyhow!("Failed to delete attendance record: {}", e))?;

        log::debug!("Successfully deleted attendance record {}", record_id);
        Ok(())
    }

    pub async fn delete_records(&self, record_ids: &[String]) -> Result<()> {
        if record_ids.is_empty() {
            return Ok(());
        }

        log::info!(
            "Batch deleting {} attendance records from bucket {}",
            record_ids.len(), self.config.bucket_name
        );

        let keys: Vec<String> = record_ids.iter().map(|id| self.get_record_key(id)).collect();
        
        self.client
            .delete_objects(&self.config.bucket_name, keys)
            .await
            .map_err(|e| anyhow!("Failed to batch delete attendance records: {}", e))?;

        log::debug!("Successfully batch deleted {} attendance records", record_ids.len());
        Ok(())
    }

    pub async fn record_exists(&self, record_id: &str) -> Result<bool> {
        let key = self.get_record_key(record_id);
        self.client
            .object_exists(&self.config.bucket_name, &key)
            .await
            .map_err(|e| anyhow!("Failed to check attendance record existence: {}", e))
    }

    pub async fn sync_records(&self, local_path: PathBuf) -> Result<SyncResult> {
        if !self.config.sync_enabled {
            log::debug!("Attendance drive sync is disabled");
            return Ok(SyncResult::default());
        }

        log::info!(
            "Syncing attendance records from {} to s3://{}/{}",
            local_path.display(), self.config.bucket_name, self.config.prefix
        );

        if !local_path.exists() {
            return Err(anyhow!("Local path does not exist: {}", local_path.display()));
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
                    log::warn!("Skipping file with invalid name: {}", path.display());
                    skipped += 1;
                    continue;
                }
            };

            if self.record_exists(&file_name).await? {
                log::debug!("Record {} already exists in drive, skipping", file_name);
                skipped += 1;
                continue;
            }

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
                    log::error!("Failed to read file {}: {}", path.display(), e);
                    failed += 1;
                }
            }
        }

        let result = SyncResult { uploaded, failed, skipped };

        log::info!(
            "Sync completed: {} uploaded, {} failed, {} skipped",
            result.uploaded, result.failed, result.skipped
        );

        Ok(result)
    }

    pub async fn get_record_metadata(&self, record_id: &str) -> Result<RecordMetadata> {
        let key = self.get_record_key(record_id);

        let metadata = self.client
            .get_object_metadata(&self.config.bucket_name, &key)
            .await
            .map_err(|e| anyhow!("Failed to get attendance record metadata: {}", e))?;

        match metadata {
            Some(m) => Ok(RecordMetadata {
                size: m.size as usize,
                last_modified: m.last_modified.and_then(|s| {
                    chrono::DateTime::parse_from_rfc2822(&s).ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                content_type: m.content_type,
                etag: m.etag,
            }),
            None => Err(anyhow!("Record not found: {}", record_id)),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub uploaded: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub size: usize,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}
