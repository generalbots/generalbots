use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ProtectionConfig {
    pub enabled: bool,
    pub max_requests_per_minute: u32,
    pub blocked_patterns: Vec<String>,
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self { enabled: true, max_requests_per_minute: 60, blocked_patterns: Vec::new() }
    }
}

pub struct ProtectionManager {
    config: ProtectionConfig,
    request_counts: Arc<RwLock<HashMap<String, u32>>>,
}

impl ProtectionManager {
    pub fn new(config: ProtectionConfig) -> Self {
        Self { config, request_counts: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn config(&self) -> &ProtectionConfig { &self.config }
}

impl std::fmt::Debug for ProtectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtectionManager").field("config", &self.config).finish()
    }
}

pub struct ProtectionTool;
impl ProtectionTool {
    pub fn new() -> Self { Self }
}
