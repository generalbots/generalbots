//! Builder and response types for the S3 facade (issue #1468).
//!
//! Split from `s3_repository` to keep both files inside the project size limit.
//! These types exist so the platform's storage helpers keep a stable shape while
//! the implementation underneath stays on `rust-s3` rather than an AWS SDK.

use anyhow::{Context, Result};
use s3::Bucket;
use std::sync::Arc;

use super::s3_repository::SharedS3Repository;

/// Metadata for an S3 object, from a HEAD request.
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
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) key: Option<String>,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) content_type: Option<String>,
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
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) key: Option<String>,
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
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) key: Option<String>,
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
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) source: Option<String>,
    pub(crate) dest: Option<String>,
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
    pub(crate) repo: Option<SharedS3Repository>,
}

impl S3ListBucketsBuilder {
    pub fn repo(mut self, repo: SharedS3Repository) -> Self { self.repo = Some(repo); self }
    pub async fn send(self) -> Result<S3ListBucketsResponse> {
        if let Some(repo) = self.repo {
            let names = repo.list_all_buckets().await?;
            Ok(S3ListBucketsResponse { buckets: names.into_iter().map(|name| S3Bucket { name }).collect() })
        } else {
            Ok(S3ListBucketsResponse { buckets: vec![] })
        }
    }
}

pub struct S3HeadBucketBuilder {
    pub(crate) bucket_name: Option<String>,
}

impl S3HeadBucketBuilder {
    pub fn bucket(mut self, name: &str) -> Self { self.bucket_name = Some(name.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        Ok(super::S3Response::default())
    }
}

pub struct S3CreateBucketBuilder {
    pub(crate) bucket_name: Option<String>,
}

impl S3CreateBucketBuilder {
    pub fn bucket(mut self, name: &str) -> Self { self.bucket_name = Some(name.to_string()); self }
    pub async fn send(self) -> Result<S3Response> {
        Ok(super::S3Response::default())
    }
}

pub struct S3ListObjectsBuilder {
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) bucket_name: Option<String>,
    pub(crate) prefix: Option<String>,
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
    pub(crate) data: Vec<u8>,
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
