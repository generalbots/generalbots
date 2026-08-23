use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

const ORIGIN: &str = "https://api.monday.com/v2";

fn param(name: &str, required: bool) -> LlmSafeParam {
    LlmSafeParam {
        name: name.to_string(),
        kind: "string".to_string(),
        required,
    }
}

pub const MONDAY_IMPLEMENTED_ACTIONS: &[&str] = &[
    "monday.boards.list",
    "monday.work_items.list",
];

/// GraphQL adapter for monday.com (POST /v2 with Bearer token).
pub struct MondayAdapter;

impl ProviderAdapter for MondayAdapter {
    fn provider(&self) -> &'static str {
        "monday"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        MONDAY_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        vec![
            LlmSafeAction {
                name: "monday.boards.list".to_string(),
                summary: "Listed monday.com boards.".to_string(),
                params: vec![param("limit", false)],
                risk: "low".to_string(),
                requires_approval: false,
            },
            LlmSafeAction {
                name: "monday.work_items.list".to_string(),
                summary: "Listed items of a board.".to_string(),
                params: vec![param("board_id", true), param("limit", false)],
                risk: "low".to_string(),
                requires_approval: false,
            },
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
                .get("token")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| rest_client::invalid("credential key token is missing".to_string()))?;
            let document = match action_key {
                "monday.boards.list" => "query Boards($limit: Int!) { boards(limit: $limit) { id name } }",
                "monday.work_items.list" => "query Items($board_id: ID!, $limit: Int!) { boards(ids: [$board_id]) { items_page(limit: $limit) { items { id name } } } }",
                _ => return Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string()),
            };
            let variables = match action_key {
                "monday.boards.list" => json!({ "limit": params.get("limit").and_then(Value::as_u64).unwrap_or(25) }),
                _ => json!({
                    "board_id": params.get("board_id"),
                    "limit": params.get("limit").and_then(Value::as_u64).unwrap_or(25)
                }),
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
            response.require_success("monday GraphQL")?;
            let body = response.json("monday GraphQL")?;
            Ok(ActionOutcome {
                summary: format!("{} completed (status {})", action_key, response.status),
                data: body.get("data").cloned().unwrap_or(Value::Null),
                truncated: false,
            })
        })
    }
}
