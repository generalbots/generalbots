//! Object operations for [`S3Repository`] (issue #1468).
//!
//! Split from `s3_repository` to keep files inside the project size limit.
//! Every method here is on the same type; only the file moved.

mod read;
mod write;
