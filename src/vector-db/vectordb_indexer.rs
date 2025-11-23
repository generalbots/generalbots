use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

// UserWorkspace struct for managing user workspace paths
#[derive(Debug, Clone)]
struct UserWorkspace {
    root: PathBuf,
    bot_id: Uuid,
    user_id: Uuid,
}

impl UserWorkspace {
    fn new(root: PathBuf, bot_id: &Uuid, user_id: &Uuid) -> Self {
        Self {
            root,
            bot_id: *bot_id,
            user_id: *user_id,
        }
    }

    fn get_path(&self) -> PathBuf {
        self.root.join(self.bot_id.to_string()).join(self.user_id.to_string())
    }
}
use crate::shared::utils::DbPool;

// VectorDB types are defined locally in this module
#[cfg(feature = "vectordb")]
use qdrant_client::prelude::*;

/// Indexing job status
#[derive(Debug, Clone, PartialEq)]
pub enum IndexingStatus {
    Idle,
    Running,
    Paused,
    Failed(String),
}

/// Indexing statistics
#[derive(Debug, Clone)]
pub struct IndexingStats {
    pub emails_indexed: u64,
    pub files_indexed: u64,
    pub emails_pending: u64,
    pub files_pending: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub errors: u64,
}

/// User indexing job
#[derive(Debug)]
struct UserIndexingJob {
    user_id: Uuid,
    bot_id: Uuid,
    workspace: UserWorkspace,
    #[cfg(all(feature = "vectordb", feature = "email"))]
    email_db: Option<UserEmailVectorDB>,
    #[cfg(feature = "vectordb")]
    drive_db: Option<UserDriveVectorDB>,
    stats: IndexingStats,
    status: IndexingStatus,
}

/// Background vector DB indexer for all users
pub struct VectorDBIndexer {
    db_pool: DbPool,
    work_root: PathBuf,
    qdrant_url: String,
    embedding_generator: Arc<EmailEmbeddingGenerator>,
    jobs: Arc<RwLock<HashMap<Uuid, UserIndexingJob>>>,
    running: Arc<RwLock<bool>>,
    interval_seconds: u64,
    batch_size: usize,
}

impl VectorDBIndexer {
    /// Create new vector DB indexer
    pub fn new(
        db_pool: DbPool,
        work_root: PathBuf,
        qdrant_url: String,
        llm_endpoint: String,
    ) -> Self {
        Self {
            db_pool,
            work_root,
            qdrant_url,
            embedding_generator: Arc::new(EmailEmbeddingGenerator::new(llm_endpoint)),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            interval_seconds: 300, // Run every 5 minutes
            batch_size: 10,        // Index 10 items at a time
        }
    }

    /// Start the background indexing service
    pub async fn start(self: Arc<Self>) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Vector DB indexer already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("🚀 Starting Vector DB Indexer background service");

        let indexer = Arc::clone(&self);
        tokio::spawn(async move {
            indexer.run_indexing_loop().await;
        });

        Ok(())
    }

    /// Stop the indexing service
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("🛑 Stopping Vector DB Indexer");
    }

    /// Main indexing loop
    async fn run_indexing_loop(self: Arc<Self>) {
        loop {
            // Check if still running
            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            info!("🔄 Running vector DB indexing cycle...");

            // Get all active users
            match self.get_active_users().await {
                Ok(users) => {
                    info!("Found {} active users to index", users.len());

                    for (user_id, bot_id) in users {
                        if let Err(e) = self.index_user_data(user_id, bot_id).await {
                            error!("Failed to index user {}: {}", user_id, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get active users: {}", e);
                }
            }

            info!("✅ Indexing cycle complete");

            // Sleep until next cycle
            sleep(Duration::from_secs(self.interval_seconds)).await;
        }

        info!("Vector DB Indexer stopped");
    }

    /// Get all active users from database
    async fn get_active_users(&self) -> Result<Vec<(Uuid, Uuid)>> {
        let conn = self.db_pool.clone();

        tokio::task::spawn_blocking(move || {
            use crate::shared::models::schema::user_sessions::dsl::*;
            use diesel::prelude::*;

            let mut db_conn = conn.get()?;

            // Get unique user_id and bot_id pairs from active sessions
            let results: Vec<(Uuid, Uuid)> = user_sessions
                .select((user_id, bot_id))
                .distinct()
                .load(&mut db_conn)?;

            Ok::<_, anyhow::Error>(results)
        })
        .await?
    }

    /// Index data for a specific user
    async fn index_user_data(&self, user_id: Uuid, bot_id: Uuid) -> Result<()> {
        info!("Indexing user: {} (bot: {})", user_id, bot_id);

        // Get or create job for this user
        let mut jobs = self.jobs.write().await;
        let job = jobs.entry(user_id).or_insert_with(|| {
            let workspace = UserWorkspace::new(self.work_root.clone(), &bot_id, &user_id);

            UserIndexingJob {
                user_id,
                bot_id,
                workspace,
                email_db: None,
                drive_db: None,
                stats: IndexingStats {
                    emails_indexed: 0,
                    files_indexed: 0,
                    emails_pending: 0,
                    files_pending: 0,
                    last_run: None,
                    errors: 0,
                },
                status: IndexingStatus::Idle,
            }
        });

        if job.status == IndexingStatus::Running {
            warn!("Job already running for user {}", user_id);
            return Ok(());
        }

        job.status = IndexingStatus::Running;

        // Initialize vector DBs if needed
        if job.email_db.is_none() {
            let mut email_db =
                UserEmailVectorDB::new(user_id, bot_id, job.workspace.email_vectordb());
            if let Err(e) = email_db.initialize(&self.qdrant_url).await {
                warn!(
                    "Failed to initialize email vector DB for user {}: {}",
                    user_id, e
                );
            } else {
                job.email_db = Some(email_db);
            }
        }

        if job.drive_db.is_none() {
            let mut drive_db =
                UserDriveVectorDB::new(user_id, bot_id, job.workspace.drive_vectordb());
            if let Err(e) = drive_db.initialize(&self.qdrant_url).await {
                warn!(
                    "Failed to initialize drive vector DB for user {}: {}",
                    user_id, e
                );
            } else {
                job.drive_db = Some(drive_db);
            }
        }

        drop(jobs);

        // Index emails
        if let Err(e) = self.index_user_emails(user_id).await {
            error!("Failed to index emails for user {}: {}", user_id, e);
        }

        // Index files
        if let Err(e) = self.index_user_files(user_id).await {
            error!("Failed to index files for user {}: {}", user_id, e);
        }

        // Update job status
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&user_id) {
            job.status = IndexingStatus::Idle;
            job.stats.last_run = Some(Utc::now());
        }

        Ok(())
    }

    /// Index user's emails
    async fn index_user_emails(&self, user_id: Uuid) -> Result<()> {
        let jobs = self.jobs.read().await;
        let job = jobs
            .get(&user_id)
            .ok_or_else(|| anyhow::anyhow!("Job not found"))?;

        let email_db = match &job.email_db {
            Some(db) => db,
            None => {
                warn!("Email vector DB not initialized for user {}", user_id);
                return Ok(());
            }
        };

        // Get user's email accounts
        let accounts = self.get_user_email_accounts(user_id).await?;

        info!(
            "Found {} email accounts for user {}",
            accounts.len(),
            user_id
        );

        for account_id in accounts {
            // Get recent unindexed emails (last 100)
            match self.get_unindexed_emails(user_id, &account_id).await {
                Ok(emails) => {
                    if emails.is_empty() {
                        continue;
                    }

                    info!(
                        "Indexing {} emails for account {}",
                        emails.len(),
                        account_id
                    );

                    // Process in batches
                    for chunk in emails.chunks(self.batch_size) {
                        for email in chunk {
                            match self.embedding_generator.generate_embedding(&email).await {
                                Ok(embedding) => {
                                    if let Err(e) = email_db.index_email(&email, embedding).await {
                                        error!("Failed to index email {}: {}", email.id, e);
                                    } else {
                                        info!("✅ Indexed email: {}", email.subject);
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to generate embedding for email {}: {}",
                                        email.id, e
                                    );
                                }
                            }
                        }

                        // Small delay between batches
                        sleep(Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to get unindexed emails for account {}: {}",
                        account_id, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Index user's files
    async fn index_user_files(&self, user_id: Uuid) -> Result<()> {
        let jobs = self.jobs.read().await;
        let job = jobs
            .get(&user_id)
            .ok_or_else(|| anyhow::anyhow!("Job not found"))?;

        let drive_db = match &job.drive_db {
            Some(db) => db,
            None => {
                warn!("Drive vector DB not initialized for user {}", user_id);
                return Ok(());
            }
        };

        // Get user's files from drive
        match self.get_unindexed_files(user_id).await {
            Ok(files) => {
                if files.is_empty() {
                    return Ok(());
                }

                info!("Indexing {} files for user {}", files.len(), user_id);

                // Process in batches
                for chunk in files.chunks(self.batch_size) {
                    for file in chunk {
                        // Check if file should be indexed
                        let mime_type = file.mime_type.as_ref().map(|s| s.as_str()).unwrap_or("");
                        if !FileContentExtractor::should_index(mime_type, file.file_size) {
                            continue;
                        }

                        // Generate embedding for file content
                        let text = format!(
                            "File: {}\nType: {}\n\n{}",
                            file.file_name, file.file_type, file.content_text
                        );

                        match self
                            .embedding_generator
                            .generate_text_embedding(&text)
                            .await
                        {
                            Ok(embedding) => {
                                if let Err(e) = drive_db.index_file(&file, embedding).await {
                                    error!("Failed to index file {}: {}", file.id, e);
                                } else {
                                    info!("✅ Indexed file: {}", file.file_name);
                                }
                            }
                            Err(e) => {
                                error!("Failed to generate embedding for file {}: {}", file.id, e);
                            }
                        }
                    }

                    // Small delay between batches
                    sleep(Duration::from_millis(100)).await;
                }
            }
            Err(e) => {
                error!("Failed to get unindexed files for user {}: {}", user_id, e);
            }
        }

        Ok(())
    }

    /// Get user's email accounts
    async fn get_user_email_accounts(&self, user_id: Uuid) -> Result<Vec<String>> {
        let conn = self.db_pool.clone();

        tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;

            let mut db_conn = conn.get()?;

            let results: Vec<String> = diesel::sql_query(
                "SELECT id::text FROM user_email_accounts WHERE user_id = $1 AND is_active = true",
            )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load(&mut db_conn)?
            .into_iter()
            .filter_map(|row: diesel::QueryableByName<diesel::pg::Pg>| {
                use diesel::sql_types::Text;
                let id: Result<String, _> = <String as diesel::deserialize::FromSql<
                    Text,
                    diesel::pg::Pg,
                >>::from_sql(row.get("id").ok()?);
                id.ok()
            })
            .collect();

            Ok::<_, anyhow::Error>(results)
        })
        .await?
    }

    /// Get unindexed emails (placeholder - needs actual implementation)
    async fn get_unindexed_emails(
        &self,
        _user_id: Uuid,
        _account_id: &str,
    ) -> Result<Vec<EmailDocument>> {
        // TODO: Implement actual email fetching from IMAP
        // This should:
        // 1. Connect to user's email account
        // 2. Fetch recent emails (last 100)
        // 3. Check which ones are not yet in vector DB
        // 4. Return list of emails to index

        Ok(Vec::new())
    }

    /// Get unindexed files (placeholder - needs actual implementation)
    async fn get_unindexed_files(&self, _user_id: Uuid) -> Result<Vec<FileDocument>> {
        // TODO: Implement actual file fetching from drive
        // This should:
        // 1. List user's files from MinIO/S3
        // 2. Check which ones are not yet in vector DB
        // 3. Extract text content from files
        // 4. Return list of files to index

        Ok(Vec::new())
    }

    /// Get indexing statistics for a user
    pub async fn get_user_stats(&self, user_id: Uuid) -> Option<IndexingStats> {
        let jobs = self.jobs.read().await;
        jobs.get(&user_id).map(|job| job.stats.clone())
    }

    /// Get overall indexing statistics
    pub async fn get_overall_stats(&self) -> IndexingStats {
        let jobs = self.jobs.read().await;

        let mut total_stats = IndexingStats {
            emails_indexed: 0,
            files_indexed: 0,
            emails_pending: 0,
            files_pending: 0,
            last_run: None,
            errors: 0,
        };

        for job in jobs.values() {
            total_stats.emails_indexed += job.stats.emails_indexed;
            total_stats.files_indexed += job.stats.files_indexed;
            total_stats.emails_pending += job.stats.emails_pending;
            total_stats.files_pending += job.stats.files_pending;
            total_stats.errors += job.stats.errors;

            if let Some(last_run) = job.stats.last_run {
                if total_stats.last_run.is_none() || total_stats.last_run.unwrap() < last_run {
                    total_stats.last_run = Some(last_run);
                }
            }
        }

        total_stats
    }

    /// Pause indexing for a specific user
    pub async fn pause_user_indexing(&self, user_id: Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&user_id) {
            job.status = IndexingStatus::Paused;
            info!("⏸️  Paused indexing for user {}", user_id);
        }
        Ok(())
    }

    /// Resume indexing for a specific user
    pub async fn resume_user_indexing(&self, user_id: Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&user_id) {
            job.status = IndexingStatus::Idle;
            info!("▶️  Resumed indexing for user {}", user_id);
        }
        Ok(())
    }

    /// Trigger immediate indexing for a user
    pub async fn trigger_user_indexing(&self, user_id: Uuid, bot_id: Uuid) -> Result<()> {
        info!("🔄 Triggering immediate indexing for user {}", user_id);
        self.index_user_data(user_id, bot_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_stats_creation() {
        let stats = IndexingStats {
            emails_indexed: 10,
            files_indexed: 5,
            emails_pending: 2,
            files_pending: 3,
            last_run: Some(Utc::now()),
            errors: 0,
        };

        assert_eq!(stats.emails_indexed, 10);
        assert_eq!(stats.files_indexed, 5);
    }
}
