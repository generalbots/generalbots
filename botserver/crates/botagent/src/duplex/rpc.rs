//! Line-delimited JSON-RPC 2.0 over a [`DuplexSession`].
//!
//! Works with peers that omit the `"jsonrpc"` member in what they send (the
//! Codex app server) and with strict ones (ACP); every message this client
//! writes carries `"jsonrpc":"2.0"`.
//!
//! - [`RpcClient::request`] sends a request with a fresh numeric id and waits
//!   for its response, up to a per-request timeout.
//! - Server-initiated requests and notifications arrive in order on the
//!   [`RpcInbox`]. Only methods named in [`RpcOptions`] reach it; any other
//!   server request is answered with `-32601` at once (fail closed).
//! - Every [`ServerRequest`] is answered exactly once: dropping it without an
//!   answer sends a `-32603` error, so a forgotten or failed handler denies.
//! - When the session ends, every pending request fails with
//!   [`RpcError::SessionEnded`] and the inbox yields [`Incoming::Ended`].

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::{mpsc, oneshot};

use super::rpc_route::dispatch;
use super::session::{DuplexSender, DuplexSession};
use super::types::{DuplexError, SessionEnd};

pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INTERNAL_ERROR: i64 = -32603;

#[derive(Debug, Clone)]
pub enum RpcError {
    /// The peer answered with a JSON-RPC error object.
    Remote {
        code: i64,
        message: String,
        data: Option<Value>,
    },
    /// No response within the request's timeout; the session stays open.
    Timeout { method: String, after: Duration },
    /// The session ended before the response arrived.
    SessionEnded(SessionEnd),
    /// The message could not be written.
    Transport(DuplexError),
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Remote { code, message, .. } => write!(f, "peer error {code}: {message}"),
            Self::Timeout { method, after } => {
                write!(f, "no answer to {method} within {} ms", after.as_millis())
            }
            Self::SessionEnded(end) => write!(
                f,
                "session ended ({:?}, exit code {:?})",
                end.reason, end.exit_code
            ),
            Self::Transport(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RpcError {}

/// Which server-initiated request methods the caller handles.
#[derive(Debug, Clone, Default)]
pub struct RpcOptions {
    accepted: HashSet<String>,
}

impl RpcOptions {
    /// Server requests named here reach the inbox; all others are refused.
    pub fn accepting<I, S>(methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            accepted: methods.into_iter().map(Into::into).collect(),
        }
    }
}

/// One message from the peer, in arrival order.
#[derive(Debug)]
pub enum Incoming {
    Notification {
        method: String,
        params: Value,
    },
    Request(ServerRequest),
    /// A line that is not a JSON-RPC object (kept for diagnostics only).
    Unparsed(String),
    /// Always last.
    Ended(SessionEnd),
}

/// A request from the peer. Answer it with [`Self::respond`] or
/// [`Self::respond_error`]; dropping it answers `-32603`.
#[derive(Debug)]
pub struct ServerRequest {
    pub method: String,
    pub params: Value,
    pub(super) id: Value,
    pub(super) reply: Option<DuplexSender>,
}

impl ServerRequest {
    pub fn id(&self) -> &Value {
        &self.id
    }

    pub fn respond(mut self, result: Value) -> Result<(), RpcError> {
        let message = json!({"jsonrpc": "2.0", "id": self.id, "result": result});
        self.send(&message)
    }

    pub fn respond_error(mut self, code: i64, message: &str) -> Result<(), RpcError> {
        let message = error_reply(&self.id, code, message);
        self.send(&message)
    }

    fn send(&mut self, message: &Value) -> Result<(), RpcError> {
        match self.reply.take() {
            Some(sender) => sender.send_json(message).map_err(RpcError::Transport),
            None => Ok(()),
        }
    }
}

impl Drop for ServerRequest {
    fn drop(&mut self) {
        if let Some(sender) = self.reply.take() {
            tracing::debug!(
                "server request {} dropped without an answer: denied",
                self.method
            );
            if let Err(error) = sender.send_json(&error_reply(
                &self.id,
                INTERNAL_ERROR,
                "request was not answered",
            )) {
                tracing::debug!("server request {} could not report its denial: {error}", self.method);
            }
        }
    }
}

pub(super) fn error_reply(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// Ordered stream of what the peer sends besides responses.
#[derive(Debug)]
pub struct RpcInbox {
    incoming: mpsc::UnboundedReceiver<Incoming>,
}

impl RpcInbox {
    /// The next message; `None` after [`Incoming::Ended`].
    pub async fn next(&mut self) -> Option<Incoming> {
        self.incoming.recv().await
    }

    /// The next message if one is already queued.
    pub fn try_next(&mut self) -> Option<Incoming> {
        self.incoming.try_recv().ok()
    }
}

pub(super) type Waiter = oneshot::Sender<Result<Value, RpcError>>;

#[derive(Default)]
pub(super) struct Pending {
    pub(super) waiters: HashMap<u64, Waiter>,
    pub(super) ended: Option<SessionEnd>,
}

pub(super) struct RpcShared {
    pub(super) sender: DuplexSender,
    pending: Mutex<Pending>,
    next_id: AtomicU64,
    pub(super) accepted: HashSet<String>,
}

impl RpcShared {
    pub(super) fn pending(&self) -> std::sync::MutexGuard<'_, Pending> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn fail_all(&self, end: &SessionEnd) {
        let waiters = {
            let mut pending = self.pending();
            pending.ended = Some(end.clone());
            std::mem::take(&mut pending.waiters)
        };
        for (_, waiter) in waiters {
            if waiter.send(Err(RpcError::SessionEnded(end.clone()))).is_err() {
                tracing::debug!("a pending request was already gone when the session ended");
            }
        }
    }
}

/// The client half. Dropping it closes the session (stdin, grace, kill).
pub struct RpcClient {
    shared: Arc<RpcShared>,
}

impl RpcClient {
    /// Takes over the session and starts reading it. Must run inside Tokio.
    pub fn start(session: DuplexSession, options: RpcOptions) -> (Self, RpcInbox) {
        let (sender, events) = session.into_parts();
        let shared = Arc::new(RpcShared {
            sender,
            pending: Mutex::new(Pending::default()),
            next_id: AtomicU64::new(1),
            accepted: options.accepted,
        });
        let (inbox_tx, inbox_rx) = mpsc::unbounded_channel();
        tokio::spawn(dispatch(events, shared.clone(), inbox_tx));
        (Self { shared }, RpcInbox { incoming: inbox_rx })
    }

    /// Sends a request and waits for its result.
    pub async fn request(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, RpcError> {
        let id = self.shared.next_id.fetch_add(1, Ordering::Relaxed);
        let (waiter, answer) = oneshot::channel();
        {
            let mut pending = self.shared.pending();
            if let Some(end) = &pending.ended {
                return Err(RpcError::SessionEnded(end.clone()));
            }
            pending.waiters.insert(id, waiter);
        }
        let mut message = json!({"jsonrpc": "2.0", "id": id, "method": method});
        if !params.is_null() {
            message["params"] = params;
        }
        if let Err(error) = self.shared.sender.send_json(&message) {
            self.shared.pending().waiters.remove(&id);
            return Err(RpcError::Transport(error));
        }
        match tokio::time::timeout(timeout, answer).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => Err(RpcError::SessionEnded(
                self.shared.sender.wait_ended().await,
            )),
            Err(_) => {
                self.shared.pending().waiters.remove(&id);
                Err(RpcError::Timeout {
                    method: method.to_string(),
                    after: timeout,
                })
            }
        }
    }

    /// Sends a notification (no id, no answer).
    pub fn notify(&self, method: &str, params: Value) -> Result<(), RpcError> {
        let mut message = json!({"jsonrpc": "2.0", "method": method});
        if !params.is_null() {
            message["params"] = params;
        }
        self.shared
            .sender
            .send_json(&message)
            .map_err(RpcError::Transport)
    }

    /// The underlying sender, for raw lines or the ended signal.
    pub fn sender(&self) -> &DuplexSender {
        &self.shared.sender
    }

    /// Closes the session and waits for its end.
    pub async fn close(self) -> SessionEnd {
        self.shared.sender.close();
        self.shared.sender.wait_ended().await
    }
}

impl Drop for RpcClient {
    fn drop(&mut self) {
        self.shared.sender.close();
    }
}
