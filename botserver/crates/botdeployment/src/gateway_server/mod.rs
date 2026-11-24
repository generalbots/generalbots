//! Deploy Gateway Server — receives requests from alm-ci and performs
//! privileged operations on the host (Incus container management, Caddy config).
//!
//! This server validates the X-Deploy-Key header and never exposes
//! Forgejo tokens, SSH keys, or Incus socket access to callers.

pub(crate) use axum::{ extract::{Path, State}, http::{HeaderMap, StatusCode}, routing::{get, post}, Json, Router, };
pub(crate) use serde::Deserialize;
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::Arc;
pub(crate) use tokio::sync::RwLock;
pub(crate) use super::types::*;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayState {
    #[serde(default)]
    deploy_keys: Vec<String>,
    /// True only when the key came from Vault/env. An ephemeral fail-closed
    /// key must never justify exposing the listener beyond loopback (#2613).
    #[serde(default)]
    key_from_config: bool,
    pub(crate) incus_socket: String,
    caddy_api_url: String,
    sites_root: String,
}

impl GatewayState {
    /// Whether a real per-deployment secret was configured (as opposed to the
    /// ephemeral key generated when nothing usable was supplied).
    pub fn deploy_key_is_configured(&self) -> bool {
        self.key_from_config
    }
}

#[derive(Debug)]
pub struct ContainerRegistry {
    containers: RwLock<HashMap<String, ContainerInfo>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub org: String,
    pub project_type: ProjectType,
    pub status: String,
    pub url: String,
}

mod published_domain;
mod gateway_key;
mod gateway_state;
mod container_registry;
mod configure_gateway_routes;
mod deploy_to_caddy;
mod rate_limit;

pub(crate) use published_domain::*;
pub(crate) use gateway_key::*;
pub use configure_gateway_routes::*;
use deploy_to_caddy::*;
use rate_limit::*;
