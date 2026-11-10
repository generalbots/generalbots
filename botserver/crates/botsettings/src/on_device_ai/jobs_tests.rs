//! Downloader helpers that can be tested without the network (#2109).

use super::sha256_file;
use std::io::Write;
use std::path::PathBuf;

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("gb-on-device-ai-{name}"));
    let mut file = std::fs::File::create(&path).expect("temp file");
    file.write_all(bytes).expect("write temp file");
    path
}

#[test]
fn hashes_the_file_contents() {
    let path = temp_file("hash.txt", b"abc");
    // Known SHA-256 of "abc".
    let digest = sha256_file(&path).expect("digest");
    assert_eq!(
        digest,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn reports_a_missing_file_instead_of_panicking() {
    let missing = std::env::temp_dir().join("gb-on-device-ai-missing.gguf");
    let _ = std::fs::remove_file(&missing);
    assert!(sha256_file(&missing).is_err());
}
