use super::*;

use super::*;

#[test]
fn provider_parsing_rejects_unknown_values() {
    assert_eq!(parse_provider("onedrive").unwrap(), Provider::OneDrive);
    assert_eq!(parse_provider("gdrive").unwrap(), Provider::GoogleDrive);
    assert_eq!(parse_provider("o365").unwrap(), Provider::OneDrive);
    assert!(parse_provider("dropbox").is_err());
    assert!(parse_provider("").is_err());
}

#[test]
fn filenames_cannot_escape_the_header() {
    assert_eq!(sanitize_filename("report\".pdf"), "report.pdf");
    assert_eq!(sanitize_filename("a\\b.txt"), "ab.txt");
    assert_eq!(sanitize_filename("line\nbreak"), "linebreak");
    assert_eq!(sanitize_filename("   "), "download");
    assert_eq!(sanitize_filename("ok.txt"), "ok.txt");
}

#[test]
fn reason_codes_are_url_encoded() {
    assert_eq!(urlencode("access_denied"), "access_denied");
    assert_eq!(urlencode("a b&c"), "a%20b%26c");
}
