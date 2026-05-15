//! Deploy Gateway — secure bridge between alm-ci CI jobs and host operations
//!
//! alm-ci calls the gateway with a scoped deploy key (not Forgejo tokens).
//! The gateway validates the key and performs privileged operations:
//! - Incus container create/stop/start for apps
//! - Static file copy to proxy container for sites
//! - Caddy route updates
//!
//! Users never see Forgejo internals, SSH keys, or Incus socket access.

use super::types::*;

#[derive(Debug, Clone)]
pub struct DeployGatewayConfig {
    pub gateway_url: String,
    pub deploy_key: String,
}

impl Default for DeployGatewayConfig {
    fn default() -> Self {
        Self {
            gateway_url: std::env::var("DEPLOY_GATEWAY_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5860".to_string()),
            deploy_key: std::env::var("DEPLOY_KEY").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeployGateway {
    config: DeployGatewayConfig,
    client: reqwest::Client,
}

impl DeployGateway {
    pub fn new(config: DeployGatewayConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn deploy(
        &self,
        request: DeployGatewayRequest,
    ) -> Result<DeployGatewayResponse, DeploymentError> {
        let url = format!("{}/deploy", self.config.gateway_url);

        let response = self
            .client
            .post(&url)
            .header("X-Deploy-Key", &self.config.deploy_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| DeploymentError::CiCdError(format!("Gateway request failed: {e}")))?;

        if response.status().is_success() {
            response
                .json::<DeployGatewayResponse>()
                .await
                .map_err(|e| DeploymentError::CiCdError(format!("Gateway response parse failed: {e}")))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DeploymentError::CiCdError(format!(
                "Gateway returned {status}: {body}"
            )))
        }
    }

    pub async fn stop_container(
        &self,
        app_name: &str,
        org: &str,
    ) -> Result<DeployGatewayResponse, DeploymentError> {
        let url = format!("{}/deploy/stop", self.config.gateway_url);

        let response = self
            .client
            .post(&url)
            .header("X-Deploy-Key", &self.config.deploy_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "app_name": app_name,
                "org": org,
            }))
            .send()
            .await
            .map_err(|e| DeploymentError::CiCdError(format!("Gateway stop request failed: {e}")))?;

        if response.status().is_success() {
            response
                .json::<DeployGatewayResponse>()
                .await
                .map_err(|e| DeploymentError::CiCdError(format!("Gateway stop response parse failed: {e}")))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DeploymentError::CiCdError(format!(
                "Gateway stop returned {status}: {body}"
            )))
        }
    }

    pub async fn start_container(
        &self,
        app_name: &str,
        org: &str,
    ) -> Result<DeployGatewayResponse, DeploymentError> {
        let url = format!("{}/deploy/start", self.config.gateway_url);

        let response = self
            .client
            .post(&url)
            .header("X-Deploy-Key", &self.config.deploy_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "app_name": app_name,
                "org": org,
            }))
            .send()
            .await
            .map_err(|e| DeploymentError::CiCdError(format!("Gateway start request failed: {e}")))?;

        if response.status().is_success() {
            response
                .json::<DeployGatewayResponse>()
                .await
                .map_err(|e| DeploymentError::CiCdError(format!("Gateway start response parse failed: {e}")))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DeploymentError::CiCdError(format!(
                "Gateway start returned {status}: {body}"
            )))
        }
    }

    pub async fn get_status(
        &self,
        app_name: &str,
        org: &str,
    ) -> Result<DeployGatewayResponse, DeploymentError> {
        let url = format!("{}/deploy/status/{org}/{app_name}", self.config.gateway_url);

        let response = self
            .client
            .get(&url)
            .header("X-Deploy-Key", &self.config.deploy_key)
            .send()
            .await
            .map_err(|e| DeploymentError::CiCdError(format!("Gateway status request failed: {e}")))?;

        if response.status().is_success() {
            response
                .json::<DeployGatewayResponse>()
                .await
                .map_err(|e| DeploymentError::CiCdError(format!("Gateway status response parse failed: {e}")))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DeploymentError::CiCdError(format!(
                "Gateway status returned {status}: {body}"
            )))
        }
    }
}
