//! `api::core` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub success: bool,
    pub capabilities: Vec<crate::capability_registry::Capability>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelRunRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineQuery {
    pub mode: Option<String>,
}

pub(crate) fn parse_use_case(s: &str) -> Option<VibeUseCase> {
    match s {
        "software_development" => Some(VibeUseCase::SoftwareDevelopment),
        "customer_support" => Some(VibeUseCase::CustomerSupport),
        "financial_analysis" => Some(VibeUseCase::FinancialAnalysis),
        _ => None,
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct SiteEnvQuery {
    #[serde(default)]
    pub(crate) env: Option<String>,
}

/// Resolve the site environment from `?env=` (query wins over body).
pub(crate) fn parse_site_env_param(
    query: &Option<String>,
    body: Option<&str>,
) -> Option<crate::site_env::SiteEnv> {
    let raw = query.as_deref().or(body)?;
    crate::site_env::SiteEnv::parse(raw)
}
