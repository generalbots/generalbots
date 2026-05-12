use botlib::traits::{DriveListResult, DriveListEntry, DriveObjectInfo, DriveObjectMetadata, DriveRepository};
use std::pin::Pin;

use super::s3_repository::S3Repository;

impl DriveRepository for S3Repository {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        let ct = content_type.map(String::from);
        Box::pin(async move {
            repo.put_object_direct(&bucket, &key, data, ct.as_deref())
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        Box::pin(async move {
            repo.get_object_direct(&bucket, &key)
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        Box::pin(async move {
            repo.delete_object_direct(&bucket, &key)
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn copy_object(
        &self,
        bucket: &str,
        from_key: &str,
        to_key: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let from = from_key.to_string();
        let to = to_key.to_string();
        Box::pin(async move {
            repo.copy_object_direct(&bucket, &from, &to)
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let prefix = prefix.map(String::from);
        Box::pin(async move {
        repo.list_objects(&bucket, prefix.as_deref())
            .await
            .map_err(|e| e.to_string())
        })
    }

    fn list_objects_with_metadata(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<DriveObjectInfo>, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let prefix = prefix.map(String::from);
        Box::pin(async move {
            let infos = repo
                .list_objects_with_metadata(&bucket, prefix.as_deref())
                .await
                .map_err(|e| e.to_string())?;
            Ok(infos
                .into_iter()
                .map(|i| DriveObjectInfo {
                    key: i.key,
                    etag: i.etag,
                    size: i.size,
                })
                .collect())
        })
    }

    fn list_all_buckets(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>> {
        let repo = self.clone();
        Box::pin(async move {
            repo.list_all_buckets().await.map_err(|e| e.to_string())
        })
    }

    fn object_exists(
        &self,
        bucket: &str,
        key: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        Box::pin(async move {
            repo.object_exists(&bucket, &key)
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn get_object_metadata(
        &self,
        bucket: &str,
        key: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<DriveObjectMetadata>, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        Box::pin(async move {
            let meta = repo
                .get_object_metadata(&bucket, &key)
                .await
                .map_err(|e| e.to_string())?;
            Ok(meta.map(|m| DriveObjectMetadata {
                size: m.size,
                content_type: m.content_type,
                last_modified: m.last_modified,
                etag: m.etag,
            }))
        })
    }

    fn create_bucket_if_not_exists(
        &self,
        bucket: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        Box::pin(async move {
            repo.create_bucket_if_not_exists(&bucket)
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn delete_objects(
        &self,
        bucket: &str,
        keys: Vec<String>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        Box::pin(async move {
            repo.delete_objects(&bucket, keys)
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn head_bucket(
        &self,
        bucket: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        Box::pin(async move {
            match repo.bucket_for(&bucket) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        })
    }

    fn list_objects_v2(
        &self,
        bucket: &str,
        prefix: &str,
        delimiter: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<DriveListResult, String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let prefix = prefix.to_string();
        let delim = delimiter.map(String::from);
        Box::pin(async move {
            let infos = repo
                .list_objects_with_metadata(&bucket, Some(prefix.as_str()))
                .await
                .map_err(|e| e.to_string())?;
            let objects: Vec<DriveListEntry> = infos
                .into_iter()
                .map(|i| DriveListEntry {
                    key: i.key,
                    size: i.size,
                })
                .collect();
            let common_prefixes = if let Some(d) = &delim {
            let prefixes = repo
                .list_objects(&bucket, Some(prefix.as_str()))
                .await
                .map_err(|e| e.to_string())?;
                prefixes
                    .iter()
                    .filter(|p| p.ends_with(d.as_str()))
                    .cloned()
                    .collect()
            } else {
                vec![]
            };
            Ok(DriveListResult {
                objects,
                common_prefixes,
            })
        })
    }

    fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
        content_type: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        let fp = file_path.to_string();
        let ct = content_type.map(String::from);
        Box::pin(async move {
            repo.upload_file(&bucket, &key, &fp, ct.as_deref())
                .await
                .map_err(|e| e.to_string())
        })
    }

    fn download_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let repo = self.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        let fp = file_path.to_string();
        Box::pin(async move {
            repo.download_file(&bucket, &key, &fp)
                .await
                .map_err(|e| e.to_string())
        })
    }
}
