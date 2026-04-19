//! API handlers for VibeCode deployment module - Phase 2.5

use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;

use crate::core::shared::state::AppState;

use super::types::*;
use super::router::DeploymentRouter;


/// Configure deployment routes
pub fn configure_deployment_routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/api/deployment/types", axum::routing::get(get_app_types))
        .route("/api/deployment/deploy", axum::routing::post(deploy_app))
}

/// Get available app types
pub async fn get_app_types(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<AppTypesResponse>, DeploymentApiError> {
    let app_types = vec![
        AppTypeInfo {
            id: "gb-native".to_string(),
            name: "GB Native".to_string(),
            description: "Optimized for General Bots platform with shared resources".to_string(),
            features: vec![
                "Shared database connection pool".to_string(),
                "Integrated GB authentication".to_string(),
                "Shared caching layer".to_string(),
                "Auto-scaling".to_string(),
                "Built-in monitoring".to_string(),
                "Zero configuration".to_string(),
            ],
        },
        AppTypeInfo {
            id: "custom-htmx".to_string(),
            name: "Custom HTMX".to_string(),
            description: "HTMX-based application with custom deployment".to_string(),
            features: vec![
                "Lightweight frontend".to_string(),
                "Server-side rendering".to_string(),
                "Custom CI/CD pipeline".to_string(),
                "Independent deployment".to_string(),
            ],
        },
        AppTypeInfo {
            id: "custom-react".to_string(),
            name: "Custom React".to_string(),
            description: "React application with custom deployment".to_string(),
            features: vec![
                "Modern React".to_string(),
                "Vite build system".to_string(),
                "Custom CI/CD pipeline".to_string(),
                "Independent deployment".to_string(),
            ],
        },
        AppTypeInfo {
            id: "custom-vue".to_string(),
            name: "Custom Vue".to_string(),
            description: "Vue.js application with custom deployment".to_string(),
            features: vec![
                "Vue 3 composition API".to_string(),
                "Vite build system".to_string(),
                "Custom CI/CD pipeline".to_string(),
                "Independent deployment".to_string(),
            ],
        },
    ];

    Ok(Json(AppTypesResponse { app_types }))
}

/// Deploy an application to Forgejo
pub async fn deploy_app(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<DeploymentRequest>,
) -> Result<Json<DeploymentResponse>, DeploymentApiError> {
    log::info!(
        "Deployment request: org={:?}, app={}, type={}, env={}",
        request.organization,
        request.app_name,
        request.app_type,
        request.environment
    );

    // Parse app type
    let app_type = match request.app_type.as_str() {
        "gb-native" => AppType::GbNative {
            shared_database: request.shared_database.unwrap_or(true),
            shared_auth: request.shared_auth.unwrap_or(true),
            shared_cache: request.shared_cache.unwrap_or(true),
        },
        custom_type if custom_type.starts_with("custom-") => {
            let framework = request.framework.clone()
                .unwrap_or_else(|| custom_type.strip_prefix("custom-").unwrap_or("unknown").to_string());

            AppType::Custom {
                framework,
                node_version: Some("20".to_string()),
                build_command: Some("npm run build".to_string()),
                output_directory: Some("dist".to_string()),
            }
        }
        _ => {
            return Err(DeploymentApiError::ValidationError(format!(
                "Unknown app type: {}",
                request.app_type
            )));
        }
    };

    // Parse environment
    let environment = match request.environment.as_str() {
        "development" => DeploymentEnvironment::Development,
        "staging" => DeploymentEnvironment::Staging,
        "production" => DeploymentEnvironment::Production,
        _ => DeploymentEnvironment::Development,
    };

    // Get Forgejo configuration
    let forgejo_url = std::env::var("FORGEJO_URL")
        .unwrap_or_else(|_| "https://alm.pragmatismo.com.br".to_string());

    let forgejo_token = std::env::var("FORGEJO_TOKEN").ok();

    // Get or default organization
    let organization = request.organization
        .or_else(|| std::env::var("FORGEJO_DEFAULT_ORG").ok())
        .unwrap_or_else(|| "generalbots".to_string());

    // Create deployment configuration
    let config = DeploymentConfig {
        organization,
        app_name: request.app_name.clone(),
        app_type,
        environment,
        custom_domain: request.custom_domain,
        ci_cd_enabled: request.ci_cd_enabled.unwrap_or(true),
    };

    // Create deployment router
    let router = DeploymentRouter::new(forgejo_url, forgejo_token);

    // Create placeholder generated app
    // In real implementation, this would come from the orchestrator
    let generated_app = GeneratedApp::new(
        config.app_name.clone(),
        format!("Generated {} application", config.app_type),
    );

    // Execute deployment
    let result = router.deploy(config, generated_app).await
        .map_err(|e| DeploymentApiError::DeploymentFailed(e.to_string()))?;

    log::info!(
        "Deployment successful: url={}, repo={}, status={:?}",
        result.url,
        result.repository,
        result.status
    );

    Ok(Json(DeploymentResponse {
        success: true,
        url: Some(result.url),
        repository: Some(result.repository),
        app_type: Some(result.app_type),
        status: Some(format!("{:?}", result.status)),
        error: None,
    }))
}
