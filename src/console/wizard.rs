//! Startup Wizard Module
//!
//! Interactive wizard for first-run configuration or --wizard flag.
//! Guides users through:
//! - LLM provider selection
//! - Component installation choices
//! - Admin user setup
//! - Organization configuration
//! - Bot template selection

use crate::core::shared::branding::platform_name;
use crate::core::shared::version::BOTSERVER_VERSION;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

/// Wizard configuration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardConfig {
    /// Selected LLM provider
    pub llm_provider: LlmProvider,

    /// LLM API key (if applicable)
    pub llm_api_key: Option<String>,

    /// Local model path (if using local LLM)
    pub local_model_path: Option<String>,

    /// Components to install
    pub components: Vec<ComponentChoice>,

    /// Admin user configuration
    pub admin: AdminConfig,

    /// Organization configuration
    pub organization: OrgConfig,

    /// Selected bot template
    pub template: Option<String>,

    /// Installation mode
    pub install_mode: InstallMode,

    /// Data directory
    pub data_dir: PathBuf,
}

/// LLM Provider options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmProvider {
    Claude,
    OpenAI,
    Gemini,
    Local,
    None,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::Claude => write!(f, "Claude (Anthropic) - Best for complex reasoning"),
            LlmProvider::OpenAI => write!(f, "GPT-4 (OpenAI) - General purpose"),
            LlmProvider::Gemini => write!(f, "Gemini (Google) - Google integration"),
            LlmProvider::Local => write!(f, "Local (Llama/Mistral) - Privacy focused"),
            LlmProvider::None => write!(f, "None - Configure later"),
        }
    }
}

/// Component installation choices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentChoice {
    Drive,     // MinIO storage
    Email,     // Email server
    Meet,      // Video meetings (LiveKit)
    Tables,    // PostgreSQL
    Cache,     // Redis
    VectorDb,  // pgvector
    Proxy,     // Caddy reverse proxy
    Directory, // LDAP/SSO
    BotModels, // AI models server
}

impl std::fmt::Display for ComponentChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentChoice::Drive => write!(f, "Drive (MinIO) - File storage"),
            ComponentChoice::Email => write!(f, "Email Server - Send/receive emails"),
            ComponentChoice::Meet => write!(f, "Meet (LiveKit) - Video meetings"),
            ComponentChoice::Tables => write!(f, "Database (PostgreSQL) - Required"),
            ComponentChoice::Cache => write!(f, "Cache (Redis) - Sessions & queues"),
            ComponentChoice::VectorDb => write!(f, "Vector DB - AI embeddings"),
            ComponentChoice::Proxy => write!(f, "Proxy (Caddy) - HTTPS & routing"),
            ComponentChoice::Directory => write!(f, "Directory - Users & SSO"),
            ComponentChoice::BotModels => write!(f, "BotModels - Local AI models"),
        }
    }
}

/// Admin user configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminConfig {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

/// Organization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgConfig {
    pub name: String,
    pub slug: String,
    pub domain: Option<String>,
}

/// Installation mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallMode {
    Development,
    Production,
    Container,
}

impl Default for WizardConfig {
    fn default() -> Self {
        Self {
            llm_provider: LlmProvider::None,
            llm_api_key: None,
            local_model_path: None,
            components: vec![
                ComponentChoice::Tables,
                ComponentChoice::Cache,
                ComponentChoice::Drive,
            ],
            admin: AdminConfig::default(),
            organization: OrgConfig::default(),
            template: None,
            install_mode: InstallMode::Development,
            data_dir: PathBuf::from("./botserver-stack"),
        }
    }
}

/// Startup Wizard
#[derive(Debug)]
pub struct StartupWizard {
    config: WizardConfig,
    current_step: usize,
    total_steps: usize,
}

impl StartupWizard {
    pub fn new() -> Self {
        Self {
            config: WizardConfig::default(),
            current_step: 0,
            total_steps: 7,
        }
    }

    /// Run the interactive wizard
    pub fn run(&mut self) -> io::Result<WizardConfig> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();

        // Clear screen and show welcome
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        self.show_welcome(&mut stdout)?;
        self.wait_for_enter()?;

        // Step 1: Installation Mode
        self.current_step = 1;
        self.step_install_mode(&mut stdout)?;

        // Step 2: LLM Provider
        self.current_step = 2;
        self.step_llm_provider(&mut stdout)?;

        // Step 3: Components
        self.current_step = 3;
        self.step_components(&mut stdout)?;

        // Step 4: Organization
        self.current_step = 4;
        self.step_organization(&mut stdout)?;

        // Step 5: Admin User
        self.current_step = 5;
        self.step_admin_user(&mut stdout)?;

        // Step 6: Template Selection
        self.current_step = 6;
        self.step_template(&mut stdout)?;

        // Step 7: Summary & Confirm
        self.current_step = 7;
        self.step_summary(&mut stdout)?;

        terminal::disable_raw_mode()?;
        Ok(self.config.clone())
    }

    fn show_welcome(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let banner = r#"
    ╔══════════════════════════════════════════════════════════════════╗
    ║                                                                  ║
    ║     ██████╗ ███████╗███╗   ██╗███████╗██████╗  █████╗ ██╗       ║
    ║    ██╔════╝ ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔══██╗██║       ║
    ║    ██║  ███╗█████╗  ██╔██╗ ██║█████╗  ██████╔╝███████║██║       ║
    ║    ██║   ██║██╔══╝  ██║╚██╗██║██╔══╝  ██╔══██╗██╔══██║██║       ║
    ║    ╚██████╔╝███████╗██║ ╚████║███████╗██║  ██║██║  ██║███████╗  ║
    ║     ╚═════╝ ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝  ║
    ║                      ██████╗  ██████╗ ████████╗███████╗          ║
    ║                      ██╔══██╗██╔═══██╗╚══██╔══╝██╔════╝          ║
    ║                      ██████╔╝██║   ██║   ██║   ███████╗          ║
    ║                      ██╔══██╗██║   ██║   ██║   ╚════██║          ║
    ║                      ██████╔╝╚██████╔╝   ██║   ███████║          ║
    ║                      ╚═════╝  ╚═════╝    ╚═╝   ╚══════╝          ║
    ║                                                                  ║
    ╚══════════════════════════════════════════════════════════════════╝
"#;

        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print(banner),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(20, 18),
            SetForegroundColor(Color::Cyan),
            Print(format!(
                "Welcome to {} Setup Wizard v{}",
                platform_name(),
                BOTSERVER_VERSION
            )),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(20, 20),
            Print("This wizard will help you configure your bot server."),
            cursor::MoveTo(20, 21),
            Print("You can re-run this wizard anytime with: "),
            SetForegroundColor(Color::Yellow),
            Print("botserver --wizard"),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(20, 24),
            SetForegroundColor(Color::DarkGrey),
            Print("Press ENTER to continue..."),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn show_step_header(&self, stdout: &mut io::Stdout, title: &str) -> io::Result<()> {
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Progress bar
        let progress = format!("Step {}/{}: {}", self.current_step, self.total_steps, title);
        let bar_width = 50;
        let filled = (self.current_step * bar_width) / self.total_steps;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("╔"),
            Print("═".repeat(bar_width + 2)),
            Print("╗\n"),
            Print("║ "),
            SetForegroundColor(Color::Green),
            Print("█".repeat(filled)),
            SetForegroundColor(Color::DarkGrey),
            Print("░".repeat(bar_width - filled)),
            SetForegroundColor(Color::Cyan),
            Print(" ║\n"),
            Print("╚"),
            Print("═".repeat(bar_width + 2)),
            Print("╝"),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(0, 4),
            SetForegroundColor(Color::White),
            Print(format!("  {}\n", progress)),
            ResetColor,
            Print("\n")
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn step_install_mode(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "Installation Mode")?;

        let options = vec![
            (
                "Development",
                "Local development with hot reload",
                InstallMode::Development,
            ),
            (
                "Production",
                "Optimized for production servers",
                InstallMode::Production,
            ),
            (
                "Container",
                "Docker/LXC container deployment",
                InstallMode::Container,
            ),
        ];

        let selected = self.select_option(stdout, &options, 0)?;
        self.config.install_mode = options[selected].2.clone();

        Ok(())
    }

    fn step_llm_provider(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "AI/LLM Provider")?;

        execute!(
            stdout,
            cursor::MoveTo(2, 7),
            Print("Select your preferred AI provider:"),
            cursor::MoveTo(2, 8),
            SetForegroundColor(Color::DarkGrey),
            Print("(You can use multiple providers later)"),
            ResetColor
        )?;

        let options = vec![
            (
                "Claude (Anthropic)",
                "Best reasoning, 200K context - Recommended",
                LlmProvider::Claude,
            ),
            (
                "GPT-4 (OpenAI)",
                "Widely compatible, good all-around",
                LlmProvider::OpenAI,
            ),
            (
                "Gemini (Google)",
                "Great for Google Workspace integration",
                LlmProvider::Gemini,
            ),
            (
                "Local Models",
                "Llama, Mistral - Full privacy, no API costs",
                LlmProvider::Local,
            ),
            (
                "Skip for now",
                "Configure AI providers later",
                LlmProvider::None,
            ),
        ];

        let selected = self.select_option(stdout, &options, 0)?;
        self.config.llm_provider = options[selected].2.clone();

        // Ask for API key if needed
        if self.config.llm_provider != LlmProvider::Local
            && self.config.llm_provider != LlmProvider::None
        {
            terminal::disable_raw_mode()?;
            execute!(
                stdout,
                cursor::MoveTo(2, 20),
                Print("Enter API key (or press Enter to skip): ")
            )?;
            stdout.flush()?;

            let mut api_key = String::new();
            io::stdin().read_line(&mut api_key)?;
            let api_key = api_key.trim().to_string();

            if !api_key.is_empty() {
                self.config.llm_api_key = Some(api_key);
            }
            terminal::enable_raw_mode()?;
        }

        if self.config.llm_provider == LlmProvider::Local {
            terminal::disable_raw_mode()?;
            execute!(
                stdout,
                cursor::MoveTo(2, 20),
                Print("Enter model path (default: ./models/llama-3.1-8b): ")
            )?;
            stdout.flush()?;

            let mut model_path = String::new();
            io::stdin().read_line(&mut model_path)?;
            let model_path = model_path.trim().to_string();

            self.config.local_model_path = Some(if model_path.is_empty() {
                "./models/llama-3.1-8b".to_string()
            } else {
                model_path
            });
            terminal::enable_raw_mode()?;
        }

        Ok(())
    }

    fn step_components(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "Components to Install")?;

        execute!(
            stdout,
            cursor::MoveTo(2, 7),
            Print("Select components to install (Space to toggle, Enter to confirm):"),
            cursor::MoveTo(2, 8),
            SetForegroundColor(Color::DarkGrey),
            Print("PostgreSQL and Redis are required and pre-selected"),
            ResetColor
        )?;

        let components = vec![
            (ComponentChoice::Tables, true, false), // required, can't toggle
            (ComponentChoice::Cache, true, false),  // required, can't toggle
            (ComponentChoice::Drive, true, true),   // default on
            (ComponentChoice::VectorDb, true, true), // default on
            (ComponentChoice::Email, false, true),  // default off
            (ComponentChoice::Meet, false, true),   // default off
            (ComponentChoice::Proxy, true, true),   // default on
            (ComponentChoice::Directory, false, true), // default off
            (ComponentChoice::BotModels, false, true), // default off
        ];

        let selected = self.multi_select(stdout, &components)?;
        self.config.components = selected;

        Ok(())
    }

    fn step_organization(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "Organization Setup")?;

        terminal::disable_raw_mode()?;

        execute!(stdout, cursor::MoveTo(2, 7), Print("Organization name: "))?;
        stdout.flush()?;

        let mut org_name = String::new();
        io::stdin().read_line(&mut org_name)?;
        self.config.organization.name = org_name.trim().to_string();

        // Generate slug from name
        self.config.organization.slug = self
            .config
            .organization
            .name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();

        execute!(
            stdout,
            cursor::MoveTo(2, 9),
            Print(format!("Slug ({}): ", self.config.organization.slug))
        )?;
        stdout.flush()?;

        let mut slug = String::new();
        io::stdin().read_line(&mut slug)?;
        let slug = slug.trim();
        if !slug.is_empty() {
            self.config.organization.slug = slug.to_string();
        }

        execute!(
            stdout,
            cursor::MoveTo(2, 11),
            Print("Domain (optional, e.g., example.com): ")
        )?;
        stdout.flush()?;

        let mut domain = String::new();
        io::stdin().read_line(&mut domain)?;
        let domain = domain.trim();
        if !domain.is_empty() {
            self.config.organization.domain = Some(domain.to_string());
        }

        terminal::enable_raw_mode()?;
        Ok(())
    }

    fn step_admin_user(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "Admin User")?;

        terminal::disable_raw_mode()?;

        execute!(stdout, cursor::MoveTo(2, 7), Print("Admin username: "))?;
        stdout.flush()?;

        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        self.config.admin.username = username.trim().to_string();

        execute!(stdout, cursor::MoveTo(2, 9), Print("Admin email: "))?;
        stdout.flush()?;

        let mut email = String::new();
        io::stdin().read_line(&mut email)?;
        self.config.admin.email = email.trim().to_string();

        execute!(stdout, cursor::MoveTo(2, 11), Print("Admin display name: "))?;
        stdout.flush()?;

        let mut display_name = String::new();
        io::stdin().read_line(&mut display_name)?;
        self.config.admin.display_name = display_name.trim().to_string();

        execute!(stdout, cursor::MoveTo(2, 13), Print("Admin password: "))?;
        stdout.flush()?;

        // Read password (in production, use rpassword for hidden input)
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        self.config.admin.password = password.trim().to_string();

        terminal::enable_raw_mode()?;
        Ok(())
    }

    fn step_template(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "Bot Template")?;

        execute!(
            stdout,
            cursor::MoveTo(2, 7),
            Print("Select a template for your first bot:"),
        )?;

        let options = vec![
            ("default", "Basic bot with weather, email, and tools"),
            ("crm", "Customer relationship management"),
            ("edu", "Educational/course management"),
            ("store", "E-commerce bot"),
            ("hr", "Human resources assistant"),
            ("healthcare", "Healthcare appointment scheduling"),
            ("none", "Start from scratch"),
        ];

        let templates: Vec<(&str, &str, Option<String>)> = options
            .iter()
            .map(|(name, desc)| {
                (
                    *name,
                    *desc,
                    if *name == "none" {
                        None
                    } else {
                        Some(name.to_string())
                    },
                )
            })
            .collect();

        let selected = self.select_option(stdout, &templates, 0)?;
        self.config.template = templates[selected].2.clone();

        Ok(())
    }

    fn step_summary(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.show_step_header(stdout, "Configuration Summary")?;

        let mode = match self.config.install_mode {
            InstallMode::Development => "Development",
            InstallMode::Production => "Production",
            InstallMode::Container => "Container",
        };

        let llm = match &self.config.llm_provider {
            LlmProvider::Claude => "Claude (Anthropic)",
            LlmProvider::OpenAI => "GPT-4 (OpenAI)",
            LlmProvider::Gemini => "Gemini (Google)",
            LlmProvider::Local => "Local Models",
            LlmProvider::None => "Not configured",
        };

        execute!(
            stdout,
            cursor::MoveTo(2, 7),
            SetForegroundColor(Color::Cyan),
            Print("═══════════════════════════════════════════════════"),
            ResetColor,
            cursor::MoveTo(2, 9),
            Print(format!("  Installation Mode:  {}", mode)),
            cursor::MoveTo(2, 10),
            Print(format!("  LLM Provider:       {}", llm)),
            cursor::MoveTo(2, 11),
            Print(format!(
                "  Organization:       {}",
                self.config.organization.name
            )),
            cursor::MoveTo(2, 12),
            Print(format!(
                "  Admin User:         {}",
                self.config.admin.username
            )),
            cursor::MoveTo(2, 13),
            Print(format!(
                "  Template:           {}",
                self.config.template.as_deref().unwrap_or("None")
            )),
            cursor::MoveTo(2, 14),
            Print(format!(
                "  Components:         {}",
                self.config.components.len()
            )),
            cursor::MoveTo(2, 16),
            SetForegroundColor(Color::Cyan),
            Print("═══════════════════════════════════════════════════"),
            ResetColor,
            cursor::MoveTo(2, 18),
            Print("Components to install:"),
        )?;

        for (i, component) in self.config.components.iter().enumerate() {
            execute!(
                stdout,
                cursor::MoveTo(4, 19 + i as u16),
                SetForegroundColor(Color::Green),
                Print("✓ "),
                ResetColor,
                Print(format!("{}", component))
            )?;
        }

        let last_line = 19 + self.config.components.len() as u16 + 2;
        execute!(
            stdout,
            cursor::MoveTo(2, last_line),
            SetForegroundColor(Color::Yellow),
            Print("Press ENTER to apply configuration, or ESC to cancel"),
            ResetColor
        )?;

        stdout.flush()?;

        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "Wizard cancelled",
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn select_option<T: Clone>(
        &self,
        stdout: &mut io::Stdout,
        options: &[(&str, &str, T)],
        default: usize,
    ) -> io::Result<usize> {
        let mut selected = default;
        let start_row = 10;

        loop {
            for (i, (name, desc, _)) in options.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(4, start_row + i as u16))?;

                if i == selected {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Green),
                        Print("▶ "),
                        Print(format!("{:<25}", name)),
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!(" {}", desc)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout,
                        Print("  "),
                        Print(format!("{:<25}", name)),
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!(" {}", desc)),
                        ResetColor
                    )?;
                }
            }

            stdout.flush()?;

            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < options.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
                    }
                    _ => {}
                }
            }
        }

        Ok(selected)
    }

    fn multi_select(
        &self,
        stdout: &mut io::Stdout,
        options: &[(ComponentChoice, bool, bool)], // (component, selected, can_toggle)
    ) -> io::Result<Vec<ComponentChoice>> {
        let mut selected: Vec<bool> = options.iter().map(|(_, s, _)| *s).collect();
        let mut cursor = 0;
        let start_row = 10;

        loop {
            for (i, (component, _, can_toggle)) in options.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(4, start_row + i as u16))?;

                let checkbox = if selected[i] { "[✓]" } else { "[ ]" };
                let prefix = if i == cursor { "▶" } else { " " };

                if !can_toggle {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!("{} {} {} (required)", prefix, checkbox, component)),
                        ResetColor
                    )?;
                } else if i == cursor {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Green),
                        Print(format!("{} {} {}", prefix, checkbox, component)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout,
                        Print(format!("{} {} {}", prefix, checkbox, component)),
                    )?;
                }
            }

            execute!(
                stdout,
                cursor::MoveTo(4, start_row + options.len() as u16 + 2),
                SetForegroundColor(Color::DarkGrey),
                Print("Use ↑↓ to navigate, SPACE to toggle, ENTER to confirm"),
                ResetColor
            )?;

            stdout.flush()?;

            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        if cursor > 0 {
                            cursor -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if cursor < options.len() - 1 {
                            cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if options[cursor].2 {
                            // can_toggle
                            selected[cursor] = !selected[cursor];
                        }
                    }
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
                    }
                    _ => {}
                }
            }
        }

        Ok(options
            .iter()
            .enumerate()
            .filter(|(i, _)| selected[*i])
            .map(|(_, (c, _, _))| c.clone())
            .collect())
    }

    fn wait_for_enter(&self) -> io::Result<()> {
        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if code == KeyCode::Enter {
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Save wizard configuration to file
pub fn save_wizard_config(config: &WizardConfig, path: &str) -> io::Result<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Load wizard configuration from file
pub fn load_wizard_config(path: &str) -> io::Result<WizardConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: WizardConfig =
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

/// Check if wizard should run (no botserver-stack exists)
pub fn should_run_wizard() -> bool {
    !std::path::Path::new("./botserver-stack").exists()
        && !std::path::Path::new("/opt/gbo").exists()
}

/// Apply wizard configuration - create directories, config files, etc.
pub fn apply_wizard_config(config: &WizardConfig) -> io::Result<()> {
    use std::fs;

    // Create data directory
    fs::create_dir_all(&config.data_dir)?;

    // Create subdirectories
    let subdirs = ["bots", "logs", "cache", "uploads", "config"];
    for subdir in &subdirs {
        fs::create_dir_all(config.data_dir.join(subdir))?;
    }

    // Save configuration
    save_wizard_config(
        config,
        &config.data_dir.join("config/wizard.toml").to_string_lossy(),
    )?;

    // Create .env file
    let mut env_content = String::new();
    env_content.push_str(&format!(
        "# Generated by {} Setup Wizard\n\n",
        platform_name()
    ));
    env_content.push_str(&format!("INSTALL_MODE={:?}\n", config.install_mode));
    env_content.push_str(&format!("ORG_NAME={}\n", config.organization.name));
    env_content.push_str(&format!("ORG_SLUG={}\n", config.organization.slug));

    if let Some(domain) = &config.organization.domain {
        env_content.push_str(&format!("DOMAIN={}\n", domain));
    }

    match &config.llm_provider {
        LlmProvider::Claude => env_content.push_str("LLM_PROVIDER=anthropic\n"),
        LlmProvider::OpenAI => env_content.push_str("LLM_PROVIDER=openai\n"),
        LlmProvider::Gemini => env_content.push_str("LLM_PROVIDER=google\n"),
        LlmProvider::Local => env_content.push_str("LLM_PROVIDER=local\n"),
        LlmProvider::None => {}
    }

    if let Some(api_key) = &config.llm_api_key {
        env_content.push_str(&format!("LLM_API_KEY={}\n", api_key));
    }

    if let Some(model_path) = &config.local_model_path {
        env_content.push_str(&format!("LOCAL_MODEL_PATH={}\n", model_path));
    }

    fs::write(config.data_dir.join(".env"), env_content)?;

    println!("\n✅ Configuration applied successfully!");
    println!("   Data directory: {}", config.data_dir.display());
    println!("\n   Next steps:");
    println!("   1. Run: botserver start");
    println!("   2. Open: http://localhost:4242");
    println!("   3. Login with: {}", config.admin.username);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WizardConfig::default();
        assert_eq!(config.llm_provider, LlmProvider::None);
        assert!(!config.components.is_empty());
    }

    #[test]
    fn test_slug_generation() {
        let mut config = WizardConfig::default();
        config.organization.name = "My Test Company".to_string();
        config.organization.slug = config
            .organization
            .name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();

        assert_eq!(config.organization.slug, "my-test-company");
    }
}
