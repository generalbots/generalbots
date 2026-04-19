//! Deployment router for VibeCode platform - Phase 2.5
//!
//! All apps are deployed to Forgejo repositories using org/app_name format.

use super::types::*;
use super::forgejo::ForgejoClient;

/// Deployment router - all apps go to Forgejo
pub struct DeploymentRouter {
    forgejo_url: String,
    forgejo_token: Option<String>,
}

impl DeploymentRouter {
    pub fn new(forgejo_url: String, forgejo_token: Option<String>) -> Self {
        Self {
            forgejo_url,
            forgejo_token,
        }
    }

    /// Deploy to Forgejo repository (org/app_name)
    pub async fn deploy(
        &self,
        config: DeploymentConfig,
        generated_app: GeneratedApp,
    ) -> Result<DeploymentResult, DeploymentError> {
        log::info!(
            "Deploying {} app: {}/{} to {} environment",
            config.app_type,
            config.organization,
            config.app_name,
            config.environment
        );

        // Get or create Forgejo client
        let token = self.forgejo_token.clone()
            .ok_or_else(|| DeploymentError::ConfigurationError("FORGEJO_TOKEN not configured".to_string()))?;

        let client = ForgejoClient::new(self.forgejo_url.clone(), token);

        // Create repository if it doesn't exist
        let repo = client.create_repository(
            &config.app_name,
            &generated_app.description,
            false, // public repo
        ).await?;

        log::info!("Repository created/verified: {}", repo.clone_url);

        // Push app to repository
        let branch = config.environment.to_string();
        client.push_app(&repo.clone_url, &generated_app, &branch).await?;

        // Create CI/CD workflow if enabled
        if config.ci_cd_enabled {
            client.create_cicd_workflow(&repo.clone_url, &config.app_type, &config.environment).await?;
            log::info!("CI/CD workflow created for {}/{}", config.organization, config.app_name);
        }

        // Build deployment URL
        let url = self.build_deployment_url(&config);

        Ok(DeploymentResult {
            url,
            repository: repo.clone_url,
            app_type: config.app_type.to_string(),
            environment: config.environment.to_string(),
            status: if config.ci_cd_enabled {
                DeploymentStatus::Building
            } else {
                DeploymentStatus::Deployed
            },
            metadata: serde_json::json!({
                "org": config.organization,
                "app_name": config.app_name,
                "repo_id": repo.id,
                "forgejo_url": self.forgejo_url,
                "custom_domain": config.custom_domain,
            }),
        })
    }

    /// Build deployment URL based on environment and domain
    fn build_deployment_url(&self, config: &DeploymentConfig) -> String {
        if let Some(ref domain) = config.custom_domain {
            format!("https://{}/", domain)
        } else {
            match config.environment {
                DeploymentEnvironment::Production => {
                    format!("https://{}.gb.solutions/", config.app_name)
                }
                DeploymentEnvironment::Staging => {
                    format!("https://{}-staging.gb.solutions/", config.app_name)
                }
                DeploymentEnvironment::Development => {
                    format!("https://{}-dev.gb.solutions/", config.app_name)
                }
            }
        }
    }
}
