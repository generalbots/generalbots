//! Unit tests for the Google Drive client: page parsing, error classification
//! and the consent URL. Split out to keep `gdrive.rs` focused on the client.

use super::*;
use serde_json::json;

#[test]
fn files_list_page_is_parsed() {
        let json = json!({
            "nextPageToken": "tok2",
            "files": [
                {
                    "id": "f1",
                    "name": "contract.pdf",
                    "mimeType": "application/pdf",
                    "size": "5120",
                    "modifiedTime": "2026-09-30T08:00:00Z",
                    "version": "7",
                    "parents": ["root"]
                },
                {
                    "id": "f2",
                    "name": "Photos",
                    "mimeType": FOLDER_MIME
                }
            ]
        });
        let page = parse_page(&json);
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.next_cursor.as_deref(), Some("tok2"));
        // Google reports size as a string; it must become a number.
        assert_eq!(page.items[0].size_bytes, 5120);
        assert_eq!(page.items[0].revision.as_deref(), Some("7"));
        assert_eq!(page.items[0].parent_remote_id.as_deref(), Some("root"));
        assert!(page.items[1].is_folder);
        assert!(page.items[0].modified_at.is_some());
}

#[test]
fn changes_page_separates_removals_from_updates() {
        let json = json!({
            "newStartPageToken": "start-99",
            "changes": [
                { "fileId": "gone", "removed": true },
                { "fileId": "trashed", "removed": false,
                  "file": { "id": "trashed", "name": "old.txt", "trashed": true } },
                { "fileId": "live", "removed": false,
                  "file": { "id": "live", "name": "new.txt", "mimeType": "text/plain" } }
            ]
        });
        let page = parse_page(&json);
        assert_eq!(page.removed, vec!["gone".to_string(), "trashed".to_string()]);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].remote_id, "live");
        assert_eq!(page.delta.as_deref(), Some("start-99"));
        assert!(page.next_cursor.is_none());
}

#[test]
fn google_403_is_a_rate_limit_only_when_the_body_says_so() {
        assert!(matches!(
            classify_status(403, r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"rateLimitExceeded"}}"#),
            SyncError::RateLimited(None)
        ));
        // A plain permission error must ask the user to reconnect, not retry.
        assert!(classify_status(403, r#"{"error":"insufficientFilePermissions"}"#).needs_reauth());
        assert!(classify_status(401, "invalid token").needs_reauth());
        assert!(matches!(classify_status(429, ""), SyncError::RateLimited(None)));
        assert!(matches!(classify_status(500, "oops"), SyncError::Transient(_)));
}

#[test]
fn authorize_url_requests_offline_access_and_readonly_scope() {
        let url = authorize_url("cid.apps.googleusercontent.com", "https://host/cb", "st/1");
        assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
        assert!(url.contains("drive.readonly"));
        assert!(url.contains("state=st%2F1"));
}

#[test]
fn entries_without_a_name_are_skipped() {
        let page = parse_page(&json!({ "files": [{ "id": "x" }, { "name": "" }] }));
        assert!(page.items.is_empty());
}
