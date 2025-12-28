use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, trace};
use reqwest::Url;
use rhai::{Array, Dynamic, Engine, EvalAltResult, Map, Position};
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;

pub fn register_web_data_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_rss_keyword(state.clone(), user.clone(), engine);
    register_scrape_keyword(state.clone(), user.clone(), engine);
    register_scrape_all_keyword(state.clone(), user.clone(), engine);
    register_scrape_table_keyword(state.clone(), user.clone(), engine);
    register_scrape_links_keyword(state.clone(), user.clone(), engine);
    register_scrape_images_keyword(state, user, engine);
}

fn register_rss_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["RSS", "$expr$"], false, move |context, inputs| {
            let url = context.eval_expression_tree(&inputs[0])?.to_string();
            trace!("RSS {}", url);
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                let result = rt.block_on(async { fetch_rss(&url, 100).await });
                let _ = tx.send(result);
            });
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!("RSS failed: {}", e).into(),
                    Position::NONE,
                ))),
                Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                    "RSS timed out".into(),
                    Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");

    engine
        .register_custom_syntax(
            ["RSS", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let limit = context
                    .eval_expression_tree(&inputs[1])?
                    .as_int()
                    .unwrap_or(10) as usize;
                trace!("RSS {} limit {}", url, limit);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                    let result = rt.block_on(async { fetch_rss(&url, limit).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("RSS failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "RSS timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid RSS syntax registration");

    debug!("Registered RSS keyword");
}

async fn fetch_rss(
    url: &str,
    limit: usize,
) -> Result<Array, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("BotServer/6.1.0")
        .timeout(Duration::from_secs(30))
        .build()?;
    let content = client.get(url).send().await?.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;
    let mut results = Array::new();
    for item in channel.items().iter().take(limit) {
        let mut entry = Map::new();
        entry.insert(
            "title".into(),
            Dynamic::from(item.title().unwrap_or("").to_string()),
        );
        entry.insert(
            "link".into(),
            Dynamic::from(item.link().unwrap_or("").to_string()),
        );
        entry.insert(
            "description".into(),
            Dynamic::from(item.description().unwrap_or("").to_string()),
        );
        entry.insert(
            "pubDate".into(),
            Dynamic::from(item.pub_date().unwrap_or("").to_string()),
        );
        entry.insert(
            "author".into(),
            Dynamic::from(item.author().unwrap_or("").to_string()),
        );
        if let Some(guid) = item.guid() {
            entry.insert("guid".into(), Dynamic::from(guid.value().to_string()));
        }
        results.push(Dynamic::from(entry));
    }
    Ok(results)
}

fn register_scrape_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SCRAPE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let selector = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("SCRAPE {} selector {}", url, selector);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                    let result = rt.block_on(async { scrape_first(&url, &selector).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("SCRAPE failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "SCRAPE timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid SCRAPE syntax registration");

    debug!("Registered SCRAPE keyword");
}

fn register_scrape_all_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SCRAPE_ALL", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let selector = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("SCRAPE_ALL {} selector {}", url, selector);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                    let result = rt.block_on(async { scrape_all(&url, &selector).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("SCRAPE_ALL failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "SCRAPE_ALL timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid SCRAPE_ALL syntax registration");

    debug!("Registered SCRAPE_ALL keyword");
}

fn register_scrape_table_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SCRAPE_TABLE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let selector = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("SCRAPE_TABLE {} selector {}", url, selector);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                    let result = rt.block_on(async { scrape_table(&url, &selector).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("SCRAPE_TABLE failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "SCRAPE_TABLE timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid SCRAPE_TABLE syntax registration");

    debug!("Registered SCRAPE_TABLE keyword");
}

fn register_scrape_links_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SCRAPE_LINKS", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("SCRAPE_LINKS {}", url);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                    let result = rt.block_on(async { scrape_links(&url).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("SCRAPE_LINKS failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "SCRAPE_LINKS timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid SCRAPE_LINKS syntax registration");

    debug!("Registered SCRAPE_LINKS keyword");
}

fn register_scrape_images_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["SCRAPE_IMAGES", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("SCRAPE_IMAGES {}", url);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {e}")).expect("Failed to create tokio runtime");
                    let result = rt.block_on(async { scrape_images(&url).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("SCRAPE_IMAGES failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "SCRAPE_IMAGES timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid SCRAPE_IMAGES syntax registration");

    debug!("Registered SCRAPE_IMAGES keyword");
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; BotServer/6.1.0)")
        .timeout(Duration::from_secs(30))
        .build()?;
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}

async fn scrape_first(
    url: &str,
    selector: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let html = fetch_page(url).await?;
    let document = Html::parse_document(&html);
    let sel = Selector::parse(selector).map_err(|e| format!("Invalid selector: {:?}", e))?;
    if let Some(element) = document.select(&sel).next() {
        let text = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        return Ok(text);
    }
    Ok(String::new())
}

async fn scrape_all(
    url: &str,
    selector: &str,
) -> Result<Array, Box<dyn std::error::Error + Send + Sync>> {
    let html = fetch_page(url).await?;
    let document = Html::parse_document(&html);
    let sel = Selector::parse(selector).map_err(|e| format!("Invalid selector: {:?}", e))?;
    let results: Array = document
        .select(&sel)
        .map(|el| {
            let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
            Dynamic::from(text)
        })
        .collect();
    Ok(results)
}

async fn scrape_table(
    url: &str,
    selector: &str,
) -> Result<Array, Box<dyn std::error::Error + Send + Sync>> {
    let html = fetch_page(url).await?;
    let document = Html::parse_document(&html);
    let table_sel = Selector::parse(selector).map_err(|e| format!("Invalid selector: {:?}", e))?;
    let tr_sel = Selector::parse("tr").expect("static tr selector");
    let th_sel = Selector::parse("th").expect("static th selector");
    let td_sel = Selector::parse("td").expect("static td selector");
    let mut results = Array::new();
    let mut headers: Vec<String> = Vec::new();
    if let Some(table) = document.select(&table_sel).next() {
        for (i, row) in table.select(&tr_sel).enumerate() {
            if i == 0 {
                headers = row
                    .select(&th_sel)
                    .chain(row.select(&td_sel))
                    .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .collect();
                if headers.is_empty() {}
            } else {
                let mut row_map = Map::new();
                for (j, cell) in row.select(&td_sel).enumerate() {
                    let key = headers
                        .get(j)
                        .cloned()
                        .unwrap_or_else(|| format!("col{}", j));
                    let value = cell.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    row_map.insert(key.into(), Dynamic::from(value));
                }
                if !row_map.is_empty() {
                    results.push(Dynamic::from(row_map));
                }
            }
        }
    }
    Ok(results)
}

async fn scrape_links(url: &str) -> Result<Array, Box<dyn std::error::Error + Send + Sync>> {
    let html = fetch_page(url).await?;
    let document = Html::parse_document(&html);
    let sel = Selector::parse("a[href]").expect("static href selector");
    let base_url = Url::parse(url)?;
    let mut results = Array::new();
    for el in document.select(&sel) {
        if let Some(href) = el.value().attr("href") {
            let absolute = base_url
                .join(href)
                .map(|u| u.to_string())
                .unwrap_or_default();
            if !absolute.is_empty() {
                let mut link = Map::new();
                link.insert("href".into(), Dynamic::from(absolute));
                link.insert(
                    "text".into(),
                    Dynamic::from(el.text().collect::<Vec<_>>().join(" ").trim().to_string()),
                );
                results.push(Dynamic::from(link));
            }
        }
    }
    Ok(results)
}

async fn scrape_images(url: &str) -> Result<Array, Box<dyn std::error::Error + Send + Sync>> {
    let html = fetch_page(url).await?;
    let document = Html::parse_document(&html);
    let sel = Selector::parse("img[src]").expect("static img selector");
    let base_url = Url::parse(url)?;
    let mut results = Array::new();
    for el in document.select(&sel) {
        if let Some(src) = el.value().attr("src") {
            let absolute = base_url
                .join(src)
                .map(|u| u.to_string())
                .unwrap_or_default();
            if !absolute.is_empty() {
                let mut img = Map::new();
                img.insert("src".into(), Dynamic::from(absolute));
                img.insert(
                    "alt".into(),
                    Dynamic::from(el.value().attr("alt").unwrap_or("").to_string()),
                );
                results.push(Dynamic::from(img));
            }
        }
    }
    Ok(results)
}
