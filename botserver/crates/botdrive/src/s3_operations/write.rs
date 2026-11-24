use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use log::{debug, info, warn};
use s3::Bucket;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::s3_repository::S3Repository;
use crate::s3_shims::{ObjectMetadata, S3ObjectInfo};

impl S3Repository {
    pub async fn list_objects_with_metadata(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<S3ObjectInfo>> {
        debug!(
            "Listing objects with metadata in S3: {} with prefix {:?}",
            bucket, prefix
        );

        let region = self.bucket.region().clone();
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;

        let target_bucket = Bucket::new(bucket, region, creds)?.with_path_style();

        let prefix_str = prefix.unwrap_or("");
        let results = target_bucket.list(prefix_str.to_string(), None).await?;
        let objects: Vec<S3ObjectInfo> = results
            .iter()
            .flat_map(|r| {
                r.contents.iter().map(|c| S3ObjectInfo {
                    key: c.key.clone(),
                    etag: c.e_tag.clone(),
                    size: c.size,
                })
            })
            .collect();
        debug!(
            "Found {} objects with metadata in bucket {}",
            objects.len(),
            bucket
        );
        Ok(objects)
    }

    /// Upload a file
    pub async fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
        _content_type: Option<&str>,
    ) -> Result<()> {
        debug!("Uploading file to S3: {} -> {}/{}", file_path, bucket, key);
        let target_bucket = self.bucket_for(bucket)?;
        let data = tokio::fs::read(file_path)
            .await
            .context("Failed to read file for upload")?;
        target_bucket.put_object(key, &data).await?;
        Ok(())
    }

    /// Download a file
    pub async fn download_file(&self, bucket: &str, key: &str, file_path: &str) -> Result<()> {
        debug!(
            "Downloading file from S3: {}/{} -> {}",
            bucket, key, file_path
        );
        let target_bucket = self.bucket_for(bucket)?;
        let response = target_bucket.get_object(key).await?;
        let data = response.to_vec();
        tokio::fs::write(file_path, data)
            .await
            .context("Failed to write downloaded file")?;
        info!(
            "Successfully downloaded file from S3: {}/{} -> {}",
            bucket, key, file_path
        );
        Ok(())
    }

    /// Delete multiple objects
    pub async fn delete_objects(&self, bucket: &str, keys: Vec<String>) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        debug!("Deleting {} objects from S3: {}", keys.len(), bucket);
        let target_bucket = self.bucket_for(bucket)?;
        let keys_count = keys.len();
        for key in keys {
            if let Err(e) = target_bucket.delete_object(&key).await {
                log::warn!("Failed to delete object {key} from bucket {bucket}: {e}");
            }
        }
        info!("Deleted {} objects from S3: {}", keys_count, bucket);
        Ok(())
    }

    /// Create bucket if not exists
    pub async fn create_bucket_if_not_exists(&self, bucket: &str) -> Result<()> {
        let target_bucket = self.bucket_for(bucket)?;

        match target_bucket.exists().await {
            Ok(true) => {
                debug!("Bucket already exists: {}", bucket);
                return Ok(());
            }
            Ok(false) => {}
            Err(e) => {
                warn!(
                    "Failed to check if bucket {} exists: {}. Attempting to create.",
                    bucket, e
                );
            }
        }

        self.create_bucket_signed(bucket).await
    }

    /// Create bucket via manually signed AWS SigV4 PUT request.
    /// Uses path-style addressing and no XML body (MinIO compatible).
    async fn create_bucket_signed(&self, bucket: &str) -> Result<()> {
        let region = self.bucket.region().clone();
        let endpoint = region.host();
        let scheme = region.scheme();
        let url_str = format!("{}://{}/{}", scheme, endpoint, bucket);

        let now = time::OffsetDateTime::now_utc();
        let short_date = now
            .format(&time::macros::format_description!("[year][month][day]"))
            .map_err(|e| anyhow::anyhow!("short date: {}", e))?;
        let long_date = now
            .format(&time::macros::format_description!(
                "[year][month][day]T[hour][minute][second]Z"
            ))
            .map_err(|e| anyhow::anyhow!("long date: {}", e))?;

        let mut headers = http::HeaderMap::new();
        headers.insert(
            "host",
            endpoint
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid host header: {}", e))?,
        );
        headers.insert(
            "x-amz-date",
            long_date
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid date header: {}", e))?,
        );
        headers.insert(
            "x-amz-content-sha256",
            http::HeaderValue::from_static(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
        );
        headers.insert("x-amz-acl", http::HeaderValue::from_static("private"));

        let url = url::Url::parse(&url_str).map_err(|e| anyhow::anyhow!("bad url: {}", e))?;

        let canonical_uri = s3::signing::canonical_uri_string(&url);
        let canonical_qs = s3::signing::canonical_query_string(&url);
        let canonical_hdrs = s3::signing::canonical_header_string(&headers)
            .map_err(|e| anyhow::anyhow!("canonical headers: {}", e))?;
        let signed_hdrs = s3::signing::signed_header_string(&headers);

        let payload_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let canonical_req = format!(
            "PUT\n{uri}\n{qs}\n{hdrs}\n\n{signed}\n{payload}",
            uri = canonical_uri,
            qs = canonical_qs,
            hdrs = canonical_hdrs,
            signed = signed_hdrs,
            payload = payload_hash,
        );

        let scope = format!(
            "{date}/{region}/s3/aws4_request",
            date = short_date,
            region = region,
        );

        let mut hasher = Sha256::default();
        hasher.update(canonical_req.as_bytes());
        let canonical_hash = hex::encode(hasher.finalize());
        let sts = format!(
            "AWS4-HMAC-SHA256\n{date}\n{scope}\n{hash}",
            date = long_date,
            scope = scope,
            hash = canonical_hash,
        );

        let secret = format!("AWS4{}", self.secret_key);
        let mut date_hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("hmac key: {}", e))?;
        date_hmac.update(short_date.as_bytes());
        let mut region_hmac = Hmac::<Sha256>::new_from_slice(&date_hmac.finalize().into_bytes())
            .map_err(|e| anyhow::anyhow!("hmac region: {}", e))?;
        region_hmac.update(region.to_string().as_bytes());
        let mut service_hmac = Hmac::<Sha256>::new_from_slice(&region_hmac.finalize().into_bytes())
            .map_err(|e| anyhow::anyhow!("hmac service: {}", e))?;
        service_hmac.update(b"s3");
        let mut signing_hmac =
            Hmac::<Sha256>::new_from_slice(&service_hmac.finalize().into_bytes())
                .map_err(|e| anyhow::anyhow!("hmac signing: {}", e))?;
        signing_hmac.update(b"aws4_request");
        let signing_key = signing_hmac.finalize().into_bytes();

        let mut sig_hmac = Hmac::<Sha256>::new_from_slice(&signing_key)
            .map_err(|e| anyhow::anyhow!("hmac sig: {}", e))?;
        sig_hmac.update(sts.as_bytes());
        let signature = hex::encode(sig_hmac.finalize().into_bytes());

        let auth = format!(
            "AWS4-HMAC-SHA256 Credential={ak}/{scope},SignedHeaders={sh},Signature={sig}",
            ak = self.access_key,
            scope = scope,
            sh = signed_hdrs,
            sig = signature,
        );
        headers.insert(
            "authorization",
            auth.parse()
                .map_err(|e| anyhow::anyhow!("invalid auth header: {}", e))?,
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let resp = client
            .put(&url_str)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if resp.status().is_success() {
            info!("Created bucket: {}", bucket);
            Ok(())
        } else {
            let status = resp.status();
            let body = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("S3 error response body unreadable: {:?}", e);
                    String::new()
                }
            };
            Err(anyhow::anyhow!(
                "Failed to create bucket {}: HTTP {} - {}",
                bucket,
                status,
                body
            ))
        }
    }

    /// Get object metadata
    pub async fn get_object_metadata(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<Option<ObjectMetadata>> {
        let target_bucket = self.bucket_for(bucket)?;
        match target_bucket.head_object(key).await {
            Ok((response, _)) => Ok(Some(ObjectMetadata {
                size: response.content_length.unwrap_or(0) as u64,
                content_type: response.content_type,
                last_modified: response.last_modified,
                etag: response.e_tag,
            })),
            Err(_) => Ok(None),
        }
    }

    // ============ Builder pattern methods for backward compatibility ============
}
