//! The S3 facade itself: the repository type, how it is built from a backend
//! selection, and the builder accessors.
//!
//! The per-operation methods live in [`s3_operations`], the builder and response
//! types in [`s3_shims`], and the backend catalog in [`storage_backends`]. Split
//! across four files to stay inside the project size limit; the type is one.

use anyhow::{Context, Result};
use s3::{Bucket, Region, creds::Credentials};
use std::sync::Arc;

use crate::s3_shims::{
    S3CopyBuilder, S3CreateBucketBuilder, S3DeleteBuilder, S3GetBuilder, S3HeadBucketBuilder,
    S3ListBucketsBuilder, S3ListObjectsBuilder, S3PutBuilder,
};
use crate::storage_backends::{EgressMeter, StorageSelection};

/// S3 Repository for basic operations
#[derive(Debug, Clone)]
pub struct S3Repository {
    pub(crate) bucket_name: String,
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
    /// Backend identity and the region requests are signed with (#1468).
    pub(crate) selection: StorageSelection,
    /// Secondary endpoint tried on a read when the primary is unreachable.
    pub(crate) fallback: Option<Arc<Bucket>>,
    /// Running egress counters, so a provider bill can be explained (#1468).
    pub(crate) egress: Arc<EgressMeter>,
}

impl S3Repository {
    /// Creates a repository for a resolved backend selection.
    ///
    /// The region is part of the selection rather than hardcoded: Backblaze B2
    /// and Cloudflare R2 both require an explicit one, and signing against the
    /// wrong region is what makes those backends unusable.
    pub fn from_selection(
        selection: &StorageSelection,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self> {
        selection.warn_if_unsuitable();

        let bucket = bucket_for_region(selection, access_key, secret_key, &selection.bucket)?;
        // The fallback handle is built eagerly so a switch costs nothing at read
        // time; a handle that cannot be built simply means no failover.
        let fallback = selection
            .fallback_endpoint
            .as_ref()
            .and_then(|endpoint| {
                let mut fallback_selection = selection.clone();
                fallback_selection.endpoint = endpoint.clone();
                bucket_for_region(&fallback_selection, access_key, secret_key, &selection.bucket)
                    .ok()
                    .map(Arc::new)
            });

        Ok(Self {
            bucket_name: selection.bucket.clone(),
            bucket: Arc::new(bucket),
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            selection: selection.clone(),
            fallback,
            egress: Arc::new(EgressMeter::new()),
        })
    }

    /// Create new S3 repository against the self-hosted default.
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str, bucket: &str) -> Result<Self> {
        Self::from_selection(
            &StorageSelection::new("minio", endpoint, "", bucket, None),
            access_key,
            secret_key,
        )
    }

    /// The backend selection this client was built from.
    #[must_use]
    pub fn selection(&self) -> &StorageSelection {
        &self.selection
    }

    /// Running egress counters for this client.
    #[must_use]
    pub fn egress(&self) -> &EgressMeter {
        &self.egress
    }

    /// True when a secondary endpoint is configured for read failover.
    #[must_use]
    pub fn has_fallback(&self) -> bool {
        self.fallback.is_some()
    }

    /// Upload data to S3 - creates bucket reference for target bucket
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

/// Builds a path-style bucket handle for a selection.
fn bucket_for_region(
    selection: &StorageSelection,
    access_key: &str,
    secret_key: &str,
    bucket: &str,
) -> Result<Bucket> {
    let region = Region::Custom {
        region: selection.region.clone(),
        endpoint: selection.endpoint.clone(),
    };
    let handle = Bucket::new(
        bucket,
        region,
        Credentials::new(Some(access_key), Some(secret_key), None, None, None)
            .context("Failed to create credentials")?,
    )?
    .with_path_style();
    Ok(*handle)
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


/// Builds a repository from the application configuration, honouring the
/// configured backend, region and fallback endpoint (#1468).
pub fn create_s3_operator_from_config(
    config: &botcore::config::AppConfig,
) -> anyhow::Result<S3Repository> {
    S3Repository::from_selection(
        &StorageSelection::new(
            &config.drive.backend,
            &config.drive.endpoint,
            &config.drive.region,
            &config.drive.bucket,
            Some(config.drive.fallback_endpoint.clone()),
        ),
        &config.drive.access_key,
        &config.drive.secret_key,
    )
}
