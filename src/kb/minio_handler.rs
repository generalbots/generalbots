use crate::shared::state::AppState;
use log::{debug, error, info};
use opendal::Operator;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct FileState {
    pub path: String,
    pub size: i64,
    pub etag: String,
    pub last_modified: Option<String>,
}

pub struct MinIOHandler {
    state: Arc<AppState>,
    watched_prefixes: Arc<tokio::sync::RwLock<Vec<String>>>,
    file_states: Arc<tokio::sync::RwLock<HashMap<String, FileState>>>,
}

impl MinIOHandler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            watched_prefixes: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            file_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn watch_prefix(&self, prefix: String) {
        let mut prefixes = self.watched_prefixes.write().await;
        if !prefixes.contains(&prefix) {
            prefixes.push(prefix.clone());
        }
    }

    pub async fn unwatch_prefix(&self, prefix: &str) {
        let mut prefixes = self.watched_prefixes.write().await;
        prefixes.retain(|p| p != prefix);
    }

    pub fn spawn(
        self: Arc<Self>,
        change_callback: Arc<dyn Fn(FileChangeEvent) + Send + Sync>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(15));
            loop {
                tick.tick().await;
                if let Err(e) = self.check_for_changes(&change_callback).await {
                    error!("Error checking for MinIO changes: {}", e);
                }
            }
        })
    }

    async fn check_for_changes(
        &self,
        callback: &Arc<dyn Fn(FileChangeEvent) + Send + Sync>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let op = match &self.state.s3_operator {
            Some(op) => op,
            None => {
                return Ok(());
            }
        };

        let prefixes = self.watched_prefixes.read().await;
        for prefix in prefixes.iter() {
            if let Err(e) = self.check_prefix_changes(op, prefix, callback).await {
                error!("Error checking prefix {}: {}", prefix, e);
            }
        }
        Ok(())
    }

    async fn check_prefix_changes(
        &self,
        op: &Operator,
        prefix: &str,
        callback: &Arc<dyn Fn(FileChangeEvent) + Send + Sync>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut current_files = HashMap::new();
        
        let mut lister = op.lister_with(prefix).recursive(true).await?;
        while let Some(entry) = lister.try_next().await? {
            let path = entry.path().to_string();
            
            if path.ends_with('/') {
                continue;
            }

            let meta = op.stat(&path).await?;
            let file_state = FileState {
                path: path.clone(),
                size: meta.content_length() as i64,
                etag: meta.etag().unwrap_or_default().to_string(),
                last_modified: meta.last_modified().map(|dt| dt.to_rfc3339()),
            };
            current_files.insert(path, file_state);
        }

        let mut file_states = self.file_states.write().await;
        for (path, current_state) in current_files.iter() {
            if let Some(previous_state) = file_states.get(path) {
                if current_state.etag != previous_state.etag
                    || current_state.size != previous_state.size
                {
                    callback(FileChangeEvent::Modified {
                        path: path.clone(),
                        size: current_state.size,
                        etag: current_state.etag.clone(),
                    });
                }
            } else {
                callback(FileChangeEvent::Created {
                    path: path.clone(),
                    size: current_state.size,
                    etag: current_state.etag.clone(),
                });
            }
        }

        let previous_paths: Vec<String> = file_states
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        for path in previous_paths {
            if !current_files.contains_key(&path) {
                callback(FileChangeEvent::Deleted { path: path.clone() });
                file_states.remove(&path);
            }
        }

        for (path, state) in current_files {
            file_states.insert(path, state);
        }

        Ok(())
    }

    pub async fn get_file_state(&self, path: &str) -> Option<FileState> {
        let states = self.file_states.read().await;
        states.get(path).cloned()
    }

    pub async fn clear_state(&self) {
        let mut states = self.file_states.write().await;
        states.clear();
    }

    pub async fn get_files_by_prefix(&self, prefix: &str) -> Vec<FileState> {
        let states = self.file_states.read().await;
        states
            .values()
            .filter(|state| state.path.starts_with(prefix))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum FileChangeEvent {
    Created {
        path: String,
        size: i64,
        etag: String,
    },
    Modified {
        path: String,
        size: i64,
        etag: String,
    },
    Deleted {
        path: String,
    },
}

impl FileChangeEvent {
    pub fn path(&self) -> &str {
        match self {
            FileChangeEvent::Created { path, .. } => path,
            FileChangeEvent::Modified { path, .. } => path,
            FileChangeEvent::Deleted { path } => path,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            FileChangeEvent::Created { .. } => "created",
            FileChangeEvent::Modified { .. } => "modified",
            FileChangeEvent::Deleted { .. } => "deleted",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_change_event_path() {
        let event = FileChangeEvent::Created {
            path: "test.txt".to_string(),
            size: 100,
            etag: "abc123".to_string(),
        };
        assert_eq!(event.path(), "test.txt");
        assert_eq!(event.event_type(), "created");
    }

    #[test]
    fn test_file_change_event_types() {
        let created = FileChangeEvent::Created {
            path: "file1.txt".to_string(),
            size: 100,
            etag: "abc".to_string(),
        };
        let modified = FileChangeEvent::Modified {
            path: "file2.txt".to_string(),
            size: 200,
            etag: "def".to_string(),
        };
        let deleted = FileChangeEvent::Deleted {
            path: "file3.txt".to_string(),
        };
        assert_eq!(created.event_type(), "created");
        assert_eq!(modified.event_type(), "modified");
        assert_eq!(deleted.event_type(), "deleted");
    }
}
