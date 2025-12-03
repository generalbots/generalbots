use crate::config::AppConfig;
use crate::package_manager::setup::{DirectorySetup, EmailSetup};
use crate::package_manager::{InstallMode, PackageManager};
use crate::shared::utils::establish_pg_connection;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use chrono;
use log::{error, info, trace, warn};
use rand::distr::Alphanumeric;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, Issuer, KeyPair,
};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
#[derive(Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
}
#[derive(Debug)]
pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
}
impl BootstrapManager {
    pub async fn new(mode: InstallMode, tenant: Option<String>) -> Self {
        trace!(
            "Initializing BootstrapManager with mode {:?} and tenant {:?}",
            mode,
            tenant
        );
        Self {
            install_mode: mode,
            tenant,
        }
    }
    pub fn start_all(&mut self) -> Result<()> {
        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
        let components = vec![
            ComponentInfo { name: "tables" },
            ComponentInfo { name: "cache" },
            ComponentInfo { name: "drive" },
            ComponentInfo { name: "llm" },
            ComponentInfo { name: "email" },
            ComponentInfo { name: "proxy" },
            ComponentInfo { name: "directory" },
            ComponentInfo { name: "alm" },
            ComponentInfo { name: "alm_ci" },
            ComponentInfo { name: "dns" },
            ComponentInfo { name: "meeting" },
            ComponentInfo {
                name: "remote_terminal",
            },
            ComponentInfo { name: "vector_db" },
            ComponentInfo { name: "host" },
        ];
        for component in components {
            if pm.is_installed(component.name) {
                match pm.start(component.name) {
                    Ok(_child) => {
                        trace!("Started component: {}", component.name);
                    }
                    Err(e) => {
                        warn!(
                            "Component {} might already be running: {}",
                            component.name, e
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn generate_secure_password(&self, length: usize) -> String {
        let mut rng = rand::rng();
        (0..length)
            .map(|_| {
                let byte = rand::Rng::sample(&mut rng, Alphanumeric);
                char::from(byte)
            })
            .collect()
    }

    /// Ensure critical services (tables and drive) are running
    pub async fn ensure_services_running(&mut self) -> Result<()> {
        info!("Ensuring critical services are running...");

        let installer = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        // Check and start PostgreSQL
        if installer.is_installed("tables") {
            info!("Starting PostgreSQL database service...");
            match installer.start("tables") {
                Ok(_child) => {
                    info!("PostgreSQL started successfully");
                    // Give PostgreSQL time to initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
                Err(e) => {
                    // Check if it's already running (start might fail if already running)
                    warn!(
                        "PostgreSQL might already be running or failed to start: {}",
                        e
                    );
                }
            }
        } else {
            warn!("PostgreSQL (tables) component not installed");
        }

        // Check and start MinIO
        if installer.is_installed("drive") {
            info!("Starting MinIO drive service...");
            match installer.start("drive") {
                Ok(_child) => {
                    info!("MinIO started successfully");
                    // Give MinIO time to initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    // MinIO is not critical, just log
                    warn!("MinIO might already be running or failed to start: {}", e);
                }
            }
        } else {
            warn!("MinIO (drive) component not installed");
        }

        Ok(())
    }

    pub async fn bootstrap(&mut self) -> Result<()> {
        // Generate certificates first
        info!("🔒 Generating TLS certificates...");
        if let Err(e) = self.generate_certificates().await {
            error!("Failed to generate certificates: {}", e);
        }

        // Directory (Zitadel) is the root service - stores all configuration
        let _directory_password = self.generate_secure_password(32);
        let _directory_masterkey = self.generate_secure_password(32);

        // Configuration is stored in Directory service, not .env files
        info!("Configuring services through Directory...");

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone()).unwrap();
        // Directory must be installed first as it's the root service
        let required_components = vec![
            "directory", // Root service - manages all other services
            "tables",    // Database - credentials stored in Directory
            "drive",     // S3 storage - credentials stored in Directory
            "cache",     // Redis cache
            "llm",       // LLM service
            "email",     // Email service integrated with Directory
            "proxy",     // Caddy reverse proxy
            "dns",       // CoreDNS for dynamic DNS
        ];
        for component in required_components {
            if !pm.is_installed(component) {
                let termination_cmd = pm
                    .components
                    .get(component)
                    .and_then(|cfg| cfg.binary_name.clone())
                    .unwrap_or_else(|| component.to_string());
                if !termination_cmd.is_empty() {
                    let check = Command::new("pgrep")
                        .arg("-f")
                        .arg(&termination_cmd)
                        .output();
                    if let Ok(output) = check {
                        if !output.stdout.is_empty() {
                            println!("Component '{}' appears to be already running from a previous install.", component);
                            println!("Do you want to terminate it? (y/n)");
                            let mut input = String::new();
                            io::stdout().flush().unwrap();
                            io::stdin().read_line(&mut input).unwrap();
                            if input.trim().eq_ignore_ascii_case("y") {
                                let _ = Command::new("pkill")
                                    .arg("-f")
                                    .arg(&termination_cmd)
                                    .status();
                                println!("Terminated existing '{}' process.", component);
                            } else {
                                println!(
                                    "Skipping start of '{}' as it is already running.",
                                    component
                                );
                                continue;
                            }
                        }
                    }
                }
                _ = pm.install(component).await;

                // Directory must be configured first as root service
                if component == "directory" {
                    info!("🔧 Configuring Directory as root service...");
                    if let Err(e) = self.setup_directory().await {
                        error!("Failed to setup Directory: {}", e);
                        return Err(anyhow::anyhow!("Directory is required as root service"));
                    }

                    // After directory is setup, configure database and drive credentials there
                    if let Err(e) = self.configure_services_in_directory().await {
                        error!("Failed to configure services in Directory: {}", e);
                    }
                }

                if component == "tables" {
                    let mut conn = establish_pg_connection().unwrap();
                    self.apply_migrations(&mut conn)?;
                }

                if component == "email" {
                    info!("🔧 Auto-configuring Email (Stalwart)...");
                    if let Err(e) = self.setup_email().await {
                        error!("Failed to setup Email: {}", e);
                    }
                }

                if component == "proxy" {
                    info!("🔧 Configuring Caddy reverse proxy...");
                    if let Err(e) = self.setup_caddy_proxy().await {
                        error!("Failed to setup Caddy: {}", e);
                    }
                }

                if component == "dns" {
                    info!("🔧 Configuring CoreDNS for dynamic DNS...");
                    if let Err(e) = self.setup_coredns().await {
                        error!("Failed to setup CoreDNS: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Configure database and drive credentials in Directory
    async fn configure_services_in_directory(&self) -> Result<()> {
        info!("Storing service credentials in Directory...");

        // Generate credentials for services
        let db_password = self.generate_secure_password(32);
        let drive_password = self.generate_secure_password(16);
        let drive_user = "gbdriveuser".to_string();

        // Create Zitadel configuration with service accounts
        let zitadel_config_path = PathBuf::from("./botserver-stack/conf/directory/zitadel.yaml");
        fs::create_dir_all(zitadel_config_path.parent().unwrap())?;

        let zitadel_config = format!(
            r#"
Database:
  postgres:
    Host: localhost
    Port: 5432
    Database: zitadel
    User: zitadel
    Password: {}
    SSL:
      Mode: require
      RootCert: /botserver-stack/conf/system/certificates/postgres/ca.crt

SystemDefaults:
  SecretGenerators:
    PasswordSaltCost: 14

ExternalSecure: true
ExternalDomain: localhost
ExternalPort: 443

# Service accounts for integrated services
ServiceAccounts:
  - Name: database-service
    Description: PostgreSQL Database Service
    Credentials:
      Username: gbuser
      Password: {}
  - Name: drive-service
    Description: MinIO S3 Storage Service
    Credentials:
      AccessKey: {}
      SecretKey: {}
  - Name: email-service
    Description: Email Service Integration
    OAuth: true
  - Name: git-service
    Description: Forgejo Git Service
    OAuth: true
"#,
            self.generate_secure_password(24),
            db_password,
            drive_user,
            drive_password
        );

        fs::write(zitadel_config_path, zitadel_config)?;

        info!("Service credentials configured in Directory");
        Ok(())
    }

    /// Setup Caddy as reverse proxy for all services
    async fn setup_caddy_proxy(&self) -> Result<()> {
        let caddy_config = PathBuf::from("./botserver-stack/conf/proxy/Caddyfile");
        fs::create_dir_all(caddy_config.parent().unwrap())?;

        let config = format!(
            r#"{{
    admin off
    auto_https disable_redirects
}}

# Main API
api.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# Directory/Auth service
auth.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# LLM service
llm.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# Mail service
mail.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}

# Meet service
meet.botserver.local {{
    tls /botserver-stack/conf/system/certificates/caddy/server.crt /botserver-stack/conf/system/certificates/caddy/server.key
    reverse_proxy {}
}}
"#,
            crate::core::urls::InternalUrls::DIRECTORY_BASE.replace("https://", ""),
            crate::core::urls::InternalUrls::DIRECTORY_BASE.replace("https://", ""),
            crate::core::urls::InternalUrls::LLM.replace("https://", ""),
            crate::core::urls::InternalUrls::EMAIL.replace("https://", ""),
            crate::core::urls::InternalUrls::LIVEKIT.replace("https://", "")
        );

        fs::write(caddy_config, config)?;
        info!("Caddy proxy configured");
        Ok(())
    }

    /// Setup CoreDNS for dynamic DNS service
    async fn setup_coredns(&self) -> Result<()> {
        let dns_config = PathBuf::from("./botserver-stack/conf/dns/Corefile");
        fs::create_dir_all(dns_config.parent().unwrap())?;

        let zone_file = PathBuf::from("./botserver-stack/conf/dns/botserver.local.zone");

        // Create Corefile
        let corefile = r#"botserver.local:53 {
    file /botserver-stack/conf/dns/botserver.local.zone
    reload 10s
    log
}

.:53 {
    forward . 8.8.8.8 8.8.4.4
    cache 30
    log
}
"#;

        fs::write(dns_config, corefile)?;

        // Create initial zone file
        let zone = r#"$ORIGIN botserver.local.
$TTL 60
@       IN      SOA     ns1.botserver.local. admin.botserver.local. (
                        2024010101      ; Serial
                        3600            ; Refresh
                        1800            ; Retry
                        604800          ; Expire
                        60              ; Minimum TTL
)
        IN      NS      ns1.botserver.local.
ns1     IN      A       127.0.0.1

; Static entries
api     IN      A       127.0.0.1
auth    IN      A       127.0.0.1
llm     IN      A       127.0.0.1
mail    IN      A       127.0.0.1
meet    IN      A       127.0.0.1

; Dynamic entries will be added below
"#;

        fs::write(zone_file, zone)?;
        info!("CoreDNS configured for dynamic DNS");
        Ok(())
    }

    /// Setup Directory (Zitadel) with default organization and user
    async fn setup_directory(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/directory_config.json");

        // Ensure config directory exists
        tokio::fs::create_dir_all("./config").await?;

        // Wait for Directory to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let mut setup = DirectorySetup::new(
            crate::core::urls::InternalUrls::DIRECTORY_BASE.to_string(),
            config_path,
        );

        // Create default organization
        let org_name = "default";
        let org_id = setup
            .create_organization(org_name, "Default Organization")
            .await?;
        info!("Created default organization: {}", org_name);

        // Generate secure passwords
        let admin_password = self.generate_secure_password(16);
        let user_password = self.generate_secure_password(16);

        // Save initial credentials to secure file
        let creds_path = PathBuf::from("./botserver-stack/conf/system/initial-credentials.txt");
        fs::create_dir_all(creds_path.parent().unwrap())?;
        let creds_content = format!(
            "INITIAL SETUP CREDENTIALS\n\
             ========================\n\
             Generated at: {}\n\n\
             Admin Account:\n\
             Username: admin@default\n\
             Password: {}\n\n\
             User Account:\n\
             Username: user@default\n\
             Password: {}\n\n\
             IMPORTANT: Delete this file after saving credentials securely.\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            admin_password,
            user_password
        );
        fs::write(&creds_path, creds_content)?;

        // Set restrictive permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&creds_path, fs::Permissions::from_mode(0o600))?;
        }

        // Create admin@default account for bot administration
        let admin_user = setup
            .create_user(
                &org_id,
                "admin",
                "admin@default",
                &admin_password,
                "Admin",
                "Default",
                true, // is_admin
            )
            .await?;
        info!("Created admin user: admin@default");

        // Create user@default account for regular bot usage
        let regular_user = setup
            .create_user(
                &org_id,
                "user",
                "user@default",
                &user_password,
                "User",
                "Default",
                false, // is_admin
            )
            .await?;
        info!("Created regular user: user@default");
        info!("   Regular user ID: {}", regular_user.id);

        // Create OAuth2 application for BotServer
        let (project_id, client_id, client_secret) =
            setup.create_oauth_application(&org_id).await?;
        info!("Created OAuth2 application in project: {}", project_id);

        // Save configuration
        let config = setup
            .save_config(
                org_id.clone(),
                org_name.to_string(),
                admin_user,
                client_id.clone(),
                client_secret,
            )
            .await?;

        info!("Directory initialized successfully!");
        info!("   Organization: default");
        info!("   Admin User: admin@default");
        info!("   Regular User: user@default");
        info!("   Client ID: {}", client_id);
        info!("   Login URL: {}", config.base_url);
        info!("");
        info!("   ⚠️  IMPORTANT: Initial credentials saved to:");
        info!("      ./botserver-stack/conf/system/initial-credentials.txt");
        info!("      Please save these credentials securely and delete the file.");

        Ok(())
    }

    /// Setup Email (Stalwart) with Directory integration
    pub async fn setup_email(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/email_config.json");
        let directory_config_path = PathBuf::from("./config/directory_config.json");

        let mut setup = EmailSetup::new(
            crate::core::urls::InternalUrls::DIRECTORY_BASE.to_string(),
            config_path,
        );

        // Try to integrate with Directory if it exists
        let directory_config = if directory_config_path.exists() {
            Some(directory_config_path)
        } else {
            None
        };

        let config = setup.initialize(directory_config).await?;

        info!("Email server initialized successfully!");
        info!("   SMTP: {}:{}", config.smtp_host, config.smtp_port);
        info!("   IMAP: {}:{}", config.imap_host, config.imap_port);
        info!("   Admin: {} / {}", config.admin_user, config.admin_pass);
        if config.directory_integration {
            info!("   🔗 Integrated with Directory for authentication");
        }

        Ok(())
    }

    async fn get_drive_client(config: &AppConfig) -> Client {
        let endpoint = if config.drive.server.ends_with('/') {
            config.drive.server.clone()
        } else {
            format!("{}/", config.drive.server)
        };
        let base_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region("auto")
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                config.drive.access_key.clone(),
                config.drive.secret_key.clone(),
                None,
                None,
                "static",
            ))
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&base_config)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    }

    pub async fn upload_templates_to_drive(&self, _config: &AppConfig) -> Result<()> {
        let mut conn = establish_pg_connection()?;
        self.create_bots_from_templates(&mut conn)?;
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }
        let client = Self::get_drive_client(_config).await;
        let mut read_dir = tokio::fs::read_dir(templates_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".gbai")
            {
                let bot_name = path.file_name().unwrap().to_string_lossy().to_string();
                let bucket = bot_name.trim_start_matches('/').to_string();
                if client.head_bucket().bucket(&bucket).send().await.is_err() {
                    match client.create_bucket().bucket(&bucket).send().await {
                        Ok(_) => {
                            self.upload_directory_recursive(&client, &path, &bucket, "/")
                                .await?;
                        }
                        Err(e) => {
                            error!("Failed to create bucket {}: {:?}", bucket, e);
                            return Err(anyhow::anyhow!("Failed to create bucket {}: {}. Check S3 credentials and endpoint configuration", bucket, e));
                        }
                    }
                } else {
                    trace!("Bucket {} already exists", bucket);
                }
            }
        }
        Ok(())
    }
    fn create_bots_from_templates(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use crate::shared::models::schema::bots;
        use diesel::prelude::*;
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.extension().map(|e| e == "gbai").unwrap_or(false) {
                let bot_folder = path.file_name().unwrap().to_string_lossy().to_string();
                let bot_name = bot_folder.trim_end_matches(".gbai");
                let existing: Option<String> = bots::table
                    .filter(bots::name.eq(&bot_name))
                    .select(bots::name)
                    .first(conn)
                    .optional()?;
                if existing.is_none() {
                    diesel::sql_query("INSERT INTO bots (id, name, description, llm_provider, llm_config, context_provider, context_config, is_active) VALUES (gen_random_uuid(), $1, $2, 'openai', '{\"model\": \"gpt-4\", \"temperature\": 0.7}', 'database', '{}', true)").bind::<diesel::sql_types::Text, _>(&bot_name).bind::<diesel::sql_types::Text, _>(format!("Bot for {} template", bot_name)).execute(conn)?;
                } else {
                    trace!("Bot {} already exists", bot_name);
                }
            }
        }
        Ok(())
    }
    fn upload_directory_recursive<'a>(
        &'a self,
        client: &'a Client,
        local_path: &'a Path,
        bucket: &'a str,
        prefix: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let _normalized_path = if local_path.to_string_lossy().ends_with('/') {
                local_path.to_string_lossy().to_string()
            } else {
                format!("{}/", local_path.display())
            };
            let mut read_dir = tokio::fs::read_dir(local_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let mut key = prefix.trim_matches('/').to_string();
                if !key.is_empty() {
                    key.push('/');
                }
                key.push_str(&file_name);
                if path.is_file() {
                    trace!(
                        "Uploading file {} to bucket {} with key {}",
                        path.display(),
                        bucket,
                        key
                    );
                    let content = tokio::fs::read(&path).await?;
                    client
                        .put_object()
                        .bucket(bucket)
                        .key(&key)
                        .body(content.into())
                        .send()
                        .await?;
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, bucket, &key)
                        .await?;
                }
            }
            Ok(())
        })
    }
    pub fn apply_migrations(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use diesel_migrations::HarnessWithOutput;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

        let mut harness = HarnessWithOutput::write_to_stdout(conn);
        if let Err(e) = harness.run_pending_migrations(MIGRATIONS) {
            error!("Failed to apply migrations: {}", e);
            return Err(anyhow::anyhow!("Migration error: {}", e));
        }

        Ok(())
    }

    /// Generate TLS certificates for all services
    async fn generate_certificates(&self) -> Result<()> {
        let cert_dir = PathBuf::from("./botserver-stack/conf/system/certificates");

        // Create certificate directory structure
        fs::create_dir_all(&cert_dir)?;
        fs::create_dir_all(cert_dir.join("ca"))?;

        // Check if CA already exists
        let ca_cert_path = cert_dir.join("ca/ca.crt");
        let ca_key_path = cert_dir.join("ca/ca.key");

        // CA params for issuer creation
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, "BR");
        dn.push(DnType::OrganizationName, "BotServer");
        dn.push(DnType::CommonName, "BotServer CA");
        ca_params.distinguished_name = dn;

        ca_params.not_before = time::OffsetDateTime::now_utc();
        ca_params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(3650);

        let ca_key_pair: KeyPair = if ca_cert_path.exists() && ca_key_path.exists() {
            info!("Using existing CA certificate");
            // Load existing CA key
            let key_pem = fs::read_to_string(&ca_key_path)?;
            KeyPair::from_pem(&key_pem)?
        } else {
            info!("Generating new CA certificate");
            let key_pair = KeyPair::generate()?;
            let cert = ca_params.self_signed(&key_pair)?;

            // Save CA certificate and key
            fs::write(&ca_cert_path, cert.pem())?;
            fs::write(&ca_key_path, key_pair.serialize_pem())?;

            key_pair
        };

        // Create issuer from CA params and key
        let ca_issuer = Issuer::from_params(&ca_params, &ca_key_pair);

        // Services that need certificates
        let services = vec![
            ("api", vec!["localhost", "127.0.0.1", "api.botserver.local"]),
            ("llm", vec!["localhost", "127.0.0.1", "llm.botserver.local"]),
            (
                "embedding",
                vec!["localhost", "127.0.0.1", "embedding.botserver.local"],
            ),
            (
                "qdrant",
                vec!["localhost", "127.0.0.1", "qdrant.botserver.local"],
            ),
            (
                "postgres",
                vec!["localhost", "127.0.0.1", "postgres.botserver.local"],
            ),
            (
                "redis",
                vec!["localhost", "127.0.0.1", "redis.botserver.local"],
            ),
            (
                "minio",
                vec!["localhost", "127.0.0.1", "minio.botserver.local"],
            ),
            (
                "directory",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "directory.botserver.local",
                    "auth.botserver.local",
                ],
            ),
            (
                "email",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "mail.botserver.local",
                    "smtp.botserver.local",
                    "imap.botserver.local",
                ],
            ),
            (
                "meet",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "meet.botserver.local",
                    "turn.botserver.local",
                ],
            ),
            (
                "caddy",
                vec![
                    "localhost",
                    "127.0.0.1",
                    "*.botserver.local",
                    "botserver.local",
                ],
            ),
        ];

        for (service, sans) in services {
            let service_dir = cert_dir.join(service);
            fs::create_dir_all(&service_dir)?;

            let cert_path = service_dir.join("server.crt");
            let key_path = service_dir.join("server.key");

            // Skip if certificate already exists
            if cert_path.exists() && key_path.exists() {
                trace!("Certificate for {} already exists", service);
                continue;
            }

            info!("Generating certificate for {}", service);

            // Generate service certificate
            let mut params = CertificateParams::default();
            params.not_before = time::OffsetDateTime::now_utc();
            params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);

            let mut dn = DistinguishedName::new();
            dn.push(DnType::CountryName, "BR");
            dn.push(DnType::OrganizationName, "BotServer");
            dn.push(DnType::CommonName, &format!("{}.botserver.local", service));
            params.distinguished_name = dn;

            // Add SANs
            for san in sans {
                params
                    .subject_alt_names
                    .push(rcgen::SanType::DnsName(san.to_string().try_into()?));
            }

            let key_pair = KeyPair::generate()?;
            let cert = params.signed_by(&key_pair, &ca_issuer)?;

            // Save certificate and key
            fs::write(cert_path, cert.pem())?;
            fs::write(key_path, key_pair.serialize_pem())?;

            // Copy CA cert to service directory for easy access
            fs::copy(&ca_cert_path, service_dir.join("ca.crt"))?;
        }

        info!("TLS certificates generated successfully");
        Ok(())
    }
}
