use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

const ORIGIN: &str = "https://api.plain.com/graphql";

fn param(name: &str, required: bool) -> LlmSafeParam {
    LlmSafeParam {
        name: name.to_string(),
        kind: "string".to_string(),
        required,
    }
}

fn action(key: &str, summary: &str, params: Vec<LlmSafeParam>) -> LlmSafeAction {
    LlmSafeAction {
        name: key.to_string(),
        summary: summary.to_string(),
        params,
        risk: if key.ends_with("list") || key.ends_with("get") || key.ends_with("search") {
            "low".to_string()
        } else {
            "medium".to_string()
        },
        requires_approval: !(key.ends_with("list") || key.ends_with("get") || key.ends_with("search")),
    }
}

pub const PLAIN_IMPLEMENTED_ACTIONS: &[&str] = &[
    "plain.tickets.list",
    "plain.tickets.get",
    "plain.tickets.search",
];

/// GraphQL adapter for Plain. Each action posts a fixed document with the
/// validated parameters as JSON variables; schema drift surfaces as a
/// provider error which maps to the standard failure sentinels.
pub struct PlainAdapter;

impl PlainAdapter {
    fn document(action: &str) -> (&'static str, &'static str) {
        match action {
            "plain.tickets.list" => (
                "query Tickets($first: Int!) { threads(first: $first) { edges { node { id title statusLabel } } } }",
                "tickets",
            ),
            "plain.tickets.get" => (
                "query Ticket($id: ID!) { thread(id: $id) { id title statusLabel } }",
                "ticket",
            ),
            "plain.tickets.search" => (
                "query Search($query: String!) { threads(first: 25, filter: { labelTypeIds: [] }) { edges { node { id title } } } }",
                "search",
            ),
            _ => ("", ""),
        }
    }
}

impl ProviderAdapter for PlainAdapter {
    fn provider(&self) -> &'static str {
        "plain"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        PLAIN_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        vec![
            action("plain.tickets.list", "List support threads.", vec![param("limit", false)]),
            action("plain.tickets.get", "Read a support thread.", vec![param("thread_id", true)]),
            action("plain.tickets.search", "Search support threads.", vec![param("query", true)]),
        ]
    }

    fn invoke<'a>(
        &'a self,
        action_key: &'a str,
        credentials: &'a Value,
        params: &'a Value,
    ) -> Pin<Box<dyn Future<Output = Result<ActionOutcome, String>> + Send + 'a>> {
        Box::pin(async move {
            let token = credentials
                .get("api_key")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| rest_client::invalid("credential key api_key is missing".to_string()))?;
            let (document, _label) = Self::document(action_key);
            if document.is_empty() {
                return Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string());
            }
            let variables = match action_key {
                "plain.tickets.list" => json!({ "first": params.get("limit").and_then(Value::as_u64).unwrap_or(25) }),
                "plain.tickets.get" => json!({ "id": params.get("thread_id") }),
                _ => json!({ "query": params.get("query") }),
            };
            let payload = json!({ "query": document, "variables": variables });
            let response = rest_client::send(RestRequest {
                method: reqwest::Method::POST,
                url: ORIGIN.to_string(),
                headers: vec![
                    ("authorization", format!("Bearer {token}")),
                    ("content-type", "application/json".to_string()),
                    ("user-agent", "generalbots-botintegrations".to_string()),
                ],
                body: Some(payload.to_string().into_bytes()),
                response_cap: MAX_RESPONSE_BYTES,
            })
            .await?;
            response.require_success("Plain GraphQL")?;
            let body = response.json("Plain GraphQL")?;
            Ok(ActionOutcome {
                summary: format!("{} completed (status {})", action_key, response.status),
                data: body.get("data").cloned().unwrap_or(Value::Null),
                truncated: false,
            })
        })
    }
}
