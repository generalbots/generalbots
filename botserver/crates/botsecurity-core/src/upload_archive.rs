//! Zip-container inspection for the upload policy (#2132).
//!
//! A zip that declares a million entries or a terabyte of decompressed data is
//! a bomb, not a document. [`inspect_archive`] opens the central directory and
//! refuses it before a handler extracts or stores anything.

use std::io::Cursor;

use crate::upload_error::UploadError;

/// Maximum entries a zip container may hold (zip-bomb guard).
pub const MAX_ARCHIVE_ENTRIES: usize = 4096;
/// Maximum total *uncompressed* size a zip container may declare.
pub const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;

/// Opens a zip container and refuses bombs: too many entries, a declared
/// uncompressed size over the cap, or an entry name that escapes the archive.
pub fn inspect_archive(data: &[u8]) -> Result<(), UploadError> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(data)).map_err(|_| UploadError::TypeMismatch {
            expected: "application/zip".to_string(),
            detected: "not a zip container".to_string(),
        })?;

    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(UploadError::ArchiveUnsafe {
            reason: "too many entries",
        });
    }

    let mut total: u64 = 0;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| UploadError::ArchiveUnsafe {
                reason: "unreadable entry",
            })?;
        total = total.saturating_add(entry.size());
        if total > MAX_ARCHIVE_UNCOMPRESSED_BYTES {
            return Err(UploadError::ArchiveUnsafe {
                reason: "decompressed size",
            });
        }
        if entry.enclosed_name().is_none() {
            return Err(UploadError::ArchiveUnsafe {
                reason: "unsafe entry name",
            });
        }
    }

    Ok(())
}

