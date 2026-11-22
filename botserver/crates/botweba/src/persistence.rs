//! Durable state for the WebA builder (#2538).
//!
//! The old builder kept everything in process-local `RwLock<HashMap<..>>`, so a
//! published site disappeared on the next restart. All state now lives in a
//! [`WebaSnapshot`] that a [`WebaStore`] loads on boot and writes after every
//! mutation. Three stores are provided:
//!
//! * [`DbStore`] — one JSONB row per branch in `weba_snapshots` (production).
//! * [`FileStore`] — one JSON file per branch (desktop-local / explicit opt-in).
//! * [`MemoryStore`] — no durability, used by unit tests and as a safe fallback.
//!
//! `WebaState::new()` picks the store from the environment, so no change to the
//! router wiring outside this crate is required for reload-on-boot to work.

use super::*;
use std::path::PathBuf;
use std::sync::Mutex;

/// An immutable, append-only copy of a site taken at publish time.
///
/// Rendering reads this instead of the draft pages, so editing a draft after a
/// publish can never alter the live site. `version` increases on every publish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedSite {
    pub app_id: Uuid,
    pub slug: String,
    pub name: String,
    pub version: i64,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub pages: Vec<WebAppPage>,
}

/// One captured form submission. `email` is the dedupe key inside a site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSubmission {
    pub id: Uuid,
    pub site_id: Uuid,
    pub slug: String,
    pub email: String,
    pub name: Option<String>,
    pub message: Option<String>,
    pub fields: HashMap<String, String>,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A click on a campaign link to a published page (#2538). `campaign_id` is
/// the id used by the marketing campaign the link was generated from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignClick {
    pub app_id: Uuid,
    pub slug: String,
    pub campaign_id: String,
    pub path: String,
    pub clicked_at: chrono::DateTime<chrono::Utc>,
}

/// Everything the builder owns, serialised as one unit per branch.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebaSnapshot {
    pub apps: Vec<WebApp>,
    pub pages: Vec<WebAppPage>,
    pub published: Vec<PublishedSite>,
    pub submissions: Vec<FormSubmission>,
    #[serde(default)]
    pub clicks: Vec<CampaignClick>,
    pub publish_counter: i64,
}

/// Store failures never panic: callers log and keep serving in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebaStoreError {
    message: String,
}

impl WebaStoreError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WebaStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WebaStoreError {}

/// Persistence boundary. Implementations must be branch-scoped: passing a
/// different `branch_id` must never return or overwrite another workspace's
/// snapshot.
pub trait WebaStore: Send + Sync {
    fn load(&self, branch_id: Uuid) -> Result<Option<WebaSnapshot>, WebaStoreError>;
    fn save(&self, branch_id: Uuid, snapshot: &WebaSnapshot) -> Result<(), WebaStoreError>;
}

/// No durability. Used in tests and when neither DB nor file storage is set up.
#[derive(Default)]
pub struct MemoryStore {
    snapshot: Mutex<Option<WebaSnapshot>>,
}

impl WebaStore for MemoryStore {
    fn load(&self, _branch_id: Uuid) -> Result<Option<WebaSnapshot>, WebaStoreError> {
        Ok(self.snapshot.lock().map_err(|_| WebaStoreError::new("snapshot lock poisoned"))?.clone())
    }

    fn save(&self, _branch_id: Uuid, snapshot: &WebaSnapshot) -> Result<(), WebaStoreError> {
        let mut guard = self.snapshot.lock().map_err(|_| WebaStoreError::new("snapshot lock poisoned"))?;
        *guard = Some(snapshot.clone());
        Ok(())
    }
}

/// Durable JSON snapshot under a caller supplied directory, one file per branch.
pub struct FileStore {
    directory: PathBuf,
}

impl FileStore {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn path_for(&self, branch_id: Uuid) -> PathBuf {
        self.directory.join(format!("weba-{branch_id}.json"))
    }
}

impl WebaStore for FileStore {
    fn load(&self, branch_id: Uuid) -> Result<Option<WebaSnapshot>, WebaStoreError> {
        let path = self.path_for(branch_id);
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(WebaStoreError::new(format!("reading {}: {error}", path.display()))),
        };
        serde_json::from_str(&raw)
            .map(Some)
            .map_err(|error| WebaStoreError::new(format!("parsing {}: {error}", path.display())))
    }

    fn save(&self, branch_id: Uuid, snapshot: &WebaSnapshot) -> Result<(), WebaStoreError> {
        std::fs::create_dir_all(&self.directory)
            .map_err(|error| WebaStoreError::new(format!("creating {}: {error}", self.directory.display())))?;
        let path = self.path_for(branch_id);
        let raw = serde_json::to_string_pretty(snapshot)
            .map_err(|error| WebaStoreError::new(format!("encoding snapshot: {error}")))?;
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, raw)
            .map_err(|error| WebaStoreError::new(format!("writing {}: {error}", temporary.display())))?;
        std::fs::rename(&temporary, &path)
            .map_err(|error| WebaStoreError::new(format!("committing {}: {error}", path.display())))
    }
}

/// Production store: the branch-scoped `weba_snapshots` row created by
/// migration `6.8.20-weba-sites`. Uses the process-wide pool so boot wiring in
/// `botserver/src` does not have to change for sites to survive a restart.
pub struct DbStore {
    pool: botcore::shared::utils::DbPool,
}

impl DbStore {
    pub fn new(pool: botcore::shared::utils::DbPool) -> Self {
        Self { pool }
    }

    /// Returns `None` when no database is configured (tests, offline shell).
    pub fn try_default() -> Option<Self> {
        if std::env::var_os("DATABASE_URL").is_none() {
            return None;
        }
        botcore::shared::utils::pool()
            .ok()
            .map(|pool| Self::new((*pool).clone()))
    }
}

#[derive(diesel::QueryableByName)]
struct PayloadRow {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    payload: serde_json::Value,
}

impl WebaStore for DbStore {
    fn load(&self, branch_id: Uuid) -> Result<Option<WebaSnapshot>, WebaStoreError> {
        use diesel::prelude::*;
        let mut conn = self
            .pool
            .get()
            .map_err(|error| WebaStoreError::new(format!("pool: {error}")))?;
        let rows: Vec<PayloadRow> = diesel::sql_query(
            "SELECT payload FROM weba_snapshots WHERE branch_id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load(&mut conn)
        .map_err(|error| WebaStoreError::new(format!("loading weba snapshot: {error}")))?;
        match rows.into_iter().next() {
            Some(row) => serde_json::from_value(row.payload)
                .map(Some)
                .map_err(|error| WebaStoreError::new(format!("decoding weba snapshot: {error}"))),
            None => Ok(None),
        }
    }

    fn save(&self, branch_id: Uuid, snapshot: &WebaSnapshot) -> Result<(), WebaStoreError> {
        use diesel::prelude::*;
        let payload = serde_json::to_value(snapshot)
            .map_err(|error| WebaStoreError::new(format!("encoding weba snapshot: {error}")))?;
        let mut conn = self
            .pool
            .get()
            .map_err(|error| WebaStoreError::new(format!("pool: {error}")))?;
        diesel::sql_query(
            "INSERT INTO weba_snapshots (branch_id, payload, updated_at) \
             VALUES ($1, $2, NOW()) \
             ON CONFLICT (branch_id) DO UPDATE SET payload = EXCLUDED.payload, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Jsonb, _>(payload)
        .execute(&mut conn)
        .map_err(|error| WebaStoreError::new(format!("saving weba snapshot: {error}")))?;
        Ok(())
    }
}
