use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::anonymous::{MessageRole, SessionMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRequest {
    pub id: Uuid,
    pub anonymous_session_id: Uuid,
    pub target_user_id: Uuid,
    pub requested_at: DateTime<Utc>,
    pub status: MigrationStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<MigrationResult>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    PartialSuccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub messages_migrated: u32,
    pub messages_failed: u32,
    pub metadata_migrated: bool,
    pub preferences_migrated: bool,
    pub new_conversation_id: Option<Uuid>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigratedMessage {
    pub id: Uuid,
    pub original_id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub original_timestamp: DateTime<Utc>,
    pub migrated_at: DateTime<Utc>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub preserve_timestamps: bool,
    pub create_new_conversation: bool,
    pub merge_into_existing: bool,
    pub existing_conversation_id: Option<Uuid>,
    pub include_system_messages: bool,
    pub add_migration_marker: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            preserve_timestamps: true,
            create_new_conversation: true,
            merge_into_existing: false,
            existing_conversation_id: None,
            include_system_messages: false,
            add_migration_marker: true,
        }
    }
}

pub struct SessionMigrationService {
    migrations: Arc<RwLock<HashMap<Uuid, MigrationRequest>>>,
    migrated_messages: Arc<RwLock<HashMap<Uuid, Vec<MigratedMessage>>>>,
    user_conversations: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl SessionMigrationService {
    pub fn new() -> Self {
        Self {
            migrations: Arc::new(RwLock::new(HashMap::new())),
            migrated_messages: Arc::new(RwLock::new(HashMap::new())),
            user_conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn migrate_session_history(
        &self,
        anonymous_session_id: Uuid,
        target_user_id: Uuid,
        messages: Vec<SessionMessage>,
        config: MigrationConfig,
    ) -> Result<MigrationRequest, MigrationError> {
        let existing = self.get_migration_by_session(anonymous_session_id).await;
        if existing.is_some() {
            return Err(MigrationError::AlreadyMigrated);
        }

        let request_id = Uuid::new_v4();
        let now = Utc::now();

        let request = MigrationRequest {
            id: request_id,
            anonymous_session_id,
            target_user_id,
            requested_at: now,
            status: MigrationStatus::InProgress,
            completed_at: None,
            result: None,
        };

        {
            let mut migrations = self.migrations.write().await;
            migrations.insert(request_id, request.clone());
        }

        let result = self
            .execute_migration(target_user_id, messages, &config)
            .await;

        let mut migrations = self.migrations.write().await;
        if let Some(req) = migrations.get_mut(&request_id) {
            match &result {
                Ok(migration_result) => {
                    req.status = if migration_result.messages_failed > 0 {
                        MigrationStatus::PartialSuccess
                    } else {
                        MigrationStatus::Completed
                    };
                    req.result = Some(migration_result.clone());
                }
                Err(_) => {
                    req.status = MigrationStatus::Failed;
                }
            }
            req.completed_at = Some(Utc::now());
        }

        let final_request = migrations.get(&request_id).cloned();
        drop(migrations);

        match result {
            Ok(_) => final_request.ok_or(MigrationError::InternalError),
            Err(e) => Err(e),
        }
    }

    async fn execute_migration(
        &self,
        user_id: Uuid,
        messages: Vec<SessionMessage>,
        config: &MigrationConfig,
    ) -> Result<MigrationResult, MigrationError> {
        let conversation_id = if config.merge_into_existing {
            config
                .existing_conversation_id
                .unwrap_or_else(Uuid::new_v4)
        } else {
            Uuid::new_v4()
        };

        let mut migrated_count: u32 = 0;
        let failed_count: u32 = 0;
        let errors = Vec::new();
        let mut migrated = Vec::new();
        let now = Utc::now();

        if config.add_migration_marker {
            let marker = MigratedMessage {
                id: Uuid::new_v4(),
                original_id: Uuid::nil(),
                user_id,
                conversation_id,
                role: MessageRole::System,
                content: "--- Conversation history migrated from anonymous session ---".to_string(),
                original_timestamp: now,
                migrated_at: now,
                metadata: Some(HashMap::from([
                    ("migration_marker".to_string(), "true".to_string()),
                ])),
            };
            migrated.push(marker);
        }

        for message in messages {
            if !config.include_system_messages && message.role == MessageRole::System {
                continue;
            }

            let migrated_message = MigratedMessage {
                id: Uuid::new_v4(),
                original_id: message.id,
                user_id,
                conversation_id,
                role: message.role,
                content: message.content,
                original_timestamp: if config.preserve_timestamps {
                    message.timestamp
                } else {
                    now
                },
                migrated_at: now,
                metadata: message.metadata,
            };

            migrated.push(migrated_message);
            migrated_count += 1;
        }

        {
            let mut messages_store = self.migrated_messages.write().await;
            messages_store
                .entry(user_id)
                .or_default()
                .extend(migrated);
        }

        {
            let mut conversations = self.user_conversations.write().await;
            conversations
                .entry(user_id)
                .or_default()
                .push(conversation_id);
        }

        Ok(MigrationResult {
            messages_migrated: migrated_count,
            messages_failed: failed_count,
            metadata_migrated: true,
            preferences_migrated: true,
            new_conversation_id: Some(conversation_id),
            errors,
        })
    }

    pub async fn get_migration(&self, migration_id: Uuid) -> Option<MigrationRequest> {
        let migrations = self.migrations.read().await;
        migrations.get(&migration_id).cloned()
    }

    pub async fn get_migration_by_session(
        &self,
        session_id: Uuid,
    ) -> Option<MigrationRequest> {
        let migrations = self.migrations.read().await;
        migrations
            .values()
            .find(|m| m.anonymous_session_id == session_id)
            .cloned()
    }

    pub async fn get_user_migrations(&self, user_id: Uuid) -> Vec<MigrationRequest> {
        let migrations = self.migrations.read().await;
        migrations
            .values()
            .filter(|m| m.target_user_id == user_id)
            .cloned()
            .collect()
    }

    pub async fn get_migrated_messages(&self, user_id: Uuid) -> Vec<MigratedMessage> {
        let messages = self.migrated_messages.read().await;
        messages.get(&user_id).cloned().unwrap_or_default()
    }

    pub async fn get_conversation_messages(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
    ) -> Vec<MigratedMessage> {
        let messages = self.migrated_messages.read().await;
        messages
            .get(&user_id)
            .map(|msgs| {
                msgs.iter()
                    .filter(|m| m.conversation_id == conversation_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn get_user_conversations(&self, user_id: Uuid) -> Vec<Uuid> {
        let conversations = self.user_conversations.read().await;
        conversations.get(&user_id).cloned().unwrap_or_default()
    }

    pub async fn rollback_migration(
        &self,
        migration_id: Uuid,
    ) -> Result<(), MigrationError> {
        let migrations = self.migrations.read().await;
        let migration = migrations
            .get(&migration_id)
            .ok_or(MigrationError::NotFound)?;

        if migration.status != MigrationStatus::Completed
            && migration.status != MigrationStatus::PartialSuccess
        {
            return Err(MigrationError::CannotRollback);
        }

        let user_id = migration.target_user_id;
        let conversation_id = migration
            .result
            .as_ref()
            .and_then(|r| r.new_conversation_id);

        drop(migrations);

        if let Some(conv_id) = conversation_id {
            let mut messages = self.migrated_messages.write().await;
            if let Some(user_messages) = messages.get_mut(&user_id) {
                user_messages.retain(|m| m.conversation_id != conv_id);
            }

            let mut conversations = self.user_conversations.write().await;
            if let Some(user_convs) = conversations.get_mut(&user_id) {
                user_convs.retain(|c| *c != conv_id);
            }
        }

        let mut migrations = self.migrations.write().await;
        migrations.remove(&migration_id);

        Ok(())
    }
}

impl Default for SessionMigrationService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum MigrationError {
    NotFound,
    AlreadyMigrated,
    SessionNotFound,
    UserNotFound,
    CannotRollback,
    InternalError,
    MessageStoreFailed(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Migration not found"),
            Self::AlreadyMigrated => write!(f, "Session has already been migrated"),
            Self::SessionNotFound => write!(f, "Anonymous session not found"),
            Self::UserNotFound => write!(f, "Target user not found"),
            Self::CannotRollback => write!(f, "Cannot rollback migration in current state"),
            Self::InternalError => write!(f, "Internal migration error"),
            Self::MessageStoreFailed(msg) => write!(f, "Failed to store messages: {msg}"),
        }
    }
}

impl std::error::Error for MigrationError {}
