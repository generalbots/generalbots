use super::*;

impl Default for ProductConfig {
    fn default() -> Self {
        let mut apps = HashSet::new();
        // #1811 — without a `.product` file every registry app is enabled. The
        // core surfaces (about/admin/settings) stay implicit and are handled by
        // the catalog, while product files override this list explicitly.
        for app in &[
            "analytics",
            "attendant",
            "automations",
            "banking",
            "billing",
            "browser",
            "calculator",
            "calendar",
            "campaigns",
            "canvas",
            "chat",
            "clock-widget",
            "compliance",
            "concierge",
            "crm",
            "database",
            "designer",
            "desktop",
            "docs",
            "drive",
            "editor",
            "fraud",
            "goals",
            "handoff",
            "hr",
            "integrations",
            "jukebox",
            "kyc",
            "learn",
            "lists",
            "mail",
            "meet",
            "memory",
            "minutes",
            "monitoring",
            "notes",
            "notepad",
            "o365",
            "paper",
            "people",
            "photos",
            "plan",
            "player",
            "plugins",
            "pos",
            "products",
            "project",
            "recycle",
            "research",
            "retail",
            "sheet",
            "slides",
            "social",
            "store",
            "tasks",
            "tax",
            "templates",
            "terminal",
            "tickets",
            "timeclock",
            "timer",
            "vibe",
            "video",
            "vision",
            "weather",
            "workspace",
        ] {
            apps.insert(app.to_string());
        }

        Self {
            name: "General Bots".to_string(),
            apps,
            preview_apps: HashSet::new(),
            theme: "sentient".to_string(),
            logo: None,
            favicon: None,
            primary_color: None,
            support_email: None,
            docs_url: None,
            copyright: None,
            search_enabled: Some(false),
            menu_launcher_enabled: Some(false),
            sidebar: None,
            suite_default_app: None,
            hidden_apps: HashSet::new(),
            no_security: false,
            shell_mode: false,
            edition: None,
            public_base_url: None,
        }
    }
}
impl ProductConfig {
    /// Load configuration from .product file
    ///
    /// Search order (first hit wins):
    /// 1. CWD-relative paths (legacy dev layout)
    /// 2. `BOTSERVER_STACK_PATH` / `GBO_STACK_PATH` (desktop sidecar)
    /// 3. Next to the running executable (packaged install)
    pub fn load() -> Result<Self, ProductConfigError> {
        #[cfg(feature = "desktop-local")]
        let names: &[&str] = &[".product.desktop-local", ".product"];
        #[cfg(not(feature = "desktop-local"))]
        let names: &[&str] = &[".product"];

        // Phase 1: CWD-relative paths (original behaviour).
        #[cfg(feature = "desktop-local")]
        let relative: &[&str] = &[
            ".product.desktop-local",
            "./botserver/.product.desktop-local",
            ".product",
            "./botserver/.product",
            "../.product",
        ];
        #[cfg(not(feature = "desktop-local"))]
        let relative: &[&str] = &[".product", "./botserver/.product", "../.product"];

        for path in relative {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path).map_err(ProductConfigError::IoError)?;
                info!("Loaded product configuration from: {path}");
                return Self::parse(&content);
            }
        }

        // Phase 2: inside the stack directory (desktop sidecar sets
        // BOTSERVER_STACK_PATH; the product file lives at the stack root).
        let stack = std::env::var("BOTSERVER_STACK_PATH")
            .or_else(|_| std::env::var("GBO_STACK_PATH"))
            .ok()
            .filter(|s| !s.trim().is_empty());
        if let Some(ref stack_dir) = stack {
            let base = Path::new(stack_dir);
            for name in names {
                let candidate = base.join(name);
                if candidate.exists() {
                    let content =
                        fs::read_to_string(&candidate).map_err(ProductConfigError::IoError)?;
                    info!(
                        "Loaded product configuration from stack: {}",
                        candidate.display()
                    );
                    return Self::parse(&content);
                }
            }
        }

        // Phase 3: next to the running executable (packaged MSI/NSIS/deb).
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                for name in names {
                    let candidate = exe_dir.join(name);
                    if candidate.exists() {
                        let content =
                            fs::read_to_string(&candidate).map_err(ProductConfigError::IoError)?;
                        info!(
                            "Loaded product configuration beside executable: {}",
                            candidate.display()
                        );
                        return Self::parse(&content);
                    }
                }
            }
        }

        warn!("No .product file found, using default configuration");
        Ok(Self::default())
    }

    /// Parse configuration from string content
    pub fn parse(content: &str) -> Result<Self, ProductConfigError> {
        let mut config = Self::default();
        let mut apps_specified = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "name" => {
                        if !value.is_empty() {
                            config.name = value.to_string();
                        }
                    }
                    "apps" => {
                        apps_specified = true;
                        config.apps.clear();
                        for app in value.split(',') {
                            let app = app.trim().to_lowercase();
                            if !app.is_empty() {
                                config.apps.insert(app);
                            }
                        }
                    }
                    "preview_apps" => {
                        config.preview_apps.clear();
                        for app in value.split(',') {
                            let app = app.trim().to_lowercase();
                            if !app.is_empty() {
                                config.preview_apps.insert(app);
                            }
                        }
                    }
                    "theme" => {
                        if !value.is_empty() {
                            config.theme = value.to_string();
                        }
                    }
                    "logo" => {
                        if !value.is_empty() {
                            config.logo = Some(value.to_string());
                        }
                    }
                    "favicon" => {
                        if !value.is_empty() {
                            config.favicon = Some(value.to_string());
                        }
                    }
                    "primary_color" => {
                        if !value.is_empty() {
                            config.primary_color = Some(value.to_string());
                        }
                    }
                    "support_email" => {
                        if !value.is_empty() {
                            config.support_email = Some(value.to_string());
                        }
                    }
                    "docs_url" => {
                        if !value.is_empty() {
                            config.docs_url = Some(value.to_string());
                        }
                    }
                    "copyright" => {
                        if !value.is_empty() {
                            config.copyright = Some(value.to_string());
                        }
                    }
                    "search_enabled" => {
                        let enabled = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                        config.search_enabled = Some(enabled);
                    }
                    "menu_launcher_enabled" => {
                        let enabled = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                        config.menu_launcher_enabled = Some(enabled);
                    }
                    "sidebar" => {
                        let enabled = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                        config.sidebar = Some(enabled);
                    }
                    "suite_default_app" => {
                        if !value.is_empty() {
                            config.suite_default_app = Some(value.to_lowercase());
                        }
                    }
                    "hidden_apps" => {
                        config.hidden_apps.clear();
                        for app in value.split(',') {
                            let app = app.trim();
                            if !app.is_empty() {
                                config.hidden_apps.insert(app.to_lowercase());
                            }
                        }
                    }
                    "no_security" => {
                        config.no_security = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                    }
                    "shell_mode" => {
                        config.shell_mode = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                    }
                    // #1889 — explicit deployment edition. Unknown values are
                    // rejected instead of silently shaping the settings tree.
                    "edition" => {
                        let normalized = value.trim().to_lowercase();
                        if normalized == "cloud" || normalized == "desktop" || normalized == "os" {
                            config.edition = Some(normalized);
                        } else {
                            warn!("Unknown product edition: {}", value);
                        }
                    }
                    "public_base_url" => {
                        if !value.is_empty() {
                            config.public_base_url =
                                Some(value.trim().trim_end_matches('/').to_string());
                        }
                    }
                    _ => {
                        warn!("Unknown product configuration key: {}", key);
                    }
                }
            }
        }

        if !apps_specified {
            info!("No apps specified in .product, all apps enabled by default");
        }

        info!(
            "Product config loaded: name='{}', apps={:?}, theme='{}'",
            config.name, config.apps, config.theme
        );

        Ok(config)
    }

    /// Check if an app is enabled
    pub fn is_app_enabled(&self, app: &str) -> bool {
        let id = app.to_lowercase();
        self.apps.contains(&id) && !self.hidden_apps.contains(&id)
    }

    /// Check whether an app is a preview application, i.e. one that is only
    /// surfaced while Preview mode is on (#1348). Preview applications are not
    /// part of `apps`, so `is_app_enabled` still answers `false` for them: the
    /// switch decides what is visible, never whether the app is installed.
    pub fn is_app_preview(&self, app: &str) -> bool {
        self.preview_apps.contains(&app.to_lowercase())
    }

    /// Check whether the profile hides an app even when it is compiled and
    /// routed (#2097: hidden wins over both the `apps=` list and the registry
    /// launcher split).
    pub fn is_app_hidden(&self, app: &str) -> bool {
        let id = app.to_lowercase();
        self.hidden_apps.contains(&id)
    }

    /// Preview applications, sorted so the catalog and the manifest are stable.
    pub fn get_preview_apps(&self) -> Vec<String> {
        let mut preview: Vec<String> = self.preview_apps.iter().cloned().collect();
        preview.sort();
        preview
    }

    /// Get the product name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the default theme
    pub fn get_theme(&self) -> &str {
        &self.theme
    }

    /// Replace "General Bots" with the product name in a string
    pub fn replace_branding(&self, text: &str) -> String {
        text.replace("General Bots", &self.name)
            .replace("general bots", &self.name.to_lowercase())
            .replace("GENERAL BOTS", &self.name.to_uppercase())
    }

    /// Get copyright text with year substitution
    pub fn get_copyright(&self) -> String {
        let year = chrono::Utc::now().format("%Y").to_string();
        let template = self
            .copyright
            .as_deref()
            .unwrap_or("© {year} {name}. All rights reserved.");

        template
            .replace("{year}", &year)
            .replace("{name}", &self.name)
    }

    /// Get all enabled apps as a vector
    pub fn get_enabled_apps(&self) -> Vec<String> {
        self.apps.iter().cloned().collect()
    }

    /// Apps the profile hides even when they are compiled and routed.
    pub fn get_hidden_apps(&self) -> Vec<String> {
        let mut hidden: Vec<String> = self.hidden_apps.iter().cloned().collect();
        hidden.sort();
        hidden
    }

    /// Reload configuration from file
    pub fn reload() -> Result<(), ProductConfigError> {
        let new_config = Self::load()?;
        let mut config = PRODUCT_CONFIG
            .write()
            .map_err(|_| ProductConfigError::LockError)?;
        *config = new_config;
        info!("Product configuration reloaded");
        Ok(())
    }
}
