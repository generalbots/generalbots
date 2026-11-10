pub(crate) use crate::{agent, BrowserSession, BrowserTab, BrowserTabMeta, SessionMap};
pub(crate) use axum::{
    extract::ws::{Message, WebSocket},
    extract::{Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    response::Json,
    routing::{delete, get, post},
    Router,
};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::sync::Arc;
pub(crate) use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct ProxyParams {
    pub url: String,
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub headless: Option<bool>,
    #[serde(default)]
    pub vm: Option<String>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub vm_controlled: Option<bool>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct AttachTabRequest {
    #[serde(default)]
    pub conversation_id: Option<String>,
}

#[derive(Deserialize)]
pub struct NavigateRequest {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ClickRequest {
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
}

#[derive(Deserialize)]
pub struct FillRequest {
    pub selector: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub script: String,
}

#[derive(Deserialize)]
pub struct AgentRequest {
    pub goal: String,
    pub max_steps: Option<usize>,
    pub llm_url: Option<String>,
    pub llm_key: Option<String>,
    pub llm_model: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct WsAgentParams {
    pub goal: String,
    pub max_steps: Option<usize>,
    pub llm_url: Option<String>,
    pub llm_key: Option<String>,
    pub llm_model: Option<String>,
}

mod configure_routes;
mod create_session;
mod handle_agent;
mod handle_agent_ws_stream;
mod handle_click;
mod handle_execute;
mod handle_fill;
mod handle_navigate;
mod handle_proxy;
mod handle_tabs;

pub use configure_routes::*;
pub use create_session::*;
pub use handle_agent::*;
pub use handle_agent_ws_stream::*;
pub use handle_click::*;
pub use handle_execute::*;
pub use handle_fill::*;
pub use handle_navigate::*;
use handle_proxy::*;
pub use handle_tabs::*;
