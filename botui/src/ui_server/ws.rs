use axum::{
    extract::{
        ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
        OriginalUri, Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use serde::Deserialize;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite,
    tungstenite::protocol::Message as TungsteniteMessage,
};

use crate::shared::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    user_id: String,
    pub bot_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OptionalWsQuery {
    task_id: Option<String>,
}

pub async fn ws_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    let path_parts: Vec<&str> = uri.path().split('/').collect();
    let bot_name = params
        .bot_name
        .filter(|name| name != "ws" && !name.is_empty())
        .or_else(|| {
            path_parts
                .iter()
                .find(|part| {
                    !part.is_empty() && **part != "chat" && **part != "app" && **part != "ws" && **part != "cloud"
                })
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "default".to_string());

    let params_with_bot = WsQuery {
        bot_name: Some(bot_name),
        ..params
    };

    ws.on_upgrade(move |socket| handle_ws_proxy(socket, state, params_with_bot))
}

async fn handle_ws_proxy(
    client_socket: WebSocket,
    state: AppState,
    params: WsQuery,
) {
    let bot_name = params.bot_name.unwrap_or_else(|| "default".to_string());
    let backend_url = format!(
        "{}/ws?bot_name={}&session_id={}&user_id={}",
        state
            .client
            .base_url()
            .replace("https://", "wss://")
            .replace("http://", "ws://"),
        bot_name,
        params.session_id,
        params.user_id
    );

    info!("Proxying WebSocket to: {backend_url}");

    let Ok(tls_connector) = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
    else {
        error!("Failed to build TLS connector for WebSocket proxy");
        return;
    };

    let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);

    let backend_result =
        connect_async_tls_with_config(&backend_url, None, false, Some(connector)).await;

    let backend_socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    > = match backend_result {
        Ok((socket, _)) => socket,
        Err(e) => {
            error!("Failed to connect to backend WebSocket: {e}");
            return;
        }
    };

    info!("Connected to backend WebSocket");

    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut backend_tx, mut backend_rx) = backend_socket.split();

    let client_to_backend = async {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(AxumMessage::Text(text)) => {
                    if backend_tx.send(TungsteniteMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Binary(data)) => {
                    if backend_tx.send(TungsteniteMessage::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Ping(data)) => {
                    if backend_tx.send(TungsteniteMessage::Ping(data)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Pong(data)) => {
                    if backend_tx.send(TungsteniteMessage::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Close(_)) | Err(_) => break,
            }
        }
    };

    let backend_to_client = async {
        while let Some(msg) = backend_rx.next().await {
            match msg {
                Ok(TungsteniteMessage::Text(text)) => {
                    if client_tx.send(AxumMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Binary(data)) => {
                    if client_tx.send(AxumMessage::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Ping(data)) => {
                    if client_tx.send(AxumMessage::Ping(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Pong(data)) => {
                    if client_tx.send(AxumMessage::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Close(_)) | Err(_) => break,
                Ok(_) => {}
            }
        }
    };

    tokio::select! {
        () = client_to_backend => info!("Client connection closed"),
        () = backend_to_client => info!("Backend connection closed"),
    }
}

pub async fn ws_task_progress_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<OptionalWsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_task_progress_ws_proxy(socket, state, params))
}

async fn handle_task_progress_ws_proxy(
    client_socket: WebSocket,
    state: AppState,
    params: OptionalWsQuery,
) {
    let mut backend_url = format!(
        "{}/ws/task-progress",
        state
            .client
            .base_url()
            .replace("https://", "wss://")
            .replace("http://", "ws://"),
    );

    if let Some(task_id) = &params.task_id {
        backend_url = format!("{}/{}", backend_url, task_id);
    }

    info!("Proxying task-progress WebSocket to: {backend_url}");

    let Ok(tls_connector) = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
    else {
        error!("Failed to build TLS connector for task-progress");
        return;
    };

    let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);

    let backend_result =
        connect_async_tls_with_config(&backend_url, None, false, Some(connector)).await;

    let backend_socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    > = match backend_result {
        Ok((socket, _)) => socket,
        Err(e) => {
            error!("Failed to connect to backend task-progress WebSocket: {e}");
            return;
        }
    };

    info!("Connected to backend task-progress WebSocket");

    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut backend_tx, mut backend_rx) = backend_socket.split();

    let client_to_backend = async {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(AxumMessage::Text(text)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Text(text)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Binary(data)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Binary(data)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Ping(data)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Ping(data)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Pong(data)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Pong(data)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Close(_)) | Err(_) => break,
            }
        }
    };

    let backend_to_client = async {
        while let Some(msg) =
            backend_rx.next().await as Option<Result<TungsteniteMessage, tungstenite::Error>>
        {
            match msg {
                Ok(TungsteniteMessage::Text(text)) => {
                    let is_manifest = text.contains("manifest_update");
                    if is_manifest {
                    } else if text.contains("task_progress") {
                        debug!("[WS_PROXY] Forwarding task_progress to client");
                    }
                    match client_tx.send(AxumMessage::Text(text)).await {
                        Ok(()) => {
                            if is_manifest {
                            }
                        }
                        Err(e) => {
                            error!("[WS_PROXY] Failed to send message to client: {:?}", e);
                            break;
                        }
                    }
                }
                Ok(TungsteniteMessage::Binary(data)) => {
                    if client_tx.send(AxumMessage::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Ping(data)) => {
                    if client_tx.send(AxumMessage::Ping(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Pong(data)) => {
                    if client_tx.send(AxumMessage::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Close(_)) | Err(_) => break,
                Ok(_) => {}
            }
        }
    };

    tokio::select! {
        () = client_to_backend => info!("Task-progress client connection closed"),
        () = backend_to_client => info!("Task-progress backend connection closed"),
    }
}
