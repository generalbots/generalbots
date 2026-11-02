#[cfg(feature = "terminal")]
pub(crate) use axum::{ extract::{ Query, State, WebSocketUpgrade, }, response::IntoResponse, routing::{get, post}, Json, Router, };
pub(crate) use axum::extract::ws::{Message, WebSocket};
pub(crate) use futures_util::{SinkExt, StreamExt};
pub(crate) use log::{error, info, warn};
pub(crate) use std::{ collections::HashMap, process::Stdio, sync::Arc, };
pub(crate) use tokio::{ io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::{Child, ChildStdin}, sync::{mpsc, Mutex, RwLock}, };
pub(crate) use botcore::shared::state::AppState;
pub(crate) use botcore::urls::ApiUrls;
pub(crate) use botsecurity::command_guard::SafeCommand;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TerminalInfo {
    pub session_id: String,
    pub container_name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct TerminalSession {
    pub session_id: String,
    pub container_name: String,
    process: Option<Child>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    output_tx: mpsc::Sender<TerminalOutput>,
    output_rx: Option<mpsc::Receiver<TerminalOutput>>,
}

#[derive(Debug, Clone)]
pub enum TerminalOutput {
    Stdout(String),
    Stderr(String),
    System(String),
}

pub struct TerminalManager {
    sessions: RwLock<HashMap<String, Arc<Mutex<TerminalSession>>>>,
}

#[derive(serde::Deserialize)]
pub struct TerminalQuery {
    session_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateTerminalRequest {
    session_id: Option<String>,
}

mod configure_terminal_routes;
mod take_output_receiver;
mod create_session;
mod terminal_ws;
mod create_terminal;

pub use configure_terminal_routes::*;
pub use terminal_ws::*;
pub use create_terminal::*;
