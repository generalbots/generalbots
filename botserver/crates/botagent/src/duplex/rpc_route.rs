//! Reads a JSON-RPC session in order: responses go to their waiting
//! requests, everything else to the inbox; unhandled server requests are
//! refused here.

use serde_json::{Map, Value};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::rpc::{
    error_reply, Incoming, RpcError, RpcShared, ServerRequest, INTERNAL_ERROR, METHOD_NOT_FOUND,
};
use super::session::DuplexEvents;
use super::types::DuplexEvent;

pub(super) async fn dispatch(
    mut events: DuplexEvents,
    shared: Arc<RpcShared>,
    inbox: mpsc::UnboundedSender<Incoming>,
) {
    while let Some(event) = events.recv().await {
        match event {
            DuplexEvent::Line(line) => route(&shared, &inbox, line),
            DuplexEvent::Ended(end) => {
                shared.fail_all(&end);
                let _ = inbox.send(Incoming::Ended(end));
                return;
            }
        }
    }
}

fn route(shared: &RpcShared, inbox: &mpsc::UnboundedSender<Incoming>, line: String) {
    let mut object = match serde_json::from_str::<Value>(&line) {
        Ok(Value::Object(object)) => object,
        _ => {
            let _ = inbox.send(Incoming::Unparsed(line));
            return;
        }
    };
    let method = match object.remove("method") {
        Some(Value::String(method)) => Some(method),
        Some(_) => {
            let _ = inbox.send(Incoming::Unparsed(line));
            return;
        }
        None => None,
    };
    let id = object.remove("id");
    let params = object.remove("params").unwrap_or(Value::Null);
    match (method, id) {
        (Some(method), Some(id)) => server_request(shared, inbox, id, method, params),
        (Some(method), None) => {
            let _ = inbox.send(Incoming::Notification { method, params });
        }
        (None, Some(id)) => complete(shared, &id, object),
        (None, None) => {
            let _ = inbox.send(Incoming::Unparsed(line));
        }
    }
}

fn server_request(
    shared: &RpcShared,
    inbox: &mpsc::UnboundedSender<Incoming>,
    id: Value,
    method: String,
    params: Value,
) {
    if !shared.accepted.contains(&method) {
        tracing::debug!("server request {method} is not handled: refused");
        let _ = shared.sender.send_json(&error_reply(
            &id,
            METHOD_NOT_FOUND,
            "method not handled by this client",
        ));
        return;
    }
    let request = ServerRequest {
        method,
        params,
        id,
        reply: Some(shared.sender.clone()),
    };
    // A closed inbox hands the request back and drops it: it is denied.
    let _ = inbox.send(Incoming::Request(request));
}

fn complete(shared: &RpcShared, id: &Value, mut object: Map<String, Value>) {
    let Some(waiter) = id
        .as_u64()
        .and_then(|key| shared.pending().waiters.remove(&key))
    else {
        tracing::debug!("response to an unknown or expired request id dropped");
        return;
    };
    let outcome = match object.remove("error") {
        Some(error) => Err(RpcError::Remote {
            code: error
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or(INTERNAL_ERROR),
            message: error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            data: error.get("data").cloned(),
        }),
        None => Ok(object.remove("result").unwrap_or(Value::Null)),
    };
    let _ = waiter.send(outcome);
}
