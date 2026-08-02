use super::{log_panel::init_logger, ActivePanel, ChatPanel, FileTree, StatusPanel, XtreeUI};
use crate::core::shared::state::AppState;
use color_eyre::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, sync::Arc};

impl XtreeUI {
    pub fn new() -> Self {
        let log_panel = Arc::new(std::sync::Mutex::new(super::LogPanel::new()));
        Self {
            app_state: None,
            file_tree: None,
            status_panel: None,
            log_panel,
            chat_panel: None,
            editor: None,
            active_panel: ActivePanel::Logs,
            should_quit: false,
            progress_channel: None,
            state_channel: None,
            bootstrap_status: "Initializing...".to_string(),
        }
    }

    pub fn set_progress_channel(
        &mut self,
        rx: Arc<
            tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<crate::BootstrapProgress>>,
        >,
    ) {
        self.progress_channel = Some(rx);
    }

    pub fn set_state_channel(
        &mut self,
        rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Arc<AppState>>>>,
    ) {
        self.state_channel = Some(rx);
    }

    pub fn set_app_state(&mut self, app_state: Arc<AppState>) {
        self.file_tree = Some(FileTree::new(app_state.clone()));
        self.status_panel = Some(StatusPanel::new(app_state.clone()));
        self.chat_panel = Some(ChatPanel::new(app_state.clone()));
        self.app_state = Some(app_state);
        self.active_panel = ActivePanel::FileTree;
        self.bootstrap_status = "Ready".to_string();
    }

    pub fn start_ui(&mut self) -> Result<()> {
        color_eyre::install()?;
        if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            return Ok(());
        }
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        if let Err(e) = init_logger(self.log_panel.clone()) {
            eprintln!("Warning: Could not initialize UI logger: {e}");
        }
        log::set_max_level(log::LevelFilter::Trace);
        let result = self.run_event_loop(&mut terminal);
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        result
    }

    fn run_event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        let mut last_update = std::time::Instant::now();
        let update_interval = std::time::Duration::from_millis(1000);
        let mut cursor_blink = false;
        let mut last_blink = std::time::Instant::now();
        let rt = tokio::runtime::Runtime::new()?;
        loop {
            if self.app_state.is_none() {
                if let Some(ref state_rx) = self.state_channel {
                    if let Ok(mut rx) = state_rx.try_lock() {
                        if let Ok(app_state) = rx.try_recv() {
                            self.file_tree = Some(FileTree::new(app_state.clone()));
                            self.status_panel = Some(StatusPanel::new(app_state.clone()));
                            self.chat_panel = Some(ChatPanel::new(app_state.clone()));
                            self.app_state = Some(app_state);
                            self.active_panel = ActivePanel::FileTree;
                            self.bootstrap_status = "Ready".to_string();

                            if let Ok(mut log_panel) = self.log_panel.lock() {
                                log_panel.add_log("AppState received - UI fully initialized");
                            }
                        }
                    }
                }
            }

            if let Some(ref progress_rx) = self.progress_channel {
                if let Ok(mut rx) = progress_rx.try_lock() {
                    while let Ok(progress) = rx.try_recv() {
                        self.bootstrap_status = match progress {
                            crate::BootstrapProgress::StartingBootstrap => {
                                "Starting bootstrap...".to_string()
                            }
                            crate::BootstrapProgress::InstallingComponent(name) => {
                                format!("Installing: {name}")
                            }
                            crate::BootstrapProgress::StartingComponent(name) => {
                                format!("Starting: {name}")
                            }
                            crate::BootstrapProgress::UploadingTemplates => {
                                "Uploading templates...".to_string()
                            }
                            crate::BootstrapProgress::ConnectingDatabase => {
                                "Connecting to database...".to_string()
                            }
                            crate::BootstrapProgress::StartingLLM => {
                                "Starting LLM servers...".to_string()
                            }
                            crate::BootstrapProgress::BootstrapComplete => {
                                "Bootstrap complete".to_string()
                            }
                            crate::BootstrapProgress::BootstrapError(msg) => {
                                format!("Error: {msg}")
                            }
                        };
                    }
                }
            }
            if last_blink.elapsed() >= std::time::Duration::from_millis(500) {
                cursor_blink = !cursor_blink;
                last_blink = std::time::Instant::now();
            }
            terminal.draw(|f| self.render(f, cursor_blink))?;
            if self.app_state.is_some() && last_update.elapsed() >= update_interval {
                if let Err(e) = rt.block_on(self.update_data()) {
                    if let Ok(mut log_panel) = self.log_panel.lock() {
                        log_panel.add_log(&format!("Update error: {e}"));
                    }
                }
                last_update = std::time::Instant::now();
            }
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if let Err(e) = rt.block_on(self.handle_input(key.code, key.modifiers)) {
                        if let Ok(mut log_panel) = self.log_panel.lock() {
                            log_panel.add_log(&format!("Input error: {e}"));
                        }
                    }
                    if self.should_quit {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
