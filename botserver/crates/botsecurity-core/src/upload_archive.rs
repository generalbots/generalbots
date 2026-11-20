//! Zip-container inspection for the upload policy (#2132).
//!
//! A zip that declares a million entries or a terabyte of decompressed data is
//! a bomb, not a document. [`inspect_archive`] opens the central directory and
//! refuses it before a handler extracts or stores anything.

use crate::upload_error::UploadError;

/// Maximum entries a zip container may hold (zip-bomb guard).
pub const MAX_ARCHIVE_ENTRIES: usize = 4096;
/// Maximum total *uncompressed* size a zip container may declare.
pub const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;

/// Opens a zip container and refuses bombs: too many entries, a declared
/// uncompressed size over the cap, or an entry name that escapes the archive.
///
/// Kept as a thin alias so callers on the older API get the full #2522 gate
/// instead of a second, weaker implementation.
pub fn inspect_archive(data: &[u8]) -> Result<(), UploadError> {
    crate::upload_hostile::inspect_hostile_upload(data).map(|_| ())
}

