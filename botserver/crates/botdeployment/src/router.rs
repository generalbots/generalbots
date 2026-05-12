//! Deployment router for VibeCode platform - Phase 2.5
//!
//! All apps are deployed to Forgejo repositories using org/app_name format.

use super::forgejo::ForgejoClient;
use super::types::*;

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

        let token = self.forgejo_token.clone()
            .ok_or_else(|| DeploymentError::ConfigurationError("FORGEJO_TOKEN not configured".to_string()))?;

        let client = ForgejoClient::new(self.forgejo_url.clone(), token);

        let repo = client.create_repository(
            &config.app_name,
            &generated_app.description,
            false,
        ).await?;

        log::info!("Repository created/verified: {}", repo.clone_url);

        let branch = config.environment.to_string();
        client.push_app(&repo.clone_url, &generated_app, &branch).await?;

        if config.ci_cd_enabled {
            client.create_cicd_workflow(&repo.clone_url, &config.app_type, &config.environment).await?;
            log::info!("CI/CD workflow created for {}/{}", config.organization, config.app_name);
        }

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
