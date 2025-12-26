use anyhow::Result;
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteCrawlConfig {
    pub url: String,
    pub max_depth: usize,
    pub max_pages: usize,
    pub crawl_delay_ms: u64,
    pub expires_policy: String,
    pub last_crawled: Option<chrono::DateTime<chrono::Utc>>,
    pub next_crawl: Option<chrono::DateTime<chrono::Utc>>,
}

impl WebsiteCrawlConfig {
    pub fn calculate_next_crawl(&mut self) {
        let now = chrono::Utc::now();
        self.last_crawled = Some(now);

        let duration = match self.expires_policy.as_str() {
            "1h" => chrono::Duration::hours(1),
            "6h" => chrono::Duration::hours(6),
            "12h" => chrono::Duration::hours(12),
            "1d" | "24h" => chrono::Duration::days(1),
            "3d" => chrono::Duration::days(3),
            "1w" | "7d" => chrono::Duration::weeks(1),
            "2w" => chrono::Duration::weeks(2),
            "1m" | "30d" => chrono::Duration::days(30),
            "3m" => chrono::Duration::days(90),
            "6m" => chrono::Duration::days(180),
            "1y" | "365d" => chrono::Duration::days(365),
            custom => {
                if let Some(hours_str) = custom.strip_suffix('h') {
                    if let Ok(hours) = hours_str.parse::<i64>() {
                        chrono::Duration::hours(hours)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if let Some(days_str) = custom.strip_suffix('d') {
                    if let Ok(days) = days_str.parse::<i64>() {
                        chrono::Duration::days(days)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if let Some(weeks_str) = custom.strip_suffix('w') {
                    if let Ok(weeks) = weeks_str.parse::<i64>() {
                        chrono::Duration::weeks(weeks)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if let Some(months_str) = custom.strip_suffix('m') {
                    if let Ok(months) = months_str.parse::<i64>() {
                        chrono::Duration::days(months * 30)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if let Some(years_str) = custom.strip_suffix('y') {
                    if let Ok(years) = years_str.parse::<i64>() {
                        chrono::Duration::days(years * 365)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else {
                    chrono::Duration::days(1)
                }
            }
        };

        self.next_crawl = Some(now + duration);
    }

    pub fn needs_crawl(&self) -> bool {
        match self.next_crawl {
            Some(next) => chrono::Utc::now() >= next,
            None => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebPage {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub meta_description: Option<String>,
    pub crawled_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct WebCrawler {
    client: reqwest::Client,
    config: WebsiteCrawlConfig,
    visited_urls: HashSet<String>,
    pages: Vec<WebPage>,
}

impl WebCrawler {
    pub fn new(config: WebsiteCrawlConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("GeneralBots/1.0 (Knowledge Base Crawler)")
            .build()
            .unwrap_or_default();

        Self {
            client,
            config,
            visited_urls: HashSet::new(),
            pages: Vec::new(),
        }
    }

    pub async fn crawl(&mut self) -> Result<Vec<WebPage>> {
        info!("Starting crawl of website: {}", self.config.url);

        self.crawl_recursive(&self.config.url.clone(), 0).await?;

        info!(
            "Crawled {} pages from {}",
            self.pages.len(),
            self.config.url
        );

        Ok(self.pages.clone())
    }

    async fn crawl_recursive(&mut self, url: &str, depth: usize) -> Result<()> {
        if depth > self.config.max_depth {
            trace!(
                "Reached max depth {} for URL: {}",
                self.config.max_depth,
                url
            );
            return Ok(());
        }

        if self.pages.len() >= self.config.max_pages {
            trace!("Reached max pages limit: {}", self.config.max_pages);
            return Ok(());
        }

        if self.visited_urls.contains(url) {
            return Ok(());
        }

        self.visited_urls.insert(url.to_string());

        if !self.visited_urls.is_empty() {
            sleep(Duration::from_millis(self.config.crawl_delay_ms)).await;
        }

        let response = match self.client.get(url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to fetch {}: {}", url, e);
                return Ok(());
            }
        };

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/html") {
            trace!("Skipping non-HTML content: {}", url);
            return Ok(());
        }

        let html_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                warn!("Failed to read response from {}: {}", url, e);
                return Ok(());
            }
        };

        let page = Self::extract_page_content(&html_text, url);
        self.pages.push(page);

        if depth < self.config.max_depth {
            let links = Self::extract_links(&html_text, url);
            for link in links {
                if Self::is_same_domain(url, &link) {
                    Box::pin(self.crawl_recursive(&link, depth + 1)).await?;
                }
            }
        }

        Ok(())
    }

    fn extract_page_content(html: &str, url: &str) -> WebPage {
        let mut text = html.to_string();

        while let Some(start) = text.find("<script") {
            if let Some(end) = text.find("</script>") {
                text.replace_range(start..=end + 8, " ");
            } else {
                break;
            }
        }

        while let Some(start) = text.find("<style") {
            if let Some(end) = text.find("</style>") {
                text.replace_range(start..=end + 7, " ");
            } else {
                break;
            }
        }

        let title = if let Some(title_start) = text.find("<title>") {
            text.find("</title>")
                .map(|title_end| text[title_start + 7..title_end].to_string())
        } else {
            None
        };

        while let Some(start) = text.find('<') {
            if let Some(end) = text.find('>') {
                if end > start {
                    text.replace_range(start..=end, " ");
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let content = text.split_whitespace().collect::<Vec<_>>().join(" ");

        WebPage {
            url: url.to_string(),
            title,
            content,
            meta_description: None,
            crawled_at: chrono::Utc::now(),
        }
    }

    fn extract_links(html: &str, base_url: &str) -> Vec<String> {
        let mut links = Vec::new();
        let mut search_pos = 0;

        while let Some(href_pos) = html[search_pos..].find("href=\"") {
            let href_start = search_pos + href_pos + 6;
            if let Some(href_end) = html[href_start..].find('"') {
                let href = &html[href_start..href_start + href_end];

                if !href.starts_with('#')
                    && !href.starts_with("javascript:")
                    && !href.starts_with("mailto:")
                    && !href.starts_with("tel:")
                {
                    let absolute_url =
                        if href.starts_with("http://") || href.starts_with("https://") {
                            href.to_string()
                        } else if href.starts_with('/') {
                            if let Some(domain_end) = base_url[8..].find('/') {
                                format!("{}{}", &base_url[..8 + domain_end], href)
                            } else {
                                format!("{}{}", base_url, href)
                            }
                        } else if let Some(last_slash) = base_url.rfind('/') {
                            format!("{}/{}", &base_url[..last_slash], href)
                        } else {
                            format!("{}/{}", base_url, href)
                        };

                    links.push(absolute_url);
                }
                search_pos = href_start + href_end;
            } else {
                break;
            }
        }

        links
    }

    fn is_same_domain(url1: &str, url2: &str) -> bool {
        let domain1 = Self::extract_domain(url1);
        let domain2 = Self::extract_domain(url2);
        domain1 == domain2
    }

    fn extract_domain(url: &str) -> String {
        let without_protocol = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        if let Some(slash_pos) = without_protocol.find('/') {
            without_protocol[..slash_pos].to_string()
        } else {
            without_protocol.to_string()
        }
    }
}
