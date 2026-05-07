use crate::types::{AutoTaskState, UserSession, generate_create_table_sql};
use crate::task_manifest::{
    FieldDefinition as ManifestField, TableDefinition as ManifestTable, TaskManifest,
};
use crate::app_logs::log_generator_info;
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pages: Vec<GeneratedFile>,
    pub tables: Vec<ManifestTable>,
    pub tools: Vec<GeneratedFile>,
    pub schedulers: Vec<GeneratedFile>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub filename: String,
    pub content: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Html,
    Css,
    Js,
    Bas,
    Json,
}

pub struct AppGenerator {
    state: Arc<dyn AutoTaskState>,
    task_id: Option<String>,
    manifest: Option<TaskManifest>,
}

impl AppGenerator {
    pub fn new(state: Arc<dyn AutoTaskState>) -> Self {
        Self { state, task_id: None, manifest: None }
    }

    pub fn with_task_id(state: Arc<dyn AutoTaskState>, task_id: impl Into<String>) -> Self {
        Self { state, task_id: Some(task_id.into()), manifest: None }
    }

    pub async fn generate(
        &self,
        intent: &str,
        session: &UserSession,
    ) -> Result<GeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = self.task_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        info!("AppGenerator generating app for task {task_id}: {}", &intent[..intent.len().min(80)]);

        self.state.emit_task_started(&task_id, "Generating application", 3);

        let app_name = self.extract_app_name(intent);
        let tables = self.infer_tables(intent);
        let pages = self.generate_pages(&app_name, &tables);
        let tools = self.generate_tools(&app_name, intent);
        let schedulers = self.generate_schedulers(&app_name, intent);

        self.state.emit_activity(&task_id, "generating_pages", "Generating pages", 1, 3,
            crate::types::AgentActivity::new("generating_pages"));

        self.state.emit_activity(&task_id, "generating_tables", "Creating database tables", 2, 3,
            crate::types::AgentActivity::new("generating_tables"));

        self.sync_tables(&tables, session.bot_id)?;

        self.state.emit_activity(&task_id, "completed", "Application generated", 3, 3,
            crate::types::AgentActivity::new("completed"));

        let app = GeneratedApp {
            id: task_id.clone(),
            name: app_name,
            description: intent.to_string(),
            pages,
            tables,
            tools,
            schedulers,
            created_at: Utc::now(),
        };

        log_generator_info(&task_id, &format!("app_generated: {}", app.name));

        Ok(app)
    }

    pub fn manifest(&self) -> Option<&TaskManifest> {
        self.manifest.as_ref()
    }

    fn extract_app_name(&self, intent: &str) -> String {
        let lower = intent.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().take(3).collect();
        words.join("-").replace([' ', '.', ',', '!'], "")
    }

    fn infer_tables(&self, intent: &str) -> Vec<ManifestTable> {
        let lower = intent.to_lowercase();
        let mut tables = Vec::new();

        if lower.contains("crm") || lower.contains("customer") {
            tables.push(ManifestTable {
                name: "customers".to_string(),
                fields: vec![
                    ManifestField { name: "id".to_string(), field_type: "uuid".to_string(), nullable: false, is_key: true, is_nullable: false, default_value: None, reference_table: None },
                    ManifestField { name: "name".to_string(), field_type: "string".to_string(), nullable: false, is_key: false, is_nullable: false, default_value: None, reference_table: None },
                    ManifestField { name: "email".to_string(), field_type: "string".to_string(), nullable: true, is_key: false, is_nullable: true, default_value: None, reference_table: None },
                    ManifestField { name: "created_at".to_string(), field_type: "datetime".to_string(), nullable: false, is_key: false, is_nullable: false, default_value: Some("NOW()".to_string()), reference_table: None },
                ],
            });
        }

        if lower.contains("inventory") || lower.contains("stock") || lower.contains("product") {
            tables.push(ManifestTable {
                name: "products".to_string(),
                fields: vec![
                    ManifestField { name: "id".to_string(), field_type: "uuid".to_string(), nullable: false, is_key: true, is_nullable: false, default_value: None, reference_table: None },
                    ManifestField { name: "name".to_string(), field_type: "string".to_string(), nullable: false, is_key: false, is_nullable: false, default_value: None, reference_table: None },
                    ManifestField { name: "quantity".to_string(), field_type: "integer".to_string(), nullable: false, is_key: false, is_nullable: false, default_value: Some("0".to_string()), reference_table: None },
                    ManifestField { name: "price".to_string(), field_type: "decimal".to_string(), nullable: true, is_key: false, is_nullable: true, default_value: None, reference_table: None },
                ],
            });
        }

        if tables.is_empty() {
            tables.push(ManifestTable {
                name: "items".to_string(),
                fields: vec![
                    ManifestField { name: "id".to_string(), field_type: "uuid".to_string(), nullable: false, is_key: true, is_nullable: false, default_value: None, reference_table: None },
                    ManifestField { name: "name".to_string(), field_type: "string".to_string(), nullable: false, is_key: false, is_nullable: false, default_value: None, reference_table: None },
                    ManifestField { name: "created_at".to_string(), field_type: "datetime".to_string(), nullable: false, is_key: false, is_nullable: false, default_value: Some("NOW()".to_string()), reference_table: None },
                ],
            });
        }

        tables
    }

    fn generate_pages(&self, app_name: &str, tables: &[ManifestTable]) -> Vec<GeneratedFile> {
        let mut pages = Vec::new();
        for table in tables {
            let html = self.generate_list_html(app_name, table);
            pages.push(GeneratedFile {
                filename: format!("{}.html", table.name),
                content: html,
                file_type: FileType::Html,
            });

            let form_html = self.generate_form_html(app_name, table);
            pages.push(GeneratedFile {
                filename: format!("{}-form.html", table.name),
                content: form_html,
                file_type: FileType::Html,
            });
        }
        pages
    }

    fn generate_list_html(&self, app_name: &str, table: &ManifestTable) -> String {
        let headers: Vec<String> = table.fields.iter()
            .filter(|f| !f.is_key || f.name != "id")
            .map(|f| f.name.clone())
            .collect();
        let header_row = headers.iter().map(|h| format!("            <th>{h}</th>")).collect::<Vec<_>>().join("\n");
        let table_name = &table.name;
        let mut html = String::from("<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\"><title>");
        html.push_str(app_name);
        html.push_str(" - ");
        html.push_str(table_name);
        html.push_str("</title>\n<link rel=\"stylesheet\" href=\"css/style.css\"></head>\n<body>\n<div class=\"container\">\n  <h1>");
        html.push_str(app_name);
        html.push_str(" - ");
        html.push_str(table_name);
        html.push_str("</h1>\n  <div id=\"list-container\" hx-get=\"/api/");
        html.push_str(table_name);
        html.push_str("\" hx-trigger=\"load\">\n    <table>\n      <thead><tr>\n");
        html.push_str(&header_row);
        html.push_str("\n      </tr></thead>\n      <tbody></tbody>\n    </table>\n  </div>\n</div>\n<script src=\"js/vendor/htmx.min.js\"></script>\n</body>\n</html>");
        html
    }

    fn generate_form_html(&self, app_name: &str, table: &ManifestTable) -> String {
        let fields: Vec<String> = table.fields.iter()
            .filter(|f| f.name != "id" && !f.name.ends_with("_at"))
            .map(|f| {
                let mut s = String::from("      <div class=\"form-group\">\n        <label for=\"");
                s.push_str(&f.name);
                s.push_str("\">");
                s.push_str(&f.name);
                s.push_str("</label>\n        <input type=\"text\" id=\"");
                s.push_str(&f.name);
                s.push_str("\" name=\"");
                s.push_str(&f.name);
                s.push_str("\" />\n      </div>");
                s
            })
            .collect();
        let fields_html = fields.join("\n");
        let table_name = &table.name;
        let mut html = String::from("<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\"><title>");
        html.push_str(app_name);
        html.push_str(" - New ");
        html.push_str(table_name);
        html.push_str("</title>\n<link rel=\"stylesheet\" href=\"css/style.css\"></head>\n<body>\n<div class=\"container\">\n  <h1>");
        html.push_str(app_name);
        html.push_str(" - New ");
        html.push_str(table_name);
        html.push_str("</h1>\n  <form hx-post=\"/api/");
        html.push_str(table_name);
        html.push_str("\" hx-target=\"#result\">\n");
        html.push_str(&fields_html);
        html.push_str("\n    <button type=\"submit\">Save</button>\n  </form>\n  <div id=\"result\"></div>\n</div>\n<script src=\"js/vendor/htmx.min.js\"></script>\n</body>\n</html>");
        html
    }

    fn generate_tools(&self, app_name: &str, intent: &str) -> Vec<GeneratedFile> {
        let tool_name = app_name.to_lowercase().replace(' ', "-");
        let _ = intent;
        vec![GeneratedFile {
            filename: format!("{tool_name}.bas"),
            content: format!("' Tool: {app_name}\nTALK \"Running {app_name}\"\n"),
            file_type: FileType::Bas,
        }]
    }

    fn generate_schedulers(&self, app_name: &str, intent: &str) -> Vec<GeneratedFile> {
        let _ = (app_name, intent);
        Vec::new()
    }

    fn sync_tables(
        &self,
        tables: &[ManifestTable],
        bot_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for table in tables {
            let sql = generate_create_table_sql(table, "postgres");
            if !sql.is_empty() {
                info!("Creating table {} for bot {}", table.name, bot_id);
                match self.state.db_pool().get() {
                    Ok(mut conn) => {
                        if let Err(e) = diesel::sql_query(&sql).execute(&mut conn) {
                            error!("Failed to create table {}: {e}", table.name);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get DB connection for table creation: {e}");
                    }
                }
            }
        }
        Ok(())
    }
}
