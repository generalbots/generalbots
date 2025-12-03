use crate::config::ConfigManager;
#[cfg(feature = "nvidia")]
use crate::nvidia;
#[cfg(feature = "nvidia")]
use crate::nvidia::get_system_metrics;
use crate::shared::models::schema::bots::dsl::*;
use crate::shared::state::AppState;
use diesel::prelude::*;
use std::sync::Arc;
use sysinfo::System;

pub struct StatusPanel {
    app_state: Arc<AppState>,
    last_update: std::time::Instant,
    cached_content: String,
    system: System,
}

impl std::fmt::Debug for StatusPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusPanel")
            .field("app_state", &"Arc<AppState>")
            .field("last_update", &self.last_update)
            .field("cached_content_len", &self.cached_content.len())
            .finish()
    }
}

impl StatusPanel {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            app_state,
            last_update: std::time::Instant::now(),
            cached_content: String::new(),
            system: System::new_all(),
        }
    }

    pub async fn update(&mut self) -> Result<(), std::io::Error> {
        self.system.refresh_all();
        // Force fresh metrics by using different token counts
        let _tokens = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 1000) as usize;
        #[cfg(feature = "nvidia")]
        let _system_metrics = nvidia::get_system_metrics().unwrap_or_default();
        self.cached_content = self.render(None);
        self.last_update = std::time::Instant::now();
        Ok(())
    }

    pub fn render(&mut self, selected_bot: Option<String>) -> String {
        let mut lines = Vec::new();

        // System metrics section
        lines.push("╔═══════════════════════════════════════╗".to_string());
        lines.push("║         SYSTEM METRICS                ║".to_string());
        lines.push("╚═══════════════════════════════════════╝".to_string());
        lines.push("".to_string());

        self.system.refresh_cpu_all();
        let cpu_usage = self.system.global_cpu_usage();
        let cpu_bar = Self::create_progress_bar(cpu_usage, 20);
        lines.push(format!(" CPU: {:5.1}% {}", cpu_usage, cpu_bar));
        #[cfg(feature = "nvidia")]
        {
            let system_metrics = get_system_metrics().unwrap_or_default();
            if let Some(gpu_usage) = system_metrics.gpu_usage {
                let gpu_bar = Self::create_progress_bar(gpu_usage, 20);
                lines.push(format!(" GPU: {:5.1}% {}", gpu_usage, gpu_bar));
            } else {
                lines.push(" GPU: Not available".to_string());
            }
        }
        #[cfg(not(feature = "nvidia"))]
        {
            lines.push(" GPU: Feature not enabled".to_string());
        }

        let total_mem = self.system.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        let used_mem = self.system.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        let mem_percentage = (used_mem / total_mem) * 100.0;
        let mem_bar = Self::create_progress_bar(mem_percentage, 20);
        lines.push(format!(
            " MEM: {:5.1}% {} ({:.1}/{:.1} GB)",
            mem_percentage, mem_bar, used_mem, total_mem
        ));

        // Components status section
        lines.push("".to_string());
        lines.push("╔═══════════════════════════════════════╗".to_string());
        lines.push("║         COMPONENTS STATUS             ║".to_string());
        lines.push("╚═══════════════════════════════════════╝".to_string());
        lines.push("".to_string());

        let components = vec![
            ("Tables", "postgres", "5432"),
            ("Cache", "valkey-server", "6379"),
            ("Drive", "minio", "9000"),
            ("LLM", "llama-server", "8081"),
        ];

        for (comp_name, process, port) in components {
            let status = if Self::check_component_running(process) {
                format!("🟢 ONLINE  [Port: {}]", port)
            } else {
                "🔴 OFFLINE".to_string()
            };
            lines.push(format!(" {:<10} {}", comp_name, status));
        }

        // Active bots section
        lines.push("".to_string());
        lines.push("╔═══════════════════════════════════════╗".to_string());
        lines.push("║         ACTIVE BOTS                   ║".to_string());
        lines.push("╚═══════════════════════════════════════╝".to_string());
        lines.push("".to_string());

        if let Ok(mut conn) = self.app_state.conn.get() {
            match bots
                .filter(is_active.eq(true))
                .select((name, id))
                .load::<(String, uuid::Uuid)>(&mut *conn)
            {
                Ok(bot_list) => {
                    if bot_list.is_empty() {
                        lines.push(" No active bots".to_string());
                    } else {
                        for (bot_name, bot_id) in bot_list {
                            let marker = if let Some(ref selected) = selected_bot {
                                if selected == &bot_name {
                                    "►"
                                } else {
                                    " "
                                }
                            } else {
                                " "
                            };
                            lines.push(format!(" {} 🤖 {}", marker, bot_name));

                            if let Some(ref selected) = selected_bot {
                                if selected == &bot_name {
                                    lines.push("".to_string());
                                    lines.push(" ┌─ Bot Configuration ─────────┐".to_string());
                                    let config_manager =
                                        ConfigManager::new(self.app_state.conn.clone());
                                    let llm_model = config_manager
                                        .get_config(&bot_id, "llm-model", None)
                                        .unwrap_or_else(|_| "N/A".to_string());
                                    lines.push(format!("  Model: {}", llm_model));
                                    let ctx_size = config_manager
                                        .get_config(&bot_id, "llm-server-ctx-size", None)
                                        .unwrap_or_else(|_| "N/A".to_string());
                                    lines.push(format!("  Context: {}", ctx_size));
                                    let temp = config_manager
                                        .get_config(&bot_id, "llm-temperature", None)
                                        .unwrap_or_else(|_| "N/A".to_string());
                                    lines.push(format!("  Temp: {}", temp));
                                    lines.push(" └─────────────────────────────┘".to_string());
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    lines.push(" Error loading bots".to_string());
                }
            }
        } else {
            lines.push(" Database locked".to_string());
        }

        // Sessions section
        lines.push("".to_string());
        lines.push("╔═══════════════════════════════════════╗".to_string());
        lines.push("║         SESSIONS                      ║".to_string());
        lines.push("╚═══════════════════════════════════════╝".to_string());

        let session_count = self
            .app_state
            .response_channels
            .try_lock()
            .map(|channels| channels.len())
            .unwrap_or(0);
        lines.push(format!(" Active Sessions: {}", session_count));

        lines.join("\n")
    }

    fn create_progress_bar(percentage: f32, width: usize) -> String {
        let filled = (percentage / 100.0 * width as f32).round() as usize;
        let empty = width.saturating_sub(filled);
        let filled_chars = "█".repeat(filled);
        let empty_chars = "░".repeat(empty);
        format!("[{}{}]", filled_chars, empty_chars)
    }

    pub fn check_component_running(process_name: &str) -> bool {
        std::process::Command::new("pgrep")
            .arg("-f")
            .arg(process_name)
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }
}
