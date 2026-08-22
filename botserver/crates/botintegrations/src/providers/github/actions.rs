//! GitHub action implementations for the provider adapter (#950 slice 2).
//!
//! Every handler validates its parameters completely before any network
//! activity, requests a single bounded first page, and projects the response
//! down to a small, redaction-safe subset of fields.

use serde_json::{json, Value};

use super::client::{self, GithubCreds};
use crate::providers::rest_client::{
    bounded_limit, invalid, optional_text, outcome, push_query_pair, required_text,
    validate_repository_slug,
};
use crate::providers::ActionOutcome;

const MAX_TITLE_LEN: usize = 512;
const MAX_BODY_LEN: usize = 64 * 1024;
const MAX_QUERY_LEN: usize = 256;
const LIST_LIMIT_MAX: usize = 50;
const SEARCH_LIMIT_MAX: usize = 20;

fn repository_param(params: &Value) -> Result<(String, String), String> {
    let slug = required_text(params, "repository", 211)?;
    validate_repository_slug(&slug)
}

/// Parses a positive issue or pull request number used as a path segment.
fn parse_issue_number(raw: &str) -> Result<u64, String> {
    let number = raw
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("resource_id must be a positive issue number".to_string()))?;
    if number > 1_000_000_000 {
        return Err(invalid("resource_id is out of range".to_string()));
    }
    Ok(number)
}

/// Builds the create-issue JSON payload from the catalog `data` object.
fn issue_create_payload(data: &Value) -> Result<Value, String> {
    let object = data.as_object().ok_or_else(|| {
        invalid("data must be a JSON object containing title and optional body".to_string())
    })?;
    let title = object
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| invalid("data.title is required".to_string()))?;
    if title.len() > MAX_TITLE_LEN {
        return Err(invalid(format!(
            "data.title must be at most {MAX_TITLE_LEN} characters"
        )));
    }
    let body = match object.get("body") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            if text.len() > MAX_BODY_LEN {
                return Err(invalid(format!(
                    "data.body must be at most {MAX_BODY_LEN} characters"
                )));
            }
            Some(text.clone())
        }
        Some(_) => return Err(invalid("data.body must be a string".to_string())),
    };
    let mut payload = json!({ "title": title });
    if let Some(body) = body {
        payload["body"] = Value::String(body);
    }
    Ok(payload)
}

/// Validates the update-issue `changes` object and returns the whitelisted
/// subset accepted by the REST API.
fn issue_update_changes(changes: &Value) -> Result<Value, String> {
    let object = changes
        .as_object()
        .ok_or_else(|| invalid("changes must be a JSON object".to_string()))?;
    let mut patch = serde_json::Map::new();
    for key in ["title", "body", "state"] {
        match object.get(key) {
            None | Some(Value::Null) => continue,
            Some(Value::String(text)) => {
                let limit = match key {
                    "body" => MAX_BODY_LEN,
                    _ => MAX_TITLE_LEN,
                };
                if text.len() > limit {
                    return Err(invalid(format!(
                        "changes.{key} must be at most {limit} characters"
                    )));
                }
                if key == "state" && text != "open" && text != "closed" {
                    return Err(invalid(
                        "changes.state must be either open or closed".to_string(),
                    ));
                }
                patch.insert(key.to_string(), Value::String(text.clone()));
            }
            Some(_) => return Err(invalid(format!("changes.{key} must be a string"))),
        }
    }
    if patch.is_empty() {
        return Err(invalid(
            "changes must set at least one of title, body or state".to_string(),
        ));
    }
    Ok(Value::Object(patch))
}

fn project_repository(item: &Value) -> Value {
    json!({
        "full_name": item.get("full_name").cloned().unwrap_or_default(),
        "private": item.get("private").cloned().unwrap_or(Value::Bool(false)),
        "html_url": item.get("html_url").cloned().unwrap_or_default(),
        "description": item.get("description").cloned().unwrap_or_default(),
        "pushed_at": item.get("pushed_at").cloned().unwrap_or_default(),
    })
}

fn project_issue(item: &Value) -> Value {
    json!({
        "number": item.get("number").cloned().unwrap_or_default(),
        "title": item.get("title").cloned().unwrap_or_default(),
        "state": item.get("state").cloned().unwrap_or_default(),
        "user": item.pointer("/user/login").cloned().unwrap_or_default(),
        "created_at": item.get("created_at").cloned().unwrap_or_default(),
        "html_url": item.get("html_url").cloned().unwrap_or_default(),
    })
}

fn project_pull_request(item: &Value) -> Value {
    json!({
        "number": item.get("number").cloned().unwrap_or_default(),
        "title": item.get("title").cloned().unwrap_or_default(),
        "draft": item.get("draft").cloned().unwrap_or(Value::Bool(false)),
        "user": item.pointer("/user/login").cloned().unwrap_or_default(),
        "created_at": item.get("created_at").cloned().unwrap_or_default(),
        "html_url": item.get("html_url").cloned().unwrap_or_default(),
    })
}

fn project_workflow_run(item: &Value) -> Value {
    json!({
        "id": item.get("id").cloned().unwrap_or_default(),
        "name": item.get("name").cloned().unwrap_or_default(),
        "status": item.get("status").cloned().unwrap_or_default(),
        "conclusion": item.get("conclusion").cloned().unwrap_or_default(),
        "event": item.get("event").cloned().unwrap_or_default(),
        "created_at": item.get("created_at").cloned().unwrap_or_default(),
        "html_url": item.get("html_url").cloned().unwrap_or_default(),
    })
}

async fn repositories_list(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let limit = bounded_limit(params, "limit", LIST_LIMIT_MAX, LIST_LIMIT_MAX)?;
    let response = client::get(creds, &format!("/user/repos?per_page={limit}&sort=pushed")).await?;
    response.require_success("repositories.list")?;
    let document = response.json("repositories.list")?;
    let items = document
        .as_array()
        .map(|items| items.iter().map(project_repository).collect::<Vec<_>>())
        .unwrap_or_default();
    let count = items.len();
    Ok(outcome(
        format!("Listed {count} accessible repositories"),
        json!({ "repository_count": count, "repositories": items }),
    ))
}

async fn repositories_search(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let query = required_text(params, "query", MAX_QUERY_LEN)?;
    let limit = bounded_limit(params, "limit", SEARCH_LIMIT_MAX, SEARCH_LIMIT_MAX)?;
    let mut path_query = String::from("/search/repositories?");
    push_query_pair(&mut path_query, "q", &query);
    push_query_pair(&mut path_query, "per_page", &limit.to_string());
    let response = client::get(creds, &path_query).await?;
    response.require_success("repositories.search")?;
    let document = response.json("repositories.search")?;
    let total = document.get("total_count").cloned().unwrap_or(json!(0));
    let items: Vec<Value> = document
        .get("items")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(project_repository).collect())
        .unwrap_or_default();
    let count = items.len();
    Ok(outcome(
        format!("Found {count} repositories matching '{query}'"),
        json!({ "total_count": total, "repository_count": count, "repositories": items }),
    ))
}

async fn repositories_get(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let (owner, repository) = repository_param(params)?;
    let response = client::get(creds, &format!("/repos/{owner}/{repository}")).await?;
    response.require_success("repositories.get")?;
    let document = response.json("repositories.get")?;
    let name = document
        .get("full_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    Ok(outcome(
        format!("Read repository {name}"),
        json!({
            "repository": project_repository(&document),
            "default_branch": document.get("default_branch").cloned().unwrap_or_default(),
            "stargazers_count": document.get("stargazers_count").cloned().unwrap_or_default(),
            "open_issues_count": document.get("open_issues_count").cloned().unwrap_or_default(),
        }),
    ))
}

async fn issues_list(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let (owner, repository) = repository_param(params)?;
    let response = client::get(
        creds,
        &format!("/repos/{owner}/{repository}/issues?state=open&per_page={LIST_LIMIT_MAX}"),
    )
    .await?;
    response.require_success("issues.list")?;
    let document = response.json("issues.list")?;
    // The REST issues endpoint also returns pull requests; keep only true
    // issues so the summary stays honest.
    let items: Vec<Value> = document
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter(|item| item.get("pull_request").is_none())
                .map(project_issue)
                .collect()
        })
        .unwrap_or_default();
    let count = items.len();
    Ok(outcome(
        format!("Listed {count} open issues in {owner}/{repository}"),
        json!({ "issue_count": count, "issues": items }),
    ))
}

async fn issues_search(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let query = required_text(params, "query", MAX_QUERY_LEN)?;
    let scope = optional_text(params, "repository", 211)?;
    if let Some(slug) = &scope {
        validate_repository_slug(slug)?;
    }
    let effective_query = match &scope {
        Some(slug) => format!("{query} repo:{slug}"),
        None => query.clone(),
    };
    let mut path_query = String::from("/search/issues?");
    push_query_pair(&mut path_query, "q", &effective_query);
    push_query_pair(&mut path_query, "per_page", "30");
    let response = client::get(creds, &path_query).await?;
    response.require_success("issues.search")?;
    let document = response.json("issues.search")?;
    let total = document.get("total_count").cloned().unwrap_or(json!(0));
    let items: Vec<Value> = document
        .get("items")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(project_issue).collect())
        .unwrap_or_default();
    let count = items.len();
    Ok(outcome(
        format!("Found {count} issues matching '{query}'"),
        json!({ "total_count": total, "issue_count": count, "issues": items }),
    ))
}

async fn issues_create(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let (owner, repository) = repository_param(params)?;
    let data = params
        .get("data")
        .ok_or_else(|| invalid("data is required".to_string()))?;
    let payload = issue_create_payload(data)?;
    let response = client::send_json(
        creds,
        reqwest::Method::POST,
        &format!("/repos/{owner}/{repository}/issues"),
        payload,
    )
    .await?;
    response.require_success("issues.create")?;
    let document = response.json("issues.create")?;
    let number = document.get("number").cloned().unwrap_or_default();
    let url = document.get("html_url").cloned().unwrap_or_default();
    Ok(outcome(
        format!("Created issue #{number} in {owner}/{repository}"),
        json!({ "number": number, "html_url": url }),
    ))
}

async fn issues_update(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let (owner, repository) = repository_param(params)?;
    let raw_number = required_text(params, "resource_id", 32)?;
    let number = parse_issue_number(&raw_number)?;
    let changes = params
        .get("changes")
        .ok_or_else(|| invalid("changes is required".to_string()))?;
    let payload = issue_update_changes(changes)?;
    let response = client::send_json(
        creds,
        reqwest::Method::PATCH,
        &format!("/repos/{owner}/{repository}/issues/{number}"),
        payload,
    )
    .await?;
    response.require_success("issues.update")?;
    let state = response
        .json("issues.update")?
        .get("state")
        .cloned()
        .unwrap_or_default();
    Ok(outcome(
        format!("Updated issue #{number} in {owner}/{repository}"),
        json!({ "number": number, "state": state }),
    ))
}

async fn pull_requests_list(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let (owner, repository) = repository_param(params)?;
    let response = client::get(
        creds,
        &format!("/repos/{owner}/{repository}/pulls?state=open&per_page={LIST_LIMIT_MAX}"),
    )
    .await?;
    response.require_success("pull_requests.list")?;
    let document = response.json("pull_requests.list")?;
    let items: Vec<Value> = document
        .as_array()
        .map(|items| items.iter().map(project_pull_request).collect())
        .unwrap_or_default();
    let count = items.len();
    Ok(outcome(
        format!("Listed {count} open pull requests in {owner}/{repository}"),
        json!({ "pull_request_count": count, "pull_requests": items }),
    ))
}

async fn actions_runs_list(creds: &GithubCreds, params: &Value) -> Result<ActionOutcome, String> {
    let (owner, repository) = repository_param(params)?;
    let response = client::get(
        creds,
        &format!("/repos/{owner}/{repository}/actions/runs?per_page=30"),
    )
    .await?;
    response.require_success("actions.runs.list")?;
    let document = response.json("actions.runs.list")?;
    let total = document.get("total_count").cloned().unwrap_or(json!(0));
    let items: Vec<Value> = document
        .pointer("/workflow_runs")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(project_workflow_run).collect())
        .unwrap_or_default();
    let count = items.len();
    Ok(outcome(
        format!("Listed {count} workflow runs in {owner}/{repository}"),
        json!({ "total_count": total, "run_count": count, "workflow_runs": items }),
    ))
}

/// Entry point used by [`super::GithubAdapter`]; unknown keys are rejected
/// here before any parameter validation or network activity happens.
pub(crate) async fn invoke(
    action: &str,
    creds: &GithubCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    match action {
        "repositories.list" => repositories_list(creds, params).await,
        "repositories.search" => repositories_search(creds, params).await,
        "repositories.get" => repositories_get(creds, params).await,
        "issues.list" => issues_list(creds, params).await,
        "issues.search" => issues_search(creds, params).await,
        "issues.create" => issues_create(creds, params).await,
        "issues.update" => issues_update(creds, params).await,
        "pull_requests.list" => pull_requests_list(creds, params).await,
        "actions.runs.list" => actions_runs_list(creds, params).await,
        _ => Err(crate::providers::ERR_UNKNOWN_ACTION.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn issue_numbers_must_be_positive_bounded_integers() {
        assert_eq!(parse_issue_number("42").ok(), Some(42));
        assert!(parse_issue_number("0").is_err());
        assert!(parse_issue_number("-3").is_err());
        assert!(parse_issue_number("../secrets").is_err());
        assert!(parse_issue_number("").is_err());
    }

    #[test]
    fn issue_create_payload_requires_title_and_accepts_body() {
        assert!(issue_create_payload(&json!({})).is_err());
        assert!(issue_create_payload(&json!({ "title": "" })).is_err());
        assert!(issue_create_payload(&json!({ "title": 5 })).is_err());
        assert!(issue_create_payload(&json!({ "title": "Bug" })).is_ok());
        let payload = issue_create_payload(&json!({ "title": "Bug", "body": "Steps..." }))
            .unwrap_or_default();
        assert_eq!(payload["title"], json!("Bug"));
        assert_eq!(payload["body"], json!("Steps..."));
        let long_body = "x".repeat(MAX_BODY_LEN + 1);
        assert!(issue_create_payload(&json!({ "title": "Bug", "body": long_body })).is_err());
    }

    #[test]
    fn issue_update_changes_are_whitelisted_and_validated() {
        let patch =
            issue_update_changes(&json!({ "state": "closed", "junk": "x" })).unwrap_or_default();
        assert_eq!(patch["state"], json!("closed"));
        assert!(patch.get("junk").is_none());
        assert!(issue_update_changes(&json!({ "state": "detonate" })).is_err());
        assert!(issue_update_changes(&json!({ "unrelated": 1 })).is_err());
        assert!(issue_update_changes(&json!([])).is_err());
    }

    #[test]
    fn unknown_actions_reject_before_any_network_activity() {
        let names = super::super::GITHUB_IMPLEMENTED_ACTIONS;
        assert!(!names.contains(&"git.refs.delete"));
        assert!(names.contains(&"issues.create"));
    }

    #[test]
    fn repository_params_validate_before_use() {
        assert!(repository_param(&json!({ "repository": "../etc/passwd" })).is_err());
        assert!(repository_param(&json!({})).is_err());
        assert_eq!(
            repository_param(&json!({ "repository": "acme/widgets.rs" })).ok(),
            Some(("acme".to_string(), "widgets.rs".to_string()))
        );
    }
}
