use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CacheRegion {
    Users,
    Organizations,
    Bots,
    Sessions,
    Permissions,
    KnowledgeBase,
    Conversations,
    Settings,
    Metrics,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub default_ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
    pub enable_statistics: bool,
    pub compression_threshold_bytes: Option<usize>,
    pub region_configs: HashMap<CacheRegion, RegionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub eviction_policy: EvictionPolicy,
    pub preload: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    Fifo,
    Ttl,
    Random,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            default_ttl_seconds: 300,
            cleanup_interval_seconds: 60,
            enable_statistics: true,
            compression_threshold_bytes: Some(1024),
            region_configs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub size_bytes: usize,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, ttl_seconds: u64, size_bytes: usize) -> Self {
        let now = Utc::now();
        Self {
            value,
            created_at: now,
            expires_at: now + Duration::seconds(ttl_seconds as i64),
            access_count: 0,
            last_accessed: now,
            size_bytes,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

#[derive(Debug, Default)]
pub struct CacheStatistics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
    pub expirations: AtomicU64,
    pub total_size_bytes: AtomicU64,
    pub entry_count: AtomicU64,
}

impl CacheStatistics {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_expiration(&self) {
        self.expirations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn to_summary(&self) -> CacheStatisticsSummary {
        CacheStatisticsSummary {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            hit_rate: self.hit_rate(),
            evictions: self.evictions.load(Ordering::Relaxed),
            expirations: self.expirations.load(Ordering::Relaxed),
            total_size_bytes: self.total_size_bytes.load(Ordering::Relaxed),
            entry_count: self.entry_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatisticsSummary {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub evictions: u64,
    pub expirations: u64,
    pub total_size_bytes: u64,
    pub entry_count: u64,
}

pub struct PerformanceCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    config: CacheConfig,
    statistics: Arc<CacheStatistics>,
    region: CacheRegion,
}

impl<K, V> PerformanceCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(region: CacheRegion, config: CacheConfig) -> Self {
        let cache = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            statistics: Arc::new(CacheStatistics::default()),
            region,
        };

        let entries = cache.entries.clone();
        let stats = cache.statistics.clone();
        let cleanup_interval = config.cleanup_interval_seconds;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(cleanup_interval)
            );
            loop {
                interval.tick().await;
                let mut map = entries.write().await;
                let before = map.len();
                map.retain(|_, entry| {
                    if entry.is_expired() {
                        stats.record_expiration();
                        false
                    } else {
                        true
                    }
                });
                let removed = before - map.len();
                if removed > 0 {
                    stats.entry_count.fetch_sub(removed as u64, Ordering::Relaxed);
                }
            }
        });

        cache
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                entries.remove(key);
                self.statistics.record_expiration();
                self.statistics.entry_count.fetch_sub(1, Ordering::Relaxed);
                self.statistics.record_miss();
                return None;
            }

            entry.touch();
            self.statistics.record_hit();
            return Some(entry.value.clone());
        }

        self.statistics.record_miss();
        None
    }

    pub async fn set(&self, key: K, value: V, size_bytes: usize) {
        self.set_with_ttl(key, value, self.config.default_ttl_seconds, size_bytes).await;
    }

    pub async fn set_with_ttl(&self, key: K, value: V, ttl_seconds: u64, size_bytes: usize) {
        let mut entries = self.entries.write().await;

        if entries.len() >= self.config.max_entries {
            self.evict_entries(&mut entries).await;
        }

        let entry = CacheEntry::new(value, ttl_seconds, size_bytes);
        let is_new = !entries.contains_key(&key);

        entries.insert(key, entry);

        if is_new {
            self.statistics.entry_count.fetch_add(1, Ordering::Relaxed);
        }
        self.statistics.total_size_bytes.fetch_add(size_bytes as u64, Ordering::Relaxed);
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.remove(key) {
            self.statistics.entry_count.fetch_sub(1, Ordering::Relaxed);
            self.statistics.total_size_bytes.fetch_sub(entry.size_bytes as u64, Ordering::Relaxed);
            return Some(entry.value);
        }

        None
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        self.statistics.entry_count.store(0, Ordering::Relaxed);
        self.statistics.total_size_bytes.store(0, Ordering::Relaxed);
    }

    pub async fn contains(&self, key: &K) -> bool {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(key) {
            return !entry.is_expired();
        }

        false
    }

    pub async fn size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    pub fn statistics(&self) -> CacheStatisticsSummary {
        self.statistics.to_summary()
    }

    async fn evict_entries(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        let region_config = self.config.region_configs.get(&self.region);
        let policy = region_config
            .map(|c| c.eviction_policy)
            .unwrap_or(EvictionPolicy::Lru);

        let entries_to_remove = entries.len() / 10 + 1;

        let keys_to_remove: Vec<K> = match policy {
            EvictionPolicy::Lru => {
                let mut sorted: Vec<_> = entries.iter().collect();
                sorted.sort_by_key(|(_, e)| e.last_accessed);
                sorted.into_iter().take(entries_to_remove).map(|(k, _)| k.clone()).collect()
            }
            EvictionPolicy::Lfu => {
                let mut sorted: Vec<_> = entries.iter().collect();
                sorted.sort_by_key(|(_, e)| e.access_count);
                sorted.into_iter().take(entries_to_remove).map(|(k, _)| k.clone()).collect()
            }
            EvictionPolicy::Fifo => {
                let mut sorted: Vec<_> = entries.iter().collect();
                sorted.sort_by_key(|(_, e)| e.created_at);
                sorted.into_iter().take(entries_to_remove).map(|(k, _)| k.clone()).collect()
            }
            EvictionPolicy::Ttl => {
                let mut sorted: Vec<_> = entries.iter().collect();
                sorted.sort_by_key(|(_, e)| e.expires_at);
                sorted.into_iter().take(entries_to_remove).map(|(k, _)| k.clone()).collect()
            }
            EvictionPolicy::Random => {
                entries.keys().take(entries_to_remove).cloned().collect()
            }
        };

        for key in keys_to_remove {
            if let Some(entry) = entries.remove(&key) {
                self.statistics.record_eviction();
                self.statistics.entry_count.fetch_sub(1, Ordering::Relaxed);
                self.statistics.total_size_bytes.fetch_sub(entry.size_bytes as u64, Ordering::Relaxed);
            }
        }
    }

    pub async fn get_or_insert<F, Fut>(&self, key: K, loader: F, size_bytes: usize) -> V
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = V>,
    {
        if let Some(value) = self.get(&key).await {
            return value;
        }

        let value = loader().await;
        self.set(key, value.clone(), size_bytes).await;
        value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationHint {
    pub use_index: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order_by: Option<Vec<(String, SortDirection)>>,
    pub select_fields: Option<Vec<String>>,
    pub include_count: bool,
    pub use_cache: bool,
    pub cache_ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for QueryOptimizationHint {
    fn default() -> Self {
        Self {
            use_index: None,
            limit: Some(100),
            offset: None,
            order_by: None,
            select_fields: None,
            include_count: false,
            use_cache: true,
            cache_ttl_seconds: Some(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub estimated_rows: u64,
    pub estimated_cost: f64,
    pub index_usage: Vec<IndexUsage>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<QuerySuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexUsage {
    pub index_name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub is_covering: bool,
    pub selectivity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub impact: Impact,
    pub implementation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    AddIndex,
    UseIndex,
    AddLimit,
    RemoveSelectStar,
    UseJoinInsteadOfSubquery,
    AddCaching,
    Denormalize,
    Partition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Impact {
    High,
    Medium,
    Low,
}

pub struct QueryOptimizer {
    slow_query_threshold_ms: u64,
    query_history: Arc<RwLock<Vec<QueryMetrics>>>,
    table_statistics: Arc<RwLock<HashMap<String, TableStats>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_hash: String,
    pub query_pattern: String,
    pub execution_count: u64,
    pub total_time_ms: u64,
    pub avg_time_ms: f64,
    pub max_time_ms: u64,
    pub min_time_ms: u64,
    pub rows_examined: u64,
    pub rows_returned: u64,
    pub last_executed: DateTime<Utc>,
    pub is_slow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: u64,
    pub avg_row_size_bytes: u64,
    pub index_count: u32,
    pub last_analyzed: DateTime<Utc>,
    pub hot_columns: Vec<String>,
}

impl QueryOptimizer {
    pub fn new(slow_query_threshold_ms: u64) -> Self {
        Self {
            slow_query_threshold_ms,
            query_history: Arc::new(RwLock::new(Vec::new())),
            table_statistics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_query(
        &self,
        query_pattern: &str,
        execution_time_ms: u64,
        rows_examined: u64,
        rows_returned: u64,
    ) {
        let query_hash = self.hash_query(query_pattern);
        let is_slow = execution_time_ms > self.slow_query_threshold_ms;

        let mut history = self.query_history.write().await;

        if let Some(existing) = history.iter_mut().find(|m| m.query_hash == query_hash) {
            existing.execution_count += 1;
            existing.total_time_ms += execution_time_ms;
            existing.avg_time_ms = existing.total_time_ms as f64 / existing.execution_count as f64;
            existing.max_time_ms = existing.max_time_ms.max(execution_time_ms);
            existing.min_time_ms = existing.min_time_ms.min(execution_time_ms);
            existing.rows_examined += rows_examined;
            existing.rows_returned += rows_returned;
            existing.last_executed = Utc::now();
            existing.is_slow = is_slow || existing.is_slow;
        } else {
            history.push(QueryMetrics {
                query_hash,
                query_pattern: query_pattern.to_string(),
                execution_count: 1,
                total_time_ms: execution_time_ms,
                avg_time_ms: execution_time_ms as f64,
                max_time_ms: execution_time_ms,
                min_time_ms: execution_time_ms,
                rows_examined,
                rows_returned,
                last_executed: Utc::now(),
                is_slow,
            });
        }

        if history.len() > 10000 {
            history.sort_by_key(|m| std::cmp::Reverse(m.last_executed));
            history.truncate(5000);
        }
    }

    pub async fn get_slow_queries(&self, limit: usize) -> Vec<QueryMetrics> {
        let history = self.query_history.read().await;
        let mut slow: Vec<_> = history.iter().filter(|m| m.is_slow).cloned().collect();
        slow.sort_by(|a, b| b.avg_time_ms.partial_cmp(&a.avg_time_ms).unwrap_or(std::cmp::Ordering::Equal));
        slow.truncate(limit);
        slow
    }

    pub async fn get_query_suggestions(&self, query_pattern: &str) -> Vec<QuerySuggestion> {
        let mut suggestions = Vec::new();

        if query_pattern.to_lowercase().contains("select *") {
            suggestions.push(QuerySuggestion {
                suggestion_type: SuggestionType::RemoveSelectStar,
                description: "Replace SELECT * with specific columns to reduce data transfer".to_string(),
                impact: Impact::Medium,
                implementation: Some("SELECT specific_column1, specific_column2 FROM ...".to_string()),
            });
        }

        if !query_pattern.to_lowercase().contains("limit") {
            suggestions.push(QuerySuggestion {
                suggestion_type: SuggestionType::AddLimit,
                description: "Add LIMIT clause to prevent fetching too many rows".to_string(),
                impact: Impact::High,
                implementation: Some("... LIMIT 100".to_string()),
            });
        }

        if query_pattern.to_lowercase().contains("where")
            && !query_pattern.to_lowercase().contains("index")
        {
            suggestions.push(QuerySuggestion {
                suggestion_type: SuggestionType::AddIndex,
                description: "Consider adding an index on filtered columns".to_string(),
                impact: Impact::High,
                implementation: None,
            });
        }

        if query_pattern.to_lowercase().contains("in (select") {
            suggestions.push(QuerySuggestion {
                suggestion_type: SuggestionType::UseJoinInsteadOfSubquery,
                description: "Replace IN subquery with JOIN for better performance".to_string(),
                impact: Impact::Medium,
                implementation: Some("Use INNER JOIN instead of IN (SELECT ...)".to_string()),
            });
        }

        suggestions
    }

    pub async fn update_table_stats(&self, table_name: &str, stats: TableStats) {
        let mut table_stats = self.table_statistics.write().await;
        table_stats.insert(table_name.to_string(), stats);
    }

    pub async fn get_table_stats(&self, table_name: &str) -> Option<TableStats> {
        let table_stats = self.table_statistics.read().await;
        table_stats.get(table_name).cloned()
    }

    pub async fn analyze_query(&self, query_pattern: &str) -> QueryPlan {
        let suggestions = self.get_query_suggestions(query_pattern).await;

        let mut warnings = Vec::new();

        if query_pattern.to_lowercase().contains("select *") {
            warnings.push("SELECT * can impact performance".to_string());
        }

        if !query_pattern.to_lowercase().contains("limit") {
            warnings.push("Query has no LIMIT clause".to_string());
        }

        QueryPlan {
            estimated_rows: 1000,
            estimated_cost: 100.0,
            index_usage: Vec::new(),
            warnings,
            suggestions,
        }
    }

    fn hash_query(&self, query: &str) -> String {
        use std::collections::hash_map::DefaultHasher;

        let normalized = self.normalize_query(query);
        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn normalize_query(&self, query: &str) -> String {
        let mut result = query.to_lowercase();

        let patterns = [
            (r"'[^']*'", "'?'"),
            (r"\d+", "?"),
            (r"\s+", " "),
        ];

        for (pattern, replacement) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }

        result.trim().to_string()
    }

    pub async fn get_performance_report(&self) -> PerformanceReport {
        let history = self.query_history.read().await;

        let total_queries: u64 = history.iter().map(|m| m.execution_count).sum();
        let slow_queries: u64 = history.iter().filter(|m| m.is_slow).map(|m| m.execution_count).sum();

        let avg_time: f64 = if total_queries > 0 {
            history.iter().map(|m| m.total_time_ms).sum::<u64>() as f64 / total_queries as f64
        } else {
            0.0
        };

        let top_slow = self.get_slow_queries(10).await;

        PerformanceReport {
            generated_at: Utc::now(),
            total_queries,
            slow_queries,
            slow_query_percentage: if total_queries > 0 {
                (slow_queries as f64 / total_queries as f64) * 100.0
            } else {
                0.0
            },
            average_query_time_ms: avg_time,
            top_slow_queries: top_slow,
            recommendations: self.generate_recommendations(&history).await,
        }
    }

    async fn generate_recommendations(&self, history: &[QueryMetrics]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let slow_count = history.iter().filter(|m| m.is_slow).count();
        if slow_count > history.len() / 10 {
            recommendations.push(
                "More than 10% of queries are slow. Consider reviewing query patterns and adding indexes."
                    .to_string(),
            );
        }

        let high_row_scans: Vec<_> = history
            .iter()
            .filter(|m| m.rows_examined > m.rows_returned * 100)
            .collect();

        if !high_row_scans.is_empty() {
            recommendations.push(
                format!(
                    "{} queries examine many more rows than returned. Consider adding appropriate indexes.",
                    high_row_scans.len()
                )
            );
        }

        if recommendations.is_empty() {
            recommendations.push("Query performance looks good. Continue monitoring.".to_string());
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub generated_at: DateTime<Utc>,
    pub total_queries: u64,
    pub slow_queries: u64,
    pub slow_query_percentage: f64,
    pub average_query_time_ms: f64,
    pub top_slow_queries: Vec<QueryMetrics>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub test_on_checkout: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 1800,
            test_on_checkout: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub waiting_requests: u32,
    pub total_connections_created: u64,
    pub total_connections_closed: u64,
    pub average_wait_time_ms: f64,
    pub pool_utilization: f64,
}

type BatchProcessorFunc<T> = Arc<dyn Fn(Vec<T>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

pub struct BatchProcessor<T> {
    batch_size: usize,
    buffer: Arc<RwLock<Vec<T>>>,
    processor: BatchProcessorFunc<T>,
}

impl<T: Clone + Send + Sync + 'static> BatchProcessor<T> {
    pub fn new<F, Fut>(batch_size: usize, flush_interval_ms: u64, processor: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let processor_arc: Arc<dyn Fn(Vec<T>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync> =
            Arc::new(move |items| Box::pin(processor(items)));

        let batch_processor = Self {
            batch_size,
            buffer: Arc::new(RwLock::new(Vec::new())),
            processor: processor_arc,
        };

        let buffer = batch_processor.buffer.clone();
        let processor = batch_processor.processor.clone();
        let batch_size_clone = batch_size;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(flush_interval_ms)
            );
            loop {
                interval.tick().await;
                let mut buf = buffer.write().await;
                if !buf.is_empty() {
                    let items: Vec<T> = buf.drain(..).collect();
                    drop(buf);

                    for chunk in items.chunks(batch_size_clone) {
                        processor(chunk.to_vec()).await;
                    }
                }
            }
        });

        batch_processor
    }

    pub async fn add(&self, item: T) {
        let mut buffer = self.buffer.write().await;
        buffer.push(item);

        if buffer.len() >= self.batch_size {
            let items: Vec<T> = buffer.drain(..).collect();
            drop(buffer);
            (self.processor)(items).await;
        }
    }

    pub async fn flush(&self) {
        let mut buffer = self.buffer.write().await;
        if !buffer.is_empty() {
            let items: Vec<T> = buffer.drain(..).collect();
            drop(buffer);
            (self.processor)(items).await;
        }
    }

    pub async fn pending_count(&self) -> usize {
        let buffer = self.buffer.read().await;
        buffer.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub default_page_size: u32,
    pub max_page_size: u32,
    pub use_cursor_pagination: bool,
    pub include_total_count: bool,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            default_page_size: 20,
            max_page_size: 100,
            use_cursor_pagination: true,
            include_total_count: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub total_count: Option<u64>,
    pub page_size: u32,
}

pub fn create_cursor(id: &Uuid, timestamp: &DateTime<Utc>) -> String {
    let data = format!("{}:{}", id, timestamp.timestamp_millis());
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data.as_bytes())
}

pub fn parse_cursor(cursor: &str) -> Option<(Uuid, DateTime<Utc>)> {
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, cursor).ok()?;
    let data = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = data.split(':').collect();

    if parts.len() != 2 {
        return None;
    }

    let id = Uuid::parse_str(parts[0]).ok()?;
    let timestamp_ms: i64 = parts[1].parse().ok()?;
    let timestamp = DateTime::from_timestamp_millis(timestamp_ms)?;

    Some((id, timestamp))
}
