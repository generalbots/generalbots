use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub status: KnowledgeStatus,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KnowledgeStatus {
    Draft,
    Published,
    Archived,
}

impl KnowledgeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeStatus::Draft => "Draft",
            KnowledgeStatus::Published => "Published",
            KnowledgeStatus::Archived => "Archived",
        }
    }

    pub fn from_str(s: &str) -> Option<KnowledgeStatus> {
        match s {
            "Draft" => Some(KnowledgeStatus::Draft),
            "Published" => Some(KnowledgeStatus::Published),
            "Archived" => Some(KnowledgeStatus::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateKnowledgeArticleRequest {
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKnowledgeArticleRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchKnowledgeQuery {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
}

type ArticleStorage = Arc<Mutex<HashMap<Uuid, KnowledgeArticle>>>;

#[derive(Clone)]
pub struct KnowledgeBase {
    storage: ArticleStorage,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        KnowledgeBase {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create(&self, req: CreateKnowledgeArticleRequest) -> Result<KnowledgeArticle, String> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let article = KnowledgeArticle {
            id,
            title: req.title,
            content: req.content,
            category: req.category,
            tags: req.tags,
            status: KnowledgeStatus::Draft,
            created_by: req.created_by,
            created_at: now,
            updated_at: now,
        };
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.insert(id, article.clone());
        Ok(article)
    }

    pub fn get(&self, id: Uuid) -> Result<KnowledgeArticle, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.get(&id).cloned().ok_or_else(|| format!("Article not found: {id}"))
    }

    pub fn update(&self, id: Uuid, req: UpdateKnowledgeArticleRequest) -> Result<KnowledgeArticle, String> {
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let article = store.get_mut(&id).ok_or_else(|| format!("Article not found: {id}"))?;
        if let Some(title) = req.title {
            article.title = title;
        }
        if let Some(content) = req.content {
            article.content = content;
        }
        if let Some(category) = req.category {
            article.category = category;
        }
        if let Some(tags) = req.tags {
            article.tags = tags;
        }
        if let Some(ref status_str) = req.status {
            if let Some(status) = KnowledgeStatus::from_str(status_str) {
                article.status = status;
            }
        }
        article.updated_at = Utc::now();
        Ok(article.clone())
    }

    pub fn approve(&self, id: Uuid) -> Result<KnowledgeArticle, String> {
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let article = store.get_mut(&id).ok_or_else(|| format!("Article not found: {id}"))?;
        article.status = KnowledgeStatus::Published;
        article.updated_at = Utc::now();
        Ok(article.clone())
    }

    pub fn archive(&self, id: Uuid) -> Result<KnowledgeArticle, String> {
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let article = store.get_mut(&id).ok_or_else(|| format!("Article not found: {id}"))?;
        article.status = KnowledgeStatus::Archived;
        article.updated_at = Utc::now();
        Ok(article.clone())
    }

    pub fn search(&self, query: SearchKnowledgeQuery) -> Result<Vec<KnowledgeArticle>, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let articles: Vec<KnowledgeArticle> = store
            .values()
            .filter(|a| {
                if let Some(ref keyword) = query.keyword {
                    let kw = keyword.to_lowercase();
                    let in_title = a.title.to_lowercase().contains(&kw);
                    let in_content = a.content.to_lowercase().contains(&kw);
                    let in_tags = a.tags.iter().any(|t| t.to_lowercase().contains(&kw));
                    if !in_title && !in_content && !in_tags {
                        return false;
                    }
                }
                if let Some(ref cat) = query.category {
                    if &a.category != cat {
                        return false;
                    }
                }
                if let Some(ref status_str) = query.status {
                    if let Some(s) = KnowledgeStatus::from_str(status_str) {
                        if a.status != s {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
        Ok(articles)
    }

    pub fn list_by_category(&self, category: &str) -> Result<Vec<KnowledgeArticle>, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let articles: Vec<KnowledgeArticle> = store
            .values()
            .filter(|a| a.category == category && a.status == KnowledgeStatus::Published)
            .cloned()
            .collect();
        Ok(articles)
    }
}
