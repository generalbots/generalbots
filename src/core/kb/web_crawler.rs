use anyhow::Result;
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;

/// Website crawl configuration
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
    /// Parse expiration policy and calculate next crawl time
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
                // Simple parsing for custom format like "2h", "5d", etc.
                if custom.ends_with('h') {
                    if let Ok(hours) = custom[..custom.len() - 1].parse::<i64>() {
                        chrono::Duration::hours(hours)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if custom.ends_with('d') {
                    if let Ok(days) = custom[..custom.len() - 1].parse::<i64>() {
                        chrono::Duration::days(days)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if custom.ends_with('w') {
                    if let Ok(weeks) = custom[..custom.len() - 1].parse::<i64>() {
                        chrono::Duration::weeks(weeks)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if custom.ends_with('m') {
                    if let Ok(months) = custom[..custom.len() - 1].parse::<i64>() {
                        chrono::Duration::days(months * 30)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else if custom.ends_with('y') {
                    if let Ok(years) = custom[..custom.len() - 1].parse::<i64>() {
                        chrono::Duration::days(years * 365)
                    } else {
                        chrono::Duration::days(1)
                    }
                } else {
                    chrono::Duration::days(1) // Default to daily if unparseable
                }
            }
        };

        self.next_crawl = Some(now + duration);
    }

    /// Check if website needs recrawling
    pub fn needs_crawl(&self) -> bool {
        match self.next_crawl {
            Some(next) => chrono::Utc::now() >= next,
            None => true, // Never crawled
        }
    }
}

/// Website content for indexing
#[derive(Debug, Clone)]
pub struct WebPage {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub meta_description: Option<String>,
    pub crawled_at: chrono::DateTime<chrono::Utc>,
}

/// Web crawler for website content
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

    /// Crawl website starting from configured URL
    pub async fn crawl(&mut self) -> Result<Vec<WebPage>> {
        info!("Starting crawl of website: {}", self.config.url);

        // Start crawling from root URL
        self.crawl_recursive(&self.config.url.clone(), 0).await?;

        info!(
            "Crawled {} pages from {}",
            self.pages.len(),
            self.config.url
        );

        Ok(self.pages.clone())
    }

    /// Recursive crawling with depth control
    async fn crawl_recursive(&mut self, url: &str, depth: usize) -> Result<()> {
        // Check depth limit
        if depth > self.config.max_depth {
            trace!(
                "Reached max depth {} for URL: {}",
                self.config.max_depth,
                url
            );
            return Ok(());
        }

        // Check page limit
        if self.pages.len() >= self.config.max_pages {
            trace!("Reached max pages limit: {}", self.config.max_pages);
            return Ok(());
        }

        // Check if already visited
        if self.visited_urls.contains(url) {
            return Ok(());
        }

        // Mark as visited
        self.visited_urls.insert(url.to_string());

        // Add crawl delay to be polite
        if !self.visited_urls.is_empty() {
            sleep(Duration::from_millis(self.config.crawl_delay_ms)).await;
        }

        // Fetch page
        let response = match self.client.get(url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to fetch {}: {}", url, e);
                return Ok(()); // Continue crawling other pages
            }
        };

        // Check if HTML
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/html") {
            trace!("Skipping non-HTML content: {}", url);
            return Ok(());
        }

        // Get page content
        let html_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                warn!("Failed to read response from {}: {}", url, e);
                return Ok(());
            }
        };

        // Extract page content
        let page = self.extract_page_content(&html_text, url);
        self.pages.push(page);

        // Extract and crawl links if not at max depth
        if depth < self.config.max_depth {
            let links = self.extract_links(&html_text, url);
            for link in links {
                // Only crawl same domain
                if self.is_same_domain(url, &link) {
                    Box::pin(self.crawl_recursive(&link, depth + 1)).await?;
                }
            }
        }

        Ok(())
    }

    /// Extract text content from HTML
    fn extract_page_content(&self, html: &str, url: &str) -> WebPage {
        // Simple HTML tag removal
        let mut text = html.to_string();

        // Remove script and style tags with their content
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

        // Extract title if present
        let title = if let Some(title_start) = text.find("<title>") {
            if let Some(title_end) = text.find("</title>") {
                Some(text[title_start + 7..title_end].to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Remove all remaining HTML tags
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

        // Clean up whitespace
        let content = text.split_whitespace().collect::<Vec<_>>().join(" ");

        WebPage {
            url: url.to_string(),
            title,
            content,
            meta_description: None,
            crawled_at: chrono::Utc::now(),
        }
    }

    /// Extract links from HTML
    fn extract_links(&self, html: &str, base_url: &str) -> Vec<String> {
        let mut links = Vec::new();
        let mut search_pos = 0;

        // Simple href extraction
        while let Some(href_pos) = html[search_pos..].find("href=\"") {
            let href_start = search_pos + href_pos + 6;
            if let Some(href_end) = html[href_start..].find('"') {
                let href = &html[href_start..href_start + href_end];

                // Skip anchors, javascript, mailto, etc.
                if !href.starts_with('#')
                    && !href.starts_with("javascript:")
                    && !href.starts_with("mailto:")
                    && !href.starts_with("tel:")
                {
                    // Convert relative URLs to absolute
                    let absolute_url =
                        if href.starts_with("http://") || href.starts_with("https://") {
                            href.to_string()
                        } else if href.starts_with('/') {
                            // Get base domain from base_url
                            if let Some(domain_end) = base_url[8..].find('/') {
                                format!("{}{}", &base_url[..8 + domain_end], href)
                            } else {
                                format!("{}{}", base_url, href)
                            }
                        } else {
                            // Relative to current page
                            if let Some(last_slash) = base_url.rfind('/') {
                                format!("{}/{}", &base_url[..last_slash], href)
                            } else {
                                format!("{}/{}", base_url, href)
                            }
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

    /// Check if two URLs are from the same domain
    fn is_same_domain(&self, url1: &str, url2: &str) -> bool {
        let domain1 = self.extract_domain(url1);
        let domain2 = self.extract_domain(url2);
        domain1 == domain2
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &str) -> String {
        let without_protocol = if url.starts_with("https://") {
            &url[8..]
        } else if url.starts_with("http://") {
            &url[7..]
        } else {
            url
        };

        if let Some(slash_pos) = without_protocol.find('/') {
            without_protocol[..slash_pos].to_string()
        } else {
            without_protocol.to_string()
        }
    }
}
