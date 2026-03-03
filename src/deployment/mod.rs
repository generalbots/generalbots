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

pub mod types;
pub mod router;
pub mod handlers;
pub mod forgejo;

// Re-export commonly used types from types module
pub use types::{
    AppType,
    DeploymentConfig,
    DeploymentEnvironment,
    DeploymentResult,
    DeploymentStatus,
    DeploymentError,
    GeneratedApp,
    GeneratedFile,
    DeploymentRequest,
    DeploymentResponse,
    AppTypesResponse,
    AppTypeInfo,
    DeploymentApiError,
};

// Re-export deployment router
pub use router::DeploymentRouter;

// Re-export route configuration function
pub use handlers::configure_deployment_routes;

// Re-export Forgejo types
pub use forgejo::{ForgejoClient, ForgejoError, ForgejoRepo};
