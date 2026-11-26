//! JSON-RPC client tests against the fake peer's `rpc` mode.

use std::time::Duration;

use serde_json::{json, Value};

use super::tests::spawn_peer;
use super::{DuplexOptions, EndReason, Incoming, RpcClient, RpcError, RpcInbox, RpcOptions};
use super::{INTERNAL_ERROR, METHOD_NOT_FOUND};

const WAIT: Duration = Duration::from_secs(20);
const APPROVAL: &str = "item/approve";

fn start() -> (RpcClient, RpcInbox) {
    let session = spawn_peer("rpc", None, DuplexOptions::default());
    RpcClient::start(session, RpcOptions::accepting([APPROVAL]))
}

async fn next(inbox: &mut RpcInbox) -> Incoming {
    tokio::time::timeout(WAIT, inbox.next())
        .await
        .expect("an inbox message in time")
        .expect("an inbox message before the end")
}

#[tokio::test]
async fn request_gets_its_result_or_its_error() {
    let (client, _inbox) = start();
    let params = json!({"text": "ação 🚀", "n": [1, 2]});
    let result = client
        .request("echo", params.clone(), WAIT)
        .await
        .expect("echo");
    assert_eq!(result, params);
    match client.request("fail", json!({"why": "test"}), WAIT).await {
        Err(RpcError::Remote {
            code,
            message,
            data,
        }) => {
            assert_eq!(code, -32000);
            assert_eq!(message, "nope");
            assert_eq!(data, Some(json!({"why": "test"})));
        }
        other => panic!("expected a remote error, got {other:?}"),
    }
    assert!(client.notify("ignored", Value::Null).is_ok());
    assert_eq!(client.close().await.reason, EndReason::Closed);
}

#[tokio::test]
async fn notifications_arrive_in_order() {
    let (client, mut inbox) = start();
    assert_eq!(
        client
            .request("notify3", Value::Null, WAIT)
            .await
            .expect("notify3"),
        json!("done")
    );
    for expected in 0..3 {
        match next(&mut inbox).await {
            Incoming::Notification { method, params } => {
                assert_eq!(method, "tick");
                assert_eq!(params["n"], json!(expected));
            }
            other => panic!("expected a notification, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn server_request_reaches_the_handler_and_its_answer_reaches_the_peer() {
    let (client, mut inbox) = start();
    let ask = client.request("ask", json!({"method": APPROVAL}), WAIT);
    let handle = async {
        match next(&mut inbox).await {
            Incoming::Request(request) => {
                assert_eq!(request.method, APPROVAL);
                assert_eq!(request.params["q"], json!("ação"));
                assert_eq!(request.id(), &json!("s-1"));
                request
                    .respond(json!({"decision": "accept"}))
                    .expect("answer");
            }
            other => panic!("expected a server request, got {other:?}"),
        }
    };
    let (seen_by_peer, ()) = tokio::join!(ask, handle);
    let seen_by_peer = seen_by_peer.expect("ask");
    assert_eq!(seen_by_peer["id"], json!("s-1"));
    assert_eq!(seen_by_peer["result"]["decision"], json!("accept"));
}

#[tokio::test]
async fn unknown_server_request_is_refused_without_reaching_the_handler() {
    let (client, mut inbox) = start();
    let seen_by_peer = client
        .request("ask", json!({"method": "weird/thing"}), WAIT)
        .await
        .expect("ask");
    assert_eq!(seen_by_peer["id"], json!("s-1"));
    assert_eq!(seen_by_peer["error"]["code"], json!(METHOD_NOT_FOUND));
    assert!(inbox.try_next().is_none(), "the handler never saw it");
}

#[tokio::test]
async fn server_request_dropped_without_an_answer_is_denied() {
    let (client, mut inbox) = start();
    let ask = client.request("ask", json!({"method": APPROVAL}), WAIT);
    let forget = async {
        match next(&mut inbox).await {
            Incoming::Request(request) => drop(request),
            other => panic!("expected a server request, got {other:?}"),
        }
    };
    let (seen_by_peer, ()) = tokio::join!(ask, forget);
    let seen_by_peer = seen_by_peer.expect("ask");
    assert_eq!(seen_by_peer["error"]["code"], json!(INTERNAL_ERROR));
}

#[tokio::test]
async fn peer_exit_fails_every_pending_request() {
    let (client, mut inbox) = start();
    let silent = client.request("silent", Value::Null, WAIT);
    let die = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        client.request("die", Value::Null, WAIT).await
    };
    let (silent, die) = tokio::join!(silent, die);
    for outcome in [silent, die] {
        match outcome {
            Err(RpcError::SessionEnded(end)) => {
                assert_eq!(end.reason, EndReason::PeerExited);
                assert_eq!(end.exit_code, Some(5));
            }
            other => panic!("expected the session end, got {other:?}"),
        }
    }
    assert!(matches!(next(&mut inbox).await, Incoming::Ended(_)));
    assert!(matches!(
        client.request("echo", Value::Null, WAIT).await,
        Err(RpcError::SessionEnded(_))
    ));
}

#[tokio::test]
async fn timed_out_request_leaves_the_session_usable() {
    let (client, _inbox) = start();
    match client
        .request("silent", Value::Null, Duration::from_millis(200))
        .await
    {
        Err(RpcError::Timeout { method, .. }) => assert_eq!(method, "silent"),
        other => panic!("expected a timeout, got {other:?}"),
    }
    assert_eq!(
        client.request("echo", json!(1), WAIT).await.expect("echo"),
        json!(1)
    );
}
