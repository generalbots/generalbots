use crate::core::shared::platform_name;
use crate::core::shared::BOTSERVER_VERSION;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardConfig {
    pub llm_provider: LlmProvider,

    pub llm_api_key: Option<String>,

    pub local_model_path: Option<String>,

    pub components: Vec<ComponentChoice>,

    pub admin: AdminConfig,

    pub organization: OrgConfig,

    pub template: Option<String>,

    pub install_mode: InstallMode,

    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
            Self::Claude => write!(f, "Claude (Anthropic) - Best for complex reasoning"),
            Self::OpenAI => write!(f, "GPT-4 (OpenAI) - General purpose"),
            Self::Gemini => write!(f, "Gemini (Google) - Google integration"),
            Self::Local => write!(f, "Local (Llama/Mistral) - Privacy focused"),
            Self::None => write!(f, "None - Configure later"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentChoice {
    Drive,
    Email,
    Meet,
    Tables,
    Cache,
    VectorDb,
    Proxy,
    Directory,
    BotModels,
}

impl std::fmt::Display for ComponentChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Drive => write!(f, "Drive (MinIO) - File storage"),
            Self::Email => write!(f, "Email Server - Send/receive emails"),
            Self::Meet => write!(f, "Meet (LiveKit) - Video meetings"),
            Self::Tables => write!(f, "Database (PostgreSQL) - Required"),
            Self::Cache => write!(f, "Cache (Redis) - Sessions & queues"),
            Self::VectorDb => write!(f, "Vector DB - AI embeddings"),
            Self::Proxy => write!(f, "Proxy (Caddy) - HTTPS & routing"),
            Self::Directory => write!(f, "Directory - Users & SSO"),
            Self::BotModels => write!(f, "BotModels - Local AI models"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminConfig {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgConfig {
    pub name: String,
    pub slug: String,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct StartupWizard {
    config: WizardConfig,
    current_step: usize,
    total_steps: usize,
}

impl Default for StartupWizard {
    fn default() -> Self {
        Self {
            config: WizardConfig::default(),
            current_step: 0,
            total_steps: 7,
        }
    }
}

impl StartupWizard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) -> io::Result<WizardConfig> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();

        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        self.show_welcome(&mut stdout)?;
        self.wait_for_enter()?;

        self.current_step = 1;
        self.step_install_mode(&mut stdout)?;

        self.current_step = 2;
        self.step_llm_provider(&mut stdout)?;

        self.current_step = 3;
        self.step_components(&mut stdout)?;

        self.current_step = 4;
        self.step_organization(&mut stdout)?;

        self.current_step = 5;
        self.step_admin_user(&mut stdout)?;

        self.current_step = 6;
        self.step_template(&mut stdout)?;

        self.current_step = 7;
        self.step_summary(&mut stdout)?;

        terminal::disable_raw_mode()?;
        Ok(self.config.clone())
    }

    fn show_welcome(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        let _ = self; // kept for API consistency
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let banner = r"
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
";

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
            (ComponentChoice::Tables, true, false),
            (ComponentChoice::Cache, true, false),
            (ComponentChoice::Drive, true, true),
            (ComponentChoice::VectorDb, true, true),
            (ComponentChoice::Email, false, true),
            (ComponentChoice::Meet, false, true),
            (ComponentChoice::Proxy, true, true),
            (ComponentChoice::Directory, false, true),
            (ComponentChoice::BotModels, false, true),
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
                        Some((*name).to_string())
                    },
                )
            })
            .collect();

        let selected = self.select_option(stdout, &templates, 0)?;
        self.config.template.clone_from(&templates[selected].2);

        Ok(())
    }

    fn step_summary(&self, stdout: &mut io::Stdout) -> io::Result<()> {
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
                Print("* "),
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
        let _ = self; // kept for API consistency
        let mut selected = default;
        let start_row = 10;

        loop {
            for (i, (name, desc, _)) in options.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(4, start_row + i as u16))?;

                if i == selected {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Green),
                        Print("> "),
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
                        selected = selected.saturating_sub(1);
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
        options: &[(ComponentChoice, bool, bool)],
    ) -> io::Result<Vec<ComponentChoice>> {
        let _ = self; // kept for API consistency
        let mut selected: Vec<bool> = options.iter().map(|(_, s, _)| *s).collect();
        let mut cursor = 0;
        let start_row = 10;

        loop {
            for (i, (component, _, can_toggle)) in options.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(4, start_row + i as u16))?;

                let checkbox = if selected[i] { "[*]" } else { "[ ]" };
                let prefix = if i == cursor { ">" } else { " " };

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
                        cursor = cursor.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if cursor < options.len() - 1 {
                            cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if options[cursor].2 {
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
        let _ = self; // kept for API consistency
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

pub fn save_wizard_config(config: &WizardConfig, path: &str) -> io::Result<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn load_wizard_config(path: &str) -> io::Result<WizardConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: WizardConfig =
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

pub fn should_run_wizard() -> bool {
    !std::path::Path::new("./botserver-stack").exists()
        && !std::path::Path::new("/opt/gbo").exists()
}

pub fn apply_wizard_config(config: &WizardConfig) -> io::Result<()> {
    use std::fs;

    fs::create_dir_all(&config.data_dir)?;

    let subdirs = ["bots", "logs", "cache", "uploads", "config"];
    for subdir in &subdirs {
        fs::create_dir_all(config.data_dir.join(subdir))?;
    }

    save_wizard_config(
        config,
        &config.data_dir.join("config/wizard.toml").to_string_lossy(),
    )?;

    let mut env_content = String::new();
    let _ = writeln!(
        env_content,
        "# Generated by {} Setup Wizard\n",
        platform_name()
    );
    let _ = writeln!(env_content, "INSTALL_MODE={:?}", config.install_mode);
    let _ = writeln!(env_content, "ORG_NAME={}", config.organization.name);
    let _ = writeln!(env_content, "ORG_SLUG={}", config.organization.slug);

    if let Some(domain) = &config.organization.domain {
        let _ = writeln!(env_content, "DOMAIN={domain}");
    }

    match &config.llm_provider {
        LlmProvider::Claude => env_content.push_str("LLM_PROVIDER=anthropic\n"),
        LlmProvider::OpenAI => env_content.push_str("LLM_PROVIDER=openai\n"),
        LlmProvider::Gemini => env_content.push_str("LLM_PROVIDER=google\n"),
        LlmProvider::Local => env_content.push_str("LLM_PROVIDER=local\n"),
        LlmProvider::None => {}
    }

    if let Some(api_key) = &config.llm_api_key {
        let _ = writeln!(env_content, "LLM_API_KEY={api_key}");
    }

    if let Some(model_path) = &config.local_model_path {
        let _ = writeln!(env_content, "LOCAL_MODEL_PATH={model_path}");
    }

    fs::write(config.data_dir.join(".env"), env_content)?;

    println!("\n Configuration applied successfully!");
    println!("   Data directory: {}", config.data_dir.display());
    println!("\n   Next steps:");
    println!("   1. Run: botserver start");
    println!("   2. Open: http://localhost:4242");
    println!("   3. Login with: {}", config.admin.username);

    Ok(())
}
