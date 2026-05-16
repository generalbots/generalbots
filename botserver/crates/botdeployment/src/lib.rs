//! Deployment module for General Bots platform
//!
//! Three project types under the ALM (Forgejo) model:
//! - **bots**: GB Native scripts in .gbdialog/ — no ALM, runs in chat mode
//! - **apps**: Custom full-stack apps — ALM repo → alm-ci builds → own Incus container
//! - **sites**: Static HTML/CSS/JS — ALM repo → alm-ci builds → Caddy serves from proxy container
//!
//! Security: Deploy Gateway uses scoped deploy keys so alm-ci never exposes
//! Forgejo internals or server access to users.
//!
//! # Architecture
//!
//! - `types` - Type definitions for deployment configuration and results
//! - `router` - Deployment router that manages the deployment process
//! - `handlers` - HTTP API handlers for deployment endpoints
//! - `forgejo` - Forgejo client for repository management
//! - `gateway` - Deploy Gateway API for secure alm-ci → host operations

pub mod forgejo;
pub mod gateway;
pub mod gateway_server;
pub mod handlers;
pub mod router;
pub mod types;

pub use forgejo::{ForgejoClient, ForgejoError, ForgejoRepo};
pub use gateway::{DeployGateway, DeployGatewayConfig};
pub use gateway_server::{configure_gateway_routes, ContainerInfo, ContainerRegistry, GatewayState};
pub use handlers::configure_deployment_routes;
pub use router::DeploymentRouter;
pub use types::{
    DeployGatewayRequest, DeployGatewayResponse, DeployKey, DeployTarget, DeploymentApiError,
    DeploymentConfig, DeploymentEnvironment, DeploymentError, DeploymentRequest,
    DeploymentResponse, DeploymentResult, DeploymentStatus, GeneratedApp, GeneratedFile,
    ProjectTypeInfo, ProjectType, ProjectTypesResponse,
};

use std::fmt;

fn log_and_sanitize(error: &dyn std::error::Error, context: &str) -> SanitizedError {
    log::warn!("Error occurred: context={}, error={}", context, error);
    SanitizedError {
        message: "An internal error occurred".to_string(),
    }
}

struct SanitizedError {
    message: String,
}

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}
