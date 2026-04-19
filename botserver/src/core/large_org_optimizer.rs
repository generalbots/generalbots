use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct LargeOrgOptimizer {
    config: OptimizerConfig,
    member_cache: Arc<RwLock<HashMap<Uuid, CachedMemberList>>>,
    permission_cache: Arc<RwLock<HashMap<PermissionCacheKey, CachedPermissions>>>,
    query_stats: Arc<RwLock<QueryStatistics>>,
    partition_manager: Arc<PartitionManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    pub large_org_threshold: usize,
    pub member_cache_ttl_seconds: u64,
    pub permission_cache_ttl_seconds: u64,
    pub batch_size: usize,
    pub parallel_queries: usize,
    pub enable_query_optimization: bool,
    pub enable_lazy_loading: bool,
    pub enable_partitioning: bool,
    pub partition_size: usize,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            large_org_threshold: 1000,
            member_cache_ttl_seconds: 300,
            permission_cache_ttl_seconds: 60,
            batch_size: 100,
            parallel_queries: 4,
            enable_query_optimization: true,
            enable_lazy_loading: true,
            enable_partitioning: true,
            partition_size: 500,
        }
    }
}

#[derive(Debug, Clone)]
struct CachedMemberList {
    member_ids: Vec<Uuid>,
    total_count: usize,
    expires_at: DateTime<Utc>,
    is_partial: bool,
    loaded_pages: Vec<usize>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PermissionCacheKey {
    organization_id: Uuid,
    user_id: Uuid,
    resource_type: String,
}

#[derive(Debug, Clone)]
struct CachedPermissions {
    permissions: Vec<String>,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
struct QueryStatistics {
    total_queries: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    slow_queries: AtomicU64,
    avg_query_time_ms: AtomicU64,
}

pub struct PartitionManager {
    partitions: RwLock<HashMap<Uuid, Vec<DataPartition>>>,
    config: PartitionConfig,
}

#[derive(Debug, Clone)]
pub struct DataPartition {
    pub id: Uuid,
    pub partition_key: String,
    pub row_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct PartitionConfig {
    auto_split_threshold: usize,
    merge_threshold: usize,
}

impl Default for PartitionConfig {
    fn default() -> Self {
        Self {
            auto_split_threshold: 8000,
            merge_threshold: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedQuery {
    pub organization_id: Uuid,
    pub page: usize,
    pub page_size: usize,
    pub sort_by: Option<String>,
    pub sort_direction: SortDirection,
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub organization_id: Uuid,
    pub member_count: usize,
    pub is_large_org: bool,
    pub cache_hit_rate: f64,
    pub avg_query_time_ms: f64,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: RecommendationCategory,
    pub description: String,
    pub impact: Impact,
    pub implementation: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    Caching,
    Indexing,
    Partitioning,
    QueryOptimization,
    DataStructure,
    BatchProcessing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Impact {
    High,
    Medium,
    Low,
}

impl LargeOrgOptimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            member_cache: Arc::new(RwLock::new(HashMap::new())),
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            query_stats: Arc::new(RwLock::new(QueryStatistics::default())),
            partition_manager: Arc::new(PartitionManager::new(PartitionConfig::default())),
        }
    }

    pub fn is_large_organization(&self, member_count: usize) -> bool {
        member_count >= self.config.large_org_threshold
    }

    pub async fn get_members_paginated(
        &self,
        query: &PaginatedQuery,
    ) -> Result<PaginatedResult<Uuid>, LargeOrgError> {
        let start = std::time::Instant::now();

        if let Some(cached) = self.get_cached_members(query.organization_id).await {
            if !cached.is_partial || cached.loaded_pages.contains(&query.page) {
                self.record_cache_hit().await;
                let result = self.paginate_cached_members(&cached, query);
                self.record_query_time(start.elapsed().as_millis() as u64).await;
                return Ok(result);
            }
        }

        self.record_cache_miss().await;

        let members = self.fetch_members_page(query).await?;
        self.record_query_time(start.elapsed().as_millis() as u64).await;

        Ok(members)
    }

    async fn get_cached_members(&self, organization_id: Uuid) -> Option<CachedMemberList> {
        let cache = self.member_cache.read().await;
        cache.get(&organization_id).and_then(|cached| {
            if Utc::now() < cached.expires_at {
                Some(cached.clone())
            } else {
                None
            }
        })
    }

    fn paginate_cached_members(
        &self,
        cached: &CachedMemberList,
        query: &PaginatedQuery,
    ) -> PaginatedResult<Uuid> {
        let start_idx = query.page * query.page_size;
        let end_idx = (start_idx + query.page_size).min(cached.member_ids.len());

        let items = if start_idx < cached.member_ids.len() {
            cached.member_ids[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        let total_pages = cached.total_count.div_ceil(query.page_size);

        PaginatedResult {
            items,
            total_count: cached.total_count,
            page: query.page,
            page_size: query.page_size,
            total_pages,
            has_next: query.page + 1 < total_pages,
            has_previous: query.page > 0,
        }
    }

    async fn fetch_members_page(
        &self,
        query: &PaginatedQuery,
    ) -> Result<PaginatedResult<Uuid>, LargeOrgError> {
        let items = Vec::new();
        let total_count: usize = 0;

        let total_pages = if total_count > 0 {
            total_count.div_ceil(query.page_size)
        } else {
            0
        };

        Ok(PaginatedResult {
            items,
            total_count,
            page: query.page,
            page_size: query.page_size,
            total_pages,
            has_next: query.page + 1 < total_pages,
            has_previous: query.page > 0,
        })
    }

    pub async fn cache_member_list(
        &self,
        organization_id: Uuid,
        member_ids: Vec<Uuid>,
        total_count: usize,
        is_partial: bool,
        pages: Vec<usize>,
    ) {
        let now = Utc::now();
        let cached = CachedMemberList {
            member_ids,
            total_count,
            expires_at: now + chrono::Duration::seconds(self.config.member_cache_ttl_seconds as i64),
            is_partial,
            loaded_pages: pages,
        };

        let mut cache = self.member_cache.write().await;
        cache.insert(organization_id, cached);
    }

    pub async fn get_cached_permissions(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        resource_type: &str,
    ) -> Option<Vec<String>> {
        let key = PermissionCacheKey {
            organization_id,
            user_id,
            resource_type: resource_type.to_string(),
        };

        let cache = self.permission_cache.read().await;
        cache.get(&key).and_then(|cached| {
            if Utc::now() < cached.expires_at {
                Some(cached.permissions.clone())
            } else {
                None
            }
        })
    }

    pub async fn cache_permissions(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        resource_type: &str,
        permissions: Vec<String>,
    ) {
        let key = PermissionCacheKey {
            organization_id,
            user_id,
            resource_type: resource_type.to_string(),
        };

        let now = Utc::now();
        let cached = CachedPermissions {
            permissions,
            expires_at: now + chrono::Duration::seconds(self.config.permission_cache_ttl_seconds as i64),
        };

        let mut cache = self.permission_cache.write().await;
        cache.insert(key, cached);
    }

    pub async fn invalidate_member_cache(&self, organization_id: Uuid) {
        let mut cache = self.member_cache.write().await;
        cache.remove(&organization_id);
    }

    pub async fn invalidate_permission_cache(&self, organization_id: Uuid, user_id: Option<Uuid>) {
        let mut cache = self.permission_cache.write().await;

        if let Some(uid) = user_id {
            cache.retain(|k, _| !(k.organization_id == organization_id && k.user_id == uid));
        } else {
            cache.retain(|k, _| k.organization_id != organization_id);
        }
    }

    pub async fn invalidate_all_caches(&self, organization_id: Uuid) {
        self.invalidate_member_cache(organization_id).await;
        self.invalidate_permission_cache(organization_id, None).await;
    }

    async fn record_cache_hit(&self) {
        let stats = self.query_stats.read().await;
        stats.cache_hits.fetch_add(1, Ordering::Relaxed);
        stats.total_queries.fetch_add(1, Ordering::Relaxed);
    }

    async fn record_cache_miss(&self) {
        let stats = self.query_stats.read().await;
        stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        stats.total_queries.fetch_add(1, Ordering::Relaxed);
    }

    async fn record_query_time(&self, time_ms: u64) {
        let stats = self.query_stats.read().await;
        let total = stats.total_queries.load(Ordering::Relaxed);
        let current_avg = stats.avg_query_time_ms.load(Ordering::Relaxed);

        if total > 0 {
            let new_avg = ((current_avg * (total - 1)) + time_ms) / total;
            stats.avg_query_time_ms.store(new_avg, Ordering::Relaxed);
        } else {
            stats.avg_query_time_ms.store(time_ms, Ordering::Relaxed);
        }

        if time_ms > 1000 {
            stats.slow_queries.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn get_cache_hit_rate(&self) -> f64 {
        let stats = self.query_stats.read().await;
        let hits = stats.cache_hits.load(Ordering::Relaxed) as f64;
        let total = stats.total_queries.load(Ordering::Relaxed) as f64;

        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    pub async fn generate_optimization_report(
        &self,
        organization_id: Uuid,
        member_count: usize,
    ) -> OptimizationReport {
        let is_large_org = self.is_large_organization(member_count);
        let cache_hit_rate = self.get_cache_hit_rate().await;

        let stats = self.query_stats.read().await;
        let avg_query_time_ms = stats.avg_query_time_ms.load(Ordering::Relaxed) as f64;
        drop(stats);

        let recommendations = self.generate_recommendations(
            member_count,
            is_large_org,
            cache_hit_rate,
            avg_query_time_ms,
        );

        OptimizationReport {
            organization_id,
            member_count,
            is_large_org,
            cache_hit_rate,
            avg_query_time_ms,
            recommendations,
            generated_at: Utc::now(),
        }
    }

    fn generate_recommendations(
        &self,
        member_count: usize,
        is_large_org: bool,
        cache_hit_rate: f64,
        avg_query_time_ms: f64,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        if cache_hit_rate < 0.5 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::Caching,
                description: "Cache hit rate is below 50%. Consider increasing cache TTL or preloading frequently accessed data.".to_string(),
                impact: Impact::High,
                implementation: "Increase member_cache_ttl_seconds and permission_cache_ttl_seconds in optimizer config".to_string(),
            });
        }

        if avg_query_time_ms > 500.0 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::QueryOptimization,
                description: "Average query time exceeds 500ms. Consider adding indexes or optimizing queries.".to_string(),
                impact: Impact::High,
                implementation: "Review slow query logs and add appropriate database indexes".to_string(),
            });
        }

        if is_large_org && !self.config.enable_partitioning {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::Partitioning,
                description: "Large organization detected without partitioning enabled.".to_string(),
                impact: Impact::Medium,
                implementation: "Enable partitioning in optimizer config for better query performance".to_string(),
            });
        }

        if member_count > 5000 && self.config.batch_size < 200 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::BatchProcessing,
                description: "Very large organization would benefit from larger batch sizes.".to_string(),
                impact: Impact::Medium,
                implementation: "Increase batch_size in optimizer config to 200 or higher".to_string(),
            });
        }

        if is_large_org && !self.config.enable_lazy_loading {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::DataStructure,
                description: "Lazy loading disabled for large organization.".to_string(),
                impact: Impact::Medium,
                implementation: "Enable lazy_loading to reduce initial load times".to_string(),
            });
        }

        recommendations
    }

    pub async fn batch_process<T, F, Fut>(
        &self,
        items: Vec<T>,
        processor: F,
    ) -> Vec<Result<(), LargeOrgError>>
    where
        T: Send + Sync + Clone + 'static,
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), LargeOrgError>> + Send,
    {
        let batch_size = self.config.batch_size;
        let mut results = Vec::new();

        for chunk in items.chunks(batch_size) {
            let result = processor(chunk.to_vec()).await;
            results.push(result);
        }

        results
    }

    pub async fn cleanup_expired_caches(&self) -> CleanupResult {
        let now = Utc::now();

        let members_removed = {
            let mut member_cache = self.member_cache.write().await;
            let original_len = member_cache.len();
            member_cache.retain(|_, v| v.expires_at > now);
            original_len - member_cache.len()
        };

        let permissions_removed = {
            let mut permission_cache = self.permission_cache.write().await;
            let original_len = permission_cache.len();
            permission_cache.retain(|_, v| v.expires_at > now);
            original_len - permission_cache.len()
        };

        CleanupResult {
            members_removed,
            permissions_removed,
            cleaned_at: now,
        }
    }

    pub fn config(&self) -> &OptimizerConfig {
        &self.config
    }

    pub fn partition_manager(&self) -> &Arc<PartitionManager> {
        &self.partition_manager
    }

    pub async fn get_statistics(&self) -> OptimizerStatistics {
        let stats = self.query_stats.read().await;
        let member_cache = self.member_cache.read().await;
        let permission_cache = self.permission_cache.read().await;

        OptimizerStatistics {
            total_queries: stats.total_queries.load(Ordering::Relaxed),
            cache_hits: stats.cache_hits.load(Ordering::Relaxed),
            cache_misses: stats.cache_misses.load(Ordering::Relaxed),
            slow_queries: stats.slow_queries.load(Ordering::Relaxed),
            avg_query_time_ms: stats.avg_query_time_ms.load(Ordering::Relaxed),
            member_cache_size: member_cache.len(),
            permission_cache_size: permission_cache.len(),
        }
    }
}

impl Default for LargeOrgOptimizer {
    fn default() -> Self {
        Self::new(OptimizerConfig::default())
    }
}

impl PartitionManager {
    pub fn new(config: PartitionConfig) -> Self {
        Self {
            partitions: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn get_partitions(&self, organization_id: Uuid) -> Vec<DataPartition> {
        let partitions = self.partitions.read().await;
        partitions.get(&organization_id).cloned().unwrap_or_default()
    }

    pub async fn create_partition(
        &self,
        organization_id: Uuid,
        partition_key: &str,
    ) -> DataPartition {
        let partition = DataPartition {
            id: Uuid::new_v4(),
            partition_key: partition_key.to_string(),
            row_count: 0,
            size_bytes: 0,
        };

        let mut partitions = self.partitions.write().await;
        partitions
            .entry(organization_id)
            .or_default()
            .push(partition.clone());

        partition
    }

    pub async fn should_split(&self, partition: &DataPartition) -> bool {
        partition.row_count >= self.config.auto_split_threshold
    }

    pub async fn should_merge(&self, partition: &DataPartition) -> bool {
        partition.row_count <= self.config.merge_threshold
    }

    pub async fn update_partition_stats(
        &self,
        organization_id: Uuid,
        partition_id: Uuid,
        row_count: usize,
        size_bytes: u64,
    ) {
        let mut partitions = self.partitions.write().await;
        if let Some(org_partitions) = partitions.get_mut(&organization_id) {
            if let Some(partition) = org_partitions.iter_mut().find(|p| p.id == partition_id) {
                partition.row_count = row_count;
                partition.size_bytes = size_bytes;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub members_removed: usize,
    pub permissions_removed: usize,
    pub cleaned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerStatistics {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub slow_queries: u64,
    pub avg_query_time_ms: u64,
    pub member_cache_size: usize,
    pub permission_cache_size: usize,
}

#[derive(Debug, Clone)]
pub enum LargeOrgError {
    DatabaseError(String),
    CacheError(String),
    PartitionError(String),
    InvalidQuery(String),
}

impl std::fmt::Display for LargeOrgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {e}"),
            Self::CacheError(e) => write!(f, "Cache error: {e}"),
            Self::PartitionError(e) => write!(f, "Partition error: {e}"),
            Self::InvalidQuery(e) => write!(f, "Invalid query: {e}"),
        }
    }
}

impl std::error::Error for LargeOrgError {}

pub async fn cache_cleanup_job(optimizer: Arc<LargeOrgOptimizer>, interval_secs: u64) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));

    loop {
        ticker.tick().await;
        let result = optimizer.cleanup_expired_caches().await;
        if result.members_removed > 0 || result.permissions_removed > 0 {
            tracing::debug!(
                "Cache cleanup: removed {} member entries, {} permission entries",
                result.members_removed,
                result.permissions_removed
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_config_default() {
        let config = OptimizerConfig::default();
        assert_eq!(config.large_org_threshold, 1000);
        assert_eq!(config.member_cache_ttl_seconds, 300);
        assert_eq!(config.permission_cache_ttl_seconds, 60);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.parallel_queries, 4);
        assert!(config.enable_query_optimization);
        assert!(config.enable_lazy_loading);
        assert!(config.enable_partitioning);
    }

    #[test]
    fn test_is_large_organization() {
        let optimizer = LargeOrgOptimizer::default();
        assert!(!optimizer.is_large_organization(500));
        assert!(!optimizer.is_large_organization(999));
        assert!(optimizer.is_large_organization(1000));
        assert!(optimizer.is_large_organization(5000));
    }

    #[test]
    fn test_sort_direction_default() {
        let direction = SortDirection::default();
        assert_eq!(direction, SortDirection::Asc);
    }

    #[tokio::test]
    async fn test_cache_member_list() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();
        let member_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        optimizer
            .cache_member_list(org_id, member_ids.clone(), 3, false, vec![0])
            .await;

        let cached = optimizer.get_cached_members(org_id).await;
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.member_ids.len(), 3);
        assert_eq!(cached.total_count, 3);
        assert!(!cached.is_partial);
    }

    #[tokio::test]
    async fn test_cache_permissions() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        optimizer
            .cache_permissions(
                org_id,
                user_id,
                "bot",
                vec!["read".to_string(), "write".to_string()],
            )
            .await;

        let cached = optimizer
            .get_cached_permissions(org_id, user_id, "bot")
            .await;
        assert!(cached.is_some());
        let perms = cached.unwrap();
        assert_eq!(perms.len(), 2);
        assert!(perms.contains(&"read".to_string()));
        assert!(perms.contains(&"write".to_string()));
    }

    #[tokio::test]
    async fn test_invalidate_member_cache() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();

        optimizer
            .cache_member_list(org_id, vec![Uuid::new_v4()], 1, false, vec![0])
            .await;

        optimizer.invalidate_member_cache(org_id).await;

        let cached = optimizer.get_cached_members(org_id).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_permission_cache_all() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();
        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();

        optimizer
            .cache_permissions(org_id, user_id1, "bot", vec!["read".to_string()])
            .await;
        optimizer
            .cache_permissions(org_id, user_id2, "bot", vec!["write".to_string()])
            .await;

        optimizer.invalidate_permission_cache(org_id, None).await;

        assert!(optimizer
            .get_cached_permissions(org_id, user_id1, "bot")
            .await
            .is_none());
        assert!(optimizer
            .get_cached_permissions(org_id, user_id2, "bot")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_invalidate_permission_cache_specific_user() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();
        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();

        optimizer
            .cache_permissions(org_id, user_id1, "bot", vec!["read".to_string()])
            .await;
        optimizer
            .cache_permissions(org_id, user_id2, "bot", vec!["write".to_string()])
            .await;

        optimizer
            .invalidate_permission_cache(org_id, Some(user_id1))
            .await;

        assert!(optimizer
            .get_cached_permissions(org_id, user_id1, "bot")
            .await
            .is_none());
        assert!(optimizer
            .get_cached_permissions(org_id, user_id2, "bot")
            .await
            .is_some());
    }

    #[tokio::test]
    async fn test_cache_hit_rate() {
        let optimizer = LargeOrgOptimizer::default();

        let rate = optimizer.get_cache_hit_rate().await;
        assert_eq!(rate, 0.0);
    }

    #[tokio::test]
    async fn test_generate_optimization_report() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();

        let report = optimizer.generate_optimization_report(org_id, 500).await;
        assert_eq!(report.organization_id, org_id);
        assert_eq!(report.member_count, 500);
        assert!(!report.is_large_org);

        let large_report = optimizer.generate_optimization_report(org_id, 2000).await;
        assert!(large_report.is_large_org);
    }

    #[tokio::test]
    async fn test_cleanup_expired_caches() {
        let optimizer = LargeOrgOptimizer::default();
        let org_id = Uuid::new_v4();

        optimizer
            .cache_member_list(org_id, vec![Uuid::new_v4()], 1, false, vec![0])
            .await;

        let result = optimizer.cleanup_expired_caches().await;
        assert_eq!(result.members_removed, 0);
        assert_eq!(result.permissions_removed, 0);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let optimizer = LargeOrgOptimizer::default();

        let stats = optimizer.get_statistics().await;
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.slow_queries, 0);
    }

    #[tokio::test]
    async fn test_partition_manager() {
        let manager = PartitionManager::new(PartitionConfig::default());
        let org_id = Uuid::new_v4();

        let partition = manager
            .create_partition(org_id, "user_id")
            .await;
        assert_eq!(partition.partition_key, "user_id");
        assert_eq!(partition.row_count, 0);

        let partitions = manager.get_partitions(org_id).await;
        assert_eq!(partitions.len(), 1);
    }

    #[tokio::test]
    async fn test_partition_split_merge_thresholds() {
        let manager = PartitionManager::new(PartitionConfig::default());
        let org_id = Uuid::new_v4();

        let partition = manager.create_partition(org_id, "id").await;

        assert!(!manager.should_split(&partition).await);
        assert!(manager.should_merge(&partition).await);

        let mut large_partition = partition.clone();
        large_partition.row_count = 9000;
        assert!(manager.should_split(&large_partition).await);
        assert!(!manager.should_merge(&large_partition).await);
    }

    #[tokio::test]
    async fn test_paginated_result() {
        let result: PaginatedResult<Uuid> = PaginatedResult {
            items: vec![Uuid::new_v4(), Uuid::new_v4()],
            total_count: 100,
            page: 0,
            page_size: 10,
            total_pages: 10,
            has_next: true,
            has_previous: false,
        };

        assert_eq!(result.items.len(), 2);
        assert!(result.has_next);
        assert!(!result.has_previous);
    }

    #[test]
    fn test_large_org_error_display() {
        let errors = vec![
            (LargeOrgError::DatabaseError("test".to_string()), "Database error: test"),
            (LargeOrgError::CacheError("test".to_string()), "Cache error: test"),
            (LargeOrgError::PartitionError("test".to_string()), "Partition error: test"),
            (LargeOrgError::InvalidQuery("test".to_string()), "Invalid query: test"),
        ];

        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_recommendation_categories() {
        let categories = vec![
            RecommendationCategory::Caching,
            RecommendationCategory::Indexing,
            RecommendationCategory::Partitioning,
            RecommendationCategory::QueryOptimization,
            RecommendationCategory::DataStructure,
            RecommendationCategory::BatchProcessing,
        ];

        for category in categories {
            let serialized = serde_json::to_string(&category).unwrap();
            let deserialized: RecommendationCategory = serde_json::from_str(&serialized).unwrap();
            assert_eq!(category, deserialized);
        }
    }

    #[test]
    fn test_impact_levels() {
        let impacts = vec![Impact::High, Impact::Medium, Impact::Low];

        for impact in impacts {
            let serialized = serde_json::to_string(&impact).unwrap();
            let deserialized: Impact = serde_json::from_str(&serialized).unwrap();
            assert_eq!(impact, deserialized);
        }
    }
}
