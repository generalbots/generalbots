use std::sync::Arc;
use crate::shared::state::AppState;
use crate::shared::models::schema::bots::dsl::*;
use crate::nvidia;
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

 // Get system metrics
 let system_metrics = match nvidia::get_system_metrics(0, 0) {
     Ok(metrics) => metrics,
     Err(_) => nvidia::SystemMetrics::default(),
 };

 // Add system metrics with progress bars
 lines.push("".to_string());
 lines.push("───────────────────────────────────────".to_string());
 lines.push("  SYSTEM METRICS".to_string());
 lines.push("───────────────────────────────────────".to_string());
 
 // CPU usage with progress bar
 let cpu_bar = Self::create_progress_bar(system_metrics.cpu_usage, 20);
 lines.push(format!("  CPU:          {:5.1}% {}", system_metrics.cpu_usage, cpu_bar));
 
 // GPU usage with progress bar (if available)
 if let Some(gpu_usage) = system_metrics.gpu_usage {
     let gpu_bar = Self::create_progress_bar(gpu_usage, 20);
     lines.push(format!("  GPU:          {:5.1}% {}", gpu_usage, gpu_bar));
 } else {
     lines.push("  GPU:          Not available".to_string());
 }

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

 /// Creates a visual progress bar for percentage values
 fn create_progress_bar(percentage: f32, width: usize) -> String {
     let filled = (percentage / 100.0 * width as f32).round() as usize;
     let empty = width.saturating_sub(filled);
     
     let filled_chars = "█".repeat(filled);
     let empty_chars = "░".repeat(empty);
     
     format!("[{}{}]", filled_chars, empty_chars)
 }
}
