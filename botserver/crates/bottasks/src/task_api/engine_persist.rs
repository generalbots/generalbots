use std::sync::Arc;

use log::info;
use uuid::Uuid;

use crate::state::TasksState;
use crate::types::TaskManifest;

pub struct EnginePersistence {
    state: Arc<TasksState>,
}

impl EnginePersistence {
    pub fn new(state: Arc<TasksState>) -> Self {
        Self { state }
    }

    pub async fn save_manifest(&self, task_id: Uuid, manifest: &TaskManifest) -> Result<(), String> {
        let manifest_json = serde_json::to_string(manifest)
            .map_err(|e| format!("Serialize error: {}", e))?;

        (self.state.cache_set)(
            format!("task_manifest:{}", task_id),
            manifest_json,
            Some(86400),
        )
        .await
        .map_err(|e| format!("Cache set error: {}", e))?;

        info!("Manifest saved for task {}", task_id);
        Ok(())
    }

    pub async fn load_manifest(&self, task_id: Uuid) -> Result<Option<TaskManifest>, String> {
        let cached = (self.state.cache_get)(format!("task_manifest:{}", task_id))
            .await
            .map_err(|e| format!("Cache get error: {}", e))?;

        match cached {
            Some(json) => {
                let manifest: TaskManifest = serde_json::from_str(&json)
                    .map_err(|e| format!("Deserialize error: {}", e))?;
                Ok(Some(manifest))
            }
            None => {
                info!("No cached manifest for task {}", task_id);
                Ok(None)
            }
        }
    }

    pub async fn delete_manifest(&self, task_id: Uuid) -> Result<(), String> {
        (self.state.cache_set)(
            format!("task_manifest:{}", task_id),
            String::new(),
            Some(0),
        )
        .await
        .map_err(|e| format!("Cache delete error: {}", e))?;

        info!("Manifest deleted for task {}", task_id);
        Ok(())
    }

    pub async fn update_manifest_status(
        &self,
        task_id: Uuid,
        status: &str,
    ) -> Result<(), String> {
        let mut manifest = self
            .load_manifest(task_id)
            .await?
            .ok_or_else(|| format!("Manifest not found for task {}", task_id))?;

        manifest.status = status.to_string();
        self.save_manifest(task_id, &manifest).await
    }

    pub async fn append_terminal_output(
        &self,
        task_id: Uuid,
        text: &str,
        line_type: Option<&str>,
    ) -> Result<(), String> {
        let mut manifest = self
            .load_manifest(task_id)
            .await?
            .ok_or_else(|| format!("Manifest not found for task {}", task_id))?;

        manifest.add_terminal_output(text, line_type);
        self.save_manifest(task_id, &manifest).await
    }
}
