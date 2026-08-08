use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeCanvas {
    pub canvas_id: Uuid,
    pub title: String,
    pub project: Option<String>,
    pub content: Value,
    pub share_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct CanvasStore {
    canvases: RwLock<Vec<VibeCanvas>>,
}

impl CanvasStore {
    pub fn new() -> Self {
        Self { canvases: RwLock::new(Vec::new()) }
    }

    pub async fn create(&self, title: String, project: Option<String>, content: Value) -> VibeCanvas {
        let canvas = VibeCanvas {
            canvas_id: Uuid::new_v4(),
            title,
            project,
            content,
            share_token: Uuid::new_v4().simple().to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let mut canvases = self.canvases.write().await;
        canvases.push(canvas.clone());
        canvas
    }

    pub async fn get(&self, canvas_id: Uuid) -> Option<VibeCanvas> {
        let canvases = self.canvases.read().await;
        canvases.iter().find(|c| c.canvas_id == canvas_id).cloned()
    }

    pub async fn list(&self, project: Option<&str>) -> Vec<VibeCanvas> {
        let canvases = self.canvases.read().await;
        canvases
            .iter()
            .filter(|c| project.is_none_or(|p| c.project.as_deref() == Some(p)))
            .cloned()
            .collect()
    }

    pub async fn update(&self, canvas_id: Uuid, title: Option<String>, content: Option<Value>) -> Option<VibeCanvas> {
        let mut canvases = self.canvases.write().await;
        let canvas = canvases.iter_mut().find(|c| c.canvas_id == canvas_id)?;
        if let Some(title) = title {
            canvas.title = title;
        }
        if let Some(content) = content {
            canvas.content = content;
        }
        canvas.updated_at = chrono::Utc::now();
        Some(canvas.clone())
    }

    pub async fn delete(&self, canvas_id: Uuid) -> bool {
        let mut canvases = self.canvases.write().await;
        let before = canvases.len();
        canvases.retain(|c| c.canvas_id != canvas_id);
        canvases.len() != before
    }

    pub async fn find_by_token(&self, token: &str) -> Option<VibeCanvas> {
        let canvases = self.canvases.read().await;
        canvases.iter().find(|c| c.share_token == token).cloned()
    }
}

impl Default for CanvasStore {
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

pub fn canvas_tools(store: Arc<CanvasStore>) -> Vec<(String, ToolSchema, ToolHandler)> {
    let create = Arc::clone(&store);
    let list = Arc::clone(&store);
    let share = Arc::clone(&store);

    let create_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&create);
        Box::pin(async move {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let project = args.get("project").and_then(|v| v.as_str()).map(String::from);
            let content = args.get("content").cloned().unwrap_or(json!({}));
            if title.is_empty() {
                return err("title is required".into());
            }
            let canvas = store.create(title, project, content).await;
            ok(json!({
                "canvas_id": canvas.canvas_id,
                "title": canvas.title,
                "share_url": format!("/api/vibe/canvases/share/{}", canvas.share_token),
            }))
        })
    });

    let list_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&list);
        Box::pin(async move {
            let project = args.get("project").and_then(|v| v.as_str()).map(String::from);
            let canvases = store.list(project.as_deref()).await;
            ok(json!({
                "canvases": canvases.iter().map(|c| json!({
                    "canvas_id": c.canvas_id,
                    "title": c.title,
                    "project": c.project,
                    "updated_at": c.updated_at,
                })).collect::<Vec<_>>(),
            }))
        })
    });

    let share_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&share);
        Box::pin(async move {
            let canvas_id = args.get("canvas_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());
            let canvas = match canvas_id {
                Some(id) => store.get(id).await,
                None => None,
            };
            match canvas {
                Some(c) => ok(json!({
                    "share_url": format!("/api/vibe/canvases/share/{}", c.share_token),
                    "canvas_id": c.canvas_id,
                })),
                None => err("canvas not found".into()),
            }
        })
    });

    vec![
        ("canvas/create".into(), ToolSchema::new("canvas/create", "Create a shareable canvas artifact").with_parameters(json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "project": {"type": "string"},
                "content": {"type": "object", "description": "Artifact content"}
            },
            "required": ["title"]
        })), create_handler),
        ("canvas/list".into(), ToolSchema::new("canvas/list", "List canvases"), list_handler),
        ("canvas/share".into(), ToolSchema::new("canvas/share", "Get the share URL for a canvas").with_parameters(json!({
            "type": "object",
            "properties": {
                "canvas_id": {"type": "string"}
            },
            "required": ["canvas_id"]
        })), share_handler),
    ]
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub title: String,
    pub project: Option<String>,
    pub content: Option<Value>,
}

#[derive(Debug, Serialize)]
struct CanvasResponse {
    success: bool,
    canvas: Option<VibeCanvas>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct CanvasesResponse {
    success: bool,
    canvases: Vec<VibeCanvas>,
}

pub fn canvases_router(store: Arc<CanvasStore>) -> Router {
    Router::new()
        .route("/api/vibe/canvases", axum::routing::get(list_canvases))
        .route("/api/vibe/canvases", axum::routing::post(create_canvas))
        .route("/api/vibe/canvases/:canvas_id", axum::routing::get(get_canvas))
        .route("/api/vibe/canvases/:canvas_id", axum::routing::delete(delete_canvas))
        .route("/api/vibe/canvases/share/:token", axum::routing::get(share_canvas))
        .layer(Extension(store))
}

async fn list_canvases(Extension(store): Extension<Arc<CanvasStore>>) -> Json<CanvasesResponse> {
    Json(CanvasesResponse { success: true, canvases: store.list(None).await })
}

async fn create_canvas(
    Extension(store): Extension<Arc<CanvasStore>>,
    Json(req): Json<CreateCanvasRequest>,
) -> Json<CanvasResponse> {
    let canvas = store.create(req.title, req.project, req.content.unwrap_or(json!({}))).await;
    Json(CanvasResponse { success: true, canvas: Some(canvas), error: None })
}

async fn get_canvas(
    Extension(store): Extension<Arc<CanvasStore>>,
    axum::extract::Path(canvas_id): axum::extract::Path<Uuid>,
) -> Json<CanvasResponse> {
    match store.get(canvas_id).await {
        Some(canvas) => Json(CanvasResponse { success: true, canvas: Some(canvas), error: None }),
        None => Json(CanvasResponse { success: false, canvas: None, error: Some("Canvas not found".into()) }),
    }
}

async fn delete_canvas(
    Extension(store): Extension<Arc<CanvasStore>>,
    axum::extract::Path(canvas_id): axum::extract::Path<Uuid>,
) -> Json<CanvasResponse> {
    let removed = store.delete(canvas_id).await;
    Json(CanvasResponse {
        success: removed,
        canvas: None,
        error: if removed { None } else { Some("Canvas not found".into()) },
    })
}

async fn share_canvas(
    Extension(store): Extension<Arc<CanvasStore>>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> axum::response::Html<String> {
    let html = match store.find_by_token(&token).await {
        Some(canvas) => format!(
            "<!DOCTYPE html><html><head><title>{}</title>\
             <style>body{{font-family:system-ui;max-width:960px;margin:40px auto;padding:0 20px;color:#1a1a2e}}\
             pre{{background:#f4f4f8;padding:16px;border-radius:8px;overflow:auto}}</style>\
             </head><body><h1>{}</h1><p>Shared canvas · updated {}</p><pre>{}</pre></body></html>",
            canvas.title,
            canvas.title,
            canvas.updated_at,
            serde_json::to_string_pretty(&canvas.content).unwrap_or_else(|_| "{}".into())
        ),
        None => "<!DOCTYPE html><html><body><h1>Canvas not found</h1></body></html>".to_string(),
    };
    axum::response::Html(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_get_update_round_trip() {
        let store = CanvasStore::new();
        let canvas = store.create("design".into(), Some("proj-a".into()), json!({"layers": 3})).await;
        let fetched = store.get(canvas.canvas_id).await.unwrap();
        assert_eq!(fetched.title, "design");
        assert_eq!(fetched.project.as_deref(), Some("proj-a"));
        assert!(!fetched.share_token.is_empty());

        let updated = store.update(canvas.canvas_id, Some("design v2".into()), Some(json!({"layers": 5}))).await.unwrap();
        assert_eq!(updated.title, "design v2");
        assert_eq!(updated.content["layers"], 5);
        assert!(updated.updated_at > canvas.updated_at || updated.updated_at == canvas.updated_at);
    }

    #[tokio::test]
    async fn list_filters_by_project() {
        let store = CanvasStore::new();
        store.create("a".into(), Some("p1".into()), json!({})).await;
        store.create("b".into(), Some("p2".into()), json!({})).await;
        store.create("c".into(), None, json!({})).await;
        assert_eq!(store.list(Some("p1")).await.len(), 1);
        assert_eq!(store.list(Some("nope")).await.len(), 0);
        assert_eq!(store.list(None).await.len(), 3);
    }

    #[tokio::test]
    async fn delete_and_find_by_token() {
        let store = CanvasStore::new();
        let canvas = store.create("x".into(), None, json!({})).await;
        assert!(store.find_by_token(&canvas.share_token).await.is_some());
        assert!(store.delete(canvas.canvas_id).await);
        assert!(!store.delete(canvas.canvas_id).await);
        assert!(store.find_by_token(&canvas.share_token).await.is_none());
    }

    #[tokio::test]
    async fn update_missing_returns_none() {
        let store = CanvasStore::new();
        assert!(store.update(Uuid::new_v4(), Some("t".into()), None).await.is_none());
    }
}
