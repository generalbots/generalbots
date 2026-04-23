/// S3 Repository - Simple facade for S3 operations using rust-s3
/// No AWS SDK - uses rust-s3 crate only
use log::{debug, info};
use std::sync::Arc;
use anyhow::{Result, Context};
use s3::{Bucket, Region, creds::Credentials};

/// S3 Repository for basic operations
#[derive(Debug, Clone)]
pub struct S3Repository {
    bucket_name: String,
    bucket: Arc<Bucket>,
    access_key: String,
    secret_key: String,
}

impl S3Repository {
    /// Create new S3 repository
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str, bucket: &str) -> Result<Self> {
        let region = Region::Custom {
            region: "auto".to_string(),
            endpoint: endpoint.trim_end_matches('/').to_string(),
        };

        let s3_bucket = Bucket::new(
            bucket,
            region,
            Credentials::new(Some(access_key), Some(secret_key), None, None, None)
                .context("Failed to create credentials")?,
        )?.with_path_style();

        Ok(Self {
            bucket_name: bucket.to_string(),
            bucket: Arc::new((*s3_bucket).clone()),
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
        })
    }

    /// Upload data to S3 - creates bucket reference for target bucket
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
        info!("Successfully uploaded to S3: {}/{}", bucket, key);
        Ok(())
    }

    /// Download data from S3 - creates bucket reference for target bucket
    pub async fn get_object_direct(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        debug!("Downloading from S3: {}/{}", bucket, key);
        let target_bucket = self.bucket_for(bucket)?;
        let response = target_bucket.get_object(key).await?;
        let data = response.to_vec();
        info!("Successfully downloaded from S3: {}/{}", bucket, key);
        Ok(data)
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
    fn bucket_for(&self, bucket_name: &str) -> Result<Arc<Bucket>> {
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
        let _target_bucket = self.bucket_for(bucket)?;
        Ok(())
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

/// Start put object builder
pub fn put_object(&self) -> S3PutBuilder {
    S3PutBuilder {
        bucket: self.bucket.clone(),
        key: None,
        body: None,
        content_type: None,
    }
}

/// Start get object builder
pub fn get_object(&self) -> S3GetBuilder {
    S3GetBuilder {
        bucket: self.bucket.clone(),
        key: None,
    }
}

/// Start delete object builder
pub fn delete_object(&self) -> S3DeleteBuilder {
    S3DeleteBuilder {
        bucket: self.bucket.clone(),
        key: None,
    }
}

/// Start copy object builder
pub fn copy_object(&self) -> S3CopyBuilder {
    S3CopyBuilder {
        bucket: self.bucket.clone(),
        source: None,
        dest: None,
    }
}

    /// List buckets
    pub fn list_buckets(&self) -> S3ListBucketsBuilder {
        S3ListBucketsBuilder { repo: Some(Arc::new(self.clone())) }
    }

    /// Head bucket
    pub fn head_bucket(&self) -> S3HeadBucketBuilder {
        S3HeadBucketBuilder {
            bucket_name: None,
        }
    }

    /// Create bucket
    pub fn create_bucket(&self) -> S3CreateBucketBuilder {
        S3CreateBucketBuilder {
            bucket_name: None,
        }
    }

    /// List objects v2
    pub fn list_objects_v2(&self) -> S3ListObjectsBuilder {
        S3ListObjectsBuilder {
            bucket: self.bucket.clone(),
            bucket_name: None,
            prefix: None,
        }
    }
}

/// Metadata for an S3 object (from HEAD request)
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub size: u64,
    pub content_type: Option<String>,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
}

/// Object info from list operations (key + etag + size)
#[derive(Debug, Clone)]
pub struct S3ObjectInfo {
    pub key: String,
    pub etag: Option<String>,
    pub size: u64,
}

// ============ Builder implementations ============

pub struct S3PutBuilder {
    bucket: Arc<Bucket>,
    key: Option<String>,
    body: Option<Vec<u8>>,
    content_type: Option<String>,
}

impl S3PutBuilder {
    pub fn bucket(self, _name: &str) -> Self { self }
    pub fn key(mut self, k: &str) -> Self { self.key = Some(k.to_string()); self }
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self { self.body = Some(body.into()); self }
    pub fn content_type(mut self, ct: &str) -> Self { self.content_type = Some(ct.to_string()); self }
    pub fn content_disposition(self, _cd: &str) -> Self { self }
    pub async fn send(self) -> Result<S3Response> {
        let key = self.key.context("Key required")?;
        let body = self.body.context("Body required")?;
        self.bucket.put_object(&key, &body).await?;
        Ok(S3Response::with_data(body))
    }
}

pub struct S3GetBuilder {
    bucket: Arc<Bucket>,
    key: Option<String>,
}

impl S3GetBuilder {
    pub fn bucket(self, _name: &str) -> Self { self }
    pub fn key(mut self, k: &str) -> Self { self.key = Some(k.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        let key = self.key.context("Key required")?;
        let response = self.bucket.get_object(&key).await?;
        let data = response.to_vec();
        Ok(S3Response::with_data(data))
    }
}

pub struct S3DeleteBuilder {
    bucket: Arc<Bucket>,
    key: Option<String>,
}

impl S3DeleteBuilder {
    pub fn bucket(self, _name: &str) -> Self { self }
    pub fn key(mut self, key: &str) -> Self { self.key = Some(key.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        let key = self.key.context("Key required")?;
        self.bucket.delete_object(&key).await?;
        Ok(S3Response::new())
    }
}

pub struct S3CopyBuilder {
    bucket: Arc<Bucket>,
    source: Option<String>,
    dest: Option<String>,
}

impl S3CopyBuilder {
    pub fn bucket(self, _name: &str) -> Self { self }
    pub fn source(mut self, source: &str) -> Self { self.source = Some(source.to_string()); self }
    pub fn dest(mut self, dest: &str) -> Self { self.dest = Some(dest.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        let source = self.source.context("Source required")?;
        let dest = self.dest.context("Dest required")?;
        let response = self.bucket.get_object(&source).await?;
        let data = response.to_vec();
        self.bucket.put_object(&dest, &data).await?;
        Ok(S3Response::new())
    }
}

pub struct S3ListBucketsBuilder {
    repo: Option<SharedS3Repository>,
}

impl S3ListBucketsBuilder {
    pub fn repo(mut self, repo: SharedS3Repository) -> Self { self.repo = Some(repo); self }
    pub async fn send(self) -> Result<S3ListBucketsResponse> {
        if let Some(repo) = self.repo {
            let names = repo.list_all_buckets().await?;
            Ok(S3ListBucketsResponse {
                buckets: names.into_iter().map(|name| S3Bucket { name }).collect(),
            })
        } else {
            Ok(S3ListBucketsResponse { buckets: vec![] })
        }
    }
}

pub struct S3HeadBucketBuilder {
    bucket_name: Option<String>,
}

impl S3HeadBucketBuilder {
    pub fn bucket(mut self, name: &str) -> Self { self.bucket_name = Some(name.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        Ok(S3Response::default())
    }
}

pub struct S3CreateBucketBuilder {
    bucket_name: Option<String>,
}

impl S3CreateBucketBuilder {
    pub fn bucket(mut self, name: &str) -> Self { self.bucket_name = Some(name.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        Ok(S3Response::default())
    }
}

pub struct S3ListObjectsBuilder {
    bucket: Arc<Bucket>,
    bucket_name: Option<String>,
    prefix: Option<String>,
}

impl S3ListObjectsBuilder {
    pub fn bucket(mut self, name: &str) -> Self { self.bucket_name = Some(name.to_string()); self }
    pub fn prefix(mut self, prefix: &str) -> Self { self.prefix = Some(prefix.to_string()); self }
    pub async fn send(self) -> Result<S3ListObjectsResponse> {
        let prefix_str = self.prefix.unwrap_or_default();
        let results = self.bucket.list(prefix_str, Some("/".to_string())).await?;
        let contents: Vec<S3Object> = results.iter()
            .flat_map(|r| r.contents.iter().map(|c| S3Object {
                key: c.key.clone(),
                size: c.size,
            }))
            .collect();
        Ok(S3ListObjectsResponse { contents })
    }
}

// ============ Response types ============

#[derive(Debug, Default)]
pub struct S3Response {
    pub body: S3ResponseBody,
}

impl S3Response {
    pub fn new() -> Self { Self::default() }
    pub fn with_data(data: Vec<u8>) -> Self { Self { body: S3ResponseBody { data } } }
}

#[derive(Debug, Default)]
pub struct S3ResponseBody {
    pub data: Vec<u8>,
}

impl S3ResponseBody {
    pub async fn collect(self) -> Result<S3CollectedBody> {
        Ok(S3CollectedBody { data: self.data })
    }
}

#[derive(Debug, Default)]
pub struct S3CollectedBody {
    data: Vec<u8>,
}

impl S3CollectedBody {
    pub fn into_bytes(self) -> Vec<u8> { self.data }
}

#[derive(Debug)]
pub struct S3ListBucketsResponse {
    pub buckets: Vec<S3Bucket>,
}

#[derive(Debug)]
pub struct S3Bucket {
    pub name: String,
}

impl S3Bucket {
    pub fn name(&self) -> Option<String> { Some(self.name.clone()) }
}

#[derive(Debug)]
pub struct S3ListObjectsResponse {
    pub contents: Vec<S3Object>,
}

impl S3ListObjectsResponse {
    pub fn contents(&self) -> &[S3Object] { &self.contents }
}

#[derive(Debug)]
pub struct S3Object {
    pub key: String,
    pub size: u64,
}

impl S3Object {
    pub fn key(&self) -> Option<String> { Some(self.key.clone()) }
}

/// Thread-safe wrapper
pub type SharedS3Repository = Arc<S3Repository>;

/// Create shared repository
pub fn create_shared_repository(
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    bucket: &str,
) -> Result<SharedS3Repository> {
    let repo = S3Repository::new(endpoint, access_key, secret_key, bucket)?;
    Ok(Arc::new(repo))
}
