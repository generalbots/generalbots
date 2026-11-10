//! Offline unit tests for the quota metric classifier (#2075).
//!
//! The quota middleware charges a request to exactly one metric.
//! `determine_metric_from_request` is the whole routing table, and a wrong
//! answer either under-counts usage (free storage) or rejects a request that
//! belongs to another quota. The order of the rules matters: `/chat/upload`
//! is a message, not storage.

use super::determine_metric_from_request;
use axum::body::Body;
use axum::http::Request;

fn metric(path: &str, method: &str) -> String {
    let request = Request::builder()
        .uri(path)
        .method(method)
        .body(Body::empty())
        .expect("valid request");
    format!("{:?}", determine_metric_from_request(&request))
}

#[test]
fn chat_conversation_paths_are_messages() {
    assert_eq!(metric("/api/chat", "POST"), "Messages");
    assert_eq!(metric("/api/message", "POST"), "Messages");
    assert_eq!(metric("/api/conversation/42", "GET"), "Messages");
}

#[test]
fn upload_and_storage_paths_are_storage() {
    assert_eq!(metric("/api/upload", "POST"), "StorageBytes");
    assert_eq!(metric("/api/file/1", "GET"), "StorageBytes");
    assert_eq!(metric("/api/storage", "GET"), "StorageBytes");
}

#[test]
fn creating_bots_and_users_counts_against_those_metrics() {
    assert_eq!(metric("/api/bot", "POST"), "Bots");
    assert_eq!(metric("/api/bots", "POST"), "Bots");
    // A read does not create a bot.
    assert_eq!(metric("/api/bot", "GET"), "ApiCalls");
    assert_eq!(metric("/api/user", "POST"), "Users");
    assert_eq!(metric("/api/user", "DELETE"), "ApiCalls");
}

#[test]
fn knowledge_base_and_app_paths_have_their_own_metrics() {
    assert_eq!(metric("/api/kb/docs", "POST"), "KbDocuments");
    assert_eq!(metric("/api/document", "PUT"), "KbDocuments");
    assert_eq!(metric("/api/app", "POST"), "Apps");
    assert_eq!(metric("/api/form", "POST"), "Apps");
    assert_eq!(metric("/api/site", "POST"), "Apps");
}

#[test]
fn everything_else_is_an_api_call() {
    assert_eq!(metric("/api/health", "GET"), "ApiCalls");
    assert_eq!(metric("/", "GET"), "ApiCalls");
    assert_eq!(metric("/api/products/7", "PATCH"), "ApiCalls");
}

#[test]
fn the_first_matching_rule_wins() {
    // Both a chat path and an upload path: messages are checked first, so the
    // upload suffix must not move the charge to the storage quota.
    assert_eq!(metric("/api/chat/upload", "POST"), "Messages");
    assert_eq!(metric("/api/conversation/file", "POST"), "Messages");
    // `/bot` beats `/kb` when both appear in the path.
    assert_eq!(metric("/api/bot/kb", "POST"), "Bots");
}
