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
            check_interval: Duration::from_secs(3600),
            running: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let service = Arc::clone(&self);

        tokio::spawn(async move {
            info!("Website crawler service started");

            let mut ticker = interval(service.check_interval);

            loop {
                ticker.tick().await;

                if *service.running.read().await {
                    warn!("Website crawler is already running, skipping this cycle");
                    continue;
                }

                *service.running.write().await = true;

                if let Err(e) = service.check_and_crawl_websites() {
                    error!("Error in website crawler service: {}", e);
                }

                *service.running.write().await = false;
            }
        })
    }

    fn check_and_crawl_websites(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Checking for websites that need recrawling");

        let mut conn = self.conn.get()?;

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
            diesel::sql_query("UPDATE website_crawls SET crawl_status = 2 WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(&website.id)
                .execute(&mut conn)?;

            let kb_manager = Arc::clone(&self.kb_manager);
            let db_pool = self.conn.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::crawl_website(website, kb_manager, db_pool).await {
                    error!("Failed to crawl website: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn crawl_website(
        website: WebsiteCrawlRecord,
        kb_manager: Arc<KnowledgeBaseManager>,
        db_pool: DbPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting crawl for website: {}", website.url);

        let config_manager = ConfigManager::new(db_pool.clone());

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

        let mut config = WebsiteCrawlConfig {
            url: website.url.clone(),
            max_depth: website_max_depth,
            max_pages: website_max_pages,
            crawl_delay_ms: 500,
            expires_policy: website.expires_policy.clone(),
            last_crawled: None,
            next_crawl: None,
        };

        let mut crawler = WebCrawler::new(config.clone());

        match crawler.crawl().await {
            Ok(pages) => {
                info!("Crawled {} pages from {}", pages.len(), website.url);

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

                let work_path = std::path::PathBuf::from("work")
                    .join(&bot_name)
                    .join(format!("{}.gbkb", bot_name))
                    .join(&kb_name);

                tokio::fs::create_dir_all(&work_path).await?;

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

                kb_manager
                    .index_kb_folder(&bot_name, &kb_name, &work_path)
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

                info!(
                    "Successfully recrawled {}, next crawl: {:?}",
                    website.url, config.next_crawl
                );
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
            }
        }

        Ok(())
    }
}

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

fn sanitize_url_for_kb(url: &str) -> String {
    url.replace("http://", "")
        .replace("https://", "")
        .replace(['/', ':', '.'], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

pub async fn ensure_crawler_service_running(
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(kb_manager) = &state.kb_manager {
        let service = Arc::new(WebsiteCrawlerService::new(
            state.conn.clone(),
            Arc::clone(kb_manager),
        ));

        drop(service.start());

        info!("Website crawler service initialized");

        Ok(())
    } else {
        warn!("KB manager not available, website crawler service not started");
        Ok(())
    }
}
