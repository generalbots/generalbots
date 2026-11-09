//! #1643/#2047 — scope, capability and connection tests for the external
//! drive types. Split out of `types.rs` so that file stays under the size
//! limit.

use super::*;

    #[test]
    fn provider_round_trips_through_strings() {
        for p in Provider::ALL {
            assert_eq!(Provider::parse(p.as_str()), Some(p));
        }
        assert_eq!(Provider::parse("  OneDrive "), Some(Provider::OneDrive));
        assert_eq!(Provider::parse("google_drive"), Some(Provider::GoogleDrive));
        assert_eq!(Provider::parse("dropbox"), None);
        assert_eq!(Provider::parse(""), None);
    }

    #[test]
    fn file_path_is_normalized() {
        let f = ExternalFile::new("id1", "notes.txt", "/Docs//notes.txt", false);
        assert_eq!(f.path, "Docs/notes.txt");
        // An empty path falls back to the name so a provider that reports only
        // a name still gets a usable breadcrumb.
        let g = ExternalFile::new("id2", "root.txt", "", false);
        assert_eq!(g.path, "root.txt");
        let h = ExternalFile::new("id3", "docs", "/", true);
        assert_eq!(h.path, "docs");
        // Backslashes and repeated separators must not create two paths for
        // the same file — the listing groups on this string.
        let i = ExternalFile::new("id4", "notes.txt", "Docs\\\\notes.txt", false);
        assert_eq!(i.path, "Docs/notes.txt");
    }

    #[test]
    fn backoff_grows_and_respects_provider_hint() {
        // A provider hint is a floor, not a suggestion to retry immediately.
        let e = SyncError::RateLimited(Some(120));
        assert_eq!(e.backoff_secs(0), 120);
        assert!(e.backoff_secs(3) >= 120);
        assert!(e.backoff_secs(0) >= 5);
        assert!(SyncError::Transient("boom".into()).backoff_secs(9) <= 3600);
    }

    #[test]
    fn delta_page_defaults_are_empty() {
        let p = DeltaPage::default();
        assert!(p.items.is_empty() && p.removed.is_empty());
        assert!(p.next_cursor.is_none() && p.delta.is_none());
    }

    #[test]
    fn only_auth_errors_require_reconnect() {
        assert!(SyncError::Auth("revoked".into()).needs_reauth());
        assert!(!SyncError::RateLimited(None).needs_reauth());
        assert!(!SyncError::Transient("502".into()).needs_reauth());
        assert!(!SyncError::Permanent("bad request".into()).needs_reauth());
    }

    #[test]
    fn connection_due_respects_backoff_and_disconnect() {
        let now = chrono::Utc::now();
        let mut c = Connection {
            id: Uuid::new_v4(),
            branch_id: Uuid::nil(),
            provider: Provider::OneDrive,
            access_token: "t".into(),
            refresh_token: None,
            expires_at: None,
            cursor: None,
            status: "connected".into(),
            sync_minutes: 30,
            attempt_count: 0,
            last_sync: None,
            next_retry_at: None,
            last_error: None,
            account: None,
            scopes: None,
        };
        assert!(c.due(now));
        c.next_retry_at = Some(now + chrono::Duration::minutes(10));
        assert!(!c.due(now));
        c.next_retry_at = None;
        c.status = "disconnected".into();
        assert!(!c.due(now));
        c.status = "connected".into();
        c.expires_at = Some(now + chrono::Duration::minutes(1));
        assert!(c.token_expired(now));
    }

    #[test]
    fn granted_scopes_decide_write_capability() {
        let gdrive = Provider::GoogleDrive;
        assert!(!gdrive.granted_scopes_allow_write(""));
        assert!(!gdrive.granted_scopes_allow_write(
            "https://www.googleapis.com/auth/drive.readonly"
        ));
        assert!(gdrive.granted_scopes_allow_write(
            "https://www.googleapis.com/auth/drive.file"
        ));
        // Case-insensitive: providers do not agree on scope casing.
        assert!(gdrive.granted_scopes_allow_write(
            "HTTPS://WWW.GOOGLEAPIS.COM/AUTH/DRIVE.FILE"
        ));

        let mut connection = Connection {
            id: Uuid::new_v4(),
            branch_id: Uuid::nil(),
            provider: Provider::OneDrive,
            access_token: "t".into(),
            refresh_token: None,
            expires_at: None,
            cursor: None,
            status: "connected".into(),
            sync_minutes: 30,
            attempt_count: 0,
            last_sync: None,
            next_retry_at: None,
            last_error: None,
            account: None,
            scopes: Some("Files.Read.All".into()),
        };
        assert!(!connection.can_write());
        connection.scopes =
            Some("Files.Read.All Files.ReadWrite.AppFolder".to_string());
        assert!(connection.can_write());
    }

    #[test]
    fn failed_report_keeps_counters() {
        let r = SyncReport {
            upserted: 3,
            removed: 1,
            ..Default::default()
        }
        .failed(&SyncError::RateLimited(None));
        assert!(!r.ok());
        assert_eq!(r.upserted, 3);
        assert_eq!(r.removed, 1);
        assert_eq!(r.error_kind.as_deref(), Some("rate_limited"));
    }
