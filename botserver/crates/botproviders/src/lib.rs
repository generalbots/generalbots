#[cfg(feature = "providers-runpod")]
pub mod runpod;
#[cfg(feature = "providers-vultr")]
pub mod vultr;
#[cfg(feature = "providers-vast")]
pub mod vast;
#[cfg(feature = "providers-contabo")]
pub mod contabo;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineSpec {
    pub cpu_cores: u32,
    pub ram_gb: u32,
    pub disk_gb: u32,
    pub gpu_type: Option<String>,
    pub gpu_count: u32,
    pub bandwidth_tb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionResult {
    pub provider: String,
    pub instance_id: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub region: String,
    pub spec: MachineSpec,
    pub hourly_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub regions: Vec<String>,
    pub available_gpus: Vec<String>,
    pub supports_spot: bool,
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Insufficient capacity: {0}")]
    Capacity(String),
    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
}

#[async_trait::async_trait]
pub trait ComputeProvider: Send + Sync {
    fn name(&self) -> &str;
    fn info(&self) -> ProviderInfo;

    async fn provision(
        &self,
        spec: &MachineSpec,
        region: &str,
        api_key: &str,
    ) -> Result<ProvisionResult, ProviderError>;

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError>;
    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError>;
    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError>;
}
