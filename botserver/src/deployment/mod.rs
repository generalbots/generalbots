use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botdeployment::{
    DeploymentRouter, ForgejoClient, ForgejoError, ForgejoRepo,
    AppType, DeploymentConfig, DeploymentEnvironment, DeploymentResult, DeploymentStatus,
    DeploymentError, GeneratedApp, GeneratedFile, DeploymentRequest, DeploymentResponse,
    AppTypesResponse, AppTypeInfo, DeploymentApiError,
};

pub fn configure_deployment_routes() -> Router<Arc<AppState>> {
    botdeployment::configure_deployment_routes::<AppState>()
}
