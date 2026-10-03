//! Object operations for [`S3Repository`] (issue #1468).
//!
//! Split from `s3_repository` to keep both files inside the project size limit.
//! Every method here is on the same type; only the file moved.

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
            None, None, None,
        ).context("Failed to create credentials")?;

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

        let query = presigned_url.query()
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
            return Err(anyhow::anyhow!("S3 GET failed with status {} for {}/{}: {}",
                response.status(), bucket, key, response.text().await.unwrap_or_default()));
        }

        let data = response.bytes()
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
    pub async fn copy_object_direct(&self, bucket: &str, from_key: &str, to_key: &str) -> Result<()> {
        debug!("Copying in S3: {}/{} -> {}/{}", bucket, from_key, bucket, to_key);
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
            None, None, None
        ).map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;
        let target = Bucket::new(bucket_name, region, creds)?.with_path_style();
        Ok(Arc::new((*target).clone()))
    }

    /// List all buckets in S3/MinIO using rust-s3 crate's list_buckets
    pub async fn list_all_buckets(&self) -> Result<Vec<String>> {
        debug!("Listing all buckets from S3");

        let region = self.bucket.region().clone();
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None, None, None
        ).map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;

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
        debug!("Listing common prefixes in S3: {} with delimiter {:?}", bucket, delimiter);

        let region = self.bucket.region().clone();
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None, None, None
        ).map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;

        let target_bucket = Bucket::new(bucket, region, creds)?.with_path_style();

        let results = target_bucket.list(String::new(), Some(delimiter.to_string())).await?;
        let common_prefixes: Vec<String> = results.iter()
            .flat_map(|r| r.common_prefixes.iter().flat_map(|v| v.iter().map(|cp| cp.prefix.clone())))
            .collect();

        debug!("Found {} common prefixes in bucket {}: {:?}", common_prefixes.len(), bucket, common_prefixes);
        Ok(common_prefixes)
    }

    /// List objects with prefix, returning only keys
    pub async fn list_objects(&self, bucket: &str, prefix: Option<&str>) -> Result<Vec<String>> {
        let infos = self.list_objects_with_metadata(bucket, prefix).await?;
        Ok(infos.into_iter().map(|i| i.key).collect())
    }

    /// List objects with prefix, returning key + etag + size for change detection
    pub async fn list_objects_with_metadata(&self, bucket: &str, prefix: Option<&str>) -> Result<Vec<S3ObjectInfo>> {
        debug!("Listing objects with metadata in S3: {} with prefix {:?}", bucket, prefix);

        let region = self.bucket.region().clone();
        let creds = s3::creds::Credentials::new(
            Some(&self.access_key),
            Some(&self.secret_key),
            None, None, None
        ).map_err(|e| anyhow::anyhow!("Failed to create credentials: {}", e))?;

        let target_bucket = Bucket::new(bucket, region, creds)?.with_path_style();

        let prefix_str = prefix.unwrap_or("");
        let results = target_bucket.list(prefix_str.to_string(), None).await?;
        let objects: Vec<S3ObjectInfo> = results.iter()
            .flat_map(|r| r.contents.iter().map(|c| S3ObjectInfo {
                key: c.key.clone(),
                etag: c.e_tag.clone(),
                size: c.size,
            }))
            .collect();
        debug!("Found {} objects with metadata in bucket {}", objects.len(), bucket);
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
        let data = tokio::fs::read(file_path).await
            .context("Failed to read file for upload")?;
        target_bucket.put_object(key, &data).await?;
        Ok(())
    }

    /// Download a file
    pub async fn download_file(&self, bucket: &str, key: &str, file_path: &str) -> Result<()> {
        debug!("Downloading file from S3: {}/{} -> {}", bucket, key, file_path);
        let target_bucket = self.bucket_for(bucket)?;
        let response = target_bucket.get_object(key).await?;
        let data = response.to_vec();
        tokio::fs::write(file_path, data).await
            .context("Failed to write downloaded file")?;
        info!("Successfully downloaded file from S3: {}/{} -> {}", bucket, key, file_path);
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
            let _ = target_bucket.delete_object(&key).await;
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
                warn!("Failed to check if bucket {} exists: {}. Attempting to create.", bucket, e);
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
        let short_date = now.format(&time::macros::format_description!("[year][month][day]"))
            .map_err(|e| anyhow::anyhow!("short date: {}", e))?;
        let long_date = now.format(&time::macros::format_description!("[year][month][day]T[hour][minute][second]Z"))
            .map_err(|e| anyhow::anyhow!("long date: {}", e))?;

        let mut headers = http::HeaderMap::new();
        headers.insert("host", endpoint.parse()
            .map_err(|e| anyhow::anyhow!("invalid host header: {}", e))?);
        headers.insert("x-amz-date", long_date.parse()
            .map_err(|e| anyhow::anyhow!("invalid date header: {}", e))?);
        headers.insert("x-amz-content-sha256", http::HeaderValue::from_static("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));
        headers.insert("x-amz-acl", http::HeaderValue::from_static("private"));

        let url = url::Url::parse(&url_str)
            .map_err(|e| anyhow::anyhow!("bad url: {}", e))?;

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

        let scope = format!("{date}/{region}/s3/aws4_request",
            date = short_date,
            region = region,
        );

        let mut hasher = Sha256::default();
        hasher.update(canonical_req.as_bytes());
        let canonical_hash = hex::encode(hasher.finalize());
        let sts = format!("AWS4-HMAC-SHA256\n{date}\n{scope}\n{hash}",
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
        let mut signing_hmac = Hmac::<Sha256>::new_from_slice(&service_hmac.finalize().into_bytes())
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
        headers.insert("authorization", auth.parse()
            .map_err(|e| anyhow::anyhow!("invalid auth header: {}", e))?);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let resp = client.put(&url_str)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if resp.status().is_success() {
            info!("Created bucket: {}", bucket);
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to create bucket {}: HTTP {} - {}", bucket, status, body))
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
