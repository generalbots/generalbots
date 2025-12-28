use crate::basic::keywords::table_definition::{
    generate_create_table_sql, FieldDefinition, TableDefinition,
};
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pages: Vec<GeneratedPage>,
    pub tables: Vec<TableDefinition>,
    pub tools: Vec<GeneratedScript>,
    pub schedulers: Vec<GeneratedScript>,
    pub created_at: DateTime<Utc>,
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

        let structure = self.analyze_app_requirements_with_llm(intent).await?;
        trace!("App structure analyzed: {:?}", structure.name);

        let tables_bas_content = self.generate_table_definitions(&structure)?;
        self.append_to_tables_bas(session.bot_id, &tables_bas_content)?;

        let sync_result = self.sync_tables_to_database(&structure.tables)?;
        info!(
            "Tables synced: {} created, {} fields added",
            sync_result.tables_created, sync_result.fields_added
        );

        let pages = self.generate_htmx_pages(&structure)?;
        trace!("Generated {} pages", pages.len());

        let bot_name = self.get_bot_name(session.bot_id)?;
        let bucket_name = format!("{}.gbai", bot_name.to_lowercase());
        let drive_app_path = format!(".gbdrive/apps/{}", structure.name);

        for page in &pages {
            let drive_path = format!("{}/{}", drive_app_path, page.filename);
            self.write_to_drive(&bucket_name, &drive_path, &page.content)
                .await?;
        }

        let designer_js = self.generate_designer_js();
        self.write_to_drive(
            &bucket_name,
            &format!("{}/designer.js", drive_app_path),
            &designer_js,
        )
        .await?;

        let css_content = self.generate_app_css();
        self.write_to_drive(
            &bucket_name,
            &format!("{}/styles.css", drive_app_path),
            &css_content,
        )
        .await?;

        let tools = self.generate_tools(&structure)?;
        for tool in &tools {
            let tool_path = format!(".gbdialog/tools/{}", tool.filename);
            self.write_to_drive(&bucket_name, &tool_path, &tool.content)
                .await?;
        }

        let schedulers = self.generate_schedulers(&structure)?;
        for scheduler in &schedulers {
            let scheduler_path = format!(".gbdialog/schedulers/{}", scheduler.filename);
            self.write_to_drive(&bucket_name, &scheduler_path, &scheduler.content)
                .await?;
        }

        self.sync_app_to_site_root(&bucket_name, &structure.name, session.bot_id)
            .await?;

        info!(
            "App '{}' generated in drive s3://{}/{} and synced to site root",
            structure.name, bucket_name, drive_app_path
        );

        Ok(GeneratedApp {
            id: Uuid::new_v4().to_string(),
            name: structure.name.clone(),
            description: structure.description.clone(),
            pages,
            tables: structure.tables,
            tools,
            schedulers,
            created_at: Utc::now(),
        })
    }

    fn get_platform_capabilities_prompt(&self) -> &'static str {
        r##"
GENERAL BOTS APP GENERATOR - PLATFORM CAPABILITIES

AVAILABLE REST APIs:

DATABASE API (/api/db/):
- GET /api/db/TABLE - List records (query: limit, offset, order_by, order_dir, search, field=value)
- GET /api/db/TABLE/ID - Get single record
- GET /api/db/TABLE/count - Get record count
- POST /api/db/TABLE - Create record (JSON body)
- PUT /api/db/TABLE/ID - Update record (JSON body)
- DELETE /api/db/TABLE/ID - Delete record

FILE STORAGE API (/api/drive/):
- GET /api/drive/list?path=/folder - List files
- GET /api/drive/download?path=/file.ext - Download file
- POST /api/drive/upload - Upload (multipart: file, path)
- DELETE /api/drive/delete?path=/file.ext - Delete file

AUTOTASK API (/api/autotask/):
- POST /api/autotask/create - Create task {"intent": "..."}
- GET /api/autotask/list - List tasks
- GET /api/autotask/stats - Get statistics
- GET /api/autotask/pending - Get pending items

DESIGNER API (/api/designer/):
- POST /api/designer/modify - AI modify app {"app_name", "current_page", "message"}

COMMUNICATION APIs:
- POST /api/mail/send - {"to", "subject", "body"}
- POST /api/whatsapp/send - {"to": "+123...", "message"}
- POST /api/llm/generate - {"prompt", "max_tokens"}
- POST /api/llm/image - {"prompt", "size"}

HTMX ATTRIBUTES:
- hx-get, hx-post, hx-put, hx-delete - HTTP methods
- hx-target="#id" - Where to put response
- hx-swap="innerHTML|outerHTML|beforeend|delete" - How to insert
- hx-trigger="click|submit|load|every 5s|keyup changed delay:300ms" - When to fire
- hx-indicator="#spinner" - Loading indicator
- hx-confirm="Are you sure?" - Confirmation dialog

BASIC AUTOMATION (.bas files):

Tools (.gbdialog/tools/*.bas):
HEAR "phrase1", "phrase2"
    result = GET FROM "table"
    TALK "Response: " + result
END HEAR

Schedulers (.gbdialog/schedulers/*.bas):
SET SCHEDULE "0 9 * * *"
    data = GET FROM "reports"
    SEND MAIL TO "email" WITH SUBJECT "Daily" BODY data
END SCHEDULE

BASIC KEYWORDS:
- TALK "message" - Send message
- ASK "question" - Get user input
- GET FROM "table" WHERE field=val - Query database
- SAVE TO "table" WITH field1, field2 - Insert record
- SEND MAIL TO "x" WITH SUBJECT "y" BODY "z"
- result = LLM "prompt" - AI text generation
- image = GENERATE IMAGE "prompt" - AI image generation

REQUIRED HTML HEAD (all pages must include):
- link rel="stylesheet" href="styles.css"
- script src="/js/vendor/htmx.min.js"
- script src="designer.js" defer

FIELD TYPES: guid, string, text, integer, decimal, boolean, date, datetime, json

RULES:
1. Always use HTMX for API calls - NO fetch() in HTML
2. Include designer.js in ALL pages
3. Make it beautiful and fully functional
4. Tables are optional - simple apps (calculator, timer) dont need them
"##
    }

    async fn analyze_app_requirements_with_llm(
        &self,
        intent: &str,
    ) -> Result<AppStructure, Box<dyn std::error::Error + Send + Sync>> {
        let capabilities = self.get_platform_capabilities_prompt();

        let prompt = format!(
            r#"You are an expert app generator for General Bots platform.

{capabilities}

USER REQUEST: "{intent}"

Generate a complete application. For simple apps (calculator, timer, game), you can use empty tables array.
For data apps (CRM, inventory), design appropriate tables.

Respond with JSON:
{{
    "name": "app-name-lowercase-dashes",
    "description": "What this app does",
    "domain": "custom|healthcare|sales|inventory|booking|etc",
    "tables": [
        {{
            "name": "table_name",
            "fields": [
                {{"name": "id", "type": "guid", "nullable": false}},
                {{"name": "field_name", "type": "string|integer|decimal|boolean|date|datetime|text", "nullable": true, "reference": "other_table or null"}}
            ]
        }}
    ],
    "features": ["list of features"],
    "custom_html": "OPTIONAL: For non-CRUD apps like calculators, provide complete HTML here",
    "custom_css": "OPTIONAL: Custom CSS styles",
    "custom_js": "OPTIONAL: Custom JavaScript"
}}

IMPORTANT:
- For simple tools (calculator, converter, timer): use custom_html/css/js, tables can be empty []
- For data apps (CRM, booking): design tables, custom_* fields are optional
- Always include id, created_at, updated_at in tables
- Make it beautiful and fully functional

Respond with valid JSON only."#
        );

        let response = self.call_llm(&prompt).await?;
        self.parse_app_structure_response(&response, intent)
    }

    /// Parse LLM response into AppStructure
    fn parse_app_structure_response(
        &self,
        response: &str,
        original_intent: &str,
    ) -> Result<AppStructure, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        struct LlmAppResponse {
            name: String,
            description: String,
            domain: String,
            tables: Vec<LlmTableResponse>,
            features: Option<Vec<String>>,
        }

        #[derive(Deserialize)]
        struct LlmTableResponse {
            name: String,
            fields: Vec<LlmFieldResponse>,
        }

        #[derive(Deserialize)]
        struct LlmFieldResponse {
            name: String,
            #[serde(rename = "type")]
            field_type: String,
            nullable: Option<bool>,
            reference: Option<String>,
        }

        match serde_json::from_str::<LlmAppResponse>(response) {
            Ok(resp) => {
                let tables = resp
                    .tables
                    .into_iter()
                    .map(|t| {
                        let fields = t
                            .fields
                            .into_iter()
                            .enumerate()
                            .map(|(i, f)| {
                                let is_id = f.name == "id";
                                FieldDefinition {
                                    name: f.name,
                                    field_type: f.field_type,
                                    length: None,
                                    precision: None,
                                    is_key: i == 0 && is_id,
                                    is_nullable: f.nullable.unwrap_or(true),
                                    default_value: None,
                                    reference_table: f.reference,
                                    field_order: i as i32,
                                }
                            })
                            .collect();

                        TableDefinition {
                            name: t.name,
                            connection_name: "default".to_string(),
                            fields,
                        }
                    })
                    .collect();

                Ok(AppStructure {
                    name: resp.name,
                    description: resp.description,
                    domain: resp.domain,
                    tables,
                    features: resp
                        .features
                        .unwrap_or_else(|| vec!["crud".to_string(), "search".to_string()]),
                })
            }
            Err(e) => {
                warn!("Failed to parse LLM response, using fallback: {e}");
                self.analyze_app_requirements_fallback(original_intent)
            }
        }
    }

    /// Fallback when LLM fails - uses heuristic patterns
    fn analyze_app_requirements_fallback(
        &self,
        intent: &str,
    ) -> Result<AppStructure, Box<dyn std::error::Error + Send + Sync>> {
        let intent_lower = intent.to_lowercase();
        let (domain, name) = self.extract_domain_and_name(&intent_lower);
        let tables = self.infer_tables_from_intent_fallback(&intent_lower, &domain)?;
        let features = vec!["crud".to_string(), "search".to_string()];

        Ok(AppStructure {
            name,
            description: intent.to_string(),
            domain,
            tables,
            features,
        })
    }

    fn extract_domain_and_name(&self, intent: &str) -> (String, String) {
        let patterns = [
            ("clínica", "healthcare", "clinic"),
            ("clinic", "healthcare", "clinic"),
            ("hospital", "healthcare", "hospital"),
            ("médico", "healthcare", "medical"),
            ("paciente", "healthcare", "patients"),
            ("crm", "sales", "crm"),
            ("vendas", "sales", "sales"),
            ("loja", "retail", "store"),
            ("estoque", "inventory", "inventory"),
            ("produto", "inventory", "products"),
            ("cliente", "sales", "customers"),
            ("restaurante", "food", "restaurant"),
            ("reserva", "booking", "reservations"),
        ];

        for (pattern, domain, name) in patterns {
            if intent.contains(pattern) {
                return (domain.to_string(), name.to_string());
            }
        }

        ("general".to_string(), "app".to_string())
    }

    fn infer_tables_from_intent_fallback(
        &self,
        intent: &str,
        domain: &str,
    ) -> Result<Vec<TableDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        let mut tables = Vec::new();

        match domain {
            "healthcare" => {
                tables.push(self.create_patients_table());
                tables.push(self.create_appointments_table());
            }
            "sales" | "retail" => {
                tables.push(self.create_customers_table());
                tables.push(self.create_products_table());
                if intent.contains("venda") || intent.contains("order") {
                    tables.push(self.create_orders_table());
                }
            }
            "inventory" => {
                tables.push(self.create_products_table());
                tables.push(self.create_suppliers_table());
            }
            _ => {
                tables.push(self.create_items_table());
            }
        }

        Ok(tables)
    }

    /// Call LLM for app generation
    async fn call_llm(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        trace!("Calling LLM for app generation");

        #[cfg(feature = "llm")]
        {
            let config = serde_json::json!({
                "temperature": 0.3,
                "max_tokens": 2000
            });
            let response = self
                .state
                .llm_provider
                .generate(prompt, &config, "gpt-4", "")
                .await?;
            return Ok(response);
        }

        #[cfg(not(feature = "llm"))]
        {
            warn!("LLM feature not enabled, using fallback");
            Ok("{}".to_string())
        }
    }

    fn create_patients_table(&self) -> TableDefinition {
        TableDefinition {
            name: "patients".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                self.create_name_field(1),
                self.create_phone_field(2),
                self.create_email_field(3),
                FieldDefinition {
                    name: "birth_date".to_string(),
                    field_type: "date".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: true,
                    default_value: None,
                    reference_table: None,
                    field_order: 4,
                },
                self.create_created_at_field(5),
            ],
        }
    }

    fn create_appointments_table(&self) -> TableDefinition {
        TableDefinition {
            name: "appointments".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                FieldDefinition {
                    name: "patient_id".to_string(),
                    field_type: "guid".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: false,
                    default_value: None,
                    reference_table: Some("patients".to_string()),
                    field_order: 1,
                },
                FieldDefinition {
                    name: "scheduled_at".to_string(),
                    field_type: "datetime".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: false,
                    default_value: None,
                    reference_table: None,
                    field_order: 2,
                },
                FieldDefinition {
                    name: "status".to_string(),
                    field_type: "string".to_string(),
                    length: Some(50),
                    precision: None,
                    is_key: false,
                    is_nullable: false,
                    default_value: Some("'scheduled'".to_string()),
                    reference_table: None,
                    field_order: 3,
                },
                self.create_created_at_field(4),
            ],
        }
    }

    fn create_customers_table(&self) -> TableDefinition {
        TableDefinition {
            name: "customers".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                self.create_name_field(1),
                self.create_phone_field(2),
                self.create_email_field(3),
                FieldDefinition {
                    name: "address".to_string(),
                    field_type: "text".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: true,
                    default_value: None,
                    reference_table: None,
                    field_order: 4,
                },
                self.create_created_at_field(5),
            ],
        }
    }

    fn create_products_table(&self) -> TableDefinition {
        TableDefinition {
            name: "products".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                self.create_name_field(1),
                FieldDefinition {
                    name: "price".to_string(),
                    field_type: "number".to_string(),
                    length: Some(10),
                    precision: Some(2),
                    is_key: false,
                    is_nullable: false,
                    default_value: Some("0".to_string()),
                    reference_table: None,
                    field_order: 2,
                },
                FieldDefinition {
                    name: "stock".to_string(),
                    field_type: "integer".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: false,
                    default_value: Some("0".to_string()),
                    reference_table: None,
                    field_order: 3,
                },
                self.create_created_at_field(4),
            ],
        }
    }

    fn create_orders_table(&self) -> TableDefinition {
        TableDefinition {
            name: "orders".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                FieldDefinition {
                    name: "customer_id".to_string(),
                    field_type: "guid".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: false,
                    default_value: None,
                    reference_table: Some("customers".to_string()),
                    field_order: 1,
                },
                FieldDefinition {
                    name: "total".to_string(),
                    field_type: "number".to_string(),
                    length: Some(10),
                    precision: Some(2),
                    is_key: false,
                    is_nullable: false,
                    default_value: Some("0".to_string()),
                    reference_table: None,
                    field_order: 2,
                },
                FieldDefinition {
                    name: "status".to_string(),
                    field_type: "string".to_string(),
                    length: Some(50),
                    precision: None,
                    is_key: false,
                    is_nullable: false,
                    default_value: Some("'pending'".to_string()),
                    reference_table: None,
                    field_order: 3,
                },
                self.create_created_at_field(4),
            ],
        }
    }

    fn create_suppliers_table(&self) -> TableDefinition {
        TableDefinition {
            name: "suppliers".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                self.create_name_field(1),
                self.create_phone_field(2),
                self.create_email_field(3),
                self.create_created_at_field(4),
            ],
        }
    }

    fn create_items_table(&self) -> TableDefinition {
        TableDefinition {
            name: "items".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                self.create_id_field(0),
                self.create_name_field(1),
                FieldDefinition {
                    name: "description".to_string(),
                    field_type: "text".to_string(),
                    length: None,
                    precision: None,
                    is_key: false,
                    is_nullable: true,
                    default_value: None,
                    reference_table: None,
                    field_order: 2,
                },
                self.create_created_at_field(3),
            ],
        }
    }

    fn create_id_field(&self, order: i32) -> FieldDefinition {
        FieldDefinition {
            name: "id".to_string(),
            field_type: "guid".to_string(),
            length: None,
            precision: None,
            is_key: true,
            is_nullable: false,
            default_value: None,
            reference_table: None,
            field_order: order,
        }
    }

    fn create_name_field(&self, order: i32) -> FieldDefinition {
        FieldDefinition {
            name: "name".to_string(),
            field_type: "string".to_string(),
            length: Some(255),
            precision: None,
            is_key: false,
            is_nullable: false,
            default_value: None,
            reference_table: None,
            field_order: order,
        }
    }

    fn create_phone_field(&self, order: i32) -> FieldDefinition {
        FieldDefinition {
            name: "phone".to_string(),
            field_type: "string".to_string(),
            length: Some(50),
            precision: None,
            is_key: false,
            is_nullable: true,
            default_value: None,
            reference_table: None,
            field_order: order,
        }
    }

    fn create_email_field(&self, order: i32) -> FieldDefinition {
        FieldDefinition {
            name: "email".to_string(),
            field_type: "string".to_string(),
            length: Some(255),
            precision: None,
            is_key: false,
            is_nullable: true,
            default_value: None,
            reference_table: None,
            field_order: order,
        }
    }

    fn create_created_at_field(&self, order: i32) -> FieldDefinition {
        FieldDefinition {
            name: "created_at".to_string(),
            field_type: "datetime".to_string(),
            length: None,
            precision: None,
            is_key: false,
            is_nullable: false,
            default_value: Some("NOW()".to_string()),
            reference_table: None,
            field_order: order,
        }
    }

    fn generate_table_definitions(
        &self,
        structure: &AppStructure,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut output = String::new();

        for table in &structure.tables {
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
                    let _ = write!(line, " DEFAULT {default}");
                }
                if let Some(ref refs) = field.reference_table {
                    let _ = write!(line, " REFERENCES {refs}");
                }
                let _ = writeln!(output, "{line}");
            }

            let _ = writeln!(output, "END TABLE");
        }

        Ok(output)
    }

    fn append_to_tables_bas(
        &self,
        bot_id: Uuid,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For tables.bas, we write to local file system since it's used by the compiler
        // The DriveMonitor will sync it to S3
        let site_path = self.get_site_path();
        let tables_bas_path = format!("{}/{}.gbai/.gbdialog/tables.bas", site_path, bot_id);

        let dir = std::path::Path::new(&tables_bas_path).parent();
        if let Some(d) = dir {
            if !d.exists() {
                std::fs::create_dir_all(d)?;
            }
        }

        let existing = std::fs::read_to_string(&tables_bas_path).unwrap_or_default();
        let new_content = format!("{existing}\n{content}");
        std::fs::write(&tables_bas_path, new_content)?;
        info!("Updated tables.bas at: {}", tables_bas_path);

        Ok(())
    }

    /// Get bot name from database
    fn get_bot_name(
        &self,
        bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::shared::models::schema::bots::dsl::{bots, id, name};
        use diesel::prelude::*;

        let mut conn = self.state.conn.get()?;
        let bot_name: String = bots
            .filter(id.eq(bot_id))
            .select(name)
            .first(&mut conn)
            .map_err(|e| format!("Failed to get bot name: {}", e))?;

        Ok(bot_name)
    }

    /// Write content to S3 drive
    async fn write_to_drive(
        &self,
        bucket: &str,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(client) = &self.state.drive else {
            warn!("S3 client not configured, falling back to local write");
            return self.write_to_local_fallback(bucket, path, content);
        };

        let key = path.to_string();
        let content_type = self.get_content_type(path);

        client
            .put_object()
            .bucket(bucket.to_lowercase())
            .key(&key)
            .body(ByteStream::from(content.as_bytes().to_vec()))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| format!("Failed to write to drive: {}", e))?;

        trace!("Wrote to drive: s3://{}/{}", bucket, key);
        Ok(())
    }

    /// Fallback to local file system when S3 is not configured
    fn write_to_local_fallback(
        &self,
        bucket: &str,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self.get_site_path();
        let full_path = format!("{}/{}/{}", site_path, bucket, path);

        if let Some(dir) = std::path::Path::new(&full_path).parent() {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }

        std::fs::write(&full_path, content)?;
        trace!("Wrote to local fallback: {}", full_path);
        Ok(())
    }

    /// Sync app from drive to SITE_ROOT for serving
    async fn sync_app_to_site_root(
        &self,
        bucket: &str,
        app_name: &str,
        bot_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self.get_site_path();

        // Target: {site_path}/{app_name}/ (clean URL)
        let target_dir = format!("{}/{}", site_path, app_name);
        std::fs::create_dir_all(&target_dir)?;

        let Some(client) = &self.state.drive else {
            info!("S3 not configured, app already written to local path");
            return Ok(());
        };

        // List all files in the app directory on drive
        let prefix = format!(".gbdrive/apps/{}/", app_name);
        let list_result = client
            .list_objects_v2()
            .bucket(bucket.to_lowercase())
            .prefix(&prefix)
            .send()
            .await
            .map_err(|e| format!("Failed to list app files: {}", e))?;

        for obj in list_result.contents.unwrap_or_default() {
            let key = obj.key().unwrap_or_default();
            if key.ends_with('/') {
                continue; // Skip directories
            }

            // Get the file from S3
            let get_result = client
                .get_object()
                .bucket(bucket.to_lowercase())
                .key(key)
                .send()
                .await
                .map_err(|e| format!("Failed to get file {}: {}", key, e))?;

            let body = get_result
                .body
                .collect()
                .await
                .map_err(|e| format!("Failed to read file body: {}", e))?;

            // Extract relative path (remove .gbdrive/apps/{app_name}/ prefix)
            let relative_path = key.strip_prefix(&prefix).unwrap_or(key);
            let local_path = format!("{}/{}", target_dir, relative_path);

            // Create parent directories if needed
            if let Some(dir) = std::path::Path::new(&local_path).parent() {
                if !dir.exists() {
                    std::fs::create_dir_all(dir)?;
                }
            }

            // Write the file
            std::fs::write(&local_path, body.into_bytes())?;
            trace!("Synced: {} -> {}", key, local_path);
        }

        info!("App '{}' synced to site root: {}", app_name, target_dir);

        // Store app metadata in database for tracking
        self.store_app_metadata(bot_id, app_name, &target_dir)?;

        Ok(())
    }

    /// Store app metadata for tracking
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
        .execute(&mut conn)
        .map_err(|e| format!("Failed to store app metadata: {}", e))?;

        Ok(())
    }

    /// Get content type based on file extension
    fn get_content_type(&self, path: &str) -> &'static str {
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" => "application/javascript; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "bas" => "text/plain; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        }
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

    fn generate_htmx_pages(
        &self,
        structure: &AppStructure,
    ) -> Result<Vec<GeneratedPage>, Box<dyn std::error::Error + Send + Sync>> {
        let mut pages = Vec::new();

        pages.push(GeneratedPage {
            filename: "index.html".to_string(),
            title: format!("{} - Dashboard", structure.name),
            page_type: PageType::Dashboard,
            content: self.generate_dashboard_html(structure),
            route: "/".to_string(),
        });

        for table in &structure.tables {
            pages.push(GeneratedPage {
                filename: format!("{}.html", table.name),
                title: format!("{} - List", table.name),
                page_type: PageType::List,
                content: self.generate_list_html(&table.name, &table.fields),
                route: format!("/{}", table.name),
            });

            pages.push(GeneratedPage {
                filename: format!("{}_form.html", table.name),
                title: format!("{} - Form", table.name),
                page_type: PageType::Form,
                content: self.generate_form_html(&table.name, &table.fields),
                route: format!("/{}/new", table.name),
            });
        }

        Ok(pages)
    }

    fn generate_dashboard_html(&self, structure: &AppStructure) -> String {
        let mut html = String::new();

        let _ = writeln!(html, "<!DOCTYPE html>");
        let _ = writeln!(html, "<html lang=\"en\">");
        let _ = writeln!(html, "<head>");
        let _ = writeln!(html, "    <meta charset=\"UTF-8\">");
        let _ = writeln!(
            html,
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        );
        let _ = writeln!(html, "    <title>{}</title>", structure.name);
        let _ = writeln!(html, "    <link rel=\"stylesheet\" href=\"styles.css\">");
        let _ = writeln!(html, "    <script src=\"/js/vendor/htmx.min.js\"></script>");
        let _ = writeln!(html, "    <script src=\"designer.js\" defer></script>");
        let _ = writeln!(html, "</head>");
        let _ = writeln!(html, "<body>");
        let _ = writeln!(html, "    <header class=\"app-header\">");
        let _ = writeln!(html, "        <h1>{}</h1>", structure.name);
        let _ = writeln!(html, "        <nav>");

        for table in &structure.tables {
            let _ = writeln!(
                html,
                "            <a href=\"{}.html\">{}</a>",
                table.name, table.name
            );
        }

        let _ = writeln!(html, "        </nav>");
        let _ = writeln!(html, "    </header>");
        let _ = writeln!(html, "    <main class=\"dashboard\">");

        for table in &structure.tables {
            let _ = writeln!(html, "        <div class=\"stat-card\" hx-get=\"/api/db/{}/count\" hx-trigger=\"load\" hx-swap=\"innerHTML\">", table.name);
            let _ = writeln!(html, "            <h3>{}</h3>", table.name);
            let _ = writeln!(html, "            <span class=\"count\">-</span>");
            let _ = writeln!(html, "        </div>");
        }

        let _ = writeln!(html, "    </main>");
        let _ = writeln!(html, "</body>");
        let _ = writeln!(html, "</html>");

        html
    }

    fn generate_list_html(&self, table_name: &str, fields: &[FieldDefinition]) -> String {
        let mut html = String::new();

        let _ = writeln!(html, "<!DOCTYPE html>");
        let _ = writeln!(html, "<html lang=\"en\">");
        let _ = writeln!(html, "<head>");
        let _ = writeln!(html, "    <meta charset=\"UTF-8\">");
        let _ = writeln!(
            html,
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        );
        let _ = writeln!(html, "    <title>{table_name} - List</title>");
        let _ = writeln!(html, "    <link rel=\"stylesheet\" href=\"styles.css\">");
        let _ = writeln!(html, "    <script src=\"/js/vendor/htmx.min.js\"></script>");
        let _ = writeln!(html, "    <script src=\"designer.js\" defer></script>");
        let _ = writeln!(html, "</head>");
        let _ = writeln!(html, "<body>");
        let _ = writeln!(html, "    <header class=\"page-header\">");
        let _ = writeln!(html, "        <h1>{table_name}</h1>");
        let _ = writeln!(
            html,
            "        <a href=\"{table_name}_form.html\" class=\"btn btn-primary\">Add New</a>"
        );
        let _ = writeln!(html, "    </header>");
        let _ = writeln!(html, "    <main>");
        let _ = writeln!(
            html,
            "        <input type=\"search\" name=\"search\" placeholder=\"Search...\""
        );
        let _ = writeln!(html, "               hx-get=\"/api/db/{table_name}\"");
        let _ = writeln!(
            html,
            "               hx-trigger=\"keyup changed delay:300ms\""
        );
        let _ = writeln!(html, "               hx-target=\"#data-table tbody\">");
        let _ = writeln!(html, "        <table id=\"data-table\">");
        let _ = writeln!(html, "            <thead><tr>");

        for field in fields {
            if field.name != "id" {
                let _ = writeln!(html, "                <th>{}</th>", field.name);
            }
        }

        let _ = writeln!(html, "                <th>Actions</th>");
        let _ = writeln!(html, "            </tr></thead>");
        let _ = writeln!(html, "            <tbody hx-get=\"/api/db/{table_name}\" hx-trigger=\"load\" hx-swap=\"innerHTML\">");
        let _ = writeln!(html, "            </tbody>");
        let _ = writeln!(html, "        </table>");
        let _ = writeln!(html, "    </main>");
        let _ = writeln!(html, "</body>");
        let _ = writeln!(html, "</html>");

        html
    }

    fn generate_form_html(&self, table_name: &str, fields: &[FieldDefinition]) -> String {
        let mut html = String::new();

        let _ = writeln!(html, "<!DOCTYPE html>");
        let _ = writeln!(html, "<html lang=\"en\">");
        let _ = writeln!(html, "<head>");
        let _ = writeln!(html, "    <meta charset=\"UTF-8\">");
        let _ = writeln!(
            html,
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        );
        let _ = writeln!(html, "    <title>{table_name} - Form</title>");
        let _ = writeln!(html, "    <link rel=\"stylesheet\" href=\"styles.css\">");
        let _ = writeln!(html, "    <script src=\"/js/vendor/htmx.min.js\"></script>");
        let _ = writeln!(html, "    <script src=\"designer.js\" defer></script>");
        let _ = writeln!(html, "</head>");
        let _ = writeln!(html, "<body>");
        let _ = writeln!(html, "    <header class=\"page-header\">");
        let _ = writeln!(html, "        <h1>New {table_name}</h1>");
        let _ = writeln!(
            html,
            "        <a href=\"{table_name}.html\" class=\"btn\">Back to List</a>"
        );
        let _ = writeln!(html, "    </header>");
        let _ = writeln!(html, "    <main>");
        let _ = writeln!(
            html,
            "        <form hx-post=\"/api/db/{table_name}\" hx-target=\"#form-result\">"
        );

        for field in fields {
            if field.name == "id" || field.name == "created_at" || field.name == "updated_at" {
                continue;
            }

            let required = if field.is_nullable { "" } else { " required" };
            let input_type = match field.field_type.as_str() {
                "number" | "integer" => "number",
                "date" => "date",
                "datetime" => "datetime-local",
                "boolean" => "checkbox",
                "text" => "textarea",
                _ => "text",
            };

            let _ = writeln!(html, "            <div class=\"form-group\">");
            let _ = writeln!(
                html,
                "                <label for=\"{}\">{}</label>",
                field.name, field.name
            );

            if input_type == "textarea" {
                let _ = writeln!(
                    html,
                    "                <textarea id=\"{}\" name=\"{}\"{}></textarea>",
                    field.name, field.name, required
                );
            } else {
                let _ = writeln!(
                    html,
                    "                <input type=\"{}\" id=\"{}\" name=\"{}\"{}>",
                    input_type, field.name, field.name, required
                );
            }

            let _ = writeln!(html, "            </div>");
        }

        let _ = writeln!(
            html,
            "            <button type=\"submit\" class=\"btn btn-primary\">Save</button>"
        );
        let _ = writeln!(html, "        </form>");
        let _ = writeln!(html, "        <div id=\"form-result\"></div>");
        let _ = writeln!(html, "    </main>");
        let _ = writeln!(html, "</body>");
        let _ = writeln!(html, "</html>");

        html
    }

    fn generate_app_css(&self) -> String {
        r#"* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: system-ui, sans-serif; line-height: 1.5; padding: 1rem; }
.app-header { display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #ddd; margin-bottom: 1rem; }
.app-header nav { display: flex; gap: 1rem; }
.app-header nav a { text-decoration: none; color: #0066cc; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.dashboard { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; }
.stat-card { background: #f5f5f5; padding: 1rem; border-radius: 8px; text-align: center; }
.stat-card .count { font-size: 2rem; font-weight: bold; }
table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
th, td { padding: 0.75rem; text-align: left; border-bottom: 1px solid #ddd; }
th { background: #f5f5f5; }
.form-group { margin-bottom: 1rem; }
.form-group label { display: block; margin-bottom: 0.25rem; font-weight: 500; }
.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 0.5rem; border: 1px solid #ddd; border-radius: 4px; }
.btn { display: inline-block; padding: 0.5rem 1rem; border: none; border-radius: 4px; cursor: pointer; text-decoration: none; }
.btn-primary { background: #0066cc; color: white; }
.btn-danger { background: #cc0000; color: white; }
.btn-secondary { background: #666; color: white; }
input[type="search"] { width: 100%; max-width: 300px; padding: 0.5rem; border: 1px solid #ddd; border-radius: 4px; }
.alert { padding: 1rem; border-radius: 4px; margin-bottom: 1rem; }
.alert-success { background: #d4edda; color: #155724; }
.alert-error { background: #f8d7da; color: #721c24; }
"#.to_string()
    }

    fn generate_tools(
        &self,
        _structure: &AppStructure,
    ) -> Result<Vec<GeneratedScript>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    fn generate_schedulers(
        &self,
        _structure: &AppStructure,
    ) -> Result<Vec<GeneratedScript>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    fn generate_designer_js(&self) -> String {
        r#"(function() {
    const style = document.createElement('style');
    style.textContent = `
        .designer-btn { position: fixed; bottom: 20px; right: 20px; width: 56px; height: 56px; border-radius: 50%; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border: none; cursor: pointer; box-shadow: 0 4px 20px rgba(102,126,234,0.4); font-size: 24px; z-index: 9999; transition: transform 0.2s, box-shadow 0.2s; }
        .designer-btn:hover { transform: scale(1.1); box-shadow: 0 6px 30px rgba(102,126,234,0.6); }
        .designer-panel { position: fixed; bottom: 90px; right: 20px; width: 380px; max-height: 500px; background: white; border-radius: 16px; box-shadow: 0 10px 40px rgba(0,0,0,0.2); z-index: 9998; display: none; flex-direction: column; overflow: hidden; }
        .designer-panel.open { display: flex; }
        .designer-header { padding: 16px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; font-weight: 600; display: flex; justify-content: space-between; align-items: center; }
        .designer-close { background: none; border: none; color: white; font-size: 20px; cursor: pointer; }
        .designer-messages { flex: 1; overflow-y: auto; padding: 16px; max-height: 300px; }
        .designer-msg { margin: 8px 0; padding: 10px 14px; border-radius: 12px; max-width: 85%; }
        .designer-msg.user { background: #667eea; color: white; margin-left: auto; }
        .designer-msg.ai { background: #f0f0f0; color: #333; }
        .designer-input { display: flex; padding: 12px; border-top: 1px solid #eee; gap: 8px; }
        .designer-input input { flex: 1; padding: 10px 14px; border: 1px solid #ddd; border-radius: 20px; outline: none; }
        .designer-input button { padding: 10px 16px; background: #667eea; color: white; border: none; border-radius: 20px; cursor: pointer; }
    `;
    document.head.appendChild(style);

    const btn = document.createElement('button');
    btn.className = 'designer-btn';
    btn.innerHTML = '🎨';
    btn.title = 'Designer AI';
    document.body.appendChild(btn);

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
            <input type="text" placeholder="e.g., Make the header blue..." />
            <button>Send</button>
        </div>
    `;
    document.body.appendChild(panel);

    btn.onclick = () => panel.classList.toggle('open');
    panel.querySelector('.designer-close').onclick = () => panel.classList.remove('open');

    const input = panel.querySelector('input');
    const sendBtn = panel.querySelector('.designer-input button');
    const messages = panel.querySelector('.designer-messages');

    const appName = window.location.pathname.split('/')[2] || 'app';
    const currentPage = window.location.pathname.split('/').pop() || 'index.html';

    async function sendMessage() {
        const msg = input.value.trim();
        if (!msg) return;

        messages.innerHTML += `<div class="designer-msg user">${msg}</div>`;
        input.value = '';
        messages.scrollTop = messages.scrollHeight;

        try {
            const res = await fetch('/api/designer/modify', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ app_name: appName, current_page: currentPage, message: msg })
            });
            const data = await res.json();
            messages.innerHTML += `<div class="designer-msg ai">${data.message || 'Done!'}</div>`;
            if (data.success && data.changes && data.changes.length > 0) {
                setTimeout(() => location.reload(), 1000);
            }
        } catch (e) {
            messages.innerHTML += `<div class="designer-msg ai">Sorry, something went wrong. Try again.</div>`;
        }
        messages.scrollTop = messages.scrollHeight;
    }

    sendBtn.onclick = sendMessage;
    input.onkeypress = (e) => { if (e.key === 'Enter') sendMessage(); };
})();"#.to_string()
    }

    fn get_site_path(&self) -> String {
        self.state
            .config
            .as_ref()
            .map(|c| c.site_path.clone())
            .unwrap_or_else(|| "./botserver-stack/sites".to_string())
    }
}
