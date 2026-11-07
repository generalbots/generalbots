use anyhow::Result;
use botvideo::safe_command::SafeCommand;
use std::sync::Arc;
#[cfg(feature = "desktop-tray")]
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
#[cfg(feature = "desktop-tray")]
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;
use tokio::sync::RwLock;

mod menu;
mod service;

pub use menu::quit_requested;
pub use service::{ServiceMonitor, ServiceStatus};

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
    NewChat,
    Files,
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

            let new_chat =
                match MenuItem::with_id(app, menu::NEW_CHAT_ID, "New Chat", true, None::<&str>) {
                    Ok(item) => item,
                    Err(e) => {
                        log::error!("Failed to create tray New Chat item: {e}");
                        return;
                    }
                };
            let files = match MenuItem::with_id(app, menu::FILES_ID, "Files", true, None::<&str>) {
                Ok(item) => item,
                Err(e) => {
                    log::error!("Failed to create tray Files item: {e}");
                    return;
                }
            };
            let settings =
                match MenuItem::with_id(app, menu::SETTINGS_ID, "Settings", true, None::<&str>) {
                    Ok(item) => item,
                    Err(e) => {
                        log::error!("Failed to create tray Settings item: {e}");
                        return;
                    }
                };
            let about = match MenuItem::with_id(app, menu::ABOUT_ID, "About", true, None::<&str>) {
                Ok(item) => item,
                Err(e) => {
                    log::error!("Failed to create tray About item: {e}");
                    return;
                }
            };
            let quit = match MenuItem::with_id(app, menu::QUIT_ID, "Quit", true, None::<&str>) {
                Ok(item) => item,
                Err(e) => {
                    log::error!("Failed to create tray Quit item: {e}");
                    return;
                }
            };
            let first_separator = match PredefinedMenuItem::separator(app) {
                Ok(item) => item,
                Err(e) => {
                    log::error!("Failed to create tray menu separator: {e}");
                    return;
                }
            };
            let second_separator = match PredefinedMenuItem::separator(app) {
                Ok(item) => item,
                Err(e) => {
                    log::error!("Failed to create tray menu separator: {e}");
                    return;
                }
            };

            let tray_menu = match Menu::with_items(
                app,
                &[
                    &new_chat,
                    &files,
                    &first_separator,
                    &settings,
                    &about,
                    &second_separator,
                    &quit,
                ],
            ) {
                Ok(menu) => menu,
                Err(e) => {
                    log::error!("Failed to create tray menu: {e}");
                    return;
                }
            };

            // The official bundled General Bots icon. It is decoded at compile
            // time from the same PNG the installer uses, so the tray can never
            // drift into a generated placeholder again (#1826).
            let icon = tauri::include_image!("icons/icon.png");

            let tray_builder = TrayIconBuilder::with_id("main")
                .menu(&tray_menu)
                .tooltip("General Bots")
                .icon(icon)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if !menu::handle_menu_event(app, event.id().as_ref()) {
                        log::debug!("Unhandled tray menu event: {}", event.id().as_ref());
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => menu::toggle_main_window(app),
                        TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => menu::new_chat(app),
                        _ => {}
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
                    log::error!("Failed to build tray icon: {e}");
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
            #[cfg(feature = "desktop-tray")]
            if let Ok(handle) = self.tray_handle.lock() {
                if let Some(tray) = handle.as_ref() {
                    if let Err(e) = tray.set_tooltip(Some(tooltip)) {
                        log::warn!("Failed to set tray tooltip: {e}");
                    }
                }
            }
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

    pub fn handle_event(&self, app: &AppHandle, event: TrayEvent) {
        let mode = self.get_mode_string();
        match event {
            TrayEvent::NewChat => {
                log::info!("Tray event: New Chat (mode: {mode})");
                menu::new_chat(app);
            }
            TrayEvent::Files => {
                log::info!("Tray event: Files (mode: {mode})");
                menu::open_app(app, "drive", false);
            }
            TrayEvent::Open => {
                log::info!("Tray event: Open main window (mode: {mode})");
                menu::show_main_window(app);
            }
            TrayEvent::Settings => {
                log::info!("Tray event: Open settings (mode: {mode})");
                menu::open_app(app, "settings", false);
            }
            TrayEvent::About => {
                log::info!("Tray event: Show about dialog (mode: {mode})");
                menu::open_app(app, "about", false);
            }
            TrayEvent::Quit => {
                log::info!("Tray event: Quit application (mode: {mode})");
                menu::quit(app);
            }
        }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}
