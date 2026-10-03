//! Shared types for the external-drive backends.
//!
//! Both providers are polled through their **native change feed** rather than
//! a full listing: OneDrive exposes `delta`, Google Drive exposes `changes`.
//! Each returns the same shape after parsing — a page of items plus the cursor
//! to resume from — so the sync engine never needs to know which provider it
//! is talking to.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A supported external file provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    /// Microsoft OneDrive (Microsoft Graph).
    OneDrive,
    /// Google Drive (Drive API v3).
    GoogleDrive,
}

impl Provider {
    /// Database representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OneDrive => "onedrive",
            Self::GoogleDrive => "gdrive",
        }
    }

    /// Human label shown in the UI.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::OneDrive => "OneDrive",
            Self::GoogleDrive => "Google Drive",
        }
    }

    /// Parse a database/API value. Returns `None` for anything unknown so a
    /// corrupt row cannot panic the sync loop.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "onedrive" | "m365" | "o365" => Some(Self::OneDrive),
            "gdrive" | "google_drive" | "googledrive" | "google" => Some(Self::GoogleDrive),
            _ => None,
        }
    }

    /// Every provider, in display order.
    pub const ALL: [Self; 2] = [Self::OneDrive, Self::GoogleDrive];
}

/// One file or folder as reported by a provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalFile {
    /// Provider-assigned identifier (`id` in Graph, `fileId`/`id` in Drive).
    pub remote_id: String,
    /// File name.
    pub name: String,
    /// Full path relative to the provider root, `/`-separated, no leading slash.
    pub path: String,
    /// MIME type, when the provider reports one.
    pub mime_type: Option<String>,
    /// Size in bytes; folders report 0.
    pub size_bytes: i64,
    /// Last modification, when reported.
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    /// True when the item is a folder.
    pub is_folder: bool,
    /// Opaque version marker (ETag / `revision`), used to detect changes.
    pub revision: Option<String>,
    /// Parent folder's remote id, when reported.
    pub parent_remote_id: Option<String>,
}

impl ExternalFile {
    /// Build an item, normalizing the path: backslashes folded to `/`, empty
    /// segments dropped (so `/Docs//notes.txt` and `Docs/notes.txt` are one
    /// path) and an empty result defaulting to the name.
    pub fn new(
        remote_id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
        is_folder: bool,
    ) -> Self {
        let name = name.into();
        let raw = path.into().replace('\\', "/");
        let normalized: Vec<&str> = raw.split('/').filter(|s| !s.is_empty()).collect();
        let path = if normalized.is_empty() {
            name.clone()
        } else {
            normalized.join("/")
        };
        Self {
            remote_id: remote_id.into(),
            name,
            path,
            mime_type: None,
            size_bytes: 0,
            modified_at: None,
            is_folder,
            revision: None,
            parent_remote_id: None,
        }
    }
}

/// A single page of a provider change feed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeltaPage {
    /// Items changed since the last cursor (additions and updates).
    pub items: Vec<ExternalFile>,
    /// Remote ids the provider reported as deleted.
    pub removed: Vec<String>,
    /// What to call next within the same run. `None` on the last page.
    ///
    /// Provider-shaped: a full URL for OneDrive (`@odata.nextLink`), a bare
    /// page token for Google Drive. Only the owning provider interprets it.
    pub next_cursor: Option<String>,
    /// Cursor to persist once the run completes. `Some` only on the page that
    /// closes a run (Graph's `@odata.deltaLink`).
    pub delta: Option<String>,
}

/// Why a provider call failed. The distinction drives backoff: a rate-limited
/// or transient provider is retried, an auth failure needs the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// Provider asked us to slow down. Carries the retry-after hint, seconds.
    RateLimited(Option<u64>),
    /// Token rejected or revoked — the user must reconnect.
    Auth(String),
    /// Network or provider 5xx — retry with backoff.
    Transient(String),
    /// The request itself is wrong; retrying cannot help.
    Permanent(String),
}

impl SyncError {
    /// Short machine-readable tag for persistence and the UI.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::RateLimited(_) => "rate_limited",
            Self::Auth(_) => "auth",
            Self::Transient(_) => "transient",
            Self::Permanent(_) => "permanent",
        }
    }

    /// True when the connection must ask the user to reconnect.
    pub const fn needs_reauth(&self) -> bool {
        matches!(self, Self::Auth(_))
    }

    /// Backoff before the next attempt, in seconds. Provider hints win.
    pub fn backoff_secs(&self, attempt: u32) -> u64 {
        let base = match self {
            Self::RateLimited(Some(secs)) => (*secs).max(1),
            Self::RateLimited(None) => 60,
            Self::Transient(_) => 30,
            Self::Auth(_) => 3600,
            Self::Permanent(_) => 3600,
        };
        // Exponential up to an hour; the provider hint is never shortened.
        let scaled = base.saturating_mul(1u64 << attempt.min(6));
        scaled.clamp(5, 3600)
    }
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited(Some(secs)) => write!(f, "rate limited (retry after {secs}s)"),
            Self::RateLimited(None) => write!(f, "rate limited"),
            Self::Auth(m) => write!(f, "auth: {m}"),
            Self::Transient(m) => write!(f, "transient: {m}"),
            Self::Permanent(m) => write!(f, "permanent: {m}"),
        }
    }
}

impl std::error::Error for SyncError {}

/// A provider connection owned by one branch.
#[derive(Debug, Clone)]
pub struct Connection {
    /// Row id.
    pub id: Uuid,
    /// Owning branch (tenant isolation — every query is bounded by it).
    pub branch_id: Uuid,
    /// Which provider.
    pub provider: Provider,
    /// Access token, decrypted.
    pub access_token: String,
    /// Refresh token, decrypted, when the provider issued one.
    pub refresh_token: Option<String>,
    /// Access-token expiry.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Provider change-feed cursor.
    pub cursor: Option<String>,
    /// `connected` | `reauth_required` | `error`.
    pub status: String,
    /// Requested sync cadence, minutes.
    pub sync_minutes: i32,
    /// Consecutive failures — drives the exponential backoff.
    pub attempt_count: i32,
    /// Last successful sync.
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    /// Do not attempt before this instant.
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last error message (never a token).
    pub last_error: Option<String>,
    /// Account label reported by the provider.
    pub account: Option<String>,
}

impl Connection {
    /// True when the access token is expired or about to expire.
    pub fn token_expired(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        self.expires_at
            .map(|at| now + chrono::Duration::minutes(5) >= at)
            .unwrap_or(false)
    }

    /// True when the connection may be synced right now.
    pub fn due(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        if self.status == "disconnected" {
            return false;
        }
        self.next_retry_at.map(|at| now >= at).unwrap_or(true)
    }
}

/// Outcome of one sync run, reported to the caller and to the UI.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncReport {
    /// Items upserted.
    pub upserted: usize,
    /// Items removed.
    pub removed: usize,
    /// Pages fetched.
    pub pages: usize,
    /// Set when the run failed; `kind` says whether a reconnect is needed.
    pub error: Option<String>,
    /// Machine-readable error kind.
    pub error_kind: Option<String>,
}

impl SyncReport {
    /// A failed run keeps the counters it managed to reach before the failure.
    pub fn failed(mut self, err: &SyncError) -> Self {
        self.error = Some(err.to_string());
        self.error_kind = Some(err.kind().to_string());
        self
    }

    /// True when the run completed without error.
    pub const fn ok(&self) -> bool {
        self.error.is_none()
    }
}

#[cfg(test)]
mod tests {
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
    fn failed_report_keeps_counters() {
        let r = SyncReport { upserted: 3, removed: 1, ..Default::default() }
            .failed(&SyncError::RateLimited(None));
        assert!(!r.ok());
        assert_eq!(r.upserted, 3);
        assert_eq!(r.removed, 1);
        assert_eq!(r.error_kind.as_deref(), Some("rate_limited"));
    }
}