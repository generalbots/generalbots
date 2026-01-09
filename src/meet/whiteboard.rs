use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Freehand,
    Text,
    Image,
    Sticky,
    Polygon,
    Triangle,
    Diamond,
    Star,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrokeStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub fn to_rgba(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0
        } else {
            1.0
        };

        Some(Color { r, g, b, a })
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeStyle {
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
    pub stroke_width: f64,
    pub stroke_style: StrokeStyle,
    pub opacity: f32,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        ShapeStyle {
            stroke_color: Color::default(),
            fill_color: None,
            stroke_width: 2.0,
            stroke_style: StrokeStyle::Solid,
            opacity: 1.0,
            font_size: None,
            font_family: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub id: Uuid,
    pub shape_type: ShapeType,
    pub points: Vec<Point>,
    pub style: ShapeStyle,
    pub text: Option<String>,
    pub image_url: Option<String>,
    pub rotation: f64,
    pub z_index: i32,
    pub locked: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub user_id: Uuid,
    pub username: String,
    pub color: Color,
    pub position: Point,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub user_id: Uuid,
    pub shape_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WhiteboardOperation {
    AddShape { shape: Shape },
    UpdateShape { shape: Shape },
    DeleteShape { shape_id: Uuid },
    MoveShapes { shape_ids: Vec<Uuid>, delta: Point },
    ResizeShape { shape_id: Uuid, bounds: ShapeBounds },
    RotateShape { shape_id: Uuid, angle: f64 },
    BringToFront { shape_id: Uuid },
    SendToBack { shape_id: Uuid },
    LockShape { shape_id: Uuid },
    UnlockShape { shape_id: Uuid },
    GroupShapes { shape_ids: Vec<Uuid>, group_id: Uuid },
    UngroupShapes { group_id: Uuid },
    Clear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRedoEntry {
    pub id: Uuid,
    pub operation: WhiteboardOperation,
    pub inverse_operation: WhiteboardOperation,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WhiteboardMessage {
    Join {
        user_id: Uuid,
        username: String,
        color: Color,
    },
    Leave {
        user_id: Uuid,
    },
    Operation {
        user_id: Uuid,
        operation: WhiteboardOperation,
    },
    CursorMove {
        cursor: CursorPosition,
    },
    Select {
        selection: Selection,
    },
    Deselect {
        user_id: Uuid,
    },
    Undo {
        user_id: Uuid,
    },
    Redo {
        user_id: Uuid,
    },
    RequestSync,
    FullSync {
        shapes: Vec<Shape>,
        cursors: Vec<CursorPosition>,
        selections: Vec<Selection>,
    },
    Ping,
    Pong,
    Error {
        message: String,
    },
    UserJoined {
        user_id: Uuid,
        username: String,
        color: Color,
    },
    UserLeft {
        user_id: Uuid,
    },
}

#[derive(Debug)]
pub struct WhiteboardState {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub name: String,
    pub shapes: HashMap<Uuid, Shape>,
    pub cursors: HashMap<Uuid, CursorPosition>,
    pub selections: HashMap<Uuid, Selection>,
    pub undo_stack: Vec<UndoRedoEntry>,
    pub redo_stack: Vec<UndoRedoEntry>,
    pub max_undo_history: usize,
    pub next_z_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WhiteboardState {
    pub fn new(id: Uuid, conversation_id: Uuid, name: String) -> Self {
        let now = Utc::now();
        WhiteboardState {
            id,
            conversation_id,
            name,
            shapes: HashMap::new(),
            cursors: HashMap::new(),
            selections: HashMap::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_history: 100,
            next_z_index: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_operation(
        &mut self,
        operation: WhiteboardOperation,
        user_id: Uuid,
    ) -> Result<Option<WhiteboardOperation>, String> {
        let inverse = self.compute_inverse(&operation)?;

        match operation {
            WhiteboardOperation::AddShape { shape } => {
                let mut shape = shape;
                shape.z_index = self.next_z_index;
                self.next_z_index += 1;
                self.shapes.insert(shape.id, shape);
            }
            WhiteboardOperation::UpdateShape { shape } => {
                if !self.shapes.contains_key(&shape.id) {
                    return Err("Shape not found".to_string());
                }
                self.shapes.insert(shape.id, shape);
            }
            WhiteboardOperation::DeleteShape { shape_id } => {
                if self.shapes.remove(&shape_id).is_none() {
                    return Err("Shape not found".to_string());
                }
                for selection in self.selections.values_mut() {
                    selection.shape_ids.retain(|id| *id != shape_id);
                }
            }
            WhiteboardOperation::MoveShapes { shape_ids, delta } => {
                for shape_id in &shape_ids {
                    if let Some(shape) = self.shapes.get_mut(shape_id) {
                        for point in &mut shape.points {
                            point.x += delta.x;
                            point.y += delta.y;
                        }
                        shape.updated_at = Utc::now();
                    }
                }
            }
            WhiteboardOperation::ResizeShape { shape_id, bounds } => {
                if let Some(shape) = self.shapes.get_mut(&shape_id) {
                    if shape.points.len() >= 2 {
                        shape.points[0] = Point {
                            x: bounds.x,
                            y: bounds.y,
                        };
                        shape.points[1] = Point {
                            x: bounds.x + bounds.width,
                            y: bounds.y + bounds.height,
                        };
                    }
                    shape.updated_at = Utc::now();
                }
            }
            WhiteboardOperation::RotateShape { shape_id, angle } => {
                if let Some(shape) = self.shapes.get_mut(&shape_id) {
                    shape.rotation = angle;
                    shape.updated_at = Utc::now();
                }
            }
            WhiteboardOperation::BringToFront { shape_id } => {
                if let Some(shape) = self.shapes.get_mut(&shape_id) {
                    shape.z_index = self.next_z_index;
                    self.next_z_index += 1;
                    shape.updated_at = Utc::now();
                }
            }
            WhiteboardOperation::SendToBack { shape_id } => {
                let min_z = self.shapes.values().map(|s| s.z_index).min().unwrap_or(0) - 1;
                if let Some(shape) = self.shapes.get_mut(&shape_id) {
                    shape.z_index = min_z;
                    shape.updated_at = Utc::now();
                }
            }
            WhiteboardOperation::LockShape { shape_id } => {
                if let Some(shape) = self.shapes.get_mut(&shape_id) {
                    shape.locked = true;
                    shape.updated_at = Utc::now();
                }
            }
            WhiteboardOperation::UnlockShape { shape_id } => {
                if let Some(shape) = self.shapes.get_mut(&shape_id) {
                    shape.locked = false;
                    shape.updated_at = Utc::now();
                }
            }
            WhiteboardOperation::GroupShapes { .. } => {}
            WhiteboardOperation::UngroupShapes { .. } => {}
            WhiteboardOperation::Clear => {
                self.shapes.clear();
                self.selections.clear();
                self.next_z_index = 0;
            }
        }

        self.updated_at = Utc::now();

        if let Some(inv) = inverse.clone() {
            let entry = UndoRedoEntry {
                id: Uuid::new_v4(),
                operation: self.reconstruct_operation(&inv),
                inverse_operation: inv,
                user_id,
                timestamp: Utc::now(),
            };
            self.undo_stack.push(entry);
            if self.undo_stack.len() > self.max_undo_history {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }

        Ok(inverse)
    }

    fn compute_inverse(&self, operation: &WhiteboardOperation) -> Result<Option<WhiteboardOperation>, String> {
        match operation {
            WhiteboardOperation::AddShape { shape } => {
                Ok(Some(WhiteboardOperation::DeleteShape { shape_id: shape.id }))
            }
            WhiteboardOperation::UpdateShape { shape } => {
                if let Some(old_shape) = self.shapes.get(&shape.id) {
                    Ok(Some(WhiteboardOperation::UpdateShape {
                        shape: old_shape.clone(),
                    }))
                } else {
                    Err("Shape not found".to_string())
                }
            }
            WhiteboardOperation::DeleteShape { shape_id } => {
                if let Some(shape) = self.shapes.get(shape_id) {
                    Ok(Some(WhiteboardOperation::AddShape {
                        shape: shape.clone(),
                    }))
                } else {
                    Err("Shape not found".to_string())
                }
            }
            WhiteboardOperation::MoveShapes { shape_ids, delta } => {
                Ok(Some(WhiteboardOperation::MoveShapes {
                    shape_ids: shape_ids.clone(),
                    delta: Point {
                        x: -delta.x,
                        y: -delta.y,
                    },
                }))
            }
            WhiteboardOperation::RotateShape { shape_id, .. } => {
                if let Some(shape) = self.shapes.get(shape_id) {
                    Ok(Some(WhiteboardOperation::RotateShape {
                        shape_id: *shape_id,
                        angle: shape.rotation,
                    }))
                } else {
                    Ok(None)
                }
            }
            WhiteboardOperation::LockShape { shape_id } => {
                Ok(Some(WhiteboardOperation::UnlockShape {
                    shape_id: *shape_id,
                }))
            }
            WhiteboardOperation::UnlockShape { shape_id } => {
                Ok(Some(WhiteboardOperation::LockShape {
                    shape_id: *shape_id,
                }))
            }
            WhiteboardOperation::Clear => {
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn reconstruct_operation(&self, inverse: &WhiteboardOperation) -> WhiteboardOperation {
        inverse.clone()
    }

    pub fn undo(&mut self, user_id: Uuid) -> Option<WhiteboardOperation> {
        let user_entries: Vec<_> = self
            .undo_stack
            .iter()
            .enumerate()
            .filter(|(_, e)| e.user_id == user_id)
            .map(|(i, _)| i)
            .collect();

        if let Some(&last_idx) = user_entries.last() {
            let entry = self.undo_stack.remove(last_idx);
            let inverse = entry.inverse_operation.clone();
            let _ = self.apply_operation_without_history(inverse.clone());
            self.redo_stack.push(entry);
            Some(inverse)
        } else {
            None
        }
    }

    pub fn redo(&mut self, user_id: Uuid) -> Option<WhiteboardOperation> {
        let user_entries: Vec<_> = self
            .redo_stack
            .iter()
            .enumerate()
            .filter(|(_, e)| e.user_id == user_id)
            .map(|(i, _)| i)
            .collect();

        if let Some(&last_idx) = user_entries.last() {
            let entry = self.redo_stack.remove(last_idx);
            let operation = entry.operation.clone();
            let _ = self.apply_operation_without_history(operation.clone());
            self.undo_stack.push(entry);
            Some(operation)
        } else {
            None
        }
    }

    fn apply_operation_without_history(&mut self, operation: WhiteboardOperation) -> Result<(), String> {
        match operation {
            WhiteboardOperation::AddShape { shape } => {
                self.shapes.insert(shape.id, shape);
            }
            WhiteboardOperation::UpdateShape { shape } => {
                self.shapes.insert(shape.id, shape);
            }
            WhiteboardOperation::DeleteShape { shape_id } => {
                self.shapes.remove(&shape_id);
            }
            WhiteboardOperation::MoveShapes { shape_ids, delta } => {
                for shape_id in &shape_ids {
                    if let Some(shape) = self.shapes.get_mut(shape_id) {
                        for point in &mut shape.points {
                            point.x += delta.x;
                            point.y += delta.y;
                        }
                    }
                }
            }
            _ => {}
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_cursor(&mut self, cursor: CursorPosition) {
        self.cursors.insert(cursor.user_id, cursor);
    }

    pub fn remove_cursor(&mut self, user_id: &Uuid) {
        self.cursors.remove(user_id);
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selections.insert(selection.user_id, selection);
    }

    pub fn clear_selection(&mut self, user_id: &Uuid) {
        self.selections.remove(user_id);
    }

    pub fn get_shapes_sorted(&self) -> Vec<&Shape> {
        let mut shapes: Vec<_> = self.shapes.values().collect();
        shapes.sort_by_key(|s| s.z_index);
        shapes
    }

    pub fn to_sync_message(&self) -> WhiteboardMessage {
        WhiteboardMessage::FullSync {
            shapes: self.shapes.values().cloned().collect(),
            cursors: self.cursors.values().cloned().collect(),
            selections: self.selections.values().cloned().collect(),
        }
    }
}

pub struct WhiteboardManager {
    whiteboards: Arc<RwLock<HashMap<Uuid, WhiteboardState>>>,
    broadcast_channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WhiteboardMessage>>>>,
}

impl Default for WhiteboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WhiteboardManager {
    pub fn new() -> Self {
        WhiteboardManager {
            whiteboards: Arc::new(RwLock::new(HashMap::new())),
            broadcast_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_whiteboard(
        &self,
        conversation_id: Uuid,
        name: String,
    ) -> Uuid {
        let whiteboard_id = Uuid::new_v4();
        let state = WhiteboardState::new(whiteboard_id, conversation_id, name);

        let (tx, _) = broadcast::channel(1024);

        {
            let mut whiteboards = self.whiteboards.write().await;
            whiteboards.insert(whiteboard_id, state);
        }

        {
            let mut channels = self.broadcast_channels.write().await;
            channels.insert(whiteboard_id, tx);
        }

        whiteboard_id
    }

    pub async fn get_whiteboard(&self, whiteboard_id: &Uuid) -> Option<WhiteboardState> {
        let whiteboards = self.whiteboards.read().await;
        whiteboards.get(whiteboard_id).cloned()
    }

    pub async fn delete_whiteboard(&self, whiteboard_id: &Uuid) -> bool {
        let mut whiteboards = self.whiteboards.write().await;
        let mut channels = self.broadcast_channels.write().await;

        channels.remove(whiteboard_id);
        whiteboards.remove(whiteboard_id).is_some()
    }

    pub async fn subscribe(&self, whiteboard_id: &Uuid) -> Option<broadcast::Receiver<WhiteboardMessage>> {
        let channels = self.broadcast_channels.read().await;
        channels.get(whiteboard_id).map(|tx| tx.subscribe())
    }

    pub async fn broadcast(&self, whiteboard_id: &Uuid, message: WhiteboardMessage) {
        let channels = self.broadcast_channels.read().await;
        if let Some(tx) = channels.get(whiteboard_id) {
            let _ = tx.send(message);
        }
    }

    pub async fn apply_operation(
        &self,
        whiteboard_id: &Uuid,
        operation: WhiteboardOperation,
        user_id: Uuid,
    ) -> Result<(), String> {
        let mut whiteboards = self.whiteboards.write().await;
        let whiteboard = whiteboards
            .get_mut(whiteboard_id)
            .ok_or("Whiteboard not found")?;

        whiteboard.apply_operation(operation.clone(), user_id)?;

        drop(whiteboards);

        self.broadcast(
            whiteboard_id,
            WhiteboardMessage::Operation { user_id, operation },
        )
        .await;

        Ok(())
    }

    pub async fn undo(&self, whiteboard_id: &Uuid, user_id: Uuid) -> Option<WhiteboardOperation> {
        let mut whiteboards = self.whiteboards.write().await;
        if let Some(whiteboard) = whiteboards.get_mut(whiteboard_id) {
            whiteboard.undo(user_id)
        } else {
            None
        }
    }

    pub async fn redo(&self, whiteboard_id: &Uuid, user_id: Uuid) -> Option<WhiteboardOperation> {
        let mut whiteboards = self.whiteboards.write().await;
        if let Some(whiteboard) = whiteboards.get_mut(whiteboard_id) {
            whiteboard.redo(user_id)
        } else {
            None
        }
    }

    pub async fn update_cursor(&self, whiteboard_id: &Uuid, cursor: CursorPosition) {
        let mut whiteboards = self.whiteboards.write().await;
        if let Some(whiteboard) = whiteboards.get_mut(whiteboard_id) {
            whiteboard.update_cursor(cursor.clone());
        }
        drop(whiteboards);

        self.broadcast(whiteboard_id, WhiteboardMessage::CursorMove { cursor })
            .await;
    }

    pub async fn user_join(
        &self,
        whiteboard_id: &Uuid,
        user_id: Uuid,
        username: String,
        color: Color,
    ) {
        self.broadcast(
            whiteboard_id,
            WhiteboardMessage::UserJoined {
                user_id,
                username,
                color,
            },
        )
        .await;
    }

    pub async fn user_leave(&self, whiteboard_id: &Uuid, user_id: Uuid) {
        {
            let mut whiteboards = self.whiteboards.write().await;
            if let Some(whiteboard) = whiteboards.get_mut(whiteboard_id) {
                whiteboard.remove_cursor(&user_id);
                whiteboard.clear_selection(&user_id);
            }
        }

        self.broadcast(whiteboard_id, WhiteboardMessage::UserLeft { user_id })
            .await;
    }
}

impl Clone for WhiteboardState {
    fn clone(&self) -> Self {
        WhiteboardState {
            id: self.id,
            conversation_id: self.conversation_id,
            name: self.name.clone(),
            shapes: self.shapes.clone(),
            cursors: self.cursors.clone(),
            selections: self.selections.clone(),
            undo_stack: self.undo_stack.clone(),
            redo_stack: self.redo_stack.clone(),
            max_undo_history: self.max_undo_history,
            next_z_index: self.next_z_index,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/whiteboard/:id/ws", get(whiteboard_websocket))
        .route("/whiteboard/create/:conversation_id", get(create_whiteboard))
}

async fn create_whiteboard(
    State(state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
) -> impl IntoResponse {
    let manager = state
        .extensions
        .get::<WhiteboardManager>()
        .await
        .unwrap_or_else(|| Arc::new(WhiteboardManager::new()));

    let whiteboard_id = manager
        .create_whiteboard(conversation_id, "New Whiteboard".to_string())
        .await;

    axum::Json(serde_json::json!({
        "whiteboard_id": whiteboard_id,
        "conversation_id": conversation_id
    }))
}

async fn whiteboard_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(whiteboard_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_whiteboard_socket(socket, state, whiteboard_id))
}

async fn handle_whiteboard_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    whiteboard_id: Uuid,
) {
    let manager = state
        .extensions
        .get::<WhiteboardManager>()
        .await
        .unwrap_or_else(|| Arc::new(WhiteboardManager::new()));

    let receiver = match manager.subscribe(&whiteboard_id).await {
        Some(rx) => rx,
        None => {
            log::error!("Whiteboard {} not found", whiteboard_id);
            return;
        }
    };

    let (mut sender, mut ws_receiver) = socket.split();
    let mut broadcast_receiver = receiver;

    let user_id = Uuid::new_v4();
    let username = format!("User-{}", &user_id.to_string()[..8]);
    let user_color = Color {
        r: (user_id.as_u128() % 200 + 55) as u8,
        g: ((user_id.as_u128() >> 8) % 200 + 55) as u8,
        b: ((user_id.as_u128() >> 16) % 200 + 55) as u8,
        a: 1.0,
    };

    manager
        .user_join(&whiteboard_id, user_id, username.clone(), user_color.clone())
        .await;

    if let Some(whiteboard) = manager.get_whiteboard(&whiteboard_id).await {
        let sync_msg = whiteboard.to_sync_message();
        if let Ok(json) = serde_json::to_string(&sync_msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    let manager_clone = manager.clone();
    let whiteboard_id_clone = whiteboard_id;

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_receiver.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let receive_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<WhiteboardMessage>(&text) {
                        match msg {
                            WhiteboardMessage::Operation { operation, .. } => {
                                let _ = manager_clone
                                    .apply_operation(&whiteboard_id_clone, operation, user_id)
                                    .await;
                            }
                            WhiteboardMessage::CursorMove { mut cursor } => {
                                cursor.user_id = user_id;
                                cursor.username = username.clone();
                                cursor.color = user_color.clone();
                                cursor.last_update = Utc::now();
                                manager_clone.update_cursor(&whiteboard_id_clone, cursor).await;
                            }
                            WhiteboardMessage::Undo { .. } => {
                                if let Some(op) = manager_clone.undo(&whiteboard_id_clone, user_id).await {
                                    manager_clone
                                        .broadcast(
                                            &whiteboard_id_clone,
                                            WhiteboardMessage::Operation {
                                                user_id,
                                                operation: op,
                                            },
                                        )
                                        .await;
                                }
                            }
                            WhiteboardMessage::Redo { .. } => {
                                if let Some(op) = manager_clone.redo(&whiteboard_id_clone, user_id).await {
                                    manager_clone
                                        .broadcast(
                                            &whiteboard_id_clone,
                                            WhiteboardMessage::Operation {
                                                user_id,
                                                operation: op,
                                            },
                                        )
                                        .await;
                                }
                            }
                            WhiteboardMessage::Select { mut selection } => {
                                selection.user_id = user_id;
                                manager_clone
                                    .broadcast(
                                        &whiteboard_id_clone,
                                        WhiteboardMessage::Select { selection },
                                    )
                                    .await;
                            }
                            WhiteboardMessage::Deselect { .. } => {
                                manager_clone
                                    .broadcast(
                                        &whiteboard_id_clone,
                                        WhiteboardMessage::Deselect { user_id },
                                    )
                                    .await;
                            }
                            WhiteboardMessage::RequestSync => {
                                if let Some(whiteboard) =
                                    manager_clone.get_whiteboard(&whiteboard_id_clone).await
                                {
                                    manager_clone
                                        .broadcast(&whiteboard_id_clone, whiteboard.to_sync_message())
                                        .await;
                                }
                            }
                            WhiteboardMessage::Ping => {
                                manager_clone
                                    .broadcast(&whiteboard_id_clone, WhiteboardMessage::Pong)
                                    .await;
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    let _ = tokio::join!(send_task, receive_task);

    manager
        .user_leave(&whiteboard_id, user_id)
        .await;
}
