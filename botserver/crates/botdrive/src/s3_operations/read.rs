use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use log::{debug, info, warn};
use s3::Bucket;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::s3_repository::S3Repository;
use crate::s3_shims::{ObjectMetadata, S3ObjectInfo};

impl S3Repository {
    pub async fn put_object_direct(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        _content_type: Option<&str>,
    ) -> Result<()> {
        debug!("Uploading to S3: {}/{}", bucket, key);
        let target_bucket = self.bucket_for(bucket)?;
        target_bucket.put_object(key, &data).await?;
        self.egress.record_write(data.len() as u64);
        info!("Successfully uploaded to S3: {}/{}", bucket, key);
        Ok(())
    }

    /// Download data from S3 - uses reqwest directly with path-style URL
    /// Bypasses rust-s3 entirely because rust-s3 0.37's Region::Custom.host()
    /// returns empty string for custom endpoints, producing malformed URLs.
    pub async fn get_object_direct(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        debug!("Downloading from S3: {}/{}", bucket, key);

        let region = self.bucket.region();

        // Extract endpoint from Region::Custom to build correct path-style URL
        let endpoint = if let s3::Region::Custom { ref endpoint, .. } = region {
            endpoint.clone()
        } else {
            // Fallback for a non-custom region, which rust-s3 never produces
            // here because the selection always builds `Region::Custom`.
            "http://localhost:9000".to_string()
        };

        // Build path-style URL directly: {endpoint}/{bucket}/{key}
        let path_url = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, key);

        // Use rust-s3's presign to generate signed query params, then rebuild
        // the URL with our correct path-style base URL
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None,
            None,
            None,
        )
        .context("Failed to create credentials")?;

        let target_bucket = Bucket::new(bucket, region.clone(), creds)
            .context("Failed to create target bucket")?
            .with_path_style();

        let presigned = target_bucket
            .presign_get(key, 3600, None)
            .await
            .map_err(|e| anyhow::anyhow!("S3 presign_get failed for {}/{}: {}", bucket, key, e))?;

        // Extract query params (signature) from presigned URL
        let presigned_url = url::Url::parse(&presigned)
            .map_err(|e| anyhow::anyhow!("Failed to parse presigned URL: {}", e))?;

        let query = presigned_url
            .query()
            .ok_or_else(|| anyhow::anyhow!("Presigned URL has no query string"))?;

        // Rebuild URL with correct base + signature params
        let signed_url = format!("{}?{}", path_url, query);

        // Download using reqwest — MUST have timeouts: a stalled S3 endpoint
        // blocks the whole boot (drive_compiler runs before the HTTP server).
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let response = client
            .get(&signed_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download from S3: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = match response.text().await {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("S3 error response body unreadable: {:?}", e);
                    String::new()
                }
            };
            return Err(anyhow::anyhow!(
                "S3 GET failed with status {} for {}/{}: {}",
                status,
                bucket,
                key,
                body
            ));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response bytes: {}", e))?
            .to_vec();
        self.egress.record_read(data.len() as u64);

        info!("Successfully downloaded from S3: {}/{}", bucket, key);
        Ok(data)
    }

    /// Reads an object through the configured backend, retrying on the secondary
    /// endpoint when the primary is unreachable.
    ///
    /// Failover is read-only on purpose. A write that silently landed on a second
    /// backend would leave two divergent copies of a bot's source tree, which is
    /// worse than a failed upload; the fallback is logged so an operator can act
    /// on a real outage rather than discover it later.
    pub async fn get_object_with_failover(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        match self.get_object_direct(bucket, key).await {
            Ok(data) => Ok(data),
            Err(primary_error) => {
                let Some(fallback) = self.fallback.clone() else {
                    return Err(primary_error);
                };
                warn!(
                    "Drive read of {key} failed on {} ({primary_error}); retrying on the fallback endpoint",
                    self.selection.backend.config_name()
                );
                let body = fallback
                    .get_object(key)
                    .await
                    .with_context(|| format!("fallback read failed for {bucket}/{key}"))?;
                let data: Vec<u8> = body.into();
                self.egress.record_read(data.len() as u64);
                Ok(data)
            }
        }
    }

    /// Delete an object from S3 - creates bucket reference for target bucket
    pub async fn delete_object_direct(&self, bucket: &str, key: &str) -> Result<()> {
        debug!("Deleting from S3: {}/{}", bucket, key);
        let target_bucket = self.bucket_for(bucket)?;
        target_bucket.delete_object(key).await?;
        info!("Successfully deleted from S3: {}/{}", bucket, key);
        Ok(())
    }

    /// Copy object - creates bucket reference for target bucket
    pub async fn copy_object_direct(
        &self,
        bucket: &str,
        from_key: &str,
        to_key: &str,
    ) -> Result<()> {
        debug!(
            "Copying in S3: {}/{} -> {}/{}",
            bucket, from_key, bucket, to_key
        );
        let target_bucket = self.bucket_for(bucket)?;
        let response = target_bucket.get_object(from_key).await?;
        let data = response.to_vec();
        target_bucket.put_object(to_key, &data).await?;
        Ok(())
    }

    /// Create a Bucket reference for a specific bucket name using stored credentials
    pub fn bucket_for(&self, bucket_name: &str) -> Result<Arc<Bucket>> {
        if bucket_name == self.bucket_name {
            return Ok(self.bucket.clone());
        }
        let region = self.bucket.region().clone();
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;
        let target = Bucket::new(bucket_name, region, creds)?.with_path_style();
        Ok(Arc::new((*target).clone()))
    }

    /// List all buckets in S3/OtterIO using rust-s3 crate's list_buckets
    pub async fn list_all_buckets(&self) -> Result<Vec<String>> {
        debug!("Listing all buckets from S3");

        let region = self.bucket.region().clone();
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;

        let response = Bucket::list_buckets(region, creds)
            .await
            .map_err(|e| anyhow::anyhow!("ListBuckets failed: {}", e))?;

        let buckets: Vec<String> = response.bucket_names().collect();
        debug!("Found {} buckets: {:?}", buckets.len(), buckets);
        Ok(buckets)
    }

    /// Check if an object exists
    pub async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool> {
        let target_bucket = self.bucket_for(bucket)?;
        Ok(target_bucket.object_exists(key).await?)
    }

    /// List common prefixes (sub-folders) within a bucket using a delimiter.
    /// Ex: list_common_prefixes("cristo.gborg", "/") → ["cristo.gbai/", "rh.gbai/"]
    pub async fn list_common_prefixes(&self, bucket: &str, delimiter: &str) -> Result<Vec<String>> {
        debug!(
            "Listing common prefixes in S3: {} with delimiter {:?}",
            bucket, delimiter
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

        let results = target_bucket
            .list(String::new(), Some(delimiter.to_string()))
            .await?;
        let common_prefixes: Vec<String> = results
            .iter()
            .flat_map(|r| {
                r.common_prefixes
                    .iter()
                    .flat_map(|v| v.iter().map(|cp| cp.prefix.clone()))
            })
            .collect();

        debug!(
            "Found {} common prefixes in bucket {}: {:?}",
            common_prefixes.len(),
            bucket,
            common_prefixes
        );
        Ok(common_prefixes)
    }

    /// List objects with prefix, returning only keys
    pub async fn list_objects(&self, bucket: &str, prefix: Option<&str>) -> Result<Vec<String>> {
        let infos = self.list_objects_with_metadata(bucket, prefix).await?;
        Ok(infos.into_iter().map(|i| i.key).collect())
    }
}
