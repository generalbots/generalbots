use crate::config::ConfigManager;
use crate::core::kb::web_crawler::{WebCrawler, WebsiteCrawlConfig};
use crate::core::kb::KnowledgeBaseManager;
use crate::shared::state::AppState;
use crate::shared::utils::DbPool;
use diesel::prelude::*;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

/// Service for periodically checking and recrawling websites
#[derive(Debug)]
pub struct WebsiteCrawlerService {
    db_pool: DbPool,
    kb_manager: Arc<KnowledgeBaseManager>,
    check_interval: Duration,
    running: Arc<tokio::sync::RwLock<bool>>,
}

impl WebsiteCrawlerService {
    pub fn new(db_pool: DbPool, kb_manager: Arc<KnowledgeBaseManager>) -> Self {
        Self {
            db_pool,
            kb_manager,
            check_interval: Duration::from_secs(3600), // Check every hour
            running: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Start the website crawler service
    pub async fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let service = Arc::clone(&self);

        tokio::spawn(async move {
            info!("Website crawler service started");

            let mut ticker = interval(service.check_interval);

            loop {
                ticker.tick().await;

                // Check if already running
                if *service.running.read().await {
                    warn!("Website crawler is already running, skipping this cycle");
                    continue;
                }

                // Set running flag
                *service.running.write().await = true;

                // Check and crawl websites
                if let Err(e) = service.check_and_crawl_websites().await {
                    error!("Error in website crawler service: {}", e);
                }

                // Clear running flag
                *service.running.write().await = false;
            }
        })
    }

    /// Check for websites that need recrawling
    async fn check_and_crawl_websites(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Checking for websites that need recrawling");

        let mut conn = self.db_pool.get()?;

        // Query websites that need recrawling
        let websites = diesel::sql_query(
            "SELECT id, bot_id, url, expires_policy, max_depth, max_pages
             FROM website_crawls
             WHERE next_crawl <= NOW()
             AND crawl_status != 2
             ORDER BY next_crawl ASC
             LIMIT 10",
        )
        .load::<WebsiteCrawlRecord>(&mut conn)?;

        info!("Found {} websites to recrawl", websites.len());

        for website in websites {
            // Mark as processing (status = 2)
            diesel::sql_query("UPDATE website_crawls SET crawl_status = 2 WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(&website.id)
                .execute(&mut conn)?;

            // Spawn crawl task
            let kb_manager = Arc::clone(&self.kb_manager);
            let db_pool = self.db_pool.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::crawl_website(website, kb_manager, db_pool).await {
                    error!("Failed to crawl website: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Crawl a single website
    async fn crawl_website(
        website: WebsiteCrawlRecord,
        kb_manager: Arc<KnowledgeBaseManager>,
        db_pool: DbPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting crawl for website: {}", website.url);

        // Get bot configuration for max_depth and max_pages
        let config_manager = ConfigManager::new(db_pool.clone());

        let website_max_depth = config_manager
            .get_bot_config_value(&website.bot_id, "website-max-depth")
            .await
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(website.max_depth as usize);

        let website_max_pages = config_manager
            .get_bot_config_value(&website.bot_id, "website-max-pages")
            .await
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(website.max_pages as usize);

        // Create crawler configuration
        let mut config = WebsiteCrawlConfig {
            url: website.url.clone(),
            max_depth: website_max_depth,
            max_pages: website_max_pages,
            crawl_delay_ms: 500,
            expires_policy: website.expires_policy.clone(),
            last_crawled: None,
            next_crawl: None,
        };

        // Create and run crawler
        let mut crawler = WebCrawler::new(config.clone());

        match crawler.crawl().await {
            Ok(pages) => {
                info!("Crawled {} pages from {}", pages.len(), website.url);

                // Get bot name
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

                // Create KB name from URL
                let kb_name = format!("website_{}", sanitize_url_for_kb(&website.url));

                // Create work directory
                let work_path = std::path::PathBuf::from("work")
                    .join(&bot_name)
                    .join(format!("{}.gbkb", bot_name))
                    .join(&kb_name);

                // Ensure directory exists
                tokio::fs::create_dir_all(&work_path).await?;

                // Write pages to files
                for (idx, page) in pages.iter().enumerate() {
                    let filename = format!("page_{:04}.txt", idx);
                    let filepath = work_path.join(&filename);

                    let content = format!(
                        "URL: {}\nTitle: {}\nCrawled: {}\n\n{}",
                        page.url,
                        page.title.as_deref().unwrap_or("Untitled"),
                        page.crawled_at,
                        page.content
                    );

                    tokio::fs::write(&filepath, content).await?;
                }

                // Index with KB manager
                kb_manager
                    .index_kb_folder(&bot_name, &kb_name, &work_path)
                    .await?;

                // Update configuration
                config.calculate_next_crawl();

                // Update database
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

                info!(
                    "Successfully recrawled {}, next crawl: {:?}",
                    website.url, config.next_crawl
                );
            }
            Err(e) => {
                error!("Failed to crawl {}: {}", website.url, e);

                // Update database with error
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
            }
        }

        Ok(())
    }
}

/// Record from website_crawls table
#[derive(QueryableByName, Debug)]
struct WebsiteCrawlRecord {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    url: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    expires_policy: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    max_depth: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    max_pages: i32,
}

/// Sanitize URL for use as KB name (duplicate from add_website.rs for isolation)
fn sanitize_url_for_kb(url: &str) -> String {
    url.replace("http://", "")
        .replace("https://", "")
        .replace('/', "_")
        .replace(':', "_")
        .replace('.', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

/// Get crawler service for a state (create if not exists)
pub async fn ensure_crawler_service_running(
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if KB manager exists
    if let Some(kb_manager) = &state.kb_manager {
        let service = Arc::new(WebsiteCrawlerService::new(
            state.conn.clone(),
            Arc::clone(kb_manager),
        ));

        // Start the service
        service.start().await;

        info!("Website crawler service started");

        Ok(())
    } else {
        warn!("KB manager not available, website crawler service not started");
        Ok(())
    }
}
