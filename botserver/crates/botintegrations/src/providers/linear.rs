use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

const ORIGIN: &str = "https://api.linear.app/graphql";

fn param(name: &str, required: bool) -> LlmSafeParam {
    LlmSafeParam {
        name: name.to_string(),
        kind: "string".to_string(),
        required,
    }
}

fn action(key: &str, summary: &str, params: Vec<LlmSafeParam>, risk: &str, approval: bool) -> LlmSafeAction {
    LlmSafeAction { name: key.to_string(), summary: summary.to_string(), params, risk: risk.to_string(), requires_approval: approval }
}

pub const LINEAR_IMPLEMENTED_ACTIONS: &[&str] = &[
    "linear.work_items.list",
    "linear.work_items.search",
    "linear.work_items.get",
    "linear.work_items.create",
    "linear.work_items.update",
    "linear.work_items.delete",
];

/// GraphQL adapter for Linear (POST /graphql, Bearer API key).
pub struct LinearAdapter;

impl LinearAdapter {
    fn document(action: &str) -> Option<(&'static str, &'static str)> {
        Some(match action {
            "linear.work_items.list" => (
                "query Issues($first: Int!) { issues(first: $first) { nodes { id identifier title state { name } } } }",
                "",
            ),
            "linear.work_items.search" => (
                "query Search($term: String!) { issueSearch(term: $term) { nodes { id identifier title } } }",
                "",
            ),
            "linear.work_items.get" => (
                "query Issue($id: String!) { issue(id: $id) { id identifier title description state { name } } }",
                "",
            ),
            "linear.work_items.create" => (
                "mutation Create($input: IssueCreateInput!) { issueCreate(input: $input) { success issue { id identifier } } }",
                "input",
            ),
            "linear.work_items.update" => (
                "mutation Update($id: String!, $input: IssueUpdateInput!) { issueUpdate(id: $id, input: $input) { success } }",
                "",
            ),
            "linear.work_items.delete" => (
                "mutation Delete($id: String!) { issueDelete(id: $id) { success } }",
                "",
            ),
            _ => return None,
        })
    }
}

impl ProviderAdapter for LinearAdapter {
    fn provider(&self) -> &'static str {
        "linear"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        LINEAR_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        vec![
            action("linear.work_items.list", "Listed Linear issues.", vec![param("limit", false)], "low", false),
            action("linear.work_items.search", "Searched Linear issues.", vec![param("query", true)], "low", false),
            action("linear.work_items.get", "Read a Linear issue.", vec![param("issue_id", true)], "low", false),
            action("linear.work_items.create", "Created a Linear issue.", vec![param("team_id", true), param("title", true), param("description", false)], "medium", true),
            action("linear.work_items.update", "Updated a Linear issue.", vec![param("issue_id", true), param("data", true)], "medium", true),
            action("linear.work_items.delete", "Deleted a Linear issue.", vec![param("issue_id", true)], "high", true),
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
            let Some((document, input_key)) = Self::document(action_key) else {
                return Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string());
            };
            let variables = match action_key {
                "linear.work_items.list" => json!({ "first": params.get("limit").and_then(Value::as_u64).unwrap_or(25) }),
                "linear.work_items.search" => json!({ "term": params.get("query") }),
                "linear.work_items.get" | "linear.work_items.delete" => json!({ "id": params.get("issue_id") }),
                "linear.work_items.create" => json!({ "input": {
                    "teamId": params.get("team_id"),
                    "title": params.get("title"),
                    "description": params.get("description")
                }}),
                _ => json!({ "id": params.get("issue_id"), "input": params.get("data") }),
            };
            let mut payload = json!({ "query": document, "variables": variables });
            if !input_key.is_empty() && action_key == "linear.work_items.update" {
                payload["variables"]["input"] = params.get("data").cloned().unwrap_or(Value::Null);
            }
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
            response.require_success("Linear GraphQL")?;
            let body = response.json("Linear GraphQL")?;
            Ok(ActionOutcome {
                summary: format!("{} completed (status {})", action_key, response.status),
                data: body.get("data").cloned().unwrap_or(Value::Null),
                truncated: false,
            })
        })
    }
}
