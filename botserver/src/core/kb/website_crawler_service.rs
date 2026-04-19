use crate::core::config::ConfigManager;
use crate::core::kb::web_crawler::{WebCrawler, WebsiteCrawlConfig};
use crate::core::kb::embedding_generator::EmbeddingConfig;
use crate::core::kb::kb_indexer::{KbIndexer, QdrantConfig};

use crate::core::shared::state::AppState;
use crate::core::shared::utils::DbPool;
use diesel::prelude::*;
use log::{error, info, trace, warn};
use regex;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug)]
pub struct WebsiteCrawlerService {
    db_pool: DbPool,
    check_interval: Duration,
    running: Arc<tokio::sync::RwLock<bool>>,
    active_crawls: Arc<tokio::sync::RwLock<HashSet<String>>>,
}

impl WebsiteCrawlerService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            check_interval: Duration::from_secs(60),
            running: Arc::new(tokio::sync::RwLock::new(false)),
            active_crawls: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        }
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let service = Arc::clone(&self);

        tokio::spawn(async move {
            trace!("Website crawler service started");

            // Reset any rows stuck at crawl_status=2 (in-progress) from a previous
            // crash or restart — they are no longer actually being crawled.
            if let Ok(mut conn) = service.db_pool.get() {
                let reset = diesel::sql_query(
                    "UPDATE website_crawls SET crawl_status = 0 WHERE crawl_status = 2"
                )
                .execute(&mut conn);
                match reset {
                    Ok(n) if n > 0 => info!("Reset {} stale in-progress crawl(s) to pending", n),
                    Ok(_) => {}
                    Err(e) => warn!("Could not reset stale crawl statuses: {}", e),
                }
            }

            let mut ticker = interval(service.check_interval);

            loop {
                ticker.tick().await;

                if *service.running.read().await {
                    warn!("Website crawler is already running, skipping this cycle");
                    continue;
                }

                *service.running.write().await = true;

                if let Err(e) = service.check_and_crawl_websites().await {
                    error!("Error in website crawler service: {}", e);
                }

                *service.running.write().await = false;
            }
        })
    }

    async fn check_and_crawl_websites(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!("Checking for websites that need recrawling");

        // First, scan for new USE WEBSITE commands in .bas files
        self.scan_and_register_websites_from_scripts()?;

        let mut conn = self.db_pool.get()?;

        // Debug: Log all websites in database
        let all_websites: Vec<WebsiteCrawlRecord> = diesel::sql_query(
            "SELECT id, bot_id, url, expires_policy, refresh_policy, max_depth, max_pages, next_crawl, crawl_status
             FROM website_crawls
             ORDER BY id DESC
             LIMIT 10"
        )
        .load(&mut conn)?;

        trace!("Total websites in database: {}", all_websites.len());
        for ws in &all_websites {
            trace!("  - URL: {}, status: {:?}, refresh: {:?}", ws.url, ws.crawl_status, ws.refresh_policy);
        }

        let websites = diesel::sql_query(
            "SELECT id, bot_id, url, expires_policy, refresh_policy, max_depth, max_pages, next_crawl, crawl_status
             FROM website_crawls
             WHERE next_crawl <= NOW()
             AND crawl_status != 2
             ORDER BY next_crawl ASC
             LIMIT 3"
        )
        .load::<WebsiteCrawlRecord>(&mut conn)?;

        trace!("Found {} websites to recrawl (next_crawl <= NOW())", websites.len());

        // Process websites sequentially to prevent memory exhaustion
        for website in websites {
            // Skip if already being crawled
            let should_skip = {
                let active = self.active_crawls.read().await;
                active.contains(&website.url)
            };

            if should_skip {
                warn!("Skipping {} - already being crawled", website.url);
                continue;
            }

            // Update status to "in progress"
            diesel::sql_query("UPDATE website_crawls SET crawl_status = 2 WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(&website.id)
                .execute(&mut conn)?;

            // Process one website at a time to control memory usage
            let db_pool = self.db_pool.clone();
            let active_crawls = Arc::clone(&self.active_crawls);

            trace!("Processing website: {}", website.url);

            match Self::crawl_website(website, db_pool, active_crawls).await {
                Ok(_) => {
                    trace!("Successfully processed website crawl");
                }
                Err(e) => {
                    error!("Failed to crawl website: {}", e);
                }
            }

            // Force memory cleanup between websites
            tokio::task::yield_now().await;

            // Add delay between crawls to prevent overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    async fn crawl_website(
        website: WebsiteCrawlRecord,
        db_pool: DbPool,
        active_crawls: Arc<tokio::sync::RwLock<HashSet<String>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if already crawling this URL
        {
            let mut active = active_crawls.write().await;
            if active.contains(&website.url) {
                warn!("Crawl already in progress for {}, skipping", website.url);
                return Ok(());
            }
            active.insert(website.url.clone());
        }

        let url_for_cleanup = website.url.clone();
        let active_crawls_cleanup = Arc::clone(&active_crawls);

        // Always remove from active_crawls at the end, regardless of success or error.
        let result = Self::do_crawl_website(website, db_pool).await;

        active_crawls_cleanup.write().await.remove(&url_for_cleanup);

        result
    }

    async fn do_crawl_website(
        website: WebsiteCrawlRecord,
        db_pool: DbPool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!("Starting crawl for website: {}", website.url);

        let config_manager = ConfigManager::new(db_pool.clone().into());

        let website_max_depth = config_manager
            .get_bot_config_value(&website.bot_id, "website-max-depth")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(website.max_depth as usize);

        let website_max_pages = config_manager
            .get_bot_config_value(&website.bot_id, "website-max-pages")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(website.max_pages as usize);

        // Strict limits to prevent memory exhaustion
        let max_depth = std::cmp::min(website_max_depth, 3); // Max 3 levels deep
        let max_pages = std::cmp::min(website_max_pages, 50); // Max 50 pages

        let mut config = WebsiteCrawlConfig {
            url: website.url.clone(),
            max_depth,
            max_pages,
            crawl_delay_ms: 1000, // Increased delay to be respectful
            expires_policy: website.expires_policy.clone(),
            refresh_policy: website.refresh_policy.clone(),
            last_crawled: None,
            next_crawl: None,
        };

        let mut crawler = WebCrawler::new(config.clone());

        match crawler.crawl().await {
            Ok(pages) => {
                trace!("Crawled {} pages from {}", pages.len(), website.url);

                let mut conn = db_pool.get()?;
                #[derive(QueryableByName)]
                struct BotNameResult {
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    name: String,
                }

                let bot_name: String = diesel::sql_query("SELECT name FROM bots WHERE id = $1")
                    .bind::<diesel::sql_types::Uuid, _>(&website.bot_id)
                    .get_result::<BotNameResult>(&mut conn)
                    .map(|r| r.name)?;

                let kb_name = format!("website_{}", sanitize_url_for_kb(&website.url));

                let work_path = std::path::PathBuf::from(crate::core::shared::utils::get_work_path())
                    .join(&bot_name)
                    .join(format!("{}.gbkb", bot_name))
                    .join(&kb_name);

                tokio::fs::create_dir_all(&work_path).await?;

                // Load embedding config: DB settings + Vault API key at gbo/{bot_name}.
                let embedding_config = embedding_config_with_vault(&db_pool, &website.bot_id, &bot_name).await;
                info!("Using embedding URL: {} for bot {}", embedding_config.embedding_url, bot_name);

            // Create bot-specific KB indexer with correct embedding config
            let qdrant_config = QdrantConfig::from_config(db_pool.clone(), &website.bot_id);
            let bot_indexer = KbIndexer::new(embedding_config, qdrant_config);

                // Process pages in small batches to prevent memory exhaustion
                const BATCH_SIZE: usize = 5;
                let total_pages = pages.len();

                for (batch_idx, batch) in pages.chunks(BATCH_SIZE).enumerate() {
                    trace!("Processing batch {} of {} pages", batch_idx + 1, total_pages.div_ceil(BATCH_SIZE));

                    for (idx, page) in batch.iter().enumerate() {
                        let global_idx = batch_idx * BATCH_SIZE + idx;
                        let filename = format!("page_{:04}.txt", global_idx);
                        let filepath = work_path.join(&filename);

                        // Limit content size to prevent memory issues
                        let content_preview = if page.content.len() > 10_000 {
                            format!("{}\n\n[Content truncated - original size: {} chars]",
                                   &page.content[..10_000], page.content.len())
                        } else {
                            page.content.clone()
                        };

                        let content = format!(
                            "URL: {}\nTitle: {}\nCrawled: {}\n\n{}",
                            page.url,
                            page.title.as_deref().unwrap_or("Untitled"),
                            page.crawled_at,
                            content_preview
                        );

                        tokio::fs::write(&filepath, content).await?;
                    }

                    // Process this batch immediately to free memory
                    if batch_idx == 0 || (batch_idx + 1) % 2 == 0 {
                        // Index every 2 batches to prevent memory buildup
                        match bot_indexer.index_kb_folder(website.bot_id, &bot_name, &kb_name, &work_path).await {
                            Ok(result) => trace!("Indexed batch {} successfully: {} docs, {} chunks",
                                batch_idx + 1, result.documents_processed, result.chunks_indexed),
                            Err(e) => warn!("Failed to index batch {}: {}", batch_idx + 1, e),
                        }

                        // Force memory cleanup
                        tokio::task::yield_now().await;
                    }
                }

                // Final indexing for any remaining content
                bot_indexer
                    .index_kb_folder(website.bot_id, &bot_name, &kb_name, &work_path)
                    .await?;

                config.calculate_next_crawl();

                diesel::sql_query(
                    "UPDATE website_crawls
                     SET last_crawled = NOW(),
                         next_crawl = $1,
                         crawl_status = 1,
                         pages_crawled = $2,
                         error_message = NULL
                     WHERE id = $3",
                )
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(
                    config.next_crawl,
                )
                .bind::<diesel::sql_types::Integer, _>(pages.len() as i32)
                .bind::<diesel::sql_types::Uuid, _>(&website.id)
                .execute(&mut conn)?;

                trace!(
                    "Successfully recrawled {}, next crawl: {:?}",
                    website.url, config.next_crawl
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to crawl {}: {}", website.url, e);

                let mut conn = db_pool.get()?;
                diesel::sql_query(
                    "UPDATE website_crawls
                     SET crawl_status = 3,
                         error_message = $1
                     WHERE id = $2",
                )
                .bind::<diesel::sql_types::Text, _>(&e.to_string())
                .bind::<diesel::sql_types::Uuid, _>(&website.id)
                .execute(&mut conn)?;

                Err(e.into())
            }
        }
    }

    fn scan_and_register_websites_from_scripts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!("Scanning .bas files for USE WEBSITE commands");

        // Use the correct work directory path instead of plain "work"
        let work_dir = std::path::PathBuf::from(crate::core::shared::utils::get_work_path());

        if !work_dir.exists() {
            return Ok(());
        }

        let mut conn = self.db_pool.get()?;

        for entry in std::fs::read_dir(work_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if path.is_dir() && file_name_str.ends_with(".gbai") {
                    let bot_name = file_name_str.replace(".gbai", "");

                // Get bot_id from database
                #[derive(QueryableByName)]
                struct BotIdResult {
                    #[diesel(sql_type = diesel::sql_types::Uuid)]
                    id: uuid::Uuid,
                }

                let bot_id_result: Result<BotIdResult, _> = diesel::sql_query("SELECT id FROM bots WHERE name = $1")
                    .bind::<diesel::sql_types::Text, _>(&bot_name)
                    .get_result(&mut conn);

                let bot_id = match bot_id_result {
                    Ok(result) => result.id,
                    Err(_) => continue, // Skip if bot not found
                };

                // Scan {bot_name}.gbdialog directory for .bas files
                let dialog_dir = path.join(format!("{}.gbdialog", bot_name));
                if dialog_dir.exists() {
                    self.scan_directory_for_websites(&dialog_dir, bot_id, &mut conn)?;
                }
                }
            }
        }

        Ok(())
    }

    pub async fn crawl_single_website(
        &self,
        website: WebsiteCrawlRecord,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::crawl_website(website, self.db_pool.clone(), Arc::clone(&self.active_crawls)).await
    }

    fn scan_directory_for_websites(
        &self,
        dir: &std::path::Path,
        bot_id: uuid::Uuid,
        conn: &mut diesel::PgConnection,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let website_regex = regex::Regex::new(r#"(?i)(?:USE\s+WEBSITE\s+"([^"]+)"(?:\s+REFRESH\s+"([^"]+)")?)|(?:USE_WEBSITE\s*\(\s*"([^"]+)"\s*(?:,\s*"([^"]+)"\s*)?\))"#)?;

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "bas") {
                let content = std::fs::read_to_string(&path)?;

                for cap in website_regex.captures_iter(&content) {
                    // Extract URL from either capture group 1 (space syntax) or group 3 (function syntax)
                    let url_str = if let Some(url) = cap.get(1) {
                        url.as_str()
                    } else if let Some(url) = cap.get(3) {
                        url.as_str()
                    } else {
                        continue;
                    };

                    // Extract refresh from either capture group 2 (space syntax) or group 4 (function syntax)
                    let refresh_str = if let Some(refresh) = cap.get(2) {
                        refresh.as_str()
                    } else if let Some(refresh) = cap.get(4) {
                        refresh.as_str()
                    } else {
                        "1m"
                    };

                    // Check if already registered
                    let exists = diesel::sql_query(
                        "SELECT COUNT(*) as count FROM website_crawls WHERE bot_id = $1 AND url = $2"
                    )
                    .bind::<diesel::sql_types::Uuid, _>(&bot_id)
                    .bind::<diesel::sql_types::Text, _>(url_str)
                    .get_result::<CountResult>(conn)
                    .map(|r| r.count)
                    .unwrap_or(0);

                    if exists == 0 {
                        trace!("Auto-registering website {} for bot {} with refresh: {}", url_str, bot_id, refresh_str);

                        // Register website for crawling with refresh policy
                        crate::basic::keywords::use_website::register_website_for_crawling_with_refresh(
                            conn, &bot_id, url_str, refresh_str
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(QueryableByName, Debug)]
pub struct WebsiteCrawlRecord {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub url: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub expires_policy: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub refresh_policy: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub max_depth: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub max_pages: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub next_crawl: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::SmallInt>)]
    pub crawl_status: Option<i16>,
}

fn sanitize_url_for_kb(url: &str) -> String {
    url.replace("http://", "")
        .replace("https://", "")
        .replace(['/', ':', '.'], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

/// Force recrawl a specific website immediately
/// Updates next_crawl to NOW() and triggers immediate crawl
pub async fn force_recrawl_website(
    db_pool: crate::core::shared::utils::DbPool,
    bot_id: uuid::Uuid,
    url: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use diesel::prelude::*;

    let mut conn = db_pool.get()?;

    // Update next_crawl to NOW() to mark for immediate recrawl
    let rows_updated = diesel::sql_query(
        "UPDATE website_crawls
         SET next_crawl = NOW(),
             crawl_status = 0,
             error_message = NULL
         WHERE bot_id = $1 AND url = $2"
    )
    .bind::<diesel::sql_types::Uuid, _>(&bot_id)
    .bind::<diesel::sql_types::Text, _>(&url)
    .execute(&mut conn)?;

    if rows_updated == 0 {
        return Err(format!("Website not found: bot_id={}, url={}", bot_id, url).into());
    }

    trace!("Updated next_crawl to NOW() for website: {}", url);

    // Get the website record for immediate crawling
    #[derive(diesel::QueryableByName)]
    struct WebsiteRecord {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        url: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        expires_policy: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        refresh_policy: String,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        max_depth: i32,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        max_pages: i32,
    }

    let website: WebsiteRecord = diesel::sql_query(
        "SELECT id, bot_id, url, expires_policy, refresh_policy, max_depth, max_pages
         FROM website_crawls
         WHERE bot_id = $1 AND url = $2"
    )
    .bind::<diesel::sql_types::Uuid, _>(&bot_id)
    .bind::<diesel::sql_types::Text, _>(&url)
    .get_result(&mut conn)?;

    // Convert to WebsiteCrawlRecord
    let website_record = WebsiteCrawlRecord {
        id: website.id,
        bot_id: website.bot_id,
        url: website.url.clone(),
        expires_policy: website.expires_policy,
        refresh_policy: Some(website.refresh_policy),
        max_depth: website.max_depth,
        max_pages: website.max_pages,
        next_crawl: None,
        crawl_status: Some(0),
    };

    // Trigger immediate crawl
    let active_crawls = Arc::new(tokio::sync::RwLock::new(HashSet::new()));

    trace!("Starting immediate crawl for website: {}", url);

    match WebsiteCrawlerService::crawl_website(website_record, db_pool, active_crawls).await {
        Ok(_) => {
            let msg = format!("Successfully triggered immediate recrawl for website: {}", url);
            info!("{}", msg);
            Ok(msg)
        }
        Err(e) => {
            let error_msg = format!("Failed to crawl website {}: {}", url, e);
            error!("{}", error_msg);
            Err(error_msg.into())
        }
    }
}

/// API Handler for force recrawl endpoint
/// POST /api/website/force-recrawl
/// Body: {"bot_id": "uuid", "url": "https://example.com"}
pub async fn handle_force_recrawl(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
    axum::Json(payload): axum::Json<ForceRecrawlRequest>,
) -> Result<axum::Json<serde_json::Value>, (axum::http::StatusCode, axum::Json<serde_json::Value>)> {
    use crate::security::error_sanitizer::log_and_sanitize_str;

    match force_recrawl_website(
        state.conn.clone(),
        payload.bot_id,
        payload.url.clone(),
    ).await {
        Ok(msg) => Ok(axum::Json(serde_json::json!({
            "success": true,
            "message": msg,
            "bot_id": payload.bot_id,
            "url": payload.url
        }))),
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "force_recrawl", None);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({"error": sanitized}))
            ))
        }
    }
}

/// Request payload for force recrawl endpoint
#[derive(serde::Deserialize)]
pub struct ForceRecrawlRequest {
    pub bot_id: uuid::Uuid,
    pub url: String,
}

pub async fn ensure_crawler_service_running(
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_kb_manager) = &state.kb_manager {
        let service = Arc::new(WebsiteCrawlerService::new(
            state.conn.clone(),
        ));

        drop(service.start());

        trace!("Website crawler service initialized");

        Ok(())
    } else {
        warn!("KB manager not available, website crawler service not started");
        Ok(())
    }
}

/// Build an `EmbeddingConfig` for a bot, reading most settings from the DB
/// but **overriding the API key from Vault** (per-bot path `gbo/{bot_name}` → `embedding-key`)
/// when available. Falls back transparently to the DB value when Vault is unavailable.
async fn embedding_config_with_vault(
    pool: &DbPool,
    bot_id: &uuid::Uuid,
    bot_name: &str,
) -> EmbeddingConfig {
    // Start from the DB-backed config (URL, model, dimensions, etc.)
    let mut config = EmbeddingConfig::from_bot_config(pool, bot_id);

    // Try to upgrade the API key from Vault using the per-bot secret path.
    if let Some(secrets) = crate::core::shared::utils::get_secrets_manager().await {
        let per_bot_path = format!("gbo/{}", bot_name);
        match secrets.get_value(&per_bot_path, "embedding-key").await {
            Ok(key) if !key.is_empty() => {
                trace!("Loaded embedding key from Vault path {} for bot {}", per_bot_path, bot_name);
                config.embedding_key = Some(key);
            }
            Ok(_) => {} // Key present but empty — keep DB value
            Err(e) => {
                trace!("Vault embedding-key not found at {} ({}), using DB value", per_bot_path, e);
            }
        }
    }

    config
}

