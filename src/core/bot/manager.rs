//! Bot Manager Module
//!
//! Manages bot lifecycle including:
//! - Creating new bots from templates
//! - MinIO bucket creation (folder = bucket)
//! - Security/access assignment
//! - Custom UI routing (/botname/gbui)

use crate::core::shared::schema::organizations;
use crate::shared::platform_name;
use crate::shared::utils::DbPool;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Unique bot ID
    pub id: Uuid,

    /// Bot name (used in URLs: /botname)
    pub name: String,

    /// Display name
    pub display_name: String,

    /// Organization ID
    pub org_id: Uuid,

    /// Organization slug
    pub org_slug: String,

    /// Template used to create this bot
    pub template: Option<String>,

    /// Bot status
    pub status: BotStatus,

    /// MinIO bucket name
    pub bucket: String,

    /// Custom UI path (optional)
    pub custom_ui: Option<String>,

    /// Bot settings
    pub settings: BotSettings,

    /// Access control
    pub access: BotAccess,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Created by user ID
    pub created_by: Uuid,
}

/// Bot status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BotStatus {
    Active,
    Inactive,
    Maintenance,
    Creating,
    Error,
}

impl std::fmt::Display for BotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotStatus::Active => write!(f, "Active"),
            BotStatus::Inactive => write!(f, "Inactive"),
            BotStatus::Maintenance => write!(f, "Maintenance"),
            BotStatus::Creating => write!(f, "Creating"),
            BotStatus::Error => write!(f, "Error"),
        }
    }
}

/// Bot settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotSettings {
    /// Default LLM model
    pub llm_model: Option<String>,

    /// Knowledge bases enabled
    pub knowledge_bases: Vec<String>,

    /// Enabled channels
    pub channels: Vec<String>,

    /// Webhook endpoints
    pub webhooks: Vec<String>,

    /// Schedule definitions
    pub schedules: Vec<String>,

    /// Custom variables
    pub variables: HashMap<String, String>,
}

/// Bot access control
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotAccess {
    /// Admin users (full access)
    pub admins: Vec<Uuid>,

    /// Editor users (can edit scripts)
    pub editors: Vec<Uuid>,

    /// Viewer users (read-only)
    pub viewers: Vec<Uuid>,

    /// Public access enabled
    pub is_public: bool,

    /// Allowed domains (for embedding)
    pub allowed_domains: Vec<String>,

    /// API key for external access
    pub api_key: Option<String>,
}

/// Available bot templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTemplate {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub files: Vec<TemplateFile>,
    pub preview_image: Option<String>,
}

/// Template file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

/// Bot creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBotRequest {
    /// Bot name (will be used in URLs)
    pub name: String,

    /// Display name
    pub display_name: Option<String>,

    /// Organization ID
    pub org_id: Uuid,

    /// Template to use (optional)
    pub template: Option<String>,

    /// Creator user ID
    pub created_by: Uuid,

    /// Initial settings
    pub settings: Option<BotSettings>,

    /// Custom UI name (optional)
    pub custom_ui: Option<String>,
}

/// Bot Manager
pub struct BotManager {
    /// MinIO client for bucket operations
    minio_endpoint: String,
    minio_access_key: String,
    minio_secret_key: String,

    /// Database connection string
    database_url: String,

    /// Templates directory
    templates_dir: PathBuf,

    /// Cached bots
    bots_cache: Arc<RwLock<HashMap<Uuid, BotConfig>>>,

    /// Available templates
    templates: Arc<RwLock<HashMap<String, BotTemplate>>>,
}

impl BotManager {
    /// Create a new BotManager
    pub fn new(
        minio_endpoint: &str,
        minio_access_key: &str,
        minio_secret_key: &str,
        database_url: &str,
        templates_dir: PathBuf,
    ) -> Self {
        Self {
            minio_endpoint: minio_endpoint.to_string(),
            minio_access_key: minio_access_key.to_string(),
            minio_secret_key: minio_secret_key.to_string(),
            database_url: database_url.to_string(),
            templates_dir,
            bots_cache: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize manager and load templates
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing Bot Manager...");

        // Load available templates
        self.load_templates().await?;

        info!("Bot Manager initialized");
        Ok(())
    }

    /// Load templates from templates directory
    async fn load_templates(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut templates = self.templates.write().await;

        // Built-in templates
        let builtin_templates = vec![
            BotTemplate {
                name: "default".to_string(),
                display_name: "Default Bot".to_string(),
                description: "Basic bot with weather, email, and calculation tools".to_string(),
                category: "General".to_string(),
                files: vec![
                    TemplateFile {
                        path: "default.gbdialog/start.bas".to_string(),
                        content: r#"REM Default start script
SET user_name = "Guest"
TALK "Hello, " + user_name + "! How can I help you today?"
HEAR user_input
response = LLM "Respond helpfully to: " + user_input
TALK response
"#
                        .to_string(),
                    },
                    TemplateFile {
                        path: "default.gbot/config.json".to_string(),
                        content: r#"{
  "name": "{{botname}}",
  "description": "Default bot created from template",
  "version": "1.0.0"
}"#
                        .to_string(),
                    },
                ],
                preview_image: None,
            },
            BotTemplate {
                name: "crm".to_string(),
                display_name: "CRM Bot".to_string(),
                description: "Customer relationship management with lead scoring".to_string(),
                category: "Business".to_string(),
                files: vec![TemplateFile {
                    path: "crm.gbdialog/lead.bas".to_string(),
                    content: r#"REM Lead capture script
PARAM name AS string LIKE "John Doe"
PARAM email AS string LIKE "john@example.com"
PARAM company AS string LIKE "Acme Inc"
DESCRIPTION "Capture and score leads"

TALK "Welcome! Let me help you get started."
TALK "What's your name?"
HEAR name

TALK "And your email?"
HEAR email

TALK "What company are you from?"
HEAR company

score = AI SCORE LEAD email, company, "interested in our product"
INSERT "leads", name, email, company, score, NOW()

IF score > 80 THEN
    CREATE TASK "Hot lead: " + name, "sales", "today"
    TALK "Great! Our sales team will reach out shortly."
ELSE
    TALK "Thanks for your interest! We'll send you some resources."
    SEND MAIL email, "Welcome!", "Thanks for reaching out..."
END IF
"#
                    .to_string(),
                }],
                preview_image: None,
            },
            BotTemplate {
                name: "edu".to_string(),
                display_name: "Education Bot".to_string(),
                description: "Course management and student enrollment".to_string(),
                category: "Education".to_string(),
                files: vec![TemplateFile {
                    path: "edu.gbdialog/enroll.bas".to_string(),
                    content: r#"REM Student enrollment script
PARAM student_name AS string LIKE "Jane Student"
PARAM course AS string LIKE "Introduction to AI"
DESCRIPTION "Enroll students in courses"

TALK "Welcome to our enrollment system!"
TALK "What's your full name?"
HEAR student_name

TALK "Which course would you like to enroll in?"
courses = FIND "courses", "status='open'"
FOR EACH course IN courses
    TALK "- " + course.name
NEXT
HEAR selected_course

INSERT "enrollments", student_name, selected_course, NOW()
TALK "You're enrolled in " + selected_course + "!"
SEND MAIL student_email, "Enrollment Confirmed", "Welcome to " + selected_course
"#
                    .to_string(),
                }],
                preview_image: None,
            },
            BotTemplate {
                name: "store".to_string(),
                display_name: "E-commerce Bot".to_string(),
                description: "Product catalog and order management".to_string(),
                category: "Business".to_string(),
                files: vec![TemplateFile {
                    path: "store.gbdialog/order.bas".to_string(),
                    content: r#"REM Order management script
DESCRIPTION "Help customers with orders"

TALK "Welcome to our store! How can I help?"
ADD SUGGESTION "Track my order"
ADD SUGGESTION "Browse products"
ADD SUGGESTION "Contact support"
HEAR choice

SWITCH choice
    CASE "Track my order"
        TALK "Please enter your order number:"
        HEAR order_id
        order = FIND "orders", "id=" + order_id
        TALK "Order status: " + order.status
    CASE "Browse products"
        products = FIND "products", "in_stock=true"
        TALK "Here are our available products:"
        FOR EACH product IN products
            TALK product.name + " - $" + product.price
        NEXT
    DEFAULT
        ticket = CREATE TASK choice, "support", "normal"
        TALK "Support ticket created: #" + ticket
END SWITCH
"#
                    .to_string(),
                }],
                preview_image: None,
            },
            BotTemplate {
                name: "hr".to_string(),
                display_name: "HR Assistant".to_string(),
                description: "Human resources and employee management".to_string(),
                category: "Business".to_string(),
                files: vec![TemplateFile {
                    path: "hr.gbdialog/leave.bas".to_string(),
                    content: r#"REM Leave request script
DESCRIPTION "Handle employee leave requests"

TALK "HR Assistant here. How can I help?"
ADD SUGGESTION "Request leave"
ADD SUGGESTION "Check balance"
ADD SUGGESTION "View policies"
HEAR request

IF request = "Request leave" THEN
    TALK "What type of leave? (vacation/sick/personal)"
    HEAR leave_type
    TALK "Start date? (YYYY-MM-DD)"
    HEAR start_date
    TALK "End date? (YYYY-MM-DD)"
    HEAR end_date

    INSERT "leave_requests", user_id, leave_type, start_date, end_date, "pending"

    manager = FIND "employees", "id=" + user.manager_id
    TALK TO manager.email, "Leave request from " + user.name
    TALK "Leave request submitted! Your manager will review it."
ELSE IF request = "Check balance" THEN
    balance = FIND "leave_balances", "user_id=" + user_id
    TALK "Your leave balance:"
    TALK "Vacation: " + balance.vacation + " days"
    TALK "Sick: " + balance.sick + " days"
END IF
"#
                    .to_string(),
                }],
                preview_image: None,
            },
            BotTemplate {
                name: "healthcare".to_string(),
                display_name: "Healthcare Bot".to_string(),
                description: "Appointment scheduling and patient management".to_string(),
                category: "Healthcare".to_string(),
                files: vec![TemplateFile {
                    path: "healthcare.gbdialog/appointment.bas".to_string(),
                    content: r#"REM Appointment scheduling
DESCRIPTION "Schedule healthcare appointments"

TALK "Welcome to our healthcare center. How can I help?"
ADD SUGGESTION "Book appointment"
ADD SUGGESTION "Cancel appointment"
ADD SUGGESTION "View my appointments"
HEAR choice

IF choice = "Book appointment" THEN
    TALK "What type of appointment? (general/specialist/lab)"
    HEAR apt_type

    TALK "Preferred date? (YYYY-MM-DD)"
    HEAR pref_date

    available = FIND "slots", "date=" + pref_date + " AND type=" + apt_type
    TALK "Available times:"
    FOR EACH slot IN available
        TALK slot.time + " - Dr. " + slot.doctor
    NEXT

    TALK "Which time would you prefer?"
    HEAR selected_time

    BOOK apt_type + " appointment", selected_time, user.email
    TALK "Appointment booked! You'll receive a confirmation email."
END IF
"#
                    .to_string(),
                }],
                preview_image: None,
            },
        ];

        for template in builtin_templates {
            templates.insert(template.name.clone(), template);
        }

        // Load templates from filesystem
        if self.templates_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.templates_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.extension().map_or(false, |e| e == "gbai") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            if !templates.contains_key(name) {
                                debug!("Found template directory: {}", name);
                                // Load template from filesystem directory
                                if let Some(template) =
                                    self.load_template_from_directory(&path, name)
                                {
                                    templates.insert(name.to_string(), template);
                                    info!("Loaded template from filesystem: {}", name);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Loaded {} templates", templates.len());
        Ok(())
    }

    /// Load a template from a filesystem directory
    fn load_template_from_directory(&self, path: &PathBuf, name: &str) -> Option<BotTemplate> {
        // Check for template metadata file
        let metadata_path = path.join("template.toml");
        let description = if metadata_path.exists() {
            std::fs::read_to_string(&metadata_path)
                .ok()
                .and_then(|content| {
                    toml::from_str::<toml::Value>(&content).ok().and_then(|v| {
                        v.get("description")
                            .and_then(|d| d.as_str().map(String::from))
                    })
                })
                .unwrap_or_else(|| format!("Template loaded from {}", name))
        } else {
            format!("Template loaded from {}", name)
        };

        // Check for dialogs
        let dialog_dir = path.join(format!("{}.gbdialog", name));
        let dialogs = if dialog_dir.exists() {
            std::fs::read_dir(&dialog_dir)
                .ok()
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|e| e.path().extension().map_or(false, |ext| ext == "bas"))
                        .filter_map(|e| {
                            let file_name = e.file_name().to_string_lossy().to_string();
                            let content = std::fs::read_to_string(e.path()).ok()?;
                            Some(DialogFile {
                                name: file_name,
                                content,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Check for preview image
        let preview_image = ["preview.png", "preview.jpg", "preview.svg"]
            .iter()
            .map(|f| path.join(f))
            .find(|p| p.exists())
            .and_then(|p| p.to_str().map(String::from));

        Some(BotTemplate {
            name: name.to_string(),
            description,
            category: "Custom".to_string(),
            dialogs,
            preview_image,
        })
    }

    /// Look up organization slug from database
    fn get_org_slug_from_db(&self, conn: &DbPool, org_id: Uuid) -> String {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get database connection for org lookup: {}", e);
                return "default".to_string();
            }
        };

        let result = organizations::table
            .filter(organizations::org_id.eq(org_id))
            .select(organizations::slug)
            .first::<String>(&mut db_conn)
            .optional();

        match result {
            Ok(Some(slug)) => {
                debug!("Found org slug '{}' for org_id {}", slug, org_id);
                slug
            }
            Ok(None) => {
                debug!("No org found for org_id {}, using 'default'", org_id);
                "default".to_string()
            }
            Err(e) => {
                warn!("Database error looking up org {}: {}", org_id, e);
                "default".to_string()
            }
        }
    }

    /// Create a new bot
    pub async fn create_bot(
        &self,
        request: CreateBotRequest,
        conn: &DbPool,
    ) -> Result<BotConfig, Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating bot: {} for org: {}", request.name, request.org_id);

        // Validate bot name
        let bot_name = self.sanitize_bot_name(&request.name);
        if bot_name.is_empty() {
            return Err("Invalid bot name".into());
        }

        // Get org slug from database
        let org_slug = self.get_org_slug_from_db(conn, request.org_id);

        // Generate bucket name: org_botname
        let bucket_name = format!("{}_{}", org_slug, bot_name);

        // Create MinIO bucket
        self.create_minio_bucket(&bucket_name).await?;

        // Create bot configuration
        let bot_id = Uuid::new_v4();
        let now = Utc::now();

        let bot_config = BotConfig {
            id: bot_id,
            name: bot_name.clone(),
            display_name: request.display_name.unwrap_or_else(|| bot_name.clone()),
            org_id: request.org_id,
            org_slug: org_slug.to_string(),
            template: request.template.clone(),
            status: BotStatus::Creating,
            bucket: bucket_name.clone(),
            custom_ui: request.custom_ui,
            settings: request.settings.unwrap_or_default(),
            access: BotAccess {
                admins: vec![request.created_by],
                ..Default::default()
            },
            created_at: now,
            updated_at: now,
            created_by: request.created_by,
        };

        // Apply template if specified
        if let Some(template_name) = &request.template {
            self.apply_template(&bucket_name, template_name, &bot_name)
                .await?;
        } else {
            // Create default directory structure
            self.create_default_structure(&bucket_name, &bot_name)
                .await?;
        }

        // Cache the bot
        {
            let mut cache = self.bots_cache.write().await;
            cache.insert(bot_id, bot_config.clone());
        }

        // Update status to active
        let mut bot_config = bot_config;
        bot_config.status = BotStatus::Active;

        info!("Bot created successfully: {} ({})", bot_name, bot_id);

        Ok(bot_config)
    }

    /// Sanitize bot name for use in URLs and buckets
    fn sanitize_bot_name(&self, name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
            .trim_matches(|c| c == '-' || c == '_')
            .to_string()
    }

    /// Create MinIO bucket for bot
    async fn create_minio_bucket(
        &self,
        bucket_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating MinIO bucket: {}", bucket_name);

        // Use mc command to create bucket
        // In production, use the AWS S3 SDK or minio-rs
        let output = tokio::process::Command::new("mc")
            .args(["mb", &format!("local/{}", bucket_name), "--ignore-existing"])
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    info!("Bucket created: {}", bucket_name);
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    if !stderr.contains("already exists") {
                        warn!("Bucket creation warning: {}", stderr);
                    }
                }
            }
            Err(e) => {
                error!("Failed to create bucket: {}", e);
                // Don't fail - bucket might be created via other means
            }
        }

        // Set bucket policy (optional - make specific paths public if needed)
        // mc admin policy attach local/ readwrite --user botuser

        Ok(())
    }

    /// Create MinIO user for bot admin access
    pub async fn create_bot_user(
        &self,
        username: &str,
        password: &str,
        bucket: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating MinIO user: {} for bucket: {}", username, bucket);

        // Create user
        // mc admin user add local/ username password
        let _ = tokio::process::Command::new("mc")
            .args(["admin", "user", "add", "local/", username, password])
            .output()
            .await;

        // Create policy for bucket access
        let policy = serde_json::json!({
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": [
                        "s3:GetObject",
                        "s3:PutObject",
                        "s3:DeleteObject",
                        "s3:ListBucket"
                    ],
                    "Resource": [
                        format!("arn:aws:s3:::{}", bucket),
                        format!("arn:aws:s3:::{}/*", bucket)
                    ]
                }
            ]
        });

        // Write policy to temp file
        let policy_path = format!("/tmp/policy_{}.json", bucket);
        std::fs::write(&policy_path, policy.to_string())?;

        // Create and attach policy
        // mc admin policy create local/ policyname policy.json
        let policy_name = format!("policy_{}", bucket);
        let _ = tokio::process::Command::new("mc")
            .args([
                "admin",
                "policy",
                "create",
                "local/",
                &policy_name,
                &policy_path,
            ])
            .output()
            .await;

        // Attach policy to user
        // mc admin policy attach local/ policyname --user username
        let _ = tokio::process::Command::new("mc")
            .args([
                "admin",
                "policy",
                "attach",
                "local/",
                &policy_name,
                "--user",
                username,
            ])
            .output()
            .await;

        // Clean up temp file
        let _ = std::fs::remove_file(&policy_path);

        info!("User created with bucket access: {}", username);
        Ok(())
    }

    /// Apply template to bot bucket
    async fn apply_template(
        &self,
        bucket: &str,
        template_name: &str,
        bot_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Applying template '{}' to bucket '{}'",
            template_name, bucket
        );

        let templates = self.templates.read().await;
        let template = templates
            .get(template_name)
            .ok_or_else(|| format!("Template not found: {}", template_name))?;

        for file in &template.files {
            // Replace template variables
            let content = file
                .content
                .replace("{{botname}}", bot_name)
                .replace("{{platform}}", platform_name());

            // Upload file to MinIO
            self.upload_file(bucket, &file.path, content.as_bytes())
                .await?;
        }

        info!(
            "Applied template '{}' ({} files)",
            template_name,
            template.files.len()
        );
        Ok(())
    }

    /// Create default directory structure for bot
    async fn create_default_structure(
        &self,
        bucket: &str,
        bot_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating default structure in bucket: {}", bucket);

        // Create directory markers (empty files with trailing /)
        let dirs = [
            format!("{}.gbdialog/", bot_name),
            format!("{}.gbkb/", bot_name),
            format!("{}.gbot/", bot_name),
            format!("{}.gbtheme/", bot_name),
            "uploads/".to_string(),
            "exports/".to_string(),
            "cache/".to_string(),
        ];

        for dir in &dirs {
            self.upload_file(bucket, dir, b"").await?;
        }

        // Create default config
        let config = serde_json::json!({
            "name": bot_name,
            "version": "1.0.0",
            "created_at": Utc::now().to_rfc3339(),
            "platform": platform_name()
        });

        self.upload_file(
            bucket,
            &format!("{}.gbot/config.json", bot_name),
            config.to_string().as_bytes(),
        )
        .await?;

        // Create default start script
        let start_script = format!(
            r#"REM {} - Start Script
TALK "Hello! I'm {}. How can I help you?"
HEAR user_input
response = LLM "Respond helpfully to: " + user_input
TALK response
"#,
            bot_name, bot_name
        );

        self.upload_file(
            bucket,
            &format!("{}.gbdialog/start.bas", bot_name),
            start_script.as_bytes(),
        )
        .await?;

        info!("Default structure created");
        Ok(())
    }

    /// Upload file to MinIO bucket
    async fn upload_file(
        &self,
        bucket: &str,
        path: &str,
        content: &[u8],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Uploading to {}/{}", bucket, path);

        // Write to temp file
        let temp_path = format!("/tmp/upload_{}", Uuid::new_v4());
        std::fs::write(&temp_path, content)?;

        // Use mc to upload
        let result = tokio::process::Command::new("mc")
            .args(["cp", &temp_path, &format!("local/{}/{}", bucket, path)])
            .output()
            .await;

        // Clean up
        let _ = std::fs::remove_file(&temp_path);

        match result {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Upload warning: {}", stderr);
                }
            }
            Err(e) => {
                warn!("Upload failed (mc not available): {}", e);
                // Fallback: write directly to filesystem if mc not available
                let fs_path = format!("./botserver-stack/minio/{}/{}", bucket, path);
                if let Some(parent) = std::path::Path::new(&fs_path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&fs_path, content)?;
            }
        }

        Ok(())
    }

    /// Get available templates
    pub async fn get_templates(&self) -> Vec<BotTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }

    /// Get bot by ID
    pub async fn get_bot(&self, bot_id: Uuid) -> Option<BotConfig> {
        let cache = self.bots_cache.read().await;
        cache.get(&bot_id).cloned()
    }

    /// Get bot by name and org
    pub async fn get_bot_by_name(&self, org_slug: &str, bot_name: &str) -> Option<BotConfig> {
        let cache = self.bots_cache.read().await;
        cache
            .values()
            .find(|b| b.org_slug == org_slug && b.name == bot_name)
            .cloned()
    }

    /// List bots for organization
    pub async fn list_bots(&self, org_id: Uuid) -> Vec<BotConfig> {
        let cache = self.bots_cache.read().await;
        cache
            .values()
            .filter(|b| b.org_id == org_id)
            .cloned()
            .collect()
    }

    /// Delete bot
    pub async fn delete_bot(
        &self,
        bot_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bot = self.get_bot(bot_id).await.ok_or("Bot not found")?;

        info!("Deleting bot: {} ({})", bot.name, bot_id);

        // Delete bucket contents
        let _ = tokio::process::Command::new("mc")
            .args([
                "rm",
                "--recursive",
                "--force",
                &format!("local/{}", bot.bucket),
            ])
            .output()
            .await;

        // Delete bucket
        let _ = tokio::process::Command::new("mc")
            .args(["rb", &format!("local/{}", bot.bucket)])
            .output()
            .await;

        // Remove from cache
        {
            let mut cache = self.bots_cache.write().await;
            cache.remove(&bot_id);
        }

        info!("Bot deleted: {}", bot_id);
        Ok(())
    }

    /// Get URL for bot
    pub fn get_bot_url(&self, bot: &BotConfig, base_url: &str) -> String {
        format!("{}/{}", base_url.trim_end_matches('/'), bot.name)
    }

    /// Get custom UI URL for bot
    pub fn get_custom_ui_url(&self, bot: &BotConfig, base_url: &str) -> Option<String> {
        bot.custom_ui.as_ref().map(|ui| {
            format!(
                "{}/{}/gbui/{}",
                base_url.trim_end_matches('/'),
                bot.name,
                ui
            )
        })
    }
}

/// Bot routing configuration for web server
#[derive(Debug, Clone)]
pub struct BotRoute {
    /// Bot name (used in URL path)
    pub name: String,

    /// Organization slug
    pub org_slug: String,

    /// Full bucket path
    pub bucket: String,

    /// Custom UI path (if any)
    pub custom_ui: Option<String>,
}

impl From<&BotConfig> for BotRoute {
    fn from(bot: &BotConfig) -> Self {
        BotRoute {
            name: bot.name.clone(),
            org_slug: bot.org_slug.clone(),
            bucket: bot.bucket.clone(),
            custom_ui: bot.custom_ui.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_bot_name() {
        let manager = BotManager::new("", "", "", "", PathBuf::new());

        assert_eq!(manager.sanitize_bot_name("My Bot"), "mybot");
        assert_eq!(manager.sanitize_bot_name("test-bot"), "test-bot");
        assert_eq!(manager.sanitize_bot_name("Bot 123"), "bot123");
        assert_eq!(manager.sanitize_bot_name("--invalid--"), "invalid");
        assert_eq!(manager.sanitize_bot_name("my_bot_name"), "my_bot_name");
    }

    #[test]
    fn test_bot_config_default() {
        let settings = BotSettings::default();
        assert!(settings.knowledge_bases.is_empty());
        assert!(settings.channels.is_empty());
    }

    #[test]
    fn test_bot_status_display() {
        assert_eq!(format!("{}", BotStatus::Active), "Active");
        assert_eq!(format!("{}", BotStatus::Creating), "Creating");
    }

    #[test]
    fn test_bot_route_from_config() {
        let config = BotConfig {
            id: Uuid::new_v4(),
            name: "testbot".to_string(),
            display_name: "Test Bot".to_string(),
            org_id: Uuid::new_v4(),
            org_slug: "myorg".to_string(),
            template: None,
            status: BotStatus::Active,
            bucket: "myorg_testbot".to_string(),
            custom_ui: Some("custom".to_string()),
            settings: BotSettings::default(),
            access: BotAccess::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };

        let route = BotRoute::from(&config);
        assert_eq!(route.name, "testbot");
        assert_eq!(route.org_slug, "myorg");
        assert_eq!(route.bucket, "myorg_testbot");
        assert_eq!(route.custom_ui, Some("custom".to_string()));
    }
}
