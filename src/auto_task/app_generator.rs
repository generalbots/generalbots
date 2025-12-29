use crate::auto_task::app_logs::{log_generator_error, log_generator_info};
use crate::basic::keywords::table_definition::{
    generate_create_table_sql, FieldDefinition, TableDefinition,
};
use crate::core::config::ConfigManager;
use crate::core::shared::get_content_type;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pages: Vec<GeneratedFile>,
    pub tables: Vec<TableDefinition>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPage {
    pub filename: String,
    pub title: String,
    pub page_type: PageType,
    pub content: String,
    pub route: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageType {
    List,
    Form,
    Detail,
    Dashboard,
}

impl std::fmt::Display for PageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List => write!(f, "list"),
            Self::Form => write!(f, "form"),
            Self::Detail => write!(f, "detail"),
            Self::Dashboard => write!(f, "dashboard"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScript {
    pub name: String,
    pub filename: String,
    pub script_type: ScriptType,
    pub content: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Tool,
    Scheduler,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStructure {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub tables: Vec<TableDefinition>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub tables_created: usize,
    pub fields_added: usize,
    pub migrations_applied: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmGeneratedApp {
    name: String,
    description: String,
    #[serde(default)]
    _domain: String,
    tables: Vec<LlmTable>,
    files: Vec<LlmFile>,
    tools: Option<Vec<LlmFile>>,
    schedulers: Option<Vec<LlmFile>>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmTable {
    name: String,
    fields: Vec<LlmField>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmField {
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    nullable: Option<bool>,
    reference: Option<String>,
    default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmFile {
    filename: String,
    content: String,
    #[serde(rename = "type", default)]
    _file_type: Option<String>,
}

pub struct AppGenerator {
    state: Arc<AppState>,
}

impl AppGenerator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn generate_app(
        &self,
        intent: &str,
        session: &UserSession,
    ) -> Result<GeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Generating app from intent: {}",
            &intent[..intent.len().min(100)]
        );

        log_generator_info(
            "pending",
            &format!(
                "Starting app generation: {}",
                &intent[..intent.len().min(50)]
            ),
        );

        let llm_app = match self.generate_complete_app_with_llm(intent, session.bot_id).await {
            Ok(app) => {
                log_generator_info(
                    &app.name,
                    "LLM successfully generated app structure and files",
                );
                app
            }
            Err(e) => {
                log_generator_error("unknown", "LLM app generation failed", &e.to_string());
                return Err(e);
            }
        };

        let tables = Self::convert_llm_tables(&llm_app.tables);

        if !tables.is_empty() {
            let tables_bas_content = Self::generate_table_definitions(&tables)?;
            if let Err(e) = self.append_to_tables_bas(session.bot_id, &tables_bas_content) {
                log_generator_error(
                    &llm_app.name,
                    "Failed to append to tables.bas",
                    &e.to_string(),
                );
            }

            match self.sync_tables_to_database(&tables) {
                Ok(result) => {
                    log_generator_info(
                        &llm_app.name,
                        &format!(
                            "Tables synced: {} created, {} fields",
                            result.tables_created, result.fields_added
                        ),
                    );
                }
                Err(e) => {
                    log_generator_error(&llm_app.name, "Failed to sync tables", &e.to_string());
                }
            }
        }

        let bot_name = self.get_bot_name(session.bot_id)?;
        let bucket_name = format!("{}.gbai", bot_name.to_lowercase());
        let drive_app_path = format!(".gbdrive/apps/{}", llm_app.name);

        let mut pages = Vec::new();
        for file in &llm_app.files {
            let drive_path = format!("{}/{}", drive_app_path, file.filename);
            if let Err(e) = self
                .write_to_drive(&bucket_name, &drive_path, &file.content)
                .await
            {
                log_generator_error(
                    &llm_app.name,
                    &format!("Failed to write {}", file.filename),
                    &e.to_string(),
                );
            }

            let file_type = Self::detect_file_type(&file.filename);
            pages.push(GeneratedFile {
                filename: file.filename.clone(),
                content: file.content.clone(),
                file_type,
            });
        }

        let designer_js = Self::generate_designer_js(&llm_app.name);
        self.write_to_drive(
            &bucket_name,
            &format!("{}/designer.js", drive_app_path),
            &designer_js,
        )
        .await?;

        let mut tools = Vec::new();
        if let Some(llm_tools) = &llm_app.tools {
            for tool in llm_tools {
                let tool_path = format!(".gbdialog/tools/{}", tool.filename);
                if let Err(e) = self
                    .write_to_drive(&bucket_name, &tool_path, &tool.content)
                    .await
                {
                    log_generator_error(
                        &llm_app.name,
                        &format!("Failed to write tool {}", tool.filename),
                        &e.to_string(),
                    );
                }
                tools.push(GeneratedFile {
                    filename: tool.filename.clone(),
                    content: tool.content.clone(),
                    file_type: FileType::Bas,
                });
            }
        }

        let mut schedulers = Vec::new();
        if let Some(llm_schedulers) = &llm_app.schedulers {
            for scheduler in llm_schedulers {
                let scheduler_path = format!(".gbdialog/schedulers/{}", scheduler.filename);
                if let Err(e) = self
                    .write_to_drive(&bucket_name, &scheduler_path, &scheduler.content)
                    .await
                {
                    log_generator_error(
                        &llm_app.name,
                        &format!("Failed to write scheduler {}", scheduler.filename),
                        &e.to_string(),
                    );
                }
                schedulers.push(GeneratedFile {
                    filename: scheduler.filename.clone(),
                    content: scheduler.content.clone(),
                    file_type: FileType::Bas,
                });
            }
        }

        self.sync_app_to_site_root(&bucket_name, &llm_app.name, session.bot_id)
            .await?;

        log_generator_info(
            &llm_app.name,
            &format!(
                "App generated: {} files, {} tables, {} tools",
                pages.len(),
                tables.len(),
                tools.len()
            ),
        );

        info!(
            "App '{}' generated in s3://{}/{}",
            llm_app.name, bucket_name, drive_app_path
        );

        Ok(GeneratedApp {
            id: Uuid::new_v4().to_string(),
            name: llm_app.name,
            description: llm_app.description,
            pages,
            tables,
            tools,
            schedulers,
            created_at: Utc::now(),
        })
    }

    fn get_platform_prompt() -> &'static str {
        r##"
GENERAL BOTS PLATFORM - APP GENERATION

You are an expert full-stack developer generating complete applications for General Bots platform.

=== AVAILABLE APIs ===

DATABASE (/api/db/):
- GET /api/db/{table} - List records (query: limit, offset, order_by, order_dir, search, field=value)
- GET /api/db/{table}/{id} - Get single record
- GET /api/db/{table}/count - Count records
- POST /api/db/{table} - Create record (JSON body)
- PUT /api/db/{table}/{id} - Update record
- DELETE /api/db/{table}/{id} - Delete record

DRIVE (/api/drive/):
- GET /api/drive/list?path=/folder - List files
- GET /api/drive/download?path=/file - Download
- POST /api/drive/upload - Upload (multipart)
- DELETE /api/drive/delete?path=/file - Delete

COMMUNICATION:
- POST /api/mail/send - {"to", "subject", "body"}
- POST /api/whatsapp/send - {"to", "message"}
- POST /api/llm/generate - {"prompt", "max_tokens"}

=== HTMX REQUIREMENTS ===

All HTML pages MUST use HTMX exclusively. NO fetch(), NO XMLHttpRequest, NO inline onclick.

Key attributes:
- hx-get, hx-post, hx-put, hx-delete - HTTP methods
- hx-target="#id" - Response destination
- hx-swap="innerHTML|outerHTML|beforeend|delete" - Insert method
- hx-trigger="click|submit|load|every 5s|keyup changed delay:300ms"
- hx-indicator="#spinner" - Loading indicator
- hx-confirm="Message?" - Confirmation
- hx-vals='{"key":"value"}' - Extra values
- hx-headers='{"X-Custom":"value"}' - Headers

=== REQUIRED HTML STRUCTURE ===

Every HTML file must include:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Title</title>
    <link rel="stylesheet" href="styles.css">
    <script src="/js/vendor/htmx.min.js"></script>
    <script src="/api/app-logs/logger.js" defer></script>
    <script src="designer.js" defer></script>
</head>
<body data-app-name="APP_NAME_HERE">
    <!-- Content -->
</body>
</html>
```

=== BASIC SCRIPTS (.bas) ===

Tools (triggered by chat):
```
HEAR "keyword1", "keyword2"
    result = GET FROM "table" WHERE field = value
    TALK "Response: " + result
END HEAR
```

Schedulers (cron-based):
```
SET SCHEDULE "0 9 * * *"
    data = GET FROM "table"
    SEND MAIL TO "email" WITH SUBJECT "Report" BODY data
END SCHEDULE
```

BASIC Keywords:
- TALK "message" - Send message
- ASK "question" - Get input
- GET FROM "table" WHERE field=val - Query
- SAVE TO "table" WITH field1, field2 - Insert
- SEND MAIL TO "x" WITH SUBJECT "y" BODY "z"
- result = LLM "prompt" - AI generation

=== FIELD TYPES ===
guid, string, text, integer, decimal, boolean, date, datetime, json

=== GENERATION RULES ===

1. Generate COMPLETE, WORKING code - no placeholders, no "...", no "add more here"
2. Use semantic HTML5 (header, main, nav, section, article, footer)
3. Include loading states (hx-indicator)
4. Include error handling
5. Make it beautiful, modern, responsive
6. Include dark mode support in CSS
7. Tables should have id, created_at, updated_at fields
8. Forms must validate required fields
9. Lists must have search, pagination, edit/delete actions
"##
    }

    async fn generate_complete_app_with_llm(
        &self,
        intent: &str,
        bot_id: Uuid,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let platform = Self::get_platform_prompt();

        let prompt = format!(
            r#"{platform}

=== USER REQUEST ===
"{intent}"

=== YOUR TASK ===
Generate a complete application based on the user's request.

Respond with a single JSON object:
{{
    "name": "app-name-lowercase-dashes",
    "description": "What this app does",
    "domain": "healthcare|sales|inventory|booking|utility|etc",
    "tables": [
        {{
            "name": "table_name",
            "fields": [
                {{"name": "id", "type": "guid", "nullable": false}},
                {{"name": "created_at", "type": "datetime", "nullable": false, "default": "now()"}},
                {{"name": "updated_at", "type": "datetime", "nullable": false, "default": "now()"}},
                {{"name": "field_name", "type": "string", "nullable": true, "reference": null}}
            ]
        }}
    ],
    "files": [
        {{"filename": "index.html", "content": "<!DOCTYPE html>...complete HTML..."}},
        {{"filename": "styles.css", "content": ":root {{...}} body {{...}} ...complete CSS..."}},
        {{"filename": "table_name.html", "content": "<!DOCTYPE html>...list page..."}},
        {{"filename": "table_name_form.html", "content": "<!DOCTYPE html>...form page..."}}
    ],
    "tools": [
        {{"filename": "app_helper.bas", "content": "HEAR \"help\"\n    TALK \"I can help with...\"\nEND HEAR"}}
    ],
    "schedulers": [
        {{"filename": "daily_report.bas", "content": "SET SCHEDULE \"0 9 * * *\"\n    ...\nEND SCHEDULE"}}
    ]
}}

IMPORTANT:
- For simple utilities (calculator, timer, converter): tables can be empty [], focus on files
- For data apps (CRM, inventory): design proper tables and CRUD pages
- Generate ALL files completely - no shortcuts
- CSS must be comprehensive with variables, responsive design, dark mode
- Every HTML page needs proper structure with all required scripts
- Replace APP_NAME_HERE with actual app name in data-app-name attribute

Respond with valid JSON only."#
        );

        let response = self.call_llm(&prompt, bot_id).await?;
        Self::parse_llm_app_response(&response)
    }

    fn parse_llm_app_response(
        response: &str,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        match serde_json::from_str::<LlmGeneratedApp>(cleaned) {
            Ok(app) => {
                if app.files.is_empty() {
                    return Err("LLM generated no files".into());
                }
                Ok(app)
            }
            Err(e) => {
                error!("Failed to parse LLM response: {}", e);
                error!("Response was: {}", &response[..response.len().min(500)]);
                Err(format!("Failed to parse LLM response: {}", e).into())
            }
        }
    }

    fn convert_llm_tables(llm_tables: &[LlmTable]) -> Vec<TableDefinition> {
        llm_tables
            .iter()
            .map(|t| {
                let fields = t
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| FieldDefinition {
                        name: f.name.clone(),
                        field_type: f.field_type.clone(),
                        is_key: f.name == "id",
                        is_nullable: f.nullable.unwrap_or(true),
                        reference_table: f.reference.clone(),
                        default_value: f.default.clone(),
                        field_order: i as i32,
                        ..Default::default()
                    })
                    .collect();

                TableDefinition {
                    name: t.name.clone(),
                    connection_name: "default".to_string(),
                    fields,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn detect_file_type(filename: &str) -> FileType {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "css" => FileType::Css,
            "js" => FileType::Js,
            "bas" => FileType::Bas,
            "json" => FileType::Json,
            _ => FileType::Html,
        }
    }

    async fn call_llm(
        &self,
        prompt: &str,
        bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "llm")]
        {
            // Get model and key from bot configuration
            let config_manager = ConfigManager::new(self.state.conn.clone());
            let model = config_manager
                .get_config(&bot_id, "llm-model", None)
                .unwrap_or_else(|_| {
                    config_manager
                        .get_config(&Uuid::nil(), "llm-model", None)
                        .unwrap_or_else(|_| "gpt-4".to_string())
                });
            let key = config_manager
                .get_config(&bot_id, "llm-key", None)
                .unwrap_or_else(|_| {
                    config_manager
                        .get_config(&Uuid::nil(), "llm-key", None)
                        .unwrap_or_default()
                });

            let llm_config = serde_json::json!({
                "temperature": 0.7,
                "max_tokens": 16000
            });

            match self
                .state
                .llm_provider
                .generate(prompt, &llm_config, &model, &key)
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("LLM call failed: {}", e);
                    return Err(e);
                }
            }
        }

        #[cfg(not(feature = "llm"))]
        {
            Err("LLM feature not enabled. App generation requires LLM.".into())
        }
    }

    fn generate_table_definitions(
        tables: &[TableDefinition],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use std::fmt::Write;
        let mut output = String::new();

        for table in tables {
            let _ = writeln!(output, "\nTABLE {}", table.name);

            for field in &table.fields {
                let mut line = format!("    {} AS {}", field.name, field.field_type.to_uppercase());
                if field.is_key {
                    line.push_str(" KEY");
                }
                if !field.is_nullable {
                    line.push_str(" REQUIRED");
                }
                if let Some(ref default) = field.default_value {
                    let _ = write!(line, " DEFAULT {}", default);
                }
                if let Some(ref refs) = field.reference_table {
                    let _ = write!(line, " REFERENCES {}", refs);
                }
                let _ = writeln!(output, "{}", line);
            }

            let _ = writeln!(output, "END TABLE\n");
        }

        Ok(output)
    }

    fn append_to_tables_bas(
        &self,
        bot_id: Uuid,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bot_name = self.get_bot_name(bot_id)?;
        let bucket = format!("{}.gbai", bot_name.to_lowercase());
        let path = ".gbdata/tables.bas";

        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct ContentRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            content: String,
        }

        let existing: Option<String> =
            sql_query("SELECT content FROM drive_files WHERE bucket = $1 AND path = $2 LIMIT 1")
                .bind::<diesel::sql_types::Text, _>(&bucket)
                .bind::<diesel::sql_types::Text, _>(path)
                .load::<ContentRow>(&mut conn)
                .ok()
                .and_then(|rows| rows.into_iter().next().map(|r| r.content));

        let new_content = match existing {
            Some(existing_content) => format!("{}\n{}", existing_content, content),
            None => content.to_string(),
        };

        sql_query(
            "INSERT INTO drive_files (id, bucket, path, content, content_type, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'text/plain', NOW(), NOW())
             ON CONFLICT (bucket, path) DO UPDATE SET content = $4, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Text, _>(&bucket)
        .bind::<diesel::sql_types::Text, _>(path)
        .bind::<diesel::sql_types::Text, _>(&new_content)
        .execute(&mut conn)?;

        Ok(())
    }

    fn get_bot_name(
        &self,
        bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct BotRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }

        let result: Vec<BotRow> = sql_query("SELECT name FROM bots WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load(&mut conn)?;

        result
            .into_iter()
            .next()
            .map(|r| r.name)
            .ok_or_else(|| format!("Bot not found: {}", bot_id).into())
    }

    async fn write_to_drive(
        &self,
        bucket: &str,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref s3) = self.state.s3_client {
            let body = ByteStream::from(content.as_bytes().to_vec());
            let content_type = get_content_type(path);

            s3.put_object()
                .bucket(bucket)
                .key(path)
                .body(body)
                .content_type(content_type)
                .send()
                .await?;

            trace!("Wrote to S3: s3://{}/{}", bucket, path);
        } else {
            self.write_to_db_fallback(bucket, path, content)?;
        }

        Ok(())
    }

    fn write_to_db_fallback(
        &self,
        bucket: &str,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;
        let content_type = get_content_type(path);

        sql_query(
            "INSERT INTO drive_files (id, bucket, path, content, content_type, size, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
             ON CONFLICT (bucket, path) DO UPDATE SET
             content = EXCLUDED.content,
             content_type = EXCLUDED.content_type,
             size = EXCLUDED.size,
             updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Text, _>(bucket)
        .bind::<diesel::sql_types::Text, _>(path)
        .bind::<diesel::sql_types::Text, _>(content)
        .bind::<diesel::sql_types::Text, _>(content_type)
        .bind::<diesel::sql_types::BigInt, _>(content.len() as i64)
        .execute(&mut conn)?;

        trace!("Wrote to DB: {}/{}", bucket, path);
        Ok(())
    }

    fn sync_tables_to_database(
        &self,
        tables: &[TableDefinition],
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut tables_created = 0;
        let mut fields_added = 0;

        let mut conn = self.state.conn.get()?;

        for table in tables {
            let create_sql = generate_create_table_sql(table, "postgres");

            match sql_query(&create_sql).execute(&mut conn) {
                Ok(_) => {
                    tables_created += 1;
                    fields_added += table.fields.len();
                    info!("Created table: {}", table.name);
                }
                Err(e) => {
                    warn!("Table {} may already exist: {}", table.name, e);
                }
            }
        }

        Ok(SyncResult {
            tables_created,
            fields_added,
            migrations_applied: tables_created,
        })
    }

    async fn sync_app_to_site_root(
        &self,
        bucket: &str,
        app_name: &str,
        bot_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let source_path = format!(".gbdrive/apps/{}", app_name);
        let site_path = Self::get_site_path(bot_id);

        if let Some(ref s3) = self.state.s3_client {
            let list_result = s3
                .list_objects_v2()
                .bucket(bucket)
                .prefix(&source_path)
                .send()
                .await?;

            if let Some(contents) = list_result.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        let relative_path =
                            key.trim_start_matches(&source_path).trim_start_matches('/');
                        let dest_key = format!("{}/{}/{}", site_path, app_name, relative_path);

                        s3.copy_object()
                            .bucket(bucket)
                            .copy_source(format!("{}/{}", bucket, key))
                            .key(&dest_key)
                            .send()
                            .await?;

                        trace!("Synced {} to {}", key, dest_key);
                    }
                }
            }
        }

        let _ = self.store_app_metadata(bot_id, app_name, &format!("{}/{}", site_path, app_name));

        info!("App synced to site root: {}/{}", site_path, app_name);
        Ok(())
    }

    fn store_app_metadata(
        &self,
        bot_id: Uuid,
        app_name: &str,
        app_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;
        let app_id = Uuid::new_v4();

        sql_query(
            "INSERT INTO generated_apps (id, bot_id, name, app_path, is_active, created_at)
             VALUES ($1, $2, $3, $4, true, NOW())
             ON CONFLICT (bot_id, name) DO UPDATE SET
             app_path = EXCLUDED.app_path,
             updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(app_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(app_name)
        .bind::<diesel::sql_types::Text, _>(app_path)
        .execute(&mut conn)?;

        Ok(())
    }

    fn get_site_path(_bot_id: Uuid) -> String {
        ".gbdrive/site".to_string()
    }

    fn generate_designer_js(app_name: &str) -> String {
        format!(
            r#"(function() {{
    const APP_NAME = '{app_name}';
    const currentPage = window.location.pathname.split('/').pop() || 'index.html';

    const style = document.createElement('style');
    style.textContent = `
        .designer-fab {{ position: fixed; bottom: 20px; right: 20px; width: 56px; height: 56px; border-radius: 50%; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border: none; cursor: pointer; box-shadow: 0 4px 12px rgba(102,126,234,0.4); font-size: 24px; z-index: 9999; transition: transform 0.2s; }}
        .designer-fab:hover {{ transform: scale(1.1); }}
        .designer-panel {{ position: fixed; bottom: 90px; right: 20px; width: 380px; max-height: 500px; background: white; border-radius: 16px; box-shadow: 0 10px 40px rgba(0,0,0,0.2); z-index: 9998; display: none; flex-direction: column; overflow: hidden; }}
        .designer-panel.open {{ display: flex; }}
        .designer-header {{ padding: 16px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; font-weight: 600; display: flex; justify-content: space-between; align-items: center; }}
        .designer-close {{ background: none; border: none; color: white; font-size: 20px; cursor: pointer; }}
        .designer-messages {{ flex: 1; overflow-y: auto; padding: 16px; max-height: 300px; }}
        .designer-msg {{ margin: 8px 0; padding: 10px 14px; border-radius: 12px; max-width: 85%; word-wrap: break-word; }}
        .designer-msg.user {{ background: #667eea; color: white; margin-left: auto; }}
        .designer-msg.ai {{ background: #f0f0f0; color: #333; }}
        .designer-input {{ display: flex; padding: 12px; border-top: 1px solid #eee; gap: 8px; }}
        .designer-input input {{ flex: 1; padding: 10px 14px; border: 1px solid #ddd; border-radius: 20px; outline: none; }}
        .designer-input button {{ padding: 10px 16px; background: #667eea; color: white; border: none; border-radius: 20px; cursor: pointer; }}
    `;
    document.head.appendChild(style);

    const fab = document.createElement('button');
    fab.className = 'designer-fab';
    fab.innerHTML = '🎨';
    fab.title = 'Designer AI';
    document.body.appendChild(fab);

    const panel = document.createElement('div');
    panel.className = 'designer-panel';
    panel.innerHTML = `
        <div class="designer-header">
            <span>🎨 Designer AI</span>
            <button class="designer-close">×</button>
        </div>
        <div class="designer-messages">
            <div class="designer-msg ai">Hi! I can help you modify this app. What would you like to change?</div>
        </div>
        <div class="designer-input">
            <input type="text" placeholder="e.g., Add a blue header..." />
            <button>Send</button>
        </div>
    `;
    document.body.appendChild(panel);

    fab.onclick = () => panel.classList.toggle('open');
    panel.querySelector('.designer-close').onclick = () => panel.classList.remove('open');

    const input = panel.querySelector('input');
    const sendBtn = panel.querySelector('.designer-input button');
    const messages = panel.querySelector('.designer-messages');

    async function sendMessage() {{
        const msg = input.value.trim();
        if (!msg) return;

        messages.innerHTML += `<div class="designer-msg user">${{msg}}</div>`;
        input.value = '';
        messages.scrollTop = messages.scrollHeight;

        try {{
            const res = await fetch('/api/designer/modify', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ app_name: APP_NAME, current_page: currentPage, message: msg }})
            }});
            const data = await res.json();
            messages.innerHTML += `<div class="designer-msg ai">${{data.message || 'Done!'}}</div>`;
            if (data.success && data.changes && data.changes.length > 0) {{
                setTimeout(() => location.reload(), 1500);
            }}
        }} catch (e) {{
            messages.innerHTML += `<div class="designer-msg ai">Sorry, something went wrong. Try again.</div>`;
            if (window.AppLogger) window.AppLogger.error('Designer error', e.toString());
        }}
        messages.scrollTop = messages.scrollHeight;
    }}

    sendBtn.onclick = sendMessage;
    input.onkeypress = (e) => {{ if (e.key === 'Enter') sendMessage(); }};
}})();"#,
            app_name = app_name
        )
    }
}
