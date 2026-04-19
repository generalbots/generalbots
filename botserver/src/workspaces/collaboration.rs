use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use super::blocks::BlockOperation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: Uuid,
    pub page_id: Uuid,
    pub active_users: Vec<ActiveUser>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveUser {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub color: String,
    pub cursor_position: Option<CursorPosition>,
    pub selection: Option<TextSelection>,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub block_id: Uuid,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelection {
    pub block_id: Uuid,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEvent {
    pub event_type: CollaborationEventType,
    pub page_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: CollaborationPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationEventType {
    UserJoined,
    UserLeft,
    CursorMoved,
    SelectionChanged,
    BlockOperation,
    PageUpdated,
    CommentAdded,
    CommentResolved,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CollaborationPayload {
    User(ActiveUser),
    Cursor(CursorPosition),
    Selection(TextSelection),
    Operation(BlockOperation),
    PageUpdate(PageUpdatePayload),
    Comment(CommentPayload),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageUpdatePayload {
    pub title: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentPayload {
    pub comment_id: Uuid,
    pub block_id: Option<Uuid>,
    pub content: String,
}

pub struct CollaborationManager {
    sessions: Arc<RwLock<HashMap<Uuid, CollaborationSession>>>,
    event_channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<CollaborationEvent>>>>,
    user_colors: Vec<String>,
}

impl CollaborationManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_channels: Arc::new(RwLock::new(HashMap::new())),
            user_colors: vec![
                "#E53935".to_string(),
                "#8E24AA".to_string(),
                "#3949AB".to_string(),
                "#039BE5".to_string(),
                "#00ACC1".to_string(),
                "#43A047".to_string(),
                "#7CB342".to_string(),
                "#FDD835".to_string(),
                "#FB8C00".to_string(),
                "#6D4C41".to_string(),
            ],
        }
    }

    pub async fn join_session(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        display_name: String,
        avatar_url: Option<String>,
    ) -> Result<(CollaborationSession, broadcast::Receiver<CollaborationEvent>), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let mut channels = self.event_channels.write().await;

        let color = self.assign_color(&sessions, page_id);

        let active_user = ActiveUser {
            user_id,
            display_name,
            avatar_url,
            color,
            cursor_position: None,
            selection: None,
            joined_at: now,
            last_seen: now,
        };

        let session = sessions.entry(page_id).or_insert_with(|| CollaborationSession {
            id: Uuid::new_v4(),
            page_id,
            active_users: Vec::new(),
            created_at: now,
            last_activity: now,
        });

        if !session.active_users.iter().any(|u| u.user_id == user_id) {
            session.active_users.push(active_user.clone());
        }
        session.last_activity = now;

        let (tx, rx) = if let Some(existing_tx) = channels.get(&page_id) {
            (existing_tx.clone(), existing_tx.subscribe())
        } else {
            let (tx, rx) = broadcast::channel(256);
            channels.insert(page_id, tx.clone());
            (tx, rx)
        };

        let event = CollaborationEvent {
            event_type: CollaborationEventType::UserJoined,
            page_id,
            user_id,
            timestamp: now,
            payload: CollaborationPayload::User(active_user),
        };

        let _ = tx.send(event);

        Ok((session.clone(), rx))
    }

    pub async fn leave_session(&self, page_id: Uuid, user_id: Uuid) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            session.active_users.retain(|u| u.user_id != user_id);
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                let event = CollaborationEvent {
                    event_type: CollaborationEventType::UserLeft,
                    page_id,
                    user_id,
                    timestamp: now,
                    payload: CollaborationPayload::Empty,
                };
                let _ = tx.send(event);
            }

            if session.active_users.is_empty() {
                sessions.remove(&page_id);
            }
        }

        Ok(())
    }

    pub async fn update_cursor(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        cursor: CursorPosition,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            if let Some(user) = session.active_users.iter_mut().find(|u| u.user_id == user_id) {
                user.cursor_position = Some(cursor.clone());
                user.last_seen = now;
            }
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                let event = CollaborationEvent {
                    event_type: CollaborationEventType::CursorMoved,
                    page_id,
                    user_id,
                    timestamp: now,
                    payload: CollaborationPayload::Cursor(cursor),
                };
                let _ = tx.send(event);
            }
        }

        Ok(())
    }

    pub async fn update_selection(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        selection: Option<TextSelection>,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            if let Some(user) = session.active_users.iter_mut().find(|u| u.user_id == user_id) {
                user.selection = selection.clone();
                user.last_seen = now;
            }
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                if let Some(sel) = selection {
                    let event = CollaborationEvent {
                        event_type: CollaborationEventType::SelectionChanged,
                        page_id,
                        user_id,
                        timestamp: now,
                        payload: CollaborationPayload::Selection(sel),
                    };
                    let _ = tx.send(event);
                }
            }
        }

        Ok(())
    }

    pub async fn broadcast_operation(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        operation: BlockOperation,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                let event = CollaborationEvent {
                    event_type: CollaborationEventType::BlockOperation,
                    page_id,
                    user_id,
                    timestamp: now,
                    payload: CollaborationPayload::Operation(operation),
                };
                let _ = tx.send(event);
            }
        }

        Ok(())
    }

    pub async fn get_session(&self, page_id: Uuid) -> Option<CollaborationSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&page_id).cloned()
    }

    pub async fn get_active_users(&self, page_id: Uuid) -> Vec<ActiveUser> {
        let sessions = self.sessions.read().await;
        sessions
            .get(&page_id)
            .map(|s| s.active_users.clone())
            .unwrap_or_default()
    }

    pub async fn cleanup_stale_sessions(&self, timeout_seconds: i64) {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(timeout_seconds);

        let mut sessions = self.sessions.write().await;
        let mut channels = self.event_channels.write().await;

        let stale_pages: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, s)| s.last_activity < cutoff)
            .map(|(id, _)| *id)
            .collect();

        for page_id in stale_pages {
            sessions.remove(&page_id);
            channels.remove(&page_id);
        }

        for session in sessions.values_mut() {
            session.active_users.retain(|u| u.last_seen >= cutoff);
        }
    }

    fn assign_color(&self, sessions: &HashMap<Uuid, CollaborationSession>, page_id: Uuid) -> String {
        if let Some(session) = sessions.get(&page_id) {
            let used_colors: Vec<&String> = session.active_users.iter().map(|u| &u.color).collect();

            for color in &self.user_colors {
                if !used_colors.contains(&color) {
                    return color.clone();
                }
            }
        }

        self.user_colors[0].clone()
    }
}

impl Default for CollaborationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalTransform {
    pub base_version: u64,
    pub operations: Vec<TransformOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOperation {
    pub op_type: TransformOpType,
    pub path: Vec<usize>,
    pub value: Option<serde_json::Value>,
    pub old_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransformOpType {
    Insert,
    Delete,
    Replace,
    Move,
}

pub fn transform_operations(
    op1: &TransformOperation,
    op2: &TransformOperation,
) -> (TransformOperation, TransformOperation) {
    let mut transformed_op1 = op1.clone();
    let mut transformed_op2 = op2.clone();

    if op1.path.is_empty() || op2.path.is_empty() {
        return (transformed_op1, transformed_op2);
    }

    let common_prefix_len = op1
        .path
        .iter()
        .zip(op2.path.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if common_prefix_len == 0 {
        return (transformed_op1, transformed_op2);
    }

    match (&op1.op_type, &op2.op_type) {
        (TransformOpType::Insert, TransformOpType::Insert) => {
            if op1.path <= op2.path {
                if let Some(idx) = transformed_op2.path.get_mut(common_prefix_len) {
                    *idx += 1;
                }
            } else if let Some(idx) = transformed_op1.path.get_mut(common_prefix_len) {
                *idx += 1;
            }
        }
        (TransformOpType::Delete, TransformOpType::Insert) => {
            if op1.path < op2.path {
                if let Some(idx) = transformed_op2.path.get_mut(common_prefix_len) {
                    *idx = idx.saturating_sub(1);
                }
            }
        }
        (TransformOpType::Insert, TransformOpType::Delete) => {
            if op2.path < op1.path {
                if let Some(idx) = transformed_op1.path.get_mut(common_prefix_len) {
                    *idx = idx.saturating_sub(1);
                }
            }
        }
        (TransformOpType::Delete, TransformOpType::Delete) => {
            if op1.path == op2.path {
                transformed_op2.op_type = TransformOpType::Replace;
                transformed_op2.value = None;
            }
        }
        _ => {}
    }

    (transformed_op1, transformed_op2)
}

#[derive(Debug, Clone)]
pub enum CollaborationError {
    SessionNotFound,
    UserNotInSession,
    BroadcastError(String),
    InvalidOperation(String),
}

impl std::fmt::Display for CollaborationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound => write!(f, "Collaboration session not found"),
            Self::UserNotInSession => write!(f, "User is not in the session"),
            Self::BroadcastError(e) => write!(f, "Broadcast error: {e}"),
            Self::InvalidOperation(e) => write!(f, "Invalid operation: {e}"),
        }
    }
}

impl std::error::Error for CollaborationError {}

pub async fn collaboration_cleanup_job(manager: Arc<CollaborationManager>, interval_seconds: u64) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));

    loop {
        ticker.tick().await;
        manager.cleanup_stale_sessions(300).await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub page_id: Uuid,
    pub users: Vec<PresenceUser>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUser {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub color: String,
    pub is_typing: bool,
    pub current_block: Option<Uuid>,
}

impl From<&ActiveUser> for PresenceUser {
    fn from(user: &ActiveUser) -> Self {
        Self {
            user_id: user.user_id,
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            color: user.color.clone(),
            is_typing: false,
            current_block: user.cursor_position.as_ref().map(|c| c.block_id),
        }
    }
}
