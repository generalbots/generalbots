use super::*;
use crate::external::types::SyncError;
use serde_json::json;

#[test]
fn delta_page_splits_updates_and_deletions() {
    let json = json!({
        "value": [
            {
                "id": "01ABC",
                "name": "invoice.pdf",
                "size": 2048,
                "lastModifiedDateTime": "2026-10-01T10:00:00Z",
                "eTag": "\"tag-1\"",
                "file": { "mimeType": "application/pdf" },
                "parentReference": { "id": "root!", "path": "/drive/root:/Reports" }
            },
            { "id": "01GONE", "deleted": { "state": "deleted" } },
            { "name": "no-id" }
        ]
    });
    let page = parse_delta(&json);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.removed, vec!["01GONE".to_string()]);
    let f = &page.items[0];
    assert_eq!(f.remote_id, "01ABC");
    assert_eq!(f.path, "drive/root:/Reports/invoice.pdf");
    assert_eq!(f.size_bytes, 2048);
    assert_eq!(f.mime_type.as_deref(), Some("application/pdf"));
    assert_eq!(f.revision.as_deref(), Some("tag-1"));
    assert!(!f.is_folder);
    assert!(f.modified_at.is_some());
}

#[test]
fn delta_page_recognizes_folders_and_cursors() {
    let paging = json!({
        "@odata.nextLink": "https://graph.microsoft.com/v1.0/me/drive/root/delta?token=2",
        "value": [{ "id": "D1", "name": "Docs", "folder": { "childCount": 3 } }]
    });
    let p = parse_delta(&paging);
    assert!(p.items[0].is_folder);
    assert_eq!(p.items[0].size_bytes, 0);
    assert_eq!(
        p.next_cursor.as_deref(),
        Some("https://graph.microsoft.com/v1.0/me/drive/root/delta?token=2")
    );
    assert!(p.delta.is_none());

    let done = json!({
        "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/drive/root/delta?token=9",
        "value": []
    });
    let d = parse_delta(&done);
    assert!(d.next_cursor.is_none());
    assert!(d.delta.is_some());
}

#[test]
fn malformed_timestamps_do_not_break_the_page() {
    let json = json!({
        "value": [{ "id": "1", "name": "a.txt", "lastModifiedDateTime": "not-a-date" }]
    });
    let page = parse_delta(&json);
    assert_eq!(page.items.len(), 1);
    assert!(page.items[0].modified_at.is_none());
}

#[test]
fn status_classification_drives_retry_behavior() {
    assert!(matches!(
        classify_status(401, "token expired"),
        SyncError::Auth(_)
    ));
    assert!(matches!(
        classify_status(403, "no consent"),
        SyncError::Auth(_)
    ));
    assert!(matches!(
        classify_status(429, ""),
        SyncError::RateLimited(None)
    ));
    assert!(matches!(
        classify_status(503, "busy"),
        SyncError::Transient(_)
    ));
    assert!(matches!(
        classify_status(400, "bad"),
        SyncError::Permanent(_)
    ));
}

#[test]
fn authorize_url_carries_state_and_offline_scope() {
    let url = authorize_url("tid", "cid", "https://host/cb", "st 1/2");
    assert!(url.starts_with("https://login.microsoftonline.com/tid/oauth2/v2.0/authorize"));
    assert!(url.contains("state=st%201%2F2"));
    assert!(url
        .contains("scope=offline_access%20Files.Read.All%20Files.ReadWrite.AppFolder%20User.Read"));
    assert!(url.contains("response_type=code"));
}

#[test]
fn empty_body_parses_to_an_empty_page() {
    let page = parse_delta(&json!({}));
    assert!(page.items.is_empty() && page.removed.is_empty());
    assert!(page.delta.is_none());
}
