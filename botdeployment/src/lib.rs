//! Deployment module for VibeCode platform - Phase 2.5
//!
//! All apps are deployed to Forgejo repositories using org/app_name format.
//! Two app types: GB Native (optimized for GB platform) and Custom (any framework).
//!
//! # Architecture
//!
//! - `types` - Type definitions for deployment configuration and results
//! - `router` - Deployment router that manages the deployment process
//! - `handlers` - HTTP API handlers for deployment endpoints
//! - `forgejo` - Forgejo client for repository management

pub mod forgejo;
pub mod handlers;
pub mod router;
pub mod types;

pub use forgejo::{ForgejoClient, ForgejoError, ForgejoRepo};
pub use handlers::configure_deployment_routes;
pub use router::DeploymentRouter;
pub use types::{
    AppType, AppTypeInfo, AppTypesResponse, DeploymentApiError, DeploymentConfig,
    DeploymentEnvironment, DeploymentError, DeploymentRequest, DeploymentResponse,
    DeploymentResult, DeploymentStatus, GeneratedApp, GeneratedFile,
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
