use crate::types::VibeState;
use crate::tool_executor::{ToolHandler, ToolSchema};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeSkill {
    pub skill_id: Uuid,
    pub name: String,
    pub description: String,
    pub content: String,
    pub triggers: Vec<String>,
    pub enabled: bool,
}

pub struct SkillStore {
    skills: RwLock<Vec<VibeSkill>>,
}

impl SkillStore {
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(Vec::new()),
        }
    }

    pub async fn register(&self, name: String, description: String, content: String, triggers: Vec<String>) -> VibeSkill {
        let skill = VibeSkill {
            skill_id: Uuid::new_v4(),
            name,
            description,
            content,
            triggers,
            enabled: true,
        };
        let mut skills = self.skills.write().await;
        skills.retain(|s| s.name != skill.name);
        skills.push(skill.clone());
        skill
    }

    pub async fn delete(&self, name: &str) -> bool {
        let mut skills = self.skills.write().await;
        let before = skills.len();
        skills.retain(|s| s.name != name);
        skills.len() != before
    }

    pub async fn list(&self) -> Vec<VibeSkill> {
        self.skills.read().await.clone()
    }

    pub async fn seed_bootstrap(&self) {
        for (name, description, content, triggers) in crate::skill_loader::bootstrap_skill_definitions() {
            self.register(name, description, content, triggers).await;
        }
    }

    pub async fn apply(&self, names: &[String]) -> String {
        let skills = self.skills.read().await;
        let mut stacked = String::new();
        let mut count = 0usize;
        for name in names {
            if count >= crate::skill_loader::MAX_STACKED_SKILLS {
                break;
            }
            for skill in skills.iter().filter(|s| s.name == *name && s.enabled) {
                if count >= crate::skill_loader::MAX_STACKED_SKILLS {
                    break;
                }
                stacked.push_str(&format!(
                    "## Skill: {}\n{}\n{}\n\n",
                    skill.name, skill.description, skill.content
                ));
                count += 1;
            }
        }
        stacked
    }

    pub async fn auto_trigger(&self, intent: &str) -> Vec<VibeSkill> {
        let skills = self.skills.read().await;
        let lower = intent.to_lowercase();
        skills
            .iter()
            .filter(|s| s.enabled && !s.triggers.is_empty())
            .filter(|s| s.triggers.iter().any(|t| lower.contains(&t.to_lowercase())))
            .cloned()
            .collect()
    }
}

impl Default for SkillStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn skill_tools(store: Arc<SkillStore>) -> Vec<(String, ToolSchema, ToolHandler)> {
    let list = Arc::clone(&store);
    let apply = Arc::clone(&store);
    let create = Arc::clone(&store);

    let list_handler: ToolHandler = Arc::new(move |_args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&list);
        Box::pin(async move {
            let skills = store.list().await;
            VibeToolResultOk::ok(json!({
                "skills": skills.iter().map(|s| json!({
                    "name": s.name,
                    "description": s.description,
                    "triggers": s.triggers,
                    "enabled": s.enabled,
                })).collect::<Vec<_>>(),
            }))
        })
    });

    let apply_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&apply);
        Box::pin(async move {
            let names: Vec<String> = args
                .get("skills")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let stacked = store.apply(&names).await;
            if stacked.is_empty() {
                VibeToolResultOk::err("No enabled skills matched the requested names".into())
            } else {
                VibeToolResultOk::ok(json!({ "applied": names, "content": stacked }))
            }
        })
    });

    let create_handler: ToolHandler = Arc::new(move |args: Value, _state: &dyn VibeState| {
        let store = Arc::clone(&create);
        Box::pin(async move {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let triggers: Vec<String> = args
                .get("triggers")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if name.is_empty() || content.is_empty() {
                return VibeToolResultOk::err("name and content are required".into());
            }
            let skill = store.register(name, description, content, triggers).await;
            VibeToolResultOk::ok(json!({ "registered": skill.name, "skill_id": skill.skill_id }))
        })
    });

    vec![
        ("skill/list".into(), ToolSchema::new("skill/list", "List registered skills"), list_handler),
        ("skill/apply".into(), ToolSchema::new("skill/apply", "Stack one or more skills into the context").with_parameters(json!({
            "type": "object",
            "properties": {
                "skills": {"type": "array", "items": {"type": "string"}, "description": "Skill names to stack"}
            },
            "required": ["skills"]
        })), apply_handler),
        ("skill/create".into(), ToolSchema::new("skill/create", "Register a new reusable skill").with_parameters(json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "description": {"type": "string"},
                "content": {"type": "string", "description": "SKILL.md body"},
                "triggers": {"type": "array", "items": {"type": "string"}, "description": "Auto-trigger keywords"}
            },
            "required": ["name", "content"]
        })), create_handler),
    ]
}

struct VibeToolResultOk;

impl VibeToolResultOk {
    fn ok(data: Value) -> crate::types::VibeToolResult {
        crate::types::VibeToolResult {
            success: true,
            data,
            error: None,
            latency_ms: 0,
        }
    }

    fn err(error: String) -> crate::types::VibeToolResult {
        crate::types::VibeToolResult {
            success: false,
            data: Value::Null,
            error: Some(error),
            latency_ms: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub triggers: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct SkillResponse {
    success: bool,
    skill: Option<VibeSkill>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsListResponse {
    success: bool,
    skills: Vec<VibeSkill>,
}

pub fn skills_router(store: Arc<SkillStore>) -> Router {
    Router::new()
        .route("/api/vibe/skills", axum::routing::get(list_skills))
        .route("/api/vibe/skills", axum::routing::post(create_skill))
        .route("/api/vibe/skills/:name", axum::routing::delete(delete_skill))
        .layer(Extension(store))
}

async fn list_skills(Extension(store): Extension<Arc<SkillStore>>) -> Json<SkillsListResponse> {
    Json(SkillsListResponse {
        success: true,
        skills: store.list().await,
    })
}

async fn create_skill(
    Extension(store): Extension<Arc<SkillStore>>,
    Json(req): Json<CreateSkillRequest>,
) -> Json<SkillResponse> {
    let skill = store
        .register(
            req.name,
            req.description.unwrap_or_default(),
            req.content,
            req.triggers.unwrap_or_default(),
        )
        .await;
    Json(SkillResponse {
        success: true,
        skill: Some(skill),
        error: None,
    })
}

async fn delete_skill(
    Extension(store): Extension<Arc<SkillStore>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<SkillResponse> {
    let removed = store.delete(&name).await;
    Json(SkillResponse {
        success: removed,
        skill: None,
        error: if removed { None } else { Some("Skill not found".into()) },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_dedupes_by_name_and_enables_by_default() {
        let store = SkillStore::new();
        let first = store.register("sql".into(), "SQL expert".into(), "write sql".into(), vec!["sql".into()]).await;
        let second = store.register("sql".into(), "New desc".into(), "write better sql".into(), vec![].into()).await;
        assert_ne!(first.skill_id, second.skill_id);
        let skills = store.list().await;
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].description, "New desc");
        assert!(skills[0].enabled);
    }

    #[tokio::test]
    async fn apply_stacks_only_enabled_matches() {
        let store = SkillStore::new();
        store.register("a".into(), "A".into(), "content-a".into(), vec![].into()).await;
        store.register("b".into(), "B".into(), "content-b".into(), vec![].into()).await;
        let stacked = store.apply(&["a".into(), "b".into(), "nope".into()]).await;
        assert!(stacked.contains("content-a"));
        assert!(stacked.contains("content-b"));
        assert!(!stacked.contains("nope"));
        assert!(store.apply(&["nope".into()]).await.is_empty());
    }

    #[tokio::test]
    async fn apply_caps_at_five_skills() {
        let store = SkillStore::new();
        for i in 0..7 {
            store.register(format!("s{i}"), format!("S{i}"), format!("content-{i}"), vec![]).await;
        }
        let names: Vec<String> = (0..7).map(|i| format!("s{i}")).collect();
        let stacked = store.apply(&names).await;
        assert!(stacked.contains("content-0"));
        assert!(stacked.contains("content-4"));
        assert!(!stacked.contains("content-5"));
        assert!(!stacked.contains("content-6"));
    }

    #[tokio::test]
    async fn seed_bootstrap_registers_all_skills() {
        let store = SkillStore::new();
        store.seed_bootstrap().await;
        let skills = store.list().await;
        assert_eq!(skills.len(), 6);
        assert!(skills.iter().all(|s| s.enabled && !s.triggers.is_empty()));
    }

    #[tokio::test]
    async fn delete_removes_and_reports() {
        let store = SkillStore::new();
        store.register("x".into(), "X".into(), "c".into(), vec![].into()).await;
        assert!(store.delete("x").await);
        assert!(!store.delete("x").await);
        assert!(store.list().await.is_empty());
    }

    #[tokio::test]
    async fn auto_trigger_matches_case_insensitively() {
        let store = SkillStore::new();
        store.register("email".into(), "Email".into(), "mail body".into(), vec!["send email".into()]).await;
        store.register("disabled".into(), "D".into(), "no".into(), vec!["send email".into()]).await;
        let mut skills = store.list().await;
        skills[1].enabled = false;
        {
            let mut guard = store.skills.write().await;
            *guard = skills;
        }
        let triggered = store.auto_trigger("Please SEND EMAIL now").await;
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].name, "email");
    }

    #[tokio::test]
    async fn auto_trigger_ignores_empty_triggers() {
        let store = SkillStore::new();
        store.register("no-trigger".into(), "N".into(), "c".into(), vec![].into()).await;
        assert!(store.auto_trigger("anything").await.is_empty());
    }

    #[test]
    fn skill_serialization_shape() {
        let skill = VibeSkill {
            skill_id: Uuid::nil(),
            name: "n".into(),
            description: "d".into(),
            content: "c".into(),
            triggers: vec!["t".into()],
            enabled: true,
        };
        let v = serde_json::to_value(&skill).unwrap();
        assert_eq!(v["name"], "n");
        assert_eq!(v["enabled"], true);
    }
}
