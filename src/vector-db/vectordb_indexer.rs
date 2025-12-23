use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[cfg(feature = "vectordb")]
use crate::drive::vectordb::UserDriveVectorDB;
#[cfg(feature = "vectordb")]
use crate::drive::vectordb::{FileContentExtractor, FileDocument};
#[cfg(all(feature = "vectordb", feature = "email"))]
use crate::email::vectordb::UserEmailVectorDB;
#[cfg(all(feature = "vectordb", feature = "email"))]
use crate::email::vectordb::{EmailDocument, EmailEmbeddingGenerator};
use crate::shared::utils::DbPool;

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
        self.root
            .join(self.bot_id.to_string())
            .join(self.user_id.to_string())
    }

    fn email_vectordb(&self) -> String {
        format!("email_{}_{}", self.bot_id, self.user_id)
    }

    fn drive_vectordb(&self) -> String {
        format!("drive_{}_{}", self.bot_id, self.user_id)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum IndexingStatus {
    Idle,
    Running,
    Paused,
    Failed(String),
}


#[derive(Debug, Clone)]
pub struct IndexingStats {
    pub emails_indexed: u64,
    pub files_indexed: u64,
    pub emails_pending: u64,
    pub files_pending: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub errors: u64,
}


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
            embedding_generator: Arc::new(EmailEmbeddingGenerator { llm_endpoint }),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            interval_seconds: 300,
            batch_size: 10,
        }
    }


    pub async fn start(self: Arc<Self>) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Vector DB indexer already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        info!(" Starting Vector DB Indexer background service");

        let indexer = Arc::clone(&self);
        tokio::spawn(async move {
            indexer.run_indexing_loop().await;
        });

        Ok(())
    }


    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("🛑 Stopping Vector DB Indexer");
    }


    async fn run_indexing_loop(self: Arc<Self>) {
        loop {

            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            info!(" Running vector DB indexing cycle...");


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

            info!(" Indexing cycle complete");


            sleep(Duration::from_secs(self.interval_seconds)).await;
        }

        info!("Vector DB Indexer stopped");
    }


    async fn get_active_users(&self) -> Result<Vec<(Uuid, Uuid)>> {
        let conn = self.db_pool.clone();

        tokio::task::spawn_blocking(move || {
            use crate::shared::models::schema::user_sessions::dsl::*;
            use diesel::prelude::*;

            let mut db_conn = conn.get()?;


            let results: Vec<(Uuid, Uuid)> = user_sessions
                .select((user_id, bot_id))
                .distinct()
                .load(&mut db_conn)?;

            Ok::<_, anyhow::Error>(results)
        })
        .await?
    }


    async fn index_user_data(&self, user_id: Uuid, bot_id: Uuid) -> Result<()> {
        info!("Indexing user: {} (bot: {})", user_id, bot_id);


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


        if job.email_db.is_none() {
            let mut email_db =
                UserEmailVectorDB::new(user_id, bot_id, job.workspace.email_vectordb().into());
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
                UserDriveVectorDB::new(user_id, bot_id, job.workspace.drive_vectordb().into());
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


        if let Err(e) = self.index_user_emails(user_id).await {
            error!("Failed to index emails for user {}: {}", user_id, e);
        }


        if let Err(e) = self.index_user_files(user_id).await {
            error!("Failed to index files for user {}: {}", user_id, e);
        }


        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&user_id) {
            job.status = IndexingStatus::Idle;
            job.stats.last_run = Some(Utc::now());
        }

        Ok(())
    }


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


        let accounts = self.get_user_email_accounts(user_id).await?;

        info!(
            "Found {} email accounts for user {}",
            accounts.len(),
            user_id
        );

        for account_id in accounts {

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


                    for chunk in emails.chunks(self.batch_size) {
                        for email in chunk {
                            match self.embedding_generator.generate_embedding(&email).await {
                                Ok(embedding) => {
                                    if let Err(e) = email_db.index_email(&email, embedding).await {
                                        error!("Failed to index email {}: {}", email.id, e);
                                    } else {
                                        info!(" Indexed email: {}", email.subject);
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


        match self.get_unindexed_files(user_id).await {
            Ok(files) => {
                if files.is_empty() {
                    return Ok(());
                }

                info!("Indexing {} files for user {}", files.len(), user_id);


                for chunk in files.chunks(self.batch_size) {
                    for file in chunk {

                        let mime_type = file.mime_type.as_ref().map(|s| s.as_str()).unwrap_or("");
                        if !FileContentExtractor::should_index(&mime_type, file.file_size) {
                            continue;
                        }


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
                                    info!(" Indexed file: {}", file.file_name);
                                }
                            }
                            Err(e) => {
                                error!("Failed to generate embedding for file {}: {}", file.id, e);
                            }
                        }
                    }


                    sleep(Duration::from_millis(100)).await;
                }
            }
            Err(e) => {
                error!("Failed to get unindexed files for user {}: {}", user_id, e);
            }
        }

        Ok(())
    }


    async fn get_user_email_accounts(&self, user_id: Uuid) -> Result<Vec<String>> {
        let conn = self.db_pool.clone();

        tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;

            let mut db_conn = conn.get()?;

            #[derive(diesel::QueryableByName)]
            struct AccountIdRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                id: String,
            }

            let results: Vec<String> = diesel::sql_query(
                "SELECT id::text FROM user_email_accounts WHERE user_id = $1 AND is_active = true",
            )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load::<AccountIdRow>(&mut db_conn)?
            .into_iter()
            .map(|row| row.id)
            .collect();

            Ok::<_, anyhow::Error>(results)
        })
        .await?
    }

    async fn get_unindexed_emails(
        &self,
        user_id: Uuid,
        account_id: &str,
    ) -> Result<Vec<EmailDocument>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.db_pool.clone();
        let account_id = account_id.to_string();

        let results = tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            let mut conn = pool.get()?;

            #[derive(diesel::QueryableByName)]
            struct EmailRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                message_id: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                subject: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                from_address: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                to_addresses: String,
                #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                body_text: Option<String>,
                #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                body_html: Option<String>,
                #[diesel(sql_type = diesel::sql_types::Timestamptz)]
                received_at: DateTime<Utc>,
                #[diesel(sql_type = diesel::sql_types::Text)]
                folder: String,
            }

            let query = r#"
                SELECT e.id, e.message_id, e.subject, e.from_address, e.to_addresses,
                       e.body_text, e.body_html, e.received_at, e.folder
                FROM emails e
                LEFT JOIN email_index_status eis ON e.id = eis.email_id
                WHERE e.user_id = $1
                  AND e.account_id = $2
                  AND (eis.indexed_at IS NULL OR eis.needs_reindex = true)
                ORDER BY e.received_at DESC
                LIMIT 100
            "#;

            let rows: Vec<EmailRow> = diesel::sql_query(query)
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .bind::<diesel::sql_types::Text, _>(&account_id)
                .load(&mut conn)
                .unwrap_or_default();

            let emails: Vec<EmailDocument> = rows
                .into_iter()
                .map(|row| {
                    EmailDocument {
                        id: row.id.to_string(),
                        account_id: account_id.clone(),
                        from_email: row.from_address.clone(),
                        from_name: row.from_address,
                        to_email: row.to_addresses,
                        subject: row.subject,
                        body_text: row.body_text.unwrap_or_default(),
                        date: row.received_at,
                        folder: row.folder,
                        has_attachments: false,
                        thread_id: None,
                    }
                })
                .collect();

            Ok::<_, anyhow::Error>(emails)
        })
        .await??;

        Ok(results)
    }

    async fn get_unindexed_files(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FileDocument>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.db_pool.clone();

        let results = tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            let mut conn = pool.get()?;

            #[derive(diesel::QueryableByName)]
            struct FileRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                file_path: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                file_name: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                file_type: String,
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                file_size: i64,
                #[diesel(sql_type = diesel::sql_types::Text)]
                bucket: String,
                #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                mime_type: Option<String>,
                #[diesel(sql_type = diesel::sql_types::Timestamptz)]
                created_at: DateTime<Utc>,
                #[diesel(sql_type = diesel::sql_types::Timestamptz)]
                modified_at: DateTime<Utc>,
            }

            let query = r#"
                SELECT f.id, f.file_path, f.file_name, f.file_type, f.file_size,
                       f.bucket, f.mime_type, f.created_at, f.modified_at
                FROM files f
                LEFT JOIN file_index_status fis ON f.id = fis.file_id
                WHERE f.user_id = $1
                  AND (fis.indexed_at IS NULL OR fis.needs_reindex = true)
                  AND f.file_size < 10485760
                ORDER BY f.modified_at DESC
                LIMIT 100
            "#;

            let rows: Vec<FileRow> = diesel::sql_query(query)
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .load(&mut conn)
                .unwrap_or_default();

            let files: Vec<FileDocument> = rows
                .into_iter()
                .map(|row| {
                    FileDocument {
                        id: row.id.to_string(),
                        file_path: row.file_path,
                        file_name: row.file_name,
                        file_type: row.file_type,
                        file_size: row.file_size as u64,
                        bucket: row.bucket,
                        content_text: String::new(),
                        content_summary: None,
                        created_at: row.created_at,
                        modified_at: row.modified_at,
                        indexed_at: Utc::now(),
                        mime_type: row.mime_type,
                        tags: Vec::new(),
                    }
                })
                .collect();

            Ok::<_, anyhow::Error>(files)
        })
        .await??;

        Ok(results)
    }


    pub async fn get_user_stats(&self, user_id: Uuid) -> Option<IndexingStats> {
        let jobs = self.jobs.read().await;
        jobs.get(&user_id).map(|job| job.stats.clone())
    }


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


    pub async fn pause_user_indexing(&self, user_id: Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&user_id) {
            job.status = IndexingStatus::Paused;
            info!("⏸️  Paused indexing for user {}", user_id);
        }
        Ok(())
    }


    pub async fn resume_user_indexing(&self, user_id: Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&user_id) {
            job.status = IndexingStatus::Idle;
            info!("▶️  Resumed indexing for user {}", user_id);
        }
        Ok(())
    }


    pub async fn trigger_user_indexing(&self, user_id: Uuid, bot_id: Uuid) -> Result<()> {
        info!(" Triggering immediate indexing for user {}", user_id);
        self.index_user_data(user_id, bot_id).await
    }
}
