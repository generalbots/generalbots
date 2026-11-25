use super::*;
use std::fs;

fn setup_cache_with_file(name: &str, data: &[u8]) -> (std::path::PathBuf, DownloadCache) {
    let base = std::env::temp_dir().join(format!("gb-cache-test-{}-{}", name, std::process::id()));
    if let Err(e) = fs::remove_dir_all(&base) {
        assert_eq!(
            e.kind(),
            std::io::ErrorKind::NotFound,
            "could not clear stale cache test dir: {e}"
        );
    }
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
    assert!(
        fs::remove_dir_all(&base).is_ok(),
        "could not clean up cache test dir"
    );
    assert!(res.is_err(), "empty checksum must be Err");
}

#[test]
fn verify_checksum_malformed_is_err() {
    let (base, cache) = setup_cache_with_file("malformed", b"hello");
    assert!(cache.verify_checksum("f.bin", "zz").is_err());
    assert!(cache.verify_checksum("f.bin", "0").is_err());
    assert!(
        fs::remove_dir_all(&base).is_ok(),
        "could not clean up cache test dir"
    );
}

#[test]
fn verify_checksum_correct_is_ok_true() {
    let (base, cache) = setup_cache_with_file("correct", b"hello world");
    let expected = sha256_hex(b"hello world");
    let res = cache.verify_checksum("f.bin", &expected).unwrap();
    assert!(
        fs::remove_dir_all(&base).is_ok(),
        "could not clean up cache test dir"
    );
    assert!(res);
}

#[test]
fn verify_checksum_wrong_is_ok_false() {
    let (base, cache) = setup_cache_with_file("wrong", b"hello");
    let wrong = "0".repeat(64);
    let res = cache.verify_checksum("f.bin", &wrong).unwrap();
    assert!(
        fs::remove_dir_all(&base).is_ok(),
        "could not clean up cache test dir"
    );
    assert!(!res);
}
