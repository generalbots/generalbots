use super::*;
use crate::forms::normalize_email;
use crate::persistence::{
    DbStore, FileStore, FormSubmission, MemoryStore, PublishedSite, WebaSnapshot, WebaStore,
};
use std::path::PathBuf;

/// Runtime state plus its durable backing store (#2538).
///
/// The maps remain for cheap reads; every mutation is mirrored to the store so
/// a restart rebuilds them from the last snapshot.
pub struct WebaState {
    pub(crate) apps: RwLock<HashMap<Uuid, WebApp>>,
    pub(crate) pages: RwLock<HashMap<Uuid, WebAppPage>>,
    pub(crate) published: RwLock<HashMap<Uuid, PublishedSite>>,
    pub(crate) submissions: RwLock<Vec<FormSubmission>>,
    pub(crate) publish_counter: std::sync::atomic::AtomicI64,
    pub(crate) rate_limiter: crate::forms::RateLimiter,
    pub(crate) branch_id: Uuid,
    store: Arc<dyn WebaStore>,
    save_lock: tokio::sync::Mutex<()>,
}

impl WebaState {
    /// Production constructor. Picks the most durable store available from the
    /// environment and reloads the branch snapshot before the first request.
    #[must_use]
    pub fn new() -> Self {
        let branch_id = branch_from_environment();
        if let Some(store) = DbStore::try_default() {
            return Self::with_store(Arc::new(store), branch_id);
        }
        if let Some(directory) = std::env::var_os("WEBA_SNAPSHOT_DIR") {
            return Self::with_store(Arc::new(FileStore::new(PathBuf::from(directory))), branch_id);
        }
        Self::with_store(Arc::new(MemoryStore::default()), branch_id)
    }

    /// Builds state around an explicit store and immediately reloads it.
    ///
    /// Loading happens synchronously before the locks exist, so boot never
    /// needs to nest a Tokio runtime inside the server's runtime.
    #[must_use]
    pub fn with_store(store: Arc<dyn WebaStore>, branch_id: Uuid) -> Self {
        let snapshot = match store.load(branch_id) {
            Ok(Some(snapshot)) => snapshot,
            Ok(None) => WebaSnapshot::default(),
            Err(error) => {
                log::error!("WEBA reload failed for branch {branch_id}: {error}");
                WebaSnapshot::default()
            }
        };
        let counter = snapshot.publish_counter;
        let apps = snapshot.apps.into_iter().map(|app| (app.id, app)).collect();
        let pages = snapshot.pages.into_iter().map(|page| (page.id, page)).collect();
        let published = snapshot
            .published
            .into_iter()
            .map(|site| (site.app_id, site))
            .collect();
        Self {
            apps: RwLock::new(apps),
            pages: RwLock::new(pages),
            published: RwLock::new(published),
            submissions: RwLock::new(snapshot.submissions),
            publish_counter: std::sync::atomic::AtomicI64::new(counter),
            rate_limiter: crate::forms::RateLimiter::new(5, 1.0 / 60.0),
            branch_id,
            store,
            save_lock: tokio::sync::Mutex::new(()),
        }
    }

    /// Convenience constructor used by unit tests: no durability.
    #[must_use]
    pub fn with_memory(branch_id: Uuid) -> Self {
        Self::with_store(Arc::new(MemoryStore::default()), branch_id)
    }

    /// File-backed constructor used by reload-on-boot tests and desktop mode.
    #[must_use]
    pub fn with_file_store(directory: impl Into<PathBuf>, branch_id: Uuid) -> Self {
        Self::with_store(Arc::new(FileStore::new(directory)), branch_id)
    }

    /// Replaces every in-memory map with the latest durable snapshot.
    /// Boot uses the synchronous load in [`Self::with_store`]; this explicit
    /// form is available for hot reloads and tests.
    pub async fn reload(&self) {
        let snapshot = match self.store.load(self.branch_id) {
            Ok(Some(snapshot)) => snapshot,
            Ok(None) => return,
            Err(error) => {
                log::error!("WEBA reload failed for branch {}: {error}", self.branch_id);
                return;
            }
        };
        let mut apps = self.apps.write().await;
        apps.clear();
        for app in snapshot.apps {
            apps.insert(app.id, app);
        }
        let mut pages = self.pages.write().await;
        pages.clear();
        for page in snapshot.pages {
            pages.insert(page.id, page);
        }
        let mut published = self.published.write().await;
        published.clear();
        for site in snapshot.published {
            published.insert(site.app_id, site);
        }
        let mut submissions = self.submissions.write().await;
        *submissions = snapshot.submissions;
        self.publish_counter.store(
            snapshot.publish_counter,
            std::sync::atomic::Ordering::SeqCst,
        );
    }

    fn current_snapshot(&self) -> impl std::future::Future<Output = WebaSnapshot> + '_ {
        async move {
            let apps = self.apps.read().await;
            let pages = self.pages.read().await;
            let published = self.published.read().await;
            let submissions = self.submissions.read().await;
            WebaSnapshot {
                apps: apps.values().cloned().collect(),
                pages: pages.values().cloned().collect(),
                published: published.values().cloned().collect(),
                submissions: submissions.clone(),
                publish_counter: self.publish_counter.load(std::sync::atomic::Ordering::SeqCst),
            }
        }
    }

    /// Writes the whole branch state as one atomic snapshot.
    pub async fn persist(&self) {
        let _guard = self.save_lock.lock().await;
        let snapshot = self.current_snapshot().await;
        if let Err(error) = self.store.save(self.branch_id, &snapshot) {
            log::error!("WEBA persist failed for branch {}: {error}", self.branch_id);
        }
    }

    /// Freezes the current draft pages into an immutable version and flips the
    /// app to published. Repeated publishes create new versions; drafts never
    /// mutate an already published snapshot.
    pub async fn publish(&self, app_id: Uuid) -> Option<PublishedSite> {
        let version = self.publish_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let (slug, name) = {
            let apps = self.apps.read().await;
            let app = apps.get(&app_id)?;
            (app.slug.clone(), app.name.clone())
        };
        let pages = {
            let pages = self.pages.read().await;
            pages.values().filter(|page| page.app_id == app_id).cloned().collect::<Vec<_>>()
        };
        let site = PublishedSite {
            app_id,
            slug,
            name,
            version,
            published_at: chrono::Utc::now(),
            pages,
        };
        self.published.write().await.insert(app_id, site.clone());
        if let Some(app) = self.apps.write().await.get_mut(&app_id) {
            app.status = WebAppStatus::Published;
            app.updated_at = chrono::Utc::now();
        }
        self.persist().await;
        Some(site)
    }

    /// Removes the live snapshot and returns the app to draft.
    pub async fn unpublish(&self, app_id: Uuid) -> bool {
        let removed = self.published.write().await.remove(&app_id).is_some();
        let mut apps = self.apps.write().await;
        if let Some(app) = apps.get_mut(&app_id) {
            app.status = WebAppStatus::Draft;
            app.updated_at = chrono::Utc::now();
        }
        drop(apps);
        if removed {
            self.persist().await;
        }
        removed
    }

    /// Returns the published snapshot for a slug, if its app is still live.
    /// The snapshot owns the slug, so renaming a draft cannot repoint a live URL.
    pub(crate) async fn live_site(&self, slug: &str) -> Option<PublishedSite> {
        let site = {
            let published = self.published.read().await;
            published.values().find(|site| site.slug == slug).cloned()?
        };
        let live = self
            .apps
            .read()
            .await
            .get(&site.app_id)
            .is_some_and(|app| matches!(app.status, WebAppStatus::Published));
        if live {
            Some(site)
        } else {
            None
        }
    }

    /// Adds or updates a submission, keyed by normalised e-mail within a slug.
    pub async fn record_submission(&self, submission: FormSubmission) {
        let mut submissions = self.submissions.write().await;
        let email = normalize_email(&submission.email);
        if let Some(existing) = submissions
            .iter_mut()
            .find(|item| item.slug == submission.slug && item.email == email)
        {
            *existing = submission;
        } else {
            submissions.push(submission);
        }
        drop(submissions);
        self.persist().await;
    }
}

impl Default for WebaState {
    fn default() -> Self {
        Self::new()
    }
}

fn branch_from_environment() -> Uuid {
    std::env::var("WEBA_BRANCH_ID")
        .ok()
        .and_then(|value| Uuid::parse_str(&value).ok())
        .unwrap_or_else(Uuid::nil)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{upsert_submission, FormSubmitRequest};
    use crate::persistence::FormSubmission;

    fn app(id: Uuid, slug: &str) -> WebApp {
        WebApp {
            id,
            name: "Test".to_string(),
            slug: slug.to_string(),
            description: None,
            template: WebAppTemplate::Blank,
            status: WebAppStatus::Draft,
            config: WebAppConfig::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn page(id: Uuid, app_id: Uuid, content: &str) -> WebAppPage {
        WebAppPage {
            id,
            app_id,
            path: "/".to_string(),
            title: "Home".to_string(),
            content: content.to_string(),
            layout: None,
            is_index: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn request() -> FormSubmitRequest {
        FormSubmitRequest {
            email: "Person@Example.com".to_string(),
            name: Some("Person".to_string()),
            message: None,
            fields: HashMap::new(),
            website: None,
        }
    }

    #[tokio::test]
    async fn persists_and_reloads_from_file() {
        let directory = std::env::temp_dir().join(format!("weba-test-{}", Uuid::new_v4()));
        let branch = Uuid::new_v4();
        let state = WebaState::with_file_store(&directory, branch);
        let app_id = Uuid::new_v4();
        state.apps.write().await.insert(app_id, app(app_id, "site"));
        let page_id = Uuid::new_v4();
        state.pages.write().await.insert(page_id, page(page_id, app_id, "v1"));
        state.persist().await;

        let reloaded = WebaState::with_file_store(&directory, branch);
        assert_eq!(reloaded.apps.read().await.len(), 1);
        assert_eq!(reloaded.pages.read().await.len(), 1);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[tokio::test]
    async fn branch_scoping_is_respected() {
        let directory = std::env::temp_dir().join(format!("weba-test-{}", Uuid::new_v4()));
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let state = WebaState::with_file_store(&directory, first);
        let app_id = Uuid::new_v4();
        state.apps.write().await.insert(app_id, app(app_id, "private"));
        state.persist().await;

        let other = WebaState::with_file_store(&directory, second);
        assert!(other.apps.read().await.is_empty());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[tokio::test]
    async fn publish_freezes_an_immutable_snapshot() {
        let state = WebaState::with_memory(Uuid::nil());
        let app_id = Uuid::new_v4();
        let page_id = Uuid::new_v4();
        state.apps.write().await.insert(app_id, app(app_id, "site"));
        state.pages.write().await.insert(page_id, page(page_id, app_id, "published text"));

        let first = state.publish(app_id).await.expect("publish creates a snapshot");
        assert_eq!(first.version, 1);
        state.pages.write().await.get_mut(&page_id).expect("page exists").content = "draft edit".to_string();
        let live = state.live_site("site").await.expect("site is live");
        assert_eq!(live.pages[0].content, "published text");
        let second = state.publish(app_id).await.expect("republish creates a new version");
        assert_eq!(second.version, 2);
    }

    #[tokio::test]
    async fn unpublish_removes_the_live_snapshot() {
        let state = WebaState::with_memory(Uuid::nil());
        let app_id = Uuid::new_v4();
        state.apps.write().await.insert(app_id, app(app_id, "site"));
        assert!(state.publish(app_id).await.is_some());
        assert!(state.unpublish(app_id).await);
        assert!(state.live_site("site").await.is_none());
    }

    #[tokio::test]
    async fn submissions_dedupe_by_email() {
        let state = WebaState::with_memory(Uuid::nil());
        let first = upsert_submission(None, Uuid::nil(), &request(), "site");
        state.record_submission(first.clone()).await;
        let updated = upsert_submission(Some(&first), Uuid::nil(), &request(), "site");
        state.record_submission(updated).await;
        assert_eq!(state.submissions.read().await.len(), 1);
    }
}
