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

    /// Scopes that allow writing files back (#2047). Google uses the
    /// per-file `drive.file` scope (the least invasive write grant: the app
    /// can only touch what it created or the user picked); OneDrive uses the
    /// app folder so a save cannot touch the rest of the account.
    pub const fn write_scopes(self) -> &'static [&'static str] {
        match self {
            Self::OneDrive => &["Files.ReadWrite.AppFolder"],
            Self::GoogleDrive => &["https://www.googleapis.com/auth/drive.file"],
        }
    }

    /// OAuth scopes requested at connect time (#1643). The consent screen and
    /// the connected card render exactly this list, so what the user is asked
    /// to allow is what the deployment actually requests — no hand-written
    /// marketing copy.
    ///
    /// #2047 — read **and** write are requested in one consent so a save target
    /// exists from the first connection. A connection created before this
    /// change keeps its recorded (read-only) scopes and reports
    /// `can_write: false` until the user grants write access by reconnecting.
    pub const fn scopes(self) -> &'static [&'static str] {
        match self {
            Self::OneDrive => &[
                "offline_access",
                "Files.Read.All",
                "Files.ReadWrite.AppFolder",
                "User.Read",
            ],
            Self::GoogleDrive => &[
                "https://www.googleapis.com/auth/drive.readonly",
                "https://www.googleapis.com/auth/drive.file",
            ],
        }
    }

    /// Plain-language description of each requested scope, in the same order
    /// as [`Self::scopes`].
    pub const fn scope_descriptions(self) -> &'static [&'static str] {
        match self {
            Self::OneDrive => &[
                "Stay connected and refresh the token without asking again",
                "Read every file and folder in the account",
                "Create files in this app's own OneDrive folder (saving artifacts)",
                "Read the account's name and e-mail",
            ],
            Self::GoogleDrive => &[
                "Read every file and folder in Google Drive",
                "Create files in Google Drive and edit only the files this app created",
            ],
        }
    }

    /// Whether a recorded consent (the space-separated `scope` the provider
    /// returned) includes a write grant. An empty or missing record answers
    /// `false`: capability is never inferred, only read from the grant.
    pub fn granted_scopes_allow_write(self, granted: &str) -> bool {
        let granted: Vec<String> = granted
            .split_whitespace()
            .map(|scope| scope.to_ascii_lowercase())
            .collect();
        if granted.is_empty() {
            return false;
        }
        self.write_scopes()
            .iter()
            .any(|scope| granted.iter().any(|value| value == scope))
    }
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
    /// #1643/#2047 — the space-separated scopes the provider reported when the
    /// user consented. Capability (`can_write`) is derived from this record,
    /// never guessed.
    pub scopes: Option<String>,
}

impl Connection {
    /// True when the access token is expired or about to expire.
    pub fn token_expired(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        self.expires_at
            .map(|at| now + chrono::Duration::minutes(5) >= at)
            .unwrap_or(false)
    }

    /// #2047 — true when this connection consented to a write scope.
    pub fn can_write(&self) -> bool {
        self.provider
            .granted_scopes_allow_write(self.scopes.as_deref().unwrap_or(""))
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
#[path = "types_tests.rs"]
mod tests;
