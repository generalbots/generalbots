/// Cache/Redis key building utilities.
///
/// Re-exports `botlib::key_utils::build_key` for convenience so both
/// `crate::cache::key_utils::build_key` and `botlib::key_utils::build_key` work.
///
/// See `botlib/src/key_utils.rs` for full docs.
pub use botlib::key_utils::build_key;
