//! Sync-to-async bridge for the AutoTask BASIC-only pipeline (#754).
//!
//! The pipeline persists generated `.bas` scripts to the bot's Drive bucket
//! (`{bot}.gbai/{bot}.gbdialog/{folder}/{file}.bas`). `botlib::traits::DriveRepository`
//! is async, while the AutoTask `DriveOps` facade is synchronous (it is invoked
//! from script handlers). This module bridges the two with a short-lived
//! current-thread Tokio runtime — the standard pattern used across keywords.

use crate::types::{BoxError, DriveOps};
use botlib::traits::DriveRepository;
use std::sync::Arc;

/// Adapter making the async botlib repository usable from AutoTask handlers.
pub struct DriveRepositoryOps(pub Arc<dyn DriveRepository>);

fn bridge<T>(fut: impl std::future::Future<Output = Result<T, String>>) -> Result<T, BoxError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| box_error(format!("spawn runtime: {e}")))?;
    rt.block_on(fut)
        .map_err(|e| box_error(format!("drive operation failed: {e}")))
}

fn box_error(msg: String) -> BoxError {
    Box::new(BridgeError(msg)) as BoxError
}

#[derive(Debug)]
struct BridgeError(String);

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BridgeError {}

impl DriveOps for DriveRepositoryOps {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), BoxError> {
        bridge(self.0.put_object(bucket, key, body, Some(content_type)))
    }

    fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, BoxError> {
        bridge(self.0.get_object(bucket, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use botlib::traits::{
        BoxFutureBool, BoxFutureDriveList, BoxFutureOptionDriveMeta, BoxFutureUnit,
        BoxFutureVecDriveObject, BoxFutureVecString, BoxFutureVecU8,
    };

    struct FakeRepo;

    impl std::fmt::Debug for FakeRepo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "FakeRepo")
        }
    }

    impl DriveRepository for FakeRepo {
        fn put_object(
            &self,
            _bucket: &str,
            _key: &str,
            _data: Vec<u8>,
            _content_type: Option<&str>,
        ) -> BoxFutureUnit {
            Box::pin(async { Ok(()) })
        }

        fn get_object(&self, _bucket: &str, _key: &str) -> BoxFutureVecU8 {
            Box::pin(async { Ok(b"hello".to_vec()) })
        }

        fn delete_object(&self, _bucket: &str, _key: &str) -> BoxFutureUnit {
            Box::pin(async { Ok(()) })
        }

        fn copy_object(&self, _bucket: &str, _from_key: &str, _to_key: &str) -> BoxFutureUnit {
            Box::pin(async { Ok(()) })
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

        fn object_exists(&self, _bucket: &str, _key: &str) -> BoxFutureBool {
            Box::pin(async { Ok(false) })
        }

        fn get_object_metadata(&self, _bucket: &str, _key: &str) -> BoxFutureOptionDriveMeta {
            Box::pin(async { Ok(None) })
        }

        fn create_bucket_if_not_exists(&self, _bucket: &str) -> BoxFutureUnit {
            Box::pin(async { Ok(()) })
        }

        fn delete_objects(&self, _bucket: &str, _keys: Vec<String>) -> BoxFutureUnit {
            let _ = _keys;
            Box::pin(async { Ok(()) })
        }

        fn head_bucket(&self, _bucket: &str) -> BoxFutureBool {
            Box::pin(async { Ok(false) })
        }

        fn list_objects_v2(
            &self,
            _bucket: &str,
            _prefix: &str,
            _delimiter: Option<&str>,
        ) -> BoxFutureDriveList {
            Box::pin(async {
                Ok(botlib::traits::DriveListResult {
                    objects: Vec::new(),
                    common_prefixes: Vec::new(),
                })
            })
        }

        fn upload_file(
            &self,
            _bucket: &str,
            _key: &str,
            _file_path: &str,
            _content_type: Option<&str>,
        ) -> BoxFutureUnit {
            Box::pin(async { Ok(()) })
        }

        fn download_file(&self, _bucket: &str, _key: &str, _file_path: &str) -> BoxFutureUnit {
            Box::pin(async { Ok(()) })
        }
    }

    #[test]
    fn put_and_get_via_bridge() {
        let ops = DriveRepositoryOps(Arc::new(FakeRepo));
        ops.put_object("b", "k", vec![1, 2], "text/plain").unwrap();
        let got = ops.get_object("b", "k").unwrap();
        assert_eq!(got, b"hello");
    }
}