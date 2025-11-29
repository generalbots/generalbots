use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(target_os = "windows")]
use trayicon::{Icon, MenuBuilder, TrayIcon, TrayIconBuilder};

#[cfg(target_os = "macos")]
use trayicon_osx::{Icon, MenuBuilder, TrayIcon, TrayIconBuilder};

#[cfg(target_os = "linux")]
use ksni::{Icon, Tray, TrayService};

use crate::core::config::ConfigManager;
use crate::core::dns::DynamicDnsService;

pub struct TrayManager {
    hostname: Arc<RwLock<Option<String>>>,
    dns_service: Option<Arc<DynamicDnsService>>,
    config_manager: Arc<ConfigManager>,
    running_mode: RunningMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunningMode {
    Server,
    Desktop,
    Client,
}

impl TrayManager {
    pub fn new(
        config_manager: Arc<ConfigManager>,
        dns_service: Option<Arc<DynamicDnsService>>,
    ) -> Self {
        let running_mode = if cfg!(feature = "desktop") {
            RunningMode::Desktop
        } else {
            RunningMode::Server
        };

        Self {
            hostname: Arc::new(RwLock::new(None)),
            dns_service,
            config_manager,
            running_mode,
        }
    }

    pub async fn start(&self) -> Result<()> {
        match self.running_mode {
            RunningMode::Desktop => {
                self.start_desktop_mode().await?;
            }
            RunningMode::Server => {
                log::info!("Running in server mode - tray icon disabled");
            }
            RunningMode::Client => {
                log::info!("Running in client mode - tray icon minimal");
            }
        }
        Ok(())
    }

    async fn start_desktop_mode(&self) -> Result<()> {
        // Check if dynamic DNS is enabled in config
        let dns_enabled = self
            .config_manager
            .get_config("default", "dns-dynamic", Some("false"))
            .unwrap_or_else(|_| "false".to_string())
            == "true";

        if dns_enabled {
            log::info!("Dynamic DNS enabled in config, registering hostname...");
            self.register_dynamic_dns().await?;
        } else {
            log::info!("Dynamic DNS disabled in config");
        }

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            self.create_tray_icon()?;
        }

        #[cfg(target_os = "linux")]
        {
            self.create_linux_tray()?;
        }

        Ok(())
    }

    async fn register_dynamic_dns(&self) -> Result<()> {
        if let Some(dns_service) = &self.dns_service {
            // Generate hostname based on machine name
            let hostname = self.generate_hostname()?;

            // Get local IP address
            let local_ip = self.get_local_ip()?;

            // Register with DNS service
            dns_service.register_hostname(&hostname, local_ip).await?;

            // Store hostname for later use
            let mut stored_hostname = self.hostname.write().await;
            *stored_hostname = Some(hostname.clone());

            log::info!("Registered dynamic DNS: {}.botserver.local", hostname);
        }
        Ok(())
    }

    fn generate_hostname(&self) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use winapi::shared::minwindef::MAX_COMPUTERNAME_LENGTH;
            use winapi::um::sysinfoapi::GetComputerNameW;

            let mut buffer = vec![0u16; MAX_COMPUTERNAME_LENGTH as usize + 1];
            let mut size = MAX_COMPUTERNAME_LENGTH + 1;

            unsafe {
                GetComputerNameW(buffer.as_mut_ptr(), &mut size);
            }

            let hostname = String::from_utf16_lossy(&buffer[..size as usize])
                .to_lowercase()
                .replace(' ', "-");

            Ok(format!("gb-{}", hostname))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let hostname = hostname::get()?
                .to_string_lossy()
                .to_lowercase()
                .replace(' ', "-");

            Ok(format!("gb-{}", hostname))
        }
    }

    fn get_local_ip(&self) -> Result<std::net::IpAddr> {
        use local_ip_address::local_ip;

        local_ip().map_err(|e| anyhow::anyhow!("Failed to get local IP: {}", e))
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    fn create_tray_icon(&self) -> Result<()> {
        let icon_bytes = include_bytes!("../../assets/icons/tray-icon.png");
        let icon = Icon::from_png(icon_bytes)?;

        let menu = MenuBuilder::new()
            .item("General Bots", |_| {})
            .separator()
            .item("Status: Running", |_| {})
            .item(&format!("Mode: {}", self.get_mode_string()), |_| {})
            .separator()
            .item("Open Dashboard", move |_| {
                let _ = webbrowser::open("https://localhost:8080");
            })
            .item("Settings", |_| {
                // Open settings window
            })
            .separator()
            .item("About", |_| {
                // Show about dialog
            })
            .item("Quit", |_| {
                std::process::exit(0);
            })
            .build()?;

        let _tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(menu)
            .with_tooltip("General Bots")
            .build()?;

        // Keep tray icon alive
        std::thread::park();

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_linux_tray(&self) -> Result<()> {
        struct GeneralBotsTray {
            mode: String,
        }

        impl Tray for GeneralBotsTray {
            fn title(&self) -> String {
                "General Bots".to_string()
            }

            fn icon_name(&self) -> &str {
                "general-bots"
            }

            fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
                use ksni::menu::*;
                vec![
                    StandardItem {
                        label: "General Bots".to_string(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                    Separator.into(),
                    StandardItem {
                        label: "Status: Running".to_string(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                    StandardItem {
                        label: format!("Mode: {}", self.mode),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                    Separator.into(),
                    StandardItem {
                        label: "Open Dashboard".to_string(),
                        activate: Box::new(|_| {
                            let _ = webbrowser::open("https://localhost:8080");
                        }),
                        ..Default::default()
                    }
                    .into(),
                    StandardItem {
                        label: "Settings".to_string(),
                        activate: Box::new(|_| {}),
                        ..Default::default()
                    }
                    .into(),
                    Separator.into(),
                    StandardItem {
                        label: "About".to_string(),
                        activate: Box::new(|_| {}),
                        ..Default::default()
                    }
                    .into(),
                    StandardItem {
                        label: "Quit".to_string(),
                        activate: Box::new(|_| {
                            std::process::exit(0);
                        }),
                        ..Default::default()
                    }
                    .into(),
                ]
            }
        }

        let tray = GeneralBotsTray {
            mode: self.get_mode_string(),
        };

        let service = TrayService::new(tray);
        service.run();

        Ok(())
    }

    fn get_mode_string(&self) -> String {
        match self.running_mode {
            RunningMode::Desktop => "Desktop".to_string(),
            RunningMode::Server => "Server".to_string(),
            RunningMode::Client => "Client".to_string(),
        }
    }

    pub async fn update_status(&self, status: &str) -> Result<()> {
        log::info!("Tray status update: {}", status);
        Ok(())
    }

    pub async fn get_hostname(&self) -> Option<String> {
        let hostname = self.hostname.read().await;
        hostname.clone()
    }
}

// Service status monitor
pub struct ServiceMonitor {
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub port: u16,
    pub url: String,
}

impl ServiceMonitor {
    pub fn new() -> Self {
        Self {
            services: vec![
                ServiceStatus {
                    name: "API".to_string(),
                    running: false,
                    port: 8080,
                    url: "https://localhost:8080".to_string(),
                },
                ServiceStatus {
                    name: "Directory".to_string(),
                    running: false,
                    port: 8080,
                    url: "https://localhost:8080".to_string(),
                },
                ServiceStatus {
                    name: "LLM".to_string(),
                    running: false,
                    port: 8081,
                    url: "https://localhost:8081".to_string(),
                },
                ServiceStatus {
                    name: "Database".to_string(),
                    running: false,
                    port: 5432,
                    url: "postgresql://localhost:5432".to_string(),
                },
                ServiceStatus {
                    name: "Cache".to_string(),
                    running: false,
                    port: 6379,
                    url: "redis://localhost:6379".to_string(),
                },
            ],
        }
    }

    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = self.check_service(&service.url).await;
        }
        self.services.clone()
    }

    async fn check_service(&self, url: &str) -> bool {
        if url.starts_with("https://") || url.starts_with("http://") {
            match reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap()
                .get(format!("{}/health", url))
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .await
            {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}
