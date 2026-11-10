//! HTTP server initialization and routing

pub(crate) use super::routes::{api_setup, security_setup, sub_router};
pub(crate) use crate::security::csrf::{csrf_middleware, CsrfConfig, CsrfManager};
pub(crate) use crate::security::headers::{
    create_security_headers_layer, security_headers_middleware, SecurityHeadersConfig,
};
pub(crate) use crate::security::panic_handler::PanicHandlerConfig;
pub(crate) use crate::security::rate_limiter::{
    auth_rate_limit_middleware, create_rate_limit_layer, CombinedRateLimiter, HttpRateLimitConfig,
};
pub(crate) use crate::security::request_id::request_id_middleware;
pub(crate) use axum::Router;
pub(crate) use botcore::shared::state::AppState;
pub(crate) use botlib::limits::SystemLimits;
pub(crate) use log::{error, info, warn};
pub(crate) use std::sync::Arc;
pub(crate) use tower_http::services::ServeDir;
pub(crate) use tower_http::trace::TraceLayer;

mod run_axum_server;
mod server_layers;

#[cfg(feature = "analytics")]
mod ops_metrics_layer;

pub use run_axum_server::*;
pub(crate) use server_layers::*;

#[cfg(feature = "analytics")]
pub(crate) use ops_metrics_layer::ops_metrics_middleware;
