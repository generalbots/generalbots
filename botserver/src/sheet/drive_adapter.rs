use std::sync::Arc;

use async_trait::async_trait;
use botlib::traits::DriveRepository;
use crate::sheet::state::DriveOps;

pub struct DriveOpsAdapter(pub Arc<dyn DriveRepository>);

#[async_trait]
impl DriveOps for DriveOpsAdapter {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        self.0
            .put_object(bucket, key, body, Some(content_type))
            .await
    }

    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<Vec<u8>, String> {
        self.0.get_object(bucket, key).await
    }

    async fn list_objects(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<String>, String> {
        self.0.list_objects(bucket, Some(prefix)).await
    }

    async fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(), String> {
        self.0.delete_object(bucket, key).await
    }
}
