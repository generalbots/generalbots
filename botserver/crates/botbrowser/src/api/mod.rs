pub(crate) use std::net::IpAddr;
pub(crate) use std::sync::Arc;
pub(crate) use axum::{ extract::{Path, Query, State, WebSocketUpgrade}, extract::ws::{Message, WebSocket}, response::IntoResponse, response::Json, routing::{delete, get, post}, Router, };
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use tokio::sync::Mutex;
pub(crate) use crate::{agent, BrowserSession, SessionMap};

#[derive(Deserialize)]
pub struct ProxyParams {
    pub url: String,
}

#[cfg(test)]
mod tests {
use super::*;

#[test]
fn private_ip_detection() {
    assert!(is_private_ip("127.0.0.1".parse().unwrap()));
    assert!(is_private_ip("192.168.1.1".parse().unwrap()));
    assert!(is_private_ip("10.0.0.1".parse().unwrap()));
    assert!(is_private_ip("::1".parse().unwrap()));
    assert!(!is_private_ip("8.8.8.8".parse().unwrap()));
    assert!(!is_private_ip("93.184.216.34".parse().unwrap()));
}
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub headless: Option<bool>,
}

#[derive(Deserialize)]
pub struct NavigateRequest {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ClickRequest {
    pub selector: String,
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
mod handle_proxy;
mod create_session;
mod handle_navigate;
mod handle_click;
mod handle_fill;
mod handle_execute;
mod handle_agent;
mod handle_agent_ws_stream;

pub use configure_routes::*;
use handle_proxy::*;
pub use create_session::*;
pub use handle_navigate::*;
pub use handle_click::*;
pub use handle_fill::*;
pub use handle_execute::*;
pub use handle_agent::*;
pub use handle_agent_ws_stream::*;
