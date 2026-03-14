use axum::{
    extract::{
        query::Query,
        State,
        WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum::extract::ws::{Message, WebSocket};
use log::{error, info, warn};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{mpsc, Mutex, RwLock},
};

use crate::core::shared::state::AppState;
use crate::core::urls::ApiUrls;

pub fn configure_terminal_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::TERMINAL_WS, get(terminal_ws))
        .route(ApiUrls::TERMINAL_LIST, get(list_terminals))
        .route(ApiUrls::TERMINAL_CREATE, post(create_terminal))
        .route(ApiUrls::TERMINAL_KILL, post(kill_terminal))
}

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
}

#[derive(Debug, Clone)]
pub enum TerminalOutput {
    Stdout(String),
    Stderr(String),
    System(String),
}

impl TerminalSession {
    pub fn new(session_id: &str) -> Self {
        let container_name = format!(
            "term-{}",
            session_id.chars().take(12).collect::<String>()
        );

        let (output_tx, _) = mpsc::channel(100);

        Self {
            session_id: session_id.to_string(),
            container_name,
            process: None,
            stdin: None,
            output_tx,
        }
    }

    pub fn output_receiver(&self) -> mpsc::Receiver<TerminalOutput> {
        self.output_tx.clone().receiver()
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if !self.container_name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err("Invalid container name".to_string());
        }

        info!("Starting LXC container: {}", self.container_name);

        let launch_output = Command::new("lxc")
            .args(["launch", "ubuntu:22.04", &self.container_name, "-e"])
            .output()
            .await
            .map_err(|e| format!("Failed to launch container: {}", e))?;

        if !launch_output.status.success() {
            let stderr = String::from_utf8_lossy(&launch_output.stderr);
            if !stderr.contains("already exists") && !stderr.contains("is already running") {
                warn!("Container launch warning: {}", stderr);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Starting bash shell in container: {}", self.container_name);

        let mut child = Command::new("lxc")
            .args(["exec", &self.container_name, "--", "bash", "-l"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn bash: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.process = Some(child);

        let tx = self.output_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx.send(TerminalOutput::Stdout(format!("{}\r\n", line))).await.is_err() {
                    break;
                }
            }
        });

        let tx = self.output_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx.send(TerminalOutput::Stderr(format!("{}\r\n", line))).await.is_err() {
                    break;
                }
            }
        });

        let tx = self.output_tx.clone();
        tx.send(TerminalOutput::System(
            "Container started. Welcome to your isolated terminal!\r\n".to_string(),
        ))
        .await
        .ok();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.send_command("export TERM=xterm-256color; clear").await?;

        Ok(())
    }

    pub async fn send_command(&self, cmd: &str) -> Result<(), String> {
        if let Some(stdin_mutex) = &self.stdin {
            let mut stdin = stdin_mutex.lock().await;
            let cmd_with_newline = format!("{}\n", cmd);
            stdin
                .write_all(cmd_with_newline.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }
        Ok(())
    }

    pub async fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
        Ok(())
    }

    pub async fn kill(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }

        let _ = Command::new("lxc")
            .args(["stop", &self.container_name, "-f"])
            .output()
            .await;

        let _ = Command::new("lxc")
            .args(["delete", &self.container_name, "-f"])
            .output()
            .await;

        info!("Container {} destroyed", self.container_name);
        Ok(())
    }
}

pub struct TerminalManager {
    sessions: RwLock<HashMap<String, TerminalSession>>,
}

impl TerminalManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub async fn create_session(&self, session_id: &str) -> Result<TerminalInfo, String> {
        let mut sessions = self.sessions.write().await;

        if sessions.contains_key(session_id) {
            return Err("Session already exists".to_string());
        }

        let mut session = TerminalSession::new(session_id);
        session.start().await?;

        let info = TerminalInfo {
            session_id: session.session_id.clone(),
            container_name: session.container_name.clone(),
            status: "running".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        sessions.insert(session_id.to_string(), session);

        Ok(info)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<TerminalSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn kill_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.remove(session_id) {
            session.kill().await?;
        }
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<TerminalInfo> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .map(|s| TerminalInfo {
                session_id: s.session_id.clone(),
                container_name: s.container_name.clone(),
                status: "running".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .collect()
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize)]
pub struct TerminalQuery {
    session_id: Option<String>,
}

pub async fn terminal_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TerminalQuery>,
) -> impl IntoResponse {
    let session_id = query.session_id.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))
            .unwrap_or_else(|_| std::time::Duration::ZERO)
            .as_millis();
        format!("term-{}", timestamp)
    });

    info!("Terminal WebSocket connection request: {}", session_id);

    ws.on_upgrade(move |socket| handle_terminal_ws(socket, state, session_id))
}

async fn handle_terminal_ws(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: String,
) {
    let (mut sender, mut receiver) = socket.split();

    let terminal_manager = state.terminal_manager.clone();
    let session = match terminal_manager.create_session(&session_id).await {
        Ok(info) => {
            info!("Created terminal session: {:?}", info);
            let welcome = serde_json::json!({
                "type": "connected",
                "session_id": session_id,
                "container": info.container_name,
                "message": "Terminal session created"
            });
            if let Ok(welcome_str) = serde_json::to_string(&welcome) {
                let _ = sender.send(Message::Text(welcome_str)).await;
            }
            terminal_manager.get_session(&session_id).await
        }
        Err(e) => {
            error!("Failed to create terminal session: {}", e);
            let error_msg = serde_json::json!({
                "type": "error",
                "message": e
            });
            let _ = sender
                .send(Message::Text(error_msg.to_string()))
                .await;
            return;
        }
    };

    let Some(mut session) = session else {
        error!("Failed to get session after creation");
        return;
    };

    let output_rx = session.output_receiver();
    let session_id_clone = session_id.clone();
    let terminal_manager_clone = terminal_manager.clone();

    let mut send_task = tokio::spawn(async move {
        let mut rx = output_rx;
        let mut sender = sender;

        while let Some(output) = rx.recv().await {
            let msg = match output {
                TerminalOutput::Stdout(s) | TerminalOutput::Stderr(s) => {
                    Message::Text(s)
                }
                TerminalOutput::System(s) => {
                    Message::Text(serde_json::json!({
                        "type": "system",
                        "message": s
                    }).to_string())
                }
            };

            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let session_id_clone2 = session_id.clone();
    let terminal_manager_clone2 = terminal_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Some(session) = terminal_manager_clone2.get_session(&session_id_clone2).await {
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        if trimmed == "\\exit" || trimmed == "exit" {
                            let _ = terminal_manager_clone2.kill_session(&session_id_clone2).await;
                            break;
                        }

                        if trimmed.starts_with("resize ") {
                            let parts: Vec<&str> = trimmed.split_whitespace().collect();
                            if parts.len() >= 3 {
                                if let (Ok(cols), Ok(rows)) = (
                                    parts[1].parse::<u16>(),
                                    parts[2].parse::<u16>(),
                                ) {
                                    let _ = session.resize(cols, rows).await;
                                }
                            }
                            continue;
                        }

                        if let Err(e) = session.send_command(trimmed).await {
                            error!("Failed to send command: {}", e);
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => break,
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => {
            warn!("Terminal send task ended");
        }
        _ = &mut recv_task => {
            info!("Terminal client disconnected");
        }
    }

    if let Err(e) = terminal_manager.kill_session(&session_id).await {
        error!("Failed to cleanup terminal session: {}", e);
    }

    info!("Terminal session {} cleaned up", session_id);
}

#[derive(serde::Deserialize)]
pub struct CreateTerminalRequest {
    session_id: Option<String>,
}

pub async fn create_terminal(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTerminalRequest>,
) -> impl IntoResponse {
    let session_id = payload.session_id.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))
            .unwrap_or_else(|_| std::time::Duration::ZERO)
            .as_millis();
        format!("term-{}", timestamp)
    });

    match state.terminal_manager.create_session(&session_id).await {
        Ok(info) => (
            axum::http::StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "terminal": info
            })),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        ),
    }
}

pub async fn kill_terminal(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if session_id.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "session_id is required"
            })),
        );
    }

    match state.terminal_manager.kill_session(session_id).await {
        Ok(()) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Terminal session killed"
            })),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        ),
    }
}

pub async fn list_terminals(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let terminals = state.terminal_manager.list_sessions().await;
    Json(serde_json::json!({
        "terminals": terminals
    }))
}
