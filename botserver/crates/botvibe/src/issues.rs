use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    Open,
    Closed,
}

impl IssueState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for IssueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeIssue {
    pub issue_id: Uuid,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub state: IssueState,
    pub assignee: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct IssueStore {
    issues: RwLock<Vec<VibeIssue>>,
}

impl IssueStore {
    pub fn new() -> Self {
        Self { issues: RwLock::new(Vec::new()) }
    }

    pub async fn create(&self, title: String, body: String, labels: Vec<String>, assignee: Option<String>) -> VibeIssue {
        let issue = VibeIssue {
            issue_id: Uuid::new_v4(),
            title,
            body,
            labels,
            state: IssueState::Open,
            assignee,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let mut issues = self.issues.write().await;
        issues.push(issue.clone());
        issue
    }

    pub async fn get(&self, issue_id: Uuid) -> Option<VibeIssue> {
        let issues = self.issues.read().await;
        issues.iter().find(|i| i.issue_id == issue_id).cloned()
    }

    pub async fn list(&self, state: Option<IssueState>) -> Vec<VibeIssue> {
        let issues = self.issues.read().await;
        issues
            .iter()
            .filter(|i| state.is_none_or(|s| i.state == s))
            .cloned()
            .collect()
    }

    pub async fn update(
        &self,
        issue_id: Uuid,
        state: Option<IssueState>,
        assignee: Option<String>,
        labels: Option<Vec<String>>,
    ) -> Option<VibeIssue> {
        let mut issues = self.issues.write().await;
        let issue = issues.iter_mut().find(|i| i.issue_id == issue_id)?;
        if let Some(state) = state {
            issue.state = state;
        }
        if let Some(assignee) = assignee {
            issue.assignee = Some(assignee);
        }
        if let Some(labels) = labels {
            issue.labels = labels;
        }
        issue.updated_at = chrono::Utc::now();
        Some(issue.clone())
    }
}

impl Default for IssueStore {
    fn default() -> Self {
        Self::new()
    }
}

fn ok(data: Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: Value::Null, error: Some(msg), latency_ms: 0 }
}

pub fn issue_tools(store: Arc<IssueStore>) -> Vec<(String, ToolSchema, ToolHandler)> {
    let create = Arc::clone(&store);
    let list = Arc::clone(&store);
    let update = Arc::clone(&store);

    let create_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&create);
        Box::pin(async move {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let labels: Vec<String> = args
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let assignee = args.get("assignee").and_then(|v| v.as_str()).map(String::from);
            if title.is_empty() {
                return err("title is required".into());
            }
            let issue = store.create(title, body, labels, assignee).await;
            ok(json!({ "issue_id": issue.issue_id, "state": issue.state.as_str() }))
        })
    });

    let list_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&list);
        Box::pin(async move {
            let state = match args.get("state").and_then(|v| v.as_str()) {
                Some("closed") => Some(IssueState::Closed),
                Some("open") => Some(IssueState::Open),
                _ => None,
            };
            let issues = store.list(state).await;
            ok(json!({
                "issues": issues.iter().map(|i| json!({
                    "issue_id": i.issue_id,
                    "title": i.title,
                    "state": i.state.as_str(),
                    "labels": i.labels,
                    "assignee": i.assignee,
                })).collect::<Vec<_>>(),
            }))
        })
    });

    let update_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&update);
        Box::pin(async move {
            let issue_id = args
                .get("issue_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            let issue_id = match issue_id {
                Some(id) => id,
                None => return err("issue_id is required".into()),
            };
            let state = match args.get("state").and_then(|v| v.as_str()) {
                Some("closed") => Some(IssueState::Closed),
                Some("open") => Some(IssueState::Open),
                _ => None,
            };
            let assignee = args.get("assignee").and_then(|v| v.as_str()).map(String::from);
            let labels: Option<Vec<String>> = args
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect());
            match store.update(issue_id, state, assignee, labels).await {
                Some(issue) => ok(json!({ "issue_id": issue.issue_id, "state": issue.state.as_str() })),
                None => err("issue not found".into()),
            }
        })
    });

    vec![
        ("issue/create".into(), ToolSchema::new("issue/create", "Create an issue").with_parameters(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "body": {"type": "string"},
                "labels": {"type": "array", "items": {"type": "string"}},
                "assignee": {"type": "string"}
            },
            "required": ["title"]
        })), create_handler),
        ("issue/list".into(), ToolSchema::new("issue/list", "List issues, optionally filtered by state").with_parameters(json!({
            "type": "object",
            "properties": {
                "state": {"type": "string", "enum": ["open", "closed"]}
            }
        })), list_handler),
        ("issue/update".into(), ToolSchema::new("issue/update", "Update issue state, assignee or labels").with_parameters(json!({
            "type": "object",
            "properties": {
                "issue_id": {"type": "string"},
                "state": {"type": "string", "enum": ["open", "closed"]},
                "assignee": {"type": "string"},
                "labels": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["issue_id"]
        })), update_handler),
    ]
}

#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssueRequest {
    pub state: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct IssueResponse {
    success: bool,
    issue: Option<VibeIssue>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct IssuesResponse {
    success: bool,
    issues: Vec<VibeIssue>,
}

pub fn issues_router(store: Arc<IssueStore>) -> Router {
    Router::new()
        .route("/api/vibe/issues", axum::routing::get(list_issues))
        .route("/api/vibe/issues", axum::routing::post(create_issue))
        .route("/api/vibe/issues/:issue_id", axum::routing::get(get_issue))
        .route("/api/vibe/issues/:issue_id", axum::routing::patch(update_issue))
        .layer(Extension(store))
}

async fn list_issues(Extension(store): Extension<Arc<IssueStore>>) -> Json<IssuesResponse> {
    Json(IssuesResponse { success: true, issues: store.list(None).await })
}

async fn create_issue(
    Extension(store): Extension<Arc<IssueStore>>,
    Json(req): Json<CreateIssueRequest>,
) -> Json<IssueResponse> {
    let issue = store
        .create(
            req.title,
            req.body.unwrap_or_default(),
            req.labels.unwrap_or_default(),
            req.assignee,
        )
        .await;
    Json(IssueResponse { success: true, issue: Some(issue), error: None })
}

async fn get_issue(
    Extension(store): Extension<Arc<IssueStore>>,
    axum::extract::Path(issue_id): axum::extract::Path<Uuid>,
) -> Json<IssueResponse> {
    match store.get(issue_id).await {
        Some(issue) => Json(IssueResponse { success: true, issue: Some(issue), error: None }),
        None => Json(IssueResponse { success: false, issue: None, error: Some("Issue not found".into()) }),
    }
}

async fn update_issue(
    Extension(store): Extension<Arc<IssueStore>>,
    axum::extract::Path(issue_id): axum::extract::Path<Uuid>,
    Json(req): Json<UpdateIssueRequest>,
) -> Json<IssueResponse> {
    let state = match req.state.as_deref() {
        Some("closed") => Some(IssueState::Closed),
        Some("open") => Some(IssueState::Open),
        _ => None,
    };
    match store.update(issue_id, state, req.assignee, req.labels).await {
        Some(issue) => Json(IssueResponse { success: true, issue: Some(issue), error: None }),
        None => Json(IssueResponse { success: false, issue: None, error: Some("Issue not found".into()) }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_state_str_and_display() {
        assert_eq!(IssueState::Open.as_str(), "open");
        assert_eq!(IssueState::Closed.as_str(), "closed");
        assert_eq!(format!("{}", IssueState::Closed), "closed");
        let v = serde_json::to_value(IssueState::Open).unwrap();
        assert_eq!(v, "open");
    }

    #[tokio::test]
    async fn create_list_filter_by_state() {
        let store = IssueStore::new();
        let open = store.create("bug".into(), "crashes".into(), vec!["bug".into()], Some("ana".into())).await;
        store.create("closed one".into(), "b".into(), vec![].into(), None).await;
        store.update(open.issue_id, Some(IssueState::Closed), None, None).await;
        assert_eq!(store.list(None).await.len(), 2);
        assert_eq!(store.list(Some(IssueState::Closed)).await.len(), 1);
        assert_eq!(store.list(Some(IssueState::Open)).await.len(), 1);
    }

    #[tokio::test]
    async fn update_changes_assignee_and_labels() {
        let store = IssueStore::new();
        let issue = store.create("t".into(), "b".into(), vec![].into(), None).await;
        let updated = store
            .update(issue.issue_id, None, Some("bob".into()), Some(vec!["urgent".into()]))
            .await
            .unwrap();
        assert_eq!(updated.assignee.as_deref(), Some("bob"));
        assert_eq!(updated.labels, vec!["urgent"]);
    }

    #[tokio::test]
    async fn update_missing_returns_none() {
        let store = IssueStore::new();
        assert!(store.update(Uuid::new_v4(), None, None, None).await.is_none());
    }
}
