use super::*;

use super::*;

/// #2143 gap A — the chat renderer resolves only `vmfile:` and `app://`
/// links, so a `drive:` URL rendered as dead text and the artifact was
/// unreachable. The emitted string must be one of those two prefixes.
/// #2283 — the bucket is included so the Drive can resolve the path in the
/// correct store instead of the currently selected one.
#[test]
fn drive_link_uses_a_prefix_the_chat_renderer_resolves() {
    let link = drive_link(
        "report.pdf",
        "beiner.gbai",
        "beiner.gbdrive/conversations/quarterly-report",
    );
    assert!(
        link.starts_with("[report.pdf](app://drive?path="),
        "unexpected link: {link}"
    );
    assert!(
        link.contains("&scope=bot&bucket="),
        "unexpected link: {link}"
    );
    assert!(!link.contains("](drive:"), "dead drive: scheme: {link}");
    assert!(
        link.starts_with("[report.pdf](app://") || link.starts_with("[report.pdf](vmfile:"),
        "not a renderer prefix: {link}"
    );
}

/// The path must survive percent-encoding, because the renderer decodes it
/// into the Drive app's `path` parameter.
#[test]
fn drive_link_encodes_the_folder_key() {
    let link = drive_link(
        "report.pdf",
        "beiner.gbai",
        "beiner.gbdrive/conversations/q3 2026",
    );
    let encoded = link
        .split("path=")
        .nth(1)
        .and_then(|rest| rest.split('&').next())
        .unwrap_or_default();
    let decoded = urlencoding::decode(encoded)
        .ok()
        .map(|value| value.into_owned())
        .unwrap_or_default();
    assert_eq!(decoded, "beiner.gbdrive/conversations/q3 2026");
    assert!(encoded.contains("%2F"), "slashes must be encoded: {link}");
    let bucket = link
        .split("bucket=")
        .nth(1)
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or_default();
    assert_eq!(bucket, "beiner.gbai");
}

#[test]
fn split_bucket_path_extracts_bucket_and_key() {
    let (bucket, key) = split_bucket_path("beiner.gbai/beiner.gbdrive/conversations/report.pdf");
    assert_eq!(bucket, "beiner.gbai");
    assert_eq!(key, "beiner.gbdrive/conversations/report.pdf");
}

/// `save_object` returns `bucket/key`; the Drive app addresses the key.
#[test]
fn relative_key_drops_the_bucket() {
    assert_eq!(
        relative_key("beiner.gbai/beiner.gbdrive/conversations/x/report.pdf"),
        "beiner.gbdrive/conversations/x/report.pdf"
    );
    assert_eq!(relative_key("no-slash"), "no-slash");
}
