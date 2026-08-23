use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

const ORIGIN: &str = "https://api.ashbyhq.com";

fn param(name: &str, required: bool) -> LlmSafeParam {
    LlmSafeParam {
        name: name.to_string(),
        kind: "string".to_string(),
        required,
    }
}

pub const ASHBY_IMPLEMENTED_ACTIONS: &[&str] = &[
    "ashby.candidates.list",
    "ashby.candidates.search",
];

/// GraphQL adapter for Ashby (POST /graphql with Bearer API key).
pub struct AshbyAdapter;

impl ProviderAdapter for AshbyAdapter {
    fn provider(&self) -> &'static str {
        "ashby"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        ASHBY_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        vec![
            LlmSafeAction {
                name: "ashby.candidates.list".to_string(),
                summary: "Listed Ashby candidates.".to_string(),
                params: vec![param("limit", false)],
                risk: "low".to_string(),
                requires_approval: false,
            },
            LlmSafeAction {
                name: "ashby.candidates.search".to_string(),
                summary: "Searched Ashby candidates by name.".to_string(),
                params: vec![param("query", true)],
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
                .get("api_key")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| rest_client::invalid("credential key api_key is missing".to_string()))?;
            let payload = match action_key {
                "ashby.candidates.list" => json!({
                    "query": "query CandidateList($first: Int!) { candidateList(first: $first) { edges { node { id name email } } } }",
                    "variables": { "first": params.get("limit").and_then(Value::as_u64).unwrap_or(25) }
                }),
                "ashby.candidates.search" => json!({
                    "query": "query CandidateSearch($name: String!) { candidateSearch(name: $name) { id name email } }",
                    "variables": { "name": params.get("query") }
                }),
                _ => return Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string()),
            };
            let response = rest_client::send(RestRequest {
                method: reqwest::Method::POST,
                url: format!("{ORIGIN}/graphql"),
                headers: vec![
                    ("authorization", format!("Bearer {token}")),
                    ("content-type", "application/json".to_string()),
                    ("user-agent", "generalbots-botintegrations".to_string()),
                ],
                body: Some(payload.to_string().into_bytes()),
                response_cap: MAX_RESPONSE_BYTES,
            })
            .await?;
            response.require_success("Ashby GraphQL")?;
            let body = response.json("Ashby GraphQL")?;
            Ok(ActionOutcome {
                summary: format!("{} completed (status {})", action_key, response.status),
                data: body.get("data").cloned().unwrap_or(Value::Null),
                truncated: false,
            })
        })
    }
}
