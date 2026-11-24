use super::unpinned_checksum_error;

#[test]
fn unpinned_is_fatal_in_strict_mode() {
    assert!(unpinned_checksum_error("https://example.invalid/bin", true).is_some());
    assert!(unpinned_checksum_error("https://example.invalid/bin", false).is_none());
}

#[test]
fn refusal_names_the_url_and_the_policy() {
    let message = unpinned_checksum_error("https://example.invalid/pinned.bin", true)
        .expect("strict mode must refuse an unpinned binary");
    assert!(message.contains("https://example.invalid/pinned.bin"));
    assert!(message.contains("GB_REQUIRE_CHECKSUMS"));
}

#[test]
fn lenient_mode_only_warns_for_any_unpinned_url() {
    for url in ["https://example.invalid/bin", "", "not-a-url"] {
        assert!(unpinned_checksum_error(url, false).is_none(), "{url}");
    }
}
