use anyhow::Result;
use botvideo::safe_command::SafeCommand;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, Emitter, Manager};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

#[derive(Clone)]
pub struct TrayManager {
    hostname: Arc<RwLock<Option<String>>>,
    running_mode: RunningMode,
    tray_active: Arc<RwLock<bool>>,
    #[cfg(feature = "desktop-tray")]
    tray_handle: Arc<std::sync::Mutex<Option<TrayIcon>>>,
}

impl std::fmt::Debug for TrayManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayManager")
            .field("hostname", &self.hostname)
            .field("running_mode", &self.running_mode)
            .field("tray_active", &self.tray_active)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunningMode {
    Server,
    Desktop,
    Client,
}

#[derive(Debug, Clone, Copy)]
pub enum TrayEvent {
    Open,
    Settings,
    About,
    Quit,
}

impl TrayManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: RunningMode::Desktop,
            tray_active: Arc::new(RwLock::new(false)),
            #[cfg(feature = "desktop-tray")]
            tray_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    #[must_use]
    pub fn with_mode(mode: RunningMode) -> Self {
        Self {
            hostname: Arc::new(RwLock::new(None)),
            running_mode: mode,
            tray_active: Arc::new(RwLock::new(false)),
            #[cfg(feature = "desktop-tray")]
            tray_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub async fn start(&self, app: &AppHandle) -> Result<()> {
        match self.running_mode {
            RunningMode::Desktop => {
                self.start_desktop_mode(app).await?;
            }
            RunningMode::Server => {
                log::info!("Running in server mode - tray icon disabled");
            }
            RunningMode::Client => {
                self.start_client_mode(app).await;
            }
        }
        Ok(())
    }

    pub async fn start_desktop_mode(&self, app: &AppHandle) -> Result<()> {
        log::info!("Starting desktop mode tray icon");
        let mut active = self.tray_active.write().await;
        *active = true;
        drop(active);

        self.setup_tray(app);
        Ok(())
    }

    fn setup_tray(&self, app: &AppHandle) {
        #[cfg(feature = "desktop-tray")]
        {
            log::info!(
                "Initializing unified system tray via tauri::tray for mode: {:?}",
                self.running_mode
            );

            // Build menu items: New Chat, Files, ---, Settings, About, ---, Quit
            macro_rules! menu_item {
                ($id:expr, $label:expr) => {
                    match MenuItem::with_id(app, $id, $label, true, None::<&str>) {
                        Ok(item) => item,
                        Err(e) => {
                            log::error!("Failed to create menu item '{}': {e}", $label);
                            return;
                        }
                    }
                };
            }
            macro_rules! separator {
                () => {
                    match PredefinedMenuItem::separator(app) {
                        Ok(item) => item,
                        Err(e) => {
                            log::error!("Failed to create separator: {e}");
                            return;
                        }
                    }
                };
            }

            let new_chat_i = menu_item!("new-chat", "New Chat");
            let files_i    = menu_item!("files",    "Files");
            let sep1       = separator!();
            let settings_i = menu_item!("settings", "Settings");
            let about_i    = menu_item!("about",    "About");
            let sep2       = separator!();
            let quit_i     = menu_item!("quit",     "Quit");

            let tray_menu = match Menu::new(app) {
                Ok(menu) => menu,
                Err(e) => {
                    log::error!("Failed to create tray menu: {e}");
                    return;
                }
            };
            let _ = tray_menu.append(&new_chat_i);
            let _ = tray_menu.append(&files_i);
            let _ = tray_menu.append(&sep1);
            let _ = tray_menu.append(&settings_i);
            let _ = tray_menu.append(&about_i);
            let _ = tray_menu.append(&sep2);
            let _ = tray_menu.append(&quit_i);

            // Use the bundled app icon (defined in tauri.conf.json) instead of a
            // generated red square.  Fall back to a plain white square only if the
            // runtime returns None (should never happen in a normal Tauri build).
            let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
                let w: u32 = 32;
                let h: u32 = 32;
                let rgba = vec![255u8; (w * h * 4) as usize]; // opaque white
                tauri::image::Image::new_owned(rgba, w, h)
            });

            let tray_builder = TrayIconBuilder::with_id("main")
                .menu(&tray_menu)
                .tooltip("General Bots")
                .icon(icon)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "new-chat" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            let _ = app.emit("tray:open-chat", ());
                        }
                        "files" => {
                            let _ = app.emit("tray:open-files", ());
                        }
                        "settings" => {
                            let _ = app.emit("tray:open-settings", ());
                        }
                        "about" => {
                            let _ = app.emit("tray:open-about", ());
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        other => {
                            log::warn!("Unknown tray menu event id: {other}");
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });

            match tray_builder.build(app) {
                Ok(tray) => {
                    if let Ok(mut handle) = self.tray_handle.lock() {
                        *handle = Some(tray);
                        log::info!("Tray icon created successfully");
                    }
                }
                Err(e) => {
                    log::error!("Failed to build tray icon: {}", e);
                }
            }
        }
    }

    async fn start_client_mode(&self, app: &AppHandle) {
        log::info!("Starting client mode with minimal tray");
        let mut active = self.tray_active.write().await;
        *active = true;
        drop(active);
        self.setup_tray(app);
    }

    #[must_use]
    pub fn get_mode_string(&self) -> String {
        match self.running_mode {
            RunningMode::Desktop => "Desktop".to_string(),
            RunningMode::Server => "Server".to_string(),
            RunningMode::Client => "Client".to_string(),
        }
    }

    pub async fn update_status(&self, status: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        let is_active = *active;
        drop(active);

        if is_active {
            log::info!("Tray status: {status}");
        }
        Ok(())
    }

    pub async fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        let is_active = *active;
        drop(active);

        if is_active {
            log::debug!("Tray tooltip: {tooltip}");
        }
        Ok(())
    }

    pub async fn show_notification(&self, title: &str, body: &str) -> Result<()> {
        let active = self.tray_active.read().await;
        let is_active = *active;
        drop(active);

        if is_active {
            log::info!("Notification: {title} - {body}");

            #[cfg(target_os = "linux")]
            {
                if let Ok(cmd) = SafeCommand::new("notify-send")
                    .and_then(|c| c.arg(title))
                    .and_then(|c| c.arg(body))
                {
                    let _ = cmd.execute();
                }
            }

            #[cfg(target_os = "macos")]
            {
                let script = format!("display notification \"{body}\" with title \"{title}\"");
                if let Ok(cmd) = SafeCommand::new("osascript")
                    .and_then(|c| c.arg("-e"))
                    .and_then(|c| c.arg(&script))
                {
                    let _ = cmd.execute();
                }
            }
        }
        Ok(())
    }

    pub async fn get_hostname(&self) -> Option<String> {
        let hostname = self.hostname.read().await;
        hostname.clone()
    }

    pub async fn set_hostname(&self, new_hostname: String) {
        let mut hostname = self.hostname.write().await;
        *hostname = Some(new_hostname);
    }

    pub async fn stop(&self) {
        let mut active = self.tray_active.write().await;
        *active = false;
        drop(active);
        log::info!("Tray manager stopped");
    }

    pub async fn is_active(&self) -> bool {
        let active = self.tray_active.read().await;
        let result = *active;
        drop(active);
        result
    }

    pub fn handle_event(&self, event: TrayEvent) {
        let mode = self.get_mode_string();
        match event {
            TrayEvent::Open => {
                log::info!("Tray event: Open main window (mode: {mode})");
            }
            TrayEvent::Settings => {
                log::info!("Tray event: Open settings (mode: {mode})");
            }
            TrayEvent::About => {
                log::info!("Tray event: Show about dialog (mode: {mode})");
            }
            TrayEvent::Quit => {
                log::info!("Tray event: Quit application (mode: {mode})");
            }
        }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ServiceMonitor {
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub port: u16,
    pub url: String,
}

impl ServiceMonitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: vec![
                ServiceStatus {
                    name: "API".to_string(),
                    running: false,
                    port: 8080,
                    url: "http://localhost:9000".to_string(),
                },
                ServiceStatus {
                    name: "UI".to_string(),
                    running: false,
                    port: 3000,
                    url: "http://localhost:3000".to_string(),
                },
            ],
        }
    }

    pub fn add_service(&mut self, name: &str, port: u16) {
        self.services.push(ServiceStatus {
            name: name.to_string(),
            running: false,
            port,
            url: format!("http://localhost:{port}"),
        });
    }

    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        for service in &mut self.services {
            service.running = Self::check_service(&service.url).await;
        }
        self.services.clone()
    }

    pub async fn check_service(url: &str) -> bool {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        else {
            return false;
        };

        let health_url = format!("{}/health", url.trim_end_matches('/'));

        client
            .get(&health_url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }

    #[must_use]
    pub fn get_service(&self, name: &str) -> Option<&ServiceStatus> {
        self.services.iter().find(|s| s.name == name)
    }

    #[must_use]
    pub fn all_running(&self) -> bool {
        self.services.iter().all(|s| s.running)
    }

    #[must_use]
    pub fn any_running(&self) -> bool {
        self.services.iter().any(|s| s.running)
    }
}

impl Default for ServiceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
