//! In-memory Drive bucket for the DOCX half of the fidelity contract.
//!
//! The save path reads the source package from the bot bucket, so a working
//! store is what makes "survives a restart" observable: the test persists, drops
//! the process cache, and reloads from here.

use std::collections::HashMap;
use std::sync::Mutex;

use botlib::traits::{
    BoxFutureBool, BoxFutureDriveList, BoxFutureOptionDriveMeta, BoxFutureUnit,
    BoxFutureVecDriveObject, BoxFutureVecString, BoxFutureVecU8, DriveListResult, DriveRepository,
};

#[derive(Debug, Default)]
pub struct MemoryDrive {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

fn object_key(bucket: &str, key: &str) -> String {
    format!("{bucket}/{key}")
}

impl MemoryDrive {
    fn stored(&self, bucket: &str, key: &str) -> Option<Vec<u8>> {
        let full = object_key(bucket, key);
        self.objects
            .lock()
            .ok()
            .and_then(|objects| objects.get(&full).cloned())
    }
}

impl DriveRepository for MemoryDrive {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        _content_type: Option<&str>,
    ) -> BoxFutureUnit {
        let full = object_key(bucket, key);
        match self.objects.lock() {
            Ok(mut objects) => {
                objects.insert(full, data);
                Box::pin(async { Ok(()) })
            }
            Err(error) => {
                let message = format!("memory drive unavailable: {error}");
                Box::pin(async move { Err(message) })
            }
        }
    }

    fn get_object(&self, bucket: &str, key: &str) -> BoxFutureVecU8 {
        let full = object_key(bucket, key);
        let found = self.stored(bucket, key);
        Box::pin(async move { found.ok_or_else(|| format!("missing object: {full}")) })
    }

    fn delete_object(&self, bucket: &str, key: &str) -> BoxFutureUnit {
        let full = object_key(bucket, key);
        if let Ok(mut objects) = self.objects.lock() {
            objects.remove(&full);
        }
        Box::pin(async { Ok(()) })
    }

    fn copy_object(&self, bucket: &str, from_key: &str, to_key: &str) -> BoxFutureUnit {
        let copied = self
            .objects
            .lock()
            .ok()
            .and_then(|mut objects| {
                let from = object_key(bucket, from_key);
                let data = objects.remove(&from)?;
                objects.insert(object_key(bucket, to_key), data)
            })
            .is_some();
        if copied {
            Box::pin(async { Ok(()) })
        } else {
            Box::pin(async { Err("copy source is missing".to_string()) })
        }
    }

    fn list_objects(&self, _bucket: &str, _prefix: Option<&str>) -> BoxFutureVecString {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn list_objects_with_metadata(
        &self,
        _bucket: &str,
        _prefix: Option<&str>,
    ) -> BoxFutureVecDriveObject {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn list_common_prefixes(&self, _bucket: &str, _delimiter: &str) -> BoxFutureVecString {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn list_all_buckets(&self) -> BoxFutureVecString {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn object_exists(&self, bucket: &str, key: &str) -> BoxFutureBool {
        let exists = self.stored(bucket, key).is_some();
        Box::pin(async move { Ok(exists) })
    }

    fn get_object_metadata(&self, _bucket: &str, _key: &str) -> BoxFutureOptionDriveMeta {
        Box::pin(async { Ok(None) })
    }

    fn create_bucket_if_not_exists(&self, _bucket: &str) -> BoxFutureUnit {
        Box::pin(async { Ok(()) })
    }

    fn delete_objects(&self, bucket: &str, keys: Vec<String>) -> BoxFutureUnit {
        if let Ok(mut objects) = self.objects.lock() {
            for key in keys {
                objects.remove(&object_key(bucket, &key));
            }
        }
        Box::pin(async { Ok(()) })
    }

    fn head_bucket(&self, _bucket: &str) -> BoxFutureBool {
        Box::pin(async { Ok(true) })
    }

    fn list_objects_v2(
        &self,
        _bucket: &str,
        _prefix: &str,
        _delimiter: Option<&str>,
    ) -> BoxFutureDriveList {
        Box::pin(async {
            Ok(DriveListResult {
                objects: Vec::new(),
                common_prefixes: Vec::new(),
            })
        })
    }

    fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
        _content_type: Option<&str>,
    ) -> BoxFutureUnit {
        let full = object_key(bucket, key);
        match (std::fs::read(file_path), self.objects.lock()) {
            (Ok(data), Ok(mut objects)) => {
                objects.insert(full, data);
                Box::pin(async { Ok(()) })
            }
            (Err(error), _) => {
                let message = format!("cannot read {file_path}: {error}");
                Box::pin(async move { Err(message) })
            }
            (_, Err(error)) => {
                let message = format!("memory drive unavailable: {error}");
                Box::pin(async move { Err(message) })
            }
        }
    }

    fn download_file(&self, bucket: &str, key: &str, file_path: &str) -> BoxFutureUnit {
        let missing = format!("missing object: {}", object_key(bucket, key));
        match self.stored(bucket, key) {
            Some(data) => match std::fs::write(file_path, data) {
                Ok(()) => Box::pin(async { Ok(()) }),
                Err(error) => {
                    let message = format!("cannot write {file_path}: {error}");
                    Box::pin(async move { Err(message) })
                }
            },
            None => Box::pin(async move { Err(missing) }),
        }
    }
}
