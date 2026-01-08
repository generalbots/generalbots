use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Double, Float, Integer, Nullable, Text, Timestamptz};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSource {
    Email,
    Drive,
    Calendar,
    Tasks,
    Transcription,
    Chat,
    Contacts,
    Notes,
}

impl std::fmt::Display for SearchSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::Drive => write!(f, "drive"),
            Self::Calendar => write!(f, "calendar"),
            Self::Tasks => write!(f, "tasks"),
            Self::Transcription => write!(f, "transcription"),
            Self::Chat => write!(f, "chat"),
            Self::Contacts => write!(f, "contacts"),
            Self::Notes => write!(f, "notes"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub source: SearchSource,
    pub title: String,
    pub snippet: String,
    pub url: Option<String>,
    pub score: f32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub sources: Option<Vec<SearchSource>>,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: i64,
    pub query_time_ms: u64,
    pub sources_searched: Vec<SearchSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub source: SearchSource,
    pub document_count: i64,
    pub last_indexed: Option<DateTime<Utc>>,
    pub index_size_bytes: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    pub enabled_sources: Vec<SearchSource>,
    pub max_results_per_source: i32,
    pub snippet_length: i32,
    pub use_tantivy: bool,
    pub postgres_fts_config: String,
    pub index_retention_days: i32,
}

impl SearchConfig {
    pub fn default_config() -> Self {
        Self {
            enabled_sources: vec![
                SearchSource::Email,
                SearchSource::Drive,
                SearchSource::Calendar,
                SearchSource::Tasks,
                SearchSource::Transcription,
            ],
            max_results_per_source: 50,
            snippet_length: 200,
            use_tantivy: false,
            postgres_fts_config: "english".to_string(),
            index_retention_days: 365,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentToIndex {
    pub id: String,
    pub source: SearchSource,
    pub organization_id: Uuid,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(QueryableByName, Debug)]
struct FtsSearchRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Text)]
    snippet: String,
    #[diesel(sql_type = Nullable<Text>)]
    url: Option<String>,
    #[diesel(sql_type = Float)]
    score: f32,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    created_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    updated_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    metadata_json: Option<String>,
}

#[derive(QueryableByName, Debug)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName, Debug)]
struct IndexStatsRow {
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = BigInt)]
    document_count: i64,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    last_indexed: Option<DateTime<Utc>>,
    #[diesel(sql_type = BigInt)]
    index_size_bytes: i64,
}

pub struct SearchService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    config: SearchConfig,
}

impl SearchService {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
        config: Option<SearchConfig>,
    ) -> Self {
        Self {
            pool,
            config: config.unwrap_or_else(SearchConfig::default_config),
        }
    }

    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse, SearchError> {
        let start_time = std::time::Instant::now();

        let sources = query
            .sources
            .clone()
            .unwrap_or_else(|| self.config.enabled_sources.clone());

        let limit = query.limit.unwrap_or(self.config.max_results_per_source);
        let offset = query.offset.unwrap_or(0);

        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            SearchError::DatabaseConnection
        })?;

        let sanitized_query = self.sanitize_query(&query.query);
        let ts_query = self.build_tsquery(&sanitized_query);

        let source_filter: Vec<String> = sources.iter().map(|s| s.to_string()).collect();
        let source_list = source_filter.join("','");

        let sql = format!(
            r#"
            SELECT
                id,
                source,
                title,
                ts_headline($1, content, plainto_tsquery($1, $2),
                    'MaxWords=35, MinWords=15, StartSel=<mark>, StopSel=</mark>') as snippet,
                url,
                ts_rank_cd(search_vector, plainto_tsquery($1, $2)) as score,
                created_at,
                updated_at,
                metadata::text as metadata_json
            FROM search_index
            WHERE organization_id = $3
              AND source IN ('{source_list}')
              AND search_vector @@ plainto_tsquery($1, $2)
              {date_filter}
            ORDER BY score DESC
            LIMIT $4 OFFSET $5
            "#,
            date_filter = self.build_date_filter(&query.from_date, &query.to_date),
        );

        let results: Vec<FtsSearchRow> = diesel::sql_query(&sql)
            .bind::<Text, _>(&self.config.postgres_fts_config)
            .bind::<Text, _>(&sanitized_query)
            .bind::<diesel::sql_types::Uuid, _>(query.organization_id)
            .bind::<Integer, _>(limit)
            .bind::<Integer, _>(offset)
            .load(&mut conn)
            .map_err(|e| {
                error!("Search query failed: {e}");
                SearchError::QueryFailed(e.to_string())
            })?;

        let count_sql = format!(
            r#"
            SELECT COUNT(*) as count
            FROM search_index
            WHERE organization_id = $1
              AND source IN ('{source_list}')
              AND search_vector @@ plainto_tsquery($2, $3)
              {date_filter}
            "#,
            date_filter = self.build_date_filter(&query.from_date, &query.to_date),
        );

        let count_result: Vec<CountRow> = diesel::sql_query(&count_sql)
            .bind::<diesel::sql_types::Uuid, _>(query.organization_id)
            .bind::<Text, _>(&self.config.postgres_fts_config)
            .bind::<Text, _>(&sanitized_query)
            .load(&mut conn)
            .map_err(|e| {
                error!("Count query failed: {e}");
                SearchError::QueryFailed(e.to_string())
            })?;

        let total_count = count_result.first().map(|r| r.count).unwrap_or(0);

        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|row| {
                let metadata: HashMap<String, String> = row
                    .metadata_json
                    .and_then(|json| serde_json::from_str(&json).ok())
                    .unwrap_or_default();

                SearchResult {
                    id: row.id,
                    source: self.parse_source(&row.source),
                    title: row.title,
                    snippet: row.snippet,
                    url: row.url,
                    score: row.score,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    metadata,
                }
            })
            .collect();

        let query_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Search completed: query='{}' results={} time={}ms",
            query.query,
            search_results.len(),
            query_time_ms
        );

        Ok(SearchResponse {
            results: search_results,
            total_count,
            query_time_ms,
            sources_searched: sources,
        })
    }

    pub async fn index_document(&self, doc: DocumentToIndex) -> Result<(), SearchError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            SearchError::DatabaseConnection
        })?;

        let metadata_json = serde_json::to_string(&doc.metadata).unwrap_or_else(|_| "{}".to_string());

        let sql = r#"
            INSERT INTO search_index (id, source, organization_id, title, content, url, metadata, search_vector, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb,
                    setweight(to_tsvector($8, $4), 'A') || setweight(to_tsvector($8, $5), 'B'),
                    $9, NOW())
            ON CONFLICT (id, source) DO UPDATE SET
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                url = EXCLUDED.url,
                metadata = EXCLUDED.metadata,
                search_vector = EXCLUDED.search_vector,
                updated_at = NOW()
        "#;

        diesel::sql_query(sql)
            .bind::<Text, _>(&doc.id)
            .bind::<Text, _>(doc.source.to_string())
            .bind::<diesel::sql_types::Uuid, _>(doc.organization_id)
            .bind::<Text, _>(&doc.title)
            .bind::<Text, _>(&doc.content)
            .bind::<Nullable<Text>, _>(doc.url.as_deref())
            .bind::<Text, _>(&metadata_json)
            .bind::<Text, _>(&self.config.postgres_fts_config)
            .bind::<Timestamptz, _>(doc.created_at)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to index document {}: {e}", doc.id);
                SearchError::IndexFailed(e.to_string())
            })?;

        debug!("Indexed document: id={} source={}", doc.id, doc.source);
        Ok(())
    }

    pub async fn index_documents(&self, docs: Vec<DocumentToIndex>) -> Result<IndexResult, SearchError> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for doc in docs {
            match self.index_document(doc.clone()).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!("{}: {e}", doc.id));
                }
            }
        }

        Ok(IndexResult {
            success_count,
            error_count,
            errors,
        })
    }

    pub async fn delete_document(&self, id: &str, source: SearchSource) -> Result<(), SearchError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            SearchError::DatabaseConnection
        })?;

        diesel::sql_query("DELETE FROM search_index WHERE id = $1 AND source = $2")
            .bind::<Text, _>(id)
            .bind::<Text, _>(source.to_string())
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to delete document {}: {e}", id);
                SearchError::DeleteFailed(e.to_string())
            })?;

        debug!("Deleted document from index: id={} source={}", id, source);
        Ok(())
    }

    pub async fn get_index_stats(&self, organization_id: Uuid) -> Result<Vec<IndexStats>, SearchError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            SearchError::DatabaseConnection
        })?;

        let sql = r#"
            SELECT
                source,
                COUNT(*) as document_count,
                MAX(updated_at) as last_indexed,
                pg_total_relation_size('search_index') /
                    GREATEST((SELECT COUNT(DISTINCT source) FROM search_index WHERE organization_id = $1), 1) as index_size_bytes
            FROM search_index
            WHERE organization_id = $1
            GROUP BY source
        "#;

        let rows: Vec<IndexStatsRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .map_err(|e| {
                error!("Failed to get index stats: {e}");
                SearchError::QueryFailed(e.to_string())
            })?;

        let stats = rows
            .into_iter()
            .map(|row| IndexStats {
                source: self.parse_source(&row.source),
                document_count: row.document_count,
                last_indexed: row.last_indexed,
                index_size_bytes: row.index_size_bytes,
            })
            .collect();

        Ok(stats)
    }

    pub async fn reindex_source(
        &self,
        organization_id: Uuid,
        source: SearchSource,
    ) -> Result<IndexResult, SearchError> {
        info!("Starting reindex for source={} org={}", source, organization_id);

        let documents = match source {
            SearchSource::Email => self.fetch_emails_for_indexing(organization_id).await?,
            SearchSource::Drive => self.fetch_drive_files_for_indexing(organization_id).await?,
            SearchSource::Calendar => self.fetch_calendar_events_for_indexing(organization_id).await?,
            SearchSource::Tasks => self.fetch_tasks_for_indexing(organization_id).await?,
            SearchSource::Transcription => {
                self.fetch_transcriptions_for_indexing(organization_id).await?
            }
            SearchSource::Chat => self.fetch_chat_messages_for_indexing(organization_id).await?,
            SearchSource::Contacts => self.fetch_contacts_for_indexing(organization_id).await?,
            SearchSource::Notes => self.fetch_notes_for_indexing(organization_id).await?,
        };

        let result = self.index_documents(documents).await?;

        info!(
            "Reindex completed for source={}: success={} errors={}",
            source, result.success_count, result.error_count
        );

        Ok(result)
    }

    pub async fn cleanup_old_index_entries(&self, before_date: DateTime<Utc>) -> Result<i64, SearchError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            SearchError::DatabaseConnection
        })?;

        let result = diesel::sql_query("DELETE FROM search_index WHERE updated_at < $1 RETURNING id")
            .bind::<Timestamptz, _>(before_date)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to cleanup old index entries: {e}");
                SearchError::DeleteFailed(e.to_string())
            })?;

        info!("Cleaned up {} old index entries", result);
        Ok(result as i64)
    }

    async fn fetch_emails_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, subject, body_text, sender, recipients, created_at
            FROM emails
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct EmailRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            subject: String,
            #[diesel(sql_type = Text)]
            body_text: String,
            #[diesel(sql_type = Text)]
            sender: String,
            #[diesel(sql_type = Nullable<Text>)]
            recipients: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<EmailRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| {
                let mut metadata = HashMap::new();
                metadata.insert("sender".to_string(), row.sender.clone());
                if let Some(ref recipients) = row.recipients {
                    metadata.insert("recipients".to_string(), recipients.clone());
                }

                DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Email,
                    organization_id,
                    title: row.subject,
                    content: row.body_text,
                    url: None,
                    metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_drive_files_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, name, content_text, mime_type, path, created_at
            FROM drive_files
            WHERE organization_id = $1 AND content_text IS NOT NULL
            ORDER BY created_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct DriveRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            name: String,
            #[diesel(sql_type = Nullable<Text>)]
            content_text: Option<String>,
            #[diesel(sql_type = Text)]
            mime_type: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<DriveRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .filter_map(|row| {
                let content = row.content_text?;
                let mut metadata = HashMap::new();
                metadata.insert("mime_type".to_string(), row.mime_type);
                metadata.insert("path".to_string(), row.path.clone());

                Some(DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Drive,
                    organization_id,
                    title: row.name,
                    content,
                    url: Some(row.path),
                    metadata,
                    created_at: row.created_at,
                })
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_calendar_events_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, title, description, location, start_time, created_at
            FROM calendar_events
            WHERE organization_id = $1
            ORDER BY start_time DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct CalendarRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            title: String,
            #[diesel(sql_type = Nullable<Text>)]
            description: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            location: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            start_time: DateTime<Utc>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<CalendarRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| {
                let mut metadata = HashMap::new();
                if let Some(ref loc) = row.location {
                    metadata.insert("location".to_string(), loc.clone());
                }
                metadata.insert("start_time".to_string(), row.start_time.to_rfc3339());

                let content = format!(
                    "{}\n{}",
                    row.description.as_deref().unwrap_or(""),
                    row.location.as_deref().unwrap_or("")
                );

                DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Calendar,
                    organization_id,
                    title: row.title,
                    content,
                    url: None,
                    metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_tasks_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, title, description, status, priority, created_at
            FROM tasks
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct TaskRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            title: String,
            #[diesel(sql_type = Nullable<Text>)]
            description: Option<String>,
            #[diesel(sql_type = Text)]
            status: String,
            #[diesel(sql_type = Nullable<Text>)]
            priority: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<TaskRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| {
                let mut metadata = HashMap::new();
                metadata.insert("status".to_string(), row.status);
                if let Some(ref pri) = row.priority {
                    metadata.insert("priority".to_string(), pri.clone());
                }

                DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Tasks,
                    organization_id,
                    title: row.title,
                    content: row.description.unwrap_or_default(),
                    url: None,
                    metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_transcriptions_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT mt.id, m.title as meeting_title, mt.content, mt.speaker, m.started_at as created_at
            FROM meeting_transcriptions mt
            JOIN meetings m ON m.id = mt.meeting_id
            WHERE m.organization_id = $1
            ORDER BY m.started_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct TranscriptionRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            meeting_title: String,
            #[diesel(sql_type = Text)]
            content: String,
            #[diesel(sql_type = Nullable<Text>)]
            speaker: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<TranscriptionRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| {
                let mut metadata = HashMap::new();
                if let Some(ref speaker) = row.speaker {
                    metadata.insert("speaker".to_string(), speaker.clone());
                }

                DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Transcription,
                    organization_id,
                    title: row.meeting_title,
                    content: row.content,
                    url: None,
                    metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_chat_messages_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, content, sender_name, channel_name, created_at
            FROM chat_messages
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct ChatRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            content: String,
            #[diesel(sql_type = Nullable<Text>)]
            sender_name: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            channel_name: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<ChatRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| {
                let mut metadata = HashMap::new();
                if let Some(ref sender) = row.sender_name {
                    metadata.insert("sender".to_string(), sender.clone());
                }
                if let Some(ref channel) = row.channel_name {
                    metadata.insert("channel".to_string(), channel.clone());
                }

                let title = row.channel_name.unwrap_or_else(|| "Chat".to_string());

                DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Chat,
                    organization_id,
                    title,
                    content: row.content,
                    url: None,
                    metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_contacts_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, name, email, phone, company, notes, created_at
            FROM contacts
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct ContactRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            name: String,
            #[diesel(sql_type = Nullable<Text>)]
            email: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            phone: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            company: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            notes: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<ContactRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| {
                let mut metadata = HashMap::new();
                if let Some(ref email) = row.email {
                    metadata.insert("email".to_string(), email.clone());
                }
                if let Some(ref phone) = row.phone {
                    metadata.insert("phone".to_string(), phone.clone());
                }
                if let Some(ref company) = row.company {
                    metadata.insert("company".to_string(), company.clone());
                }

                let content = format!(
                    "{} {} {} {}",
                    row.email.as_deref().unwrap_or(""),
                    row.phone.as_deref().unwrap_or(""),
                    row.company.as_deref().unwrap_or(""),
                    row.notes.as_deref().unwrap_or("")
                );

                DocumentToIndex {
                    id: row.id,
                    source: SearchSource::Contacts,
                    organization_id,
                    title: row.name,
                    content,
                    url: None,
                    metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(documents)
    }

    async fn fetch_notes_for_indexing(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DocumentToIndex>, SearchError> {
        let mut conn = self.pool.get().map_err(|_| SearchError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, title, content, created_at
            FROM notes
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT 10000
        "#;

        #[derive(QueryableByName)]
        struct NoteRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            title: String,
            #[diesel(sql_type = Text)]
            content: String,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<NoteRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(organization_id)
            .load(&mut conn)
            .unwrap_or_default();

        let documents = rows
            .into_iter()
            .map(|row| DocumentToIndex {
                id: row.id,
                source: SearchSource::Notes,
                organization_id,
                title: row.title,
                content: row.content,
                url: None,
                metadata: HashMap::new(),
                created_at: row.created_at,
            })
            .collect();

        Ok(documents)
    }

    fn sanitize_query(&self, query: &str) -> String {
        query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect::<String>()
            .trim()
            .to_string()
    }

    fn build_tsquery(&self, query: &str) -> String {
        query
            .split_whitespace()
            .map(|word| format!("{}:*", word))
            .collect::<Vec<_>>()
            .join(" & ")
    }

    fn build_date_filter(
        &self,
        from_date: &Option<DateTime<Utc>>,
        to_date: &Option<DateTime<Utc>>,
    ) -> String {
        let mut filter = String::new();
        if let Some(from) = from_date {
            filter.push_str(&format!(" AND created_at >= '{}'", from.to_rfc3339()));
        }
        if let Some(to) = to_date {
            filter.push_str(&format!(" AND created_at <= '{}'", to.to_rfc3339()));
        }
        filter
    }

    fn parse_source(&self, source: &str) -> SearchSource {
        match source {
            "email" => SearchSource::Email,
            "drive" => SearchSource::Drive,
            "calendar" => SearchSource::Calendar,
            "tasks" => SearchSource::Tasks,
            "transcription" => SearchSource::Transcription,
            "chat" => SearchSource::Chat,
            "contacts" => SearchSource::Contacts,
            "notes" => SearchSource::Notes,
            _ => SearchSource::Drive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    pub success_count: i32,
    pub error_count: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SearchError {
    DatabaseConnection,
    QueryFailed(String),
    IndexFailed(String),
    DeleteFailed(String),
    InvalidQuery(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::QueryFailed(msg) => write!(f, "Query failed: {msg}"),
            Self::IndexFailed(msg) => write!(f, "Index failed: {msg}"),
            Self::DeleteFailed(msg) => write!(f, "Delete failed: {msg}"),
            Self::InvalidQuery(msg) => write!(f, "Invalid query: {msg}"),
        }
    }
}

impl std::error::Error for SearchError {}

pub fn create_search_index_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS search_index (
        id TEXT NOT NULL,
        source TEXT NOT NULL,
        organization_id UUID NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        url TEXT,
        metadata JSONB DEFAULT '{}',
        search_vector TSVECTOR,
        created_at TIMESTAMPTZ NOT NULL,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        PRIMARY KEY (id, source)
    );

    CREATE INDEX IF NOT EXISTS idx_search_index_org ON search_index(organization_id);
    CREATE INDEX IF NOT EXISTS idx_search_index_source ON search_index(source);
    CREATE INDEX IF NOT EXISTS idx_search_index_vector ON search_index USING GIN(search_vector);
    CREATE INDEX IF NOT EXISTS idx_search_index_created ON search_index(created_at);

    CREATE OR REPLACE FUNCTION search_index_update_trigger() RETURNS trigger AS $$
    BEGIN
        NEW.search_vector :=
            setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
            setweight(to_tsvector('english', COALESCE(NEW.content, '')), 'B');
        NEW.updated_at := NOW();
        RETURN NEW;
    END
    $$ LANGUAGE plpgsql;

    DROP TRIGGER IF EXISTS search_index_vector_update ON search_index;
    CREATE TRIGGER search_index_vector_update
        BEFORE INSERT OR UPDATE ON search_index
        FOR EACH ROW EXECUTE FUNCTION search_index_update_trigger();
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_source_display() {
        assert_eq!(SearchSource::Email.to_string(), "email");
        assert_eq!(SearchSource::Drive.to_string(), "drive");
        assert_eq!(SearchSource::Calendar.to_string(), "calendar");
        assert_eq!(SearchSource::Tasks.to_string(), "tasks");
        assert_eq!(SearchSource::Transcription.to_string(), "transcription");
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default_config();
        assert!(!config.enabled_sources.is_empty());
        assert_eq!(config.max_results_per_source, 50);
        assert_eq!(config.snippet_length, 200);
        assert!(!config.use_tantivy);
    }

    #[test]
    fn test_search_error_display() {
        let err = SearchError::DatabaseConnection;
        assert_eq!(err.to_string(), "Database connection failed");

        let err = SearchError::QueryFailed("test error".to_string());
        assert_eq!(err.to_string(), "Query failed: test error");
    }

    #[test]
    fn test_document_to_index_creation() {
        let doc = DocumentToIndex {
            id: "test-123".to_string(),
            source: SearchSource::Email,
            organization_id: Uuid::new_v4(),
            title: "Test Email".to_string(),
            content: "This is test content".to_string(),
            url: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };
        assert_eq!(doc.source, SearchSource::Email);
        assert_eq!(doc.title, "Test Email");
    }

    #[test]
    fn test_index_result() {
        let result = IndexResult {
            success_count: 10,
            error_count: 2,
            errors: vec!["error1".to_string(), "error2".to_string()],
        };
        assert_eq!(result.success_count, 10);
        assert_eq!(result.error_count, 2);
        assert_eq!(result.errors.len(), 2);
    }
}
