use super::*;
use std::fs;

use super::*;
use std::fs;

fn setup_cache_with_file(name: &str, data: &[u8]) -> (std::path::PathBuf, DownloadCache) {
    let base = std::env::temp_dir().join(format!("gb-cache-test-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let cache_dir = base.join("botserver-installers");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join("f.bin"), data).unwrap();
    let cache = DownloadCache::new(&base).unwrap();
    (base, cache)
}

#[test]
fn verify_checksum_empty_is_err() {
    let (base, cache) = setup_cache_with_file("empty", b"hello");
    let res = cache.verify_checksum("f.bin", "");
    let _ = fs::remove_dir_all(&base);
    assert!(res.is_err(), "empty checksum must be Err");
}

#[test]
fn verify_checksum_malformed_is_err() {
    let (base, cache) = setup_cache_with_file("malformed", b"hello");
    assert!(cache.verify_checksum("f.bin", "zz").is_err());
    assert!(cache.verify_checksum("f.bin", "0").is_err());
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn verify_checksum_correct_is_ok_true() {
    let (base, cache) = setup_cache_with_file("correct", b"hello world");
    let expected = sha256_hex(b"hello world");
    let res = cache.verify_checksum("f.bin", &expected).unwrap();
    let _ = fs::remove_dir_all(&base);
    assert!(res);
}

#[test]
fn verify_checksum_wrong_is_ok_false() {
    let (base, cache) = setup_cache_with_file("wrong", b"hello");
    let wrong = "0".repeat(64);
    let res = cache.verify_checksum("f.bin", &wrong).unwrap();
    let _ = fs::remove_dir_all(&base);
    assert!(!res);
}
