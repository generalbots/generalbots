//! A no-op `DriveRepository` used when no storage backend is configured.
//!
//! Some consumers (e.g. the docs app) require a concrete `DriveRepository`.
//! This implementation satisfies the trait without panicking: every mutation
//! returns an error, and every read returns an empty result. It never touches
//! the network and is only a compile-time fallback for an absent backend.

use botlib::traits::{
    BoxFutureBool, BoxFutureDriveList, BoxFutureOptionDriveMeta, BoxFutureUnit,
    BoxFutureVecDriveObject, BoxFutureVecString, BoxFutureVecU8, DriveListResult,
    DriveRepository,
};

#[derive(Debug, Default)]
pub struct NoopDrive;

const NOOP_ERR: &str = "drive backend is not configured (NoopDrive)";

fn err_unit() -> BoxFutureUnit {
    Box::pin(async { Err(NOOP_ERR.to_string()) })
}

fn err_vec_string() -> BoxFutureVecString {
    Box::pin(async { Ok(Vec::new()) })
}

impl DriveRepository for NoopDrive {
    fn put_object(
        &self,
        _bucket: &str,
        _key: &str,
        _data: Vec<u8>,
        _content_type: Option<&str>,
    ) -> BoxFutureUnit {
        err_unit()
    }

    fn get_object(
        &self,
        _bucket: &str,
        _key: &str,
    ) -> BoxFutureVecU8 {
        Box::pin(async { Err(NOOP_ERR.to_string()) })
    }

    fn delete_object(
        &self,
        _bucket: &str,
        _key: &str,
    ) -> BoxFutureUnit {
        err_unit()
    }

    fn copy_object(
        &self,
        _bucket: &str,
        _from: &str,
        _to: &str,
    ) -> BoxFutureUnit {
        err_unit()
    }

    fn list_objects(
        &self,
        _bucket: &str,
        _prefix: Option<&str>,
    ) -> BoxFutureVecString {
        err_vec_string()
    }

    fn list_objects_with_metadata(
        &self,
        _bucket: &str,
        _prefix: Option<&str>,
    ) -> BoxFutureVecDriveObject {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn list_common_prefixes(
        &self,
        _bucket: &str,
        _delimiter: &str,
    ) -> BoxFutureVecString {
        err_vec_string()
    }

    fn list_all_buckets(
        &self,
    ) -> BoxFutureVecString {
        err_vec_string()
    }

    fn object_exists(
        &self,
        _bucket: &str,
        _key: &str,
    ) -> BoxFutureBool {
        Box::pin(async { Ok(false) })
    }

    fn get_object_metadata(
        &self,
        _bucket: &str,
        _key: &str,
    ) -> BoxFutureOptionDriveMeta {
        Box::pin(async { Ok(None) })
    }

    fn create_bucket_if_not_exists(
        &self,
        _bucket: &str,
    ) -> BoxFutureUnit {
        err_unit()
    }

    fn delete_objects(
        &self,
        _bucket: &str,
        _keys: Vec<String>,
    ) -> BoxFutureUnit {
        let _ = _keys;
        err_unit()
    }

    fn head_bucket(
        &self,
        _bucket: &str,
    ) -> BoxFutureBool {
        Box::pin(async { Ok(false) })
    }

    fn list_objects_v2(
        &self,
        _bucket: &str,
        _prefix: &str,
        _delimiter: Option<&str>,
    ) -> BoxFutureDriveList {
        let _ = (_prefix, _delimiter);
        Box::pin(async {
            Ok(DriveListResult {
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
        err_unit()
    }

    fn download_file(
        &self,
        _bucket: &str,
        _key: &str,
        _file_path: &str,
    ) -> BoxFutureUnit {
        err_unit()
    }
}