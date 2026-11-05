//! Drive adapter for the slides app.
//!
//! `botslides` is generic over its own `DriveOps` trait; the server's shared
//! `DriveRepository` is adapted here the same way `sheet::drive_adapter` does
//! it. Without this the slides router could not be mounted (it needs
//! `SlidesState<D>` with a concrete drive implementation).

use std::sync::Arc;

use botlib::traits::DriveRepository;

use crate::slides::storage::DriveOps;

#[derive(Clone)]
pub struct DriveOpsAdapter(pub Arc<dyn DriveRepository>);

impl DriveOps for DriveOpsAdapter {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let repository = self.0.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        let content_type = content_type.to_string();
        async move {
            repository
                .put_object(&bucket, &key, body, Some(&content_type))
                .await
        }
    }

    fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, String>> + Send {
        let repository = self.0.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        async move { repository.get_object(&bucket, &key).await }
    }

    fn list_objects(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> impl std::future::Future<Output = Result<Vec<String>, String>> + Send {
        let repository = self.0.clone();
        let bucket = bucket.to_string();
        let prefix = prefix.to_string();
        async move { repository.list_objects(&bucket, Some(&prefix)).await }
    }

    fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let repository = self.0.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        async move { repository.delete_object(&bucket, &key).await }
    }
}
