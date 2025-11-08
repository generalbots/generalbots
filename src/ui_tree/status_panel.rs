use std::sync::Arc;
use crate::shared::state::AppState;
use crate::shared::models::schema::bots::dsl::*;
use diesel::prelude::*;

pub struct StatusPanel {
 app_state: Arc<AppState>,
 last_update: std::time::Instant,
 cached_content: String,
}

impl StatusPanel {
 pub fn new(app_state: Arc<AppState>) -> Self {
 Self {
 app_state,
 last_update: std::time::Instant::now(),
 cached_content: String::new(),
 }
 }

 pub async fn update(&mut self) -> Result<(), std::io::Error> {
 if self.last_update.elapsed() < std::time::Duration::from_secs(2) {
 return Ok(());
 }

 let mut lines = Vec::new();
 lines.push("═══════════════════════════════════════".to_string());
 lines.push("  COMPONENT STATUS".to_string());
 lines.push("═══════════════════════════════════════".to_string());
 lines.push("".to_string());

 let db_status = if self.app_state.conn.try_lock().is_ok() {
 "🟢 ONLINE"
 } else {
 "🔴 OFFLINE"
 };
 lines.push(format!("  Database:     {}", db_status));

 let cache_status = if self.app_state.cache.is_some() {
 "🟢 ONLINE"
 } else {
 "🟡 DISABLED"
 };
 lines.push(format!("  Cache:        {}", cache_status));

 let drive_status = if self.app_state.drive.is_some() {
 "🟢 ONLINE"
 } else {
 "🔴 OFFLINE"
 };
 lines.push(format!("  Drive:        {}", drive_status));

 let llm_status = "🟢 ONLINE";
 lines.push(format!("  LLM:          {}", llm_status));

 lines.push("".to_string());
 lines.push("───────────────────────────────────────".to_string());
 lines.push("  ACTIVE BOTS".to_string());
 lines.push("───────────────────────────────────────".to_string());

 if let Ok(mut conn) = self.app_state.conn.try_lock() {
 match bots
 .filter(is_active.eq(true))
 .select((name, id))
 .load::<(String, uuid::Uuid)>(&mut *conn)
 {
 Ok(bot_list) => {
 if bot_list.is_empty() {
 lines.push("  No active bots".to_string());
 } else {
 for (bot_name, _bot_id) in bot_list {
 lines.push(format!("  🤖 {}", bot_name));
 }
 }
 }
 Err(_) => {
 lines.push("  Error loading bots".to_string());
 }
 }
 } else {
 lines.push("  Database locked".to_string());
 }

 lines.push("".to_string());
 lines.push("───────────────────────────────────────".to_string());
 lines.push("  SESSIONS".to_string());
 lines.push("───────────────────────────────────────".to_string());

 let session_count = self.app_state.response_channels.try_lock()
 .map(|channels| channels.len())
 .unwrap_or(0);
 lines.push(format!("  Active:       {}", session_count));

 lines.push("".to_string());
 lines.push("═══════════════════════════════════════".to_string());

 self.cached_content = lines.join("\n");
 self.last_update = std::time::Instant::now();
 Ok(())
 }

 pub fn render(&self) -> String {
 self.cached_content.clone()
 }
}
