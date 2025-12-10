use crate::drive::convert_tree_to_items;
use crate::shared::state::AppState;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
mod chat_panel;
mod editor;
pub mod file_tree;
mod log_panel;
mod status_panel;
pub mod wizard;
use chat_panel::ChatPanel;
use editor::Editor;
use file_tree::{FileTree, TreeNode};
use log_panel::{init_logger, LogPanel};
use status_panel::StatusPanel;
#[derive(Debug)]
pub struct XtreeUI {
    app_state: Option<Arc<AppState>>,
    file_tree: Option<FileTree>,
    status_panel: Option<StatusPanel>,
    log_panel: Arc<Mutex<LogPanel>>,
    chat_panel: Option<ChatPanel>,
    editor: Option<Editor>,
    active_panel: ActivePanel,
    should_quit: bool,
    progress_channel: Option<
        Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<crate::BootstrapProgress>>>,
    >,
    state_channel: Option<Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Arc<AppState>>>>>,
    bootstrap_status: String,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum ActivePanel {
    FileTree,
    Editor,
    Status,
    Logs,
    Chat,
}
impl XtreeUI {
    pub fn new() -> Self {
        let log_panel = Arc::new(Mutex::new(LogPanel::new()));
        Self {
            app_state: None,
            file_tree: None,
            status_panel: None,
            log_panel: log_panel.clone(),
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
        rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<crate::BootstrapProgress>>>,
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
        // Initialize UI logger to capture logs for the log panel
        // This works because env_logger is not initialized when console UI is enabled
        if let Err(e) = init_logger(self.log_panel.clone()) {
            eprintln!("Warning: Could not initialize UI logger: {}", e);
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
            // Poll for AppState updates from the main thread
            if self.app_state.is_none() {
                if let Some(ref state_rx) = self.state_channel {
                    if let Ok(mut rx) = state_rx.try_lock() {
                        if let Ok(app_state) = rx.try_recv() {
                            // Initialize all panels with the new state
                            self.file_tree = Some(FileTree::new(app_state.clone()));
                            self.status_panel = Some(StatusPanel::new(app_state.clone()));
                            self.chat_panel = Some(ChatPanel::new(app_state.clone()));
                            self.app_state = Some(app_state);
                            self.active_panel = ActivePanel::FileTree;
                            self.bootstrap_status = "Ready".to_string();

                            // Log that we received the state
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
                                format!("Installing: {}", name)
                            }
                            crate::BootstrapProgress::StartingComponent(name) => {
                                format!("Starting: {}", name)
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
                                format!("Error: {}", msg)
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
                    let mut log_panel = self.log_panel.lock().unwrap();
                    log_panel.add_log(&format!("Update error: {}", e));
                }
                last_update = std::time::Instant::now();
            }
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if let Err(e) = rt.block_on(self.handle_input(key.code, key.modifiers)) {
                        let mut log_panel = self.log_panel.lock().unwrap();
                        log_panel.add_log(&format!("Input error: {}", e));
                    }
                    if self.should_quit {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    fn render(&mut self, f: &mut Frame, cursor_blink: bool) {
        let bg = Color::Rgb(0, 30, 100);
        let border_active = Color::Rgb(85, 255, 255);
        let border_inactive = Color::Rgb(170, 170, 170);
        let text = Color::Rgb(255, 255, 255);
        let highlight = Color::Rgb(0, 170, 170);
        let title_bg = Color::Rgb(170, 170, 170);
        let title_fg = Color::Rgb(0, 0, 0);
        if self.app_state.is_none() {
            self.render_loading(f, bg, text, border_active, title_bg, title_fg);
            return;
        }
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(12),
            ])
            .split(f.area());
        self.render_header(f, main_chunks[0], bg, title_bg, title_fg);
        if self.editor.is_some() {
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(40),
                    Constraint::Percentage(35),
                ])
                .split(main_chunks[1]);
            self.render_file_tree(
                f,
                content_chunks[0],
                bg,
                text,
                border_active,
                border_inactive,
                highlight,
                title_bg,
                title_fg,
            );
            if let Some(editor) = &self.editor {
                self.render_editor(
                    f,
                    content_chunks[1],
                    editor,
                    bg,
                    text,
                    border_active,
                    border_inactive,
                    highlight,
                    title_bg,
                    title_fg,
                    cursor_blink,
                );
            }
            self.render_chat(
                f,
                content_chunks[2],
                bg,
                text,
                border_active,
                border_inactive,
                highlight,
                title_bg,
                title_fg,
            );
        } else {
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(40),
                    Constraint::Percentage(35),
                ])
                .split(main_chunks[1]);
            self.render_file_tree(
                f,
                content_chunks[0],
                bg,
                text,
                border_active,
                border_inactive,
                highlight,
                title_bg,
                title_fg,
            );
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_chunks[1]);
            self.render_status(
                f,
                right_chunks[0],
                bg,
                text,
                border_active,
                border_inactive,
                highlight,
                title_bg,
                title_fg,
            );
            self.render_chat(
                f,
                content_chunks[2],
                bg,
                text,
                border_active,
                border_inactive,
                highlight,
                title_bg,
                title_fg,
            );
        }
        self.render_logs(
            f,
            main_chunks[2],
            bg,
            text,
            border_active,
            border_inactive,
            highlight,
            title_bg,
            title_fg,
        );
    }
    fn render_header(
        &self,
        f: &mut Frame,
        area: Rect,
        _bg: Color,
        title_bg: Color,
        title_fg: Color,
    ) {
        let block = Block::default().style(Style::default().bg(title_bg));
        f.render_widget(block, area);
        let title = if self.app_state.is_some() {
            let components = vec![
                ("Tables", "postgres", "5432"),
                ("Cache", "valkey-server", "6379"),
                ("Drive", "minio", "9000"),
                ("LLM", "llama-server", "8081"),
            ];
            let statuses: Vec<String> = components
                .iter()
                .map(|(comp_name, process, _port)| {
                    let status = if status_panel::StatusPanel::check_component_running(process) {
                        format!(" {}", comp_name)
                    } else {
                        format!(" {}", comp_name)
                    };
                    status
                })
                .collect();
            format!(" GENERAL BOTS ┃ {} ", statuses.join(" ┃ "))
        } else {
            " GENERAL BOTS ".to_string()
        };
        let title_len = title.len() as u16;
        let centered_x = (area.width.saturating_sub(title_len)) / 2;
        let centered_y = area.y + 1;
        let x = area.x + centered_x;
        let max_width = area.width.saturating_sub(x - area.x);
        let width = title_len.min(max_width);
        let title_span = Span::styled(
            title,
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(
            Paragraph::new(Line::from(title_span)),
            Rect {
                x,
                y: centered_y,
                width,
                height: 1,
            },
        );
    }
    fn render_loading(
        &self,
        f: &mut Frame,
        bg: Color,
        text: Color,
        border: Color,
        title_bg: Color,
        title_fg: Color,
    ) {
        // Same layout as the real UI - header, content, logs
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(12),
            ])
            .split(f.area());

        // Render header with GENERAL BOTS title
        let header_block = Block::default().style(Style::default().bg(title_bg));
        f.render_widget(header_block, main_chunks[0]);

        let title = " GENERAL BOTS ";
        let title_len = title.len() as u16;
        let centered_x = (main_chunks[0].width.saturating_sub(title_len)) / 2;
        let title_span = Span::styled(
            title,
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(
            Paragraph::new(Line::from(title_span)),
            Rect {
                x: main_chunks[0].x + centered_x,
                y: main_chunks[0].y + 1,
                width: title_len,
                height: 1,
            },
        );

        // Content area - same 3 columns as real UI
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(40),
                Constraint::Percentage(35),
            ])
            .split(main_chunks[1]);

        // Left panel - FILE EXPLORER (loading)
        let file_block = Block::default()
            .title(Span::styled(
                " FILE EXPLORER ",
                Style::default().fg(title_fg).bg(title_bg),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(bg));
        let file_text = Paragraph::new("\n\n     Loading files...")
            .block(file_block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(file_text, content_chunks[0]);

        // Middle panel - STATUS (loading with bootstrap info)
        let middle_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[1]);

        let status_block = Block::default()
            .title(Span::styled(
                " STATUS ",
                Style::default().fg(title_fg).bg(title_bg),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(bg));

        let status_text = format!(
            "\n   {}\n\n  Components:\n    ○ Vault\n    ○ Database\n    ○ Drive\n    ○ Cache\n    ○ LLM",
            self.bootstrap_status
        );
        let status_para = Paragraph::new(status_text)
            .block(status_block)
            .style(Style::default().fg(text));
        f.render_widget(status_para, middle_chunks[0]);

        // Empty space below status (will be editor later)
        let empty_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(bg));
        f.render_widget(empty_block, middle_chunks[1]);

        // Right panel - CHAT (loading)
        let chat_block = Block::default()
            .title(Span::styled(
                " CHAT ",
                Style::default().fg(title_fg).bg(title_bg),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(bg));
        let chat_text = Paragraph::new("\n\n     Connecting...")
            .block(chat_block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(chat_text, content_chunks[2]);

        // Bottom panel - LOGS (showing bootstrap progress)
        let logs_block = Block::default()
            .title(Span::styled(
                " SYSTEM LOGS ",
                Style::default().fg(title_fg).bg(title_bg),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(bg));

        // Calculate visible lines for log panel (area height minus borders)
        let logs_visible_lines = main_chunks[2].height.saturating_sub(2) as usize;
        let logs_content = if let Ok(panel) = self.log_panel.lock() {
            panel.render(logs_visible_lines)
        } else {
            String::from("  Waiting for logs...")
        };

        let logs_para = Paragraph::new(logs_content)
            .block(logs_block)
            .style(Style::default().fg(text))
            .wrap(Wrap { trim: false });
        f.render_widget(logs_para, main_chunks[2]);
    }
    fn render_file_tree(
        &self,
        f: &mut Frame,
        area: Rect,
        bg: Color,
        text: Color,
        border_active: Color,
        border_inactive: Color,
        highlight: Color,
        title_bg: Color,
        title_fg: Color,
    ) {
        if let Some(file_tree) = &self.file_tree {
            let items = file_tree.render_items();
            let selected = file_tree.selected_index();
            // Use convert_tree_to_items to get detailed file metadata
            let _file_items = convert_tree_to_items(file_tree);
            let list_items: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(idx, (display, _))| {
                    let style = if idx == selected {
                        Style::default()
                            .bg(highlight)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(text)
                    };
                    ListItem::new(Line::from(Span::styled(display.clone(), style)))
                })
                .collect();
            let is_active = self.active_panel == ActivePanel::FileTree;
            let border_color = if is_active {
                border_active
            } else {
                border_inactive
            };
            let title_style = if is_active {
                Style::default()
                    .fg(title_fg)
                    .bg(title_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(title_fg).bg(title_bg)
            };
            let block = Block::default()
                .title(Span::styled(" FILE EXPLORER ", title_style))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(bg));
            let list = List::new(list_items).block(block);
            f.render_widget(list, area);
        }
    }
    fn render_status(
        &mut self,
        f: &mut Frame,
        area: Rect,
        bg: Color,
        text: Color,
        border_active: Color,
        border_inactive: Color,
        _highlight: Color,
        title_bg: Color,
        title_fg: Color,
    ) {
        let selected_bot_opt = self.file_tree.as_ref().and_then(|ft| ft.get_selected_bot());
        let status_text = if let Some(status_panel) = &mut self.status_panel {
            match selected_bot_opt {
                Some(bot) => status_panel.render(Some(bot)),
                None => status_panel.render(None),
            }
        } else {
            "Waiting for initialization...".to_string()
        };
        let is_active = self.active_panel == ActivePanel::Status;
        let border_color = if is_active {
            border_active
        } else {
            border_inactive
        };
        let title_style = if is_active {
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(title_fg).bg(title_bg)
        };
        let block = Block::default()
            .title(Span::styled(" SYSTEM STATUS ", title_style))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(bg));
        let paragraph = Paragraph::new(status_text)
            .block(block)
            .style(Style::default().fg(text))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }
    fn render_editor(
        &self,
        f: &mut Frame,
        area: Rect,
        editor: &Editor,
        bg: Color,
        text: Color,
        border_active: Color,
        border_inactive: Color,
        _highlight: Color,
        title_bg: Color,
        title_fg: Color,
        cursor_blink: bool,
    ) {
        let is_active = self.active_panel == ActivePanel::Editor;
        let border_color = if is_active {
            border_active
        } else {
            border_inactive
        };
        let title_style = if is_active {
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(title_fg).bg(title_bg)
        };
        let title_text = format!(" EDITOR: {} ", editor.file_path());
        let block = Block::default()
            .title(Span::styled(title_text, title_style))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(bg));
        let content = editor.render(cursor_blink);
        let paragraph = Paragraph::new(content)
            .block(block)
            .style(Style::default().fg(text))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }
    fn render_chat(
        &self,
        f: &mut Frame,
        area: Rect,
        bg: Color,
        text: Color,
        border_active: Color,
        border_inactive: Color,
        _highlight: Color,
        title_bg: Color,
        title_fg: Color,
    ) {
        if let Some(chat_panel) = &self.chat_panel {
            let is_active = self.active_panel == ActivePanel::Chat;
            let border_color = if is_active {
                border_active
            } else {
                border_inactive
            };
            let title_style = if is_active {
                Style::default()
                    .fg(title_fg)
                    .bg(title_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(title_fg).bg(title_bg)
            };
            let selected_bot = if let Some(file_tree) = &self.file_tree {
                file_tree
                    .get_selected_bot()
                    .unwrap_or("No bot selected".to_string())
            } else {
                "No bot selected".to_string()
            };
            let title_text = format!(" CHAT: {} ", selected_bot);
            let block = Block::default()
                .title(Span::styled(title_text, title_style))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(bg));
            let content = chat_panel.render();
            let paragraph = Paragraph::new(content)
                .block(block)
                .style(Style::default().fg(text))
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        }
    }
    fn render_logs(
        &self,
        f: &mut Frame,
        area: Rect,
        bg: Color,
        text: Color,
        border_active: Color,
        border_inactive: Color,
        _highlight: Color,
        title_bg: Color,
        title_fg: Color,
    ) {
        // Calculate visible lines (area height minus borders)
        let visible_lines = area.height.saturating_sub(2) as usize;

        let log_panel = self.log_panel.try_lock();
        let (log_lines, can_scroll_up, can_scroll_down, logs_count, auto_scroll) =
            if let Ok(panel) = log_panel {
                let (content, up, down) = panel.render_with_scroll_indicator(visible_lines);
                (
                    content,
                    up,
                    down,
                    panel.logs_count(),
                    panel.is_auto_scroll(),
                )
            } else {
                ("Loading logs...".to_string(), false, false, 0, true)
            };

        let is_active = self.active_panel == ActivePanel::Logs;
        let border_color = if is_active {
            border_active
        } else {
            border_inactive
        };
        let title_style = if is_active {
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(title_fg).bg(title_bg)
        };

        // Build title with scroll indicators
        let scroll_indicator = if can_scroll_up && can_scroll_down {
            " [^v] "
        } else if can_scroll_up {
            " [^] "
        } else if can_scroll_down {
            " [v] "
        } else {
            ""
        };

        let auto_indicator = if auto_scroll { "" } else { " [SCROLL] " };
        let title_text = format!(
            " SYSTEM LOGS ({}) {}{}",
            logs_count, scroll_indicator, auto_indicator
        );

        let block = Block::default()
            .title(Span::styled(title_text, title_style))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(bg));
        let paragraph = Paragraph::new(log_lines)
            .block(block)
            .style(Style::default().fg(text))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }
    async fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key {
                KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.should_quit = true;
                    return Ok(());
                }
                KeyCode::Char('s') => {
                    if let Some(editor) = &mut self.editor {
                        if let Some(app_state) = &self.app_state {
                            if let Err(e) = editor.save(app_state).await {
                                let mut log_panel = self.log_panel.lock().unwrap();
                                log_panel.add_log(&format!("Save failed: {}", e));
                            } else {
                                let mut log_panel = self.log_panel.lock().unwrap();
                                log_panel.add_log(&format!("Saved: {}", editor.file_path()));
                            }
                        }
                    }
                    return Ok(());
                }
                KeyCode::Char('w') => {
                    if self.editor.is_some() {
                        self.editor = None;
                        self.active_panel = ActivePanel::FileTree;
                        let mut log_panel = self.log_panel.lock().unwrap();
                        log_panel.add_log("Closed editor");
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
        if self.app_state.is_none() {
            return Ok(());
        }
        match self.active_panel {
            ActivePanel::FileTree => match key {
                KeyCode::Up => {
                    if let Some(file_tree) = &mut self.file_tree {
                        file_tree.move_up();
                    }
                }
                KeyCode::Down => {
                    if let Some(file_tree) = &mut self.file_tree {
                        file_tree.move_down();
                    }
                }
                KeyCode::Enter => {
                    if let Err(e) = self.handle_tree_enter().await {
                        let mut log_panel = self.log_panel.lock().unwrap();
                        log_panel.add_log(&format!("Enter error: {}", e));
                    }
                }
                KeyCode::Backspace => {
                    if let Some(file_tree) = &mut self.file_tree {
                        if file_tree.go_up() {
                            if let Err(e) = file_tree.refresh_current().await {
                                let mut log_panel = self.log_panel.lock().unwrap();
                                log_panel.add_log(&format!("Navigation error: {}", e));
                            }
                        }
                    }
                }
                KeyCode::Tab => {
                    if self.editor.is_some() {
                        self.active_panel = ActivePanel::Editor;
                    } else {
                        self.active_panel = ActivePanel::Logs;
                    }
                }
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::F(5) => {
                    if let Some(file_tree) = &mut self.file_tree {
                        if let Err(e) = file_tree.refresh_current().await {
                            let mut log_panel = self.log_panel.lock().unwrap();
                            log_panel.add_log(&format!("Refresh failed: {}", e));
                        } else {
                            let mut log_panel = self.log_panel.lock().unwrap();
                            log_panel.add_log("Refreshed");
                        }
                    }
                }
                _ => {}
            },
            ActivePanel::Editor => {
                if let Some(editor) = &mut self.editor {
                    match key {
                        KeyCode::Up => editor.move_up(),
                        KeyCode::Down => editor.move_down(),
                        KeyCode::Left => editor.move_left(),
                        KeyCode::Right => editor.move_right(),
                        KeyCode::PageUp => editor.page_up(),
                        KeyCode::PageDown => editor.page_down(),
                        KeyCode::Home => {
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                editor.goto_line(1);
                            }
                        }
                        KeyCode::End => {
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                let line_count = editor.file_path().lines().count().max(1);
                                editor.goto_line(line_count);
                            }
                        }
                        KeyCode::Char(c) => editor.insert_char(c),
                        KeyCode::Backspace => editor.backspace(),
                        KeyCode::Enter => editor.insert_newline(),
                        KeyCode::Tab => {
                            self.active_panel = ActivePanel::Chat;
                        }
                        KeyCode::Esc => {
                            self.editor = None;
                            self.active_panel = ActivePanel::FileTree;
                            let mut log_panel = self.log_panel.lock().unwrap();
                            log_panel.add_log("Closed editor");
                        }
                        _ => {}
                    }
                }
            }
            ActivePanel::Logs => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Ok(mut panel) = self.log_panel.lock() {
                        panel.scroll_up(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Ok(mut panel) = self.log_panel.lock() {
                        panel.scroll_down(1, 10); // approximate visible lines
                    }
                }
                KeyCode::PageUp => {
                    if let Ok(mut panel) = self.log_panel.lock() {
                        panel.page_up(10);
                    }
                }
                KeyCode::PageDown => {
                    if let Ok(mut panel) = self.log_panel.lock() {
                        panel.page_down(10);
                    }
                }
                KeyCode::Home => {
                    if let Ok(mut panel) = self.log_panel.lock() {
                        panel.scroll_to_top();
                    }
                }
                KeyCode::End => {
                    if let Ok(mut panel) = self.log_panel.lock() {
                        panel.scroll_to_bottom();
                    }
                }
                KeyCode::Tab => {
                    self.active_panel = ActivePanel::Chat;
                }
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                _ => {}
            },
            ActivePanel::Chat => match key {
                KeyCode::Tab => {
                    if self.editor.is_some() {
                        self.active_panel = ActivePanel::Editor;
                    } else {
                        self.active_panel = ActivePanel::FileTree;
                    }
                }
                KeyCode::Enter => {
                    if let (Some(chat_panel), Some(file_tree), Some(app_state)) =
                        (&mut self.chat_panel, &self.file_tree, &self.app_state)
                    {
                        if let Some(bot_name) = file_tree.get_selected_bot() {
                            if let Err(e) = chat_panel.send_message(&bot_name, app_state).await {
                                let mut log_panel = self.log_panel.lock().unwrap();
                                log_panel.add_log(&format!("Chat error: {}", e));
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(chat_panel) = &mut self.chat_panel {
                        chat_panel.add_char(c);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(chat_panel) = &mut self.chat_panel {
                        chat_panel.backspace();
                    }
                }
                _ => {}
            },
            ActivePanel::Status => match key {
                KeyCode::Tab => {
                    self.active_panel = ActivePanel::Chat;
                }
                _ => {}
            },
        }
        Ok(())
    }
    async fn handle_tree_enter(&mut self) -> Result<()> {
        if let (Some(file_tree), Some(app_state)) = (&mut self.file_tree, &self.app_state) {
            if let Some(node) = file_tree.get_selected_node().cloned() {
                match node {
                    TreeNode::Bucket { name, .. } => {
                        file_tree.enter_bucket(name.clone()).await?;
                        let mut log_panel = self.log_panel.lock().unwrap();
                        log_panel.add_log(&format!("Opened bucket: {}", name));
                    }
                    TreeNode::Folder { bucket, path, .. } => {
                        file_tree.enter_folder(bucket.clone(), path.clone()).await?;
                        let mut log_panel = self.log_panel.lock().unwrap();
                        log_panel.add_log(&format!("Opened folder: {}", path));
                    }
                    TreeNode::File { bucket, path, .. } => {
                        match Editor::load(app_state, &bucket, &path).await {
                            Ok(editor) => {
                                self.editor = Some(editor);
                                self.active_panel = ActivePanel::Editor;
                                let mut log_panel = self.log_panel.lock().unwrap();
                                log_panel.add_log(&format!("Editing: {}", path));
                            }
                            Err(e) => {
                                let mut log_panel = self.log_panel.lock().unwrap();
                                log_panel.add_log(&format!("Failed to load file: {}", e));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    async fn update_data(&mut self) -> Result<()> {
        if let Some(status_panel) = &mut self.status_panel {
            status_panel.update().await?;
        }
        if let Some(file_tree) = &self.file_tree {
            if file_tree.render_items().is_empty() {
                if let Some(file_tree) = &mut self.file_tree {
                    file_tree.load_root().await?;
                }
            }
        }
        if let (Some(chat_panel), Some(file_tree)) = (&mut self.chat_panel, &self.file_tree) {
            if let Some(bot_name) = file_tree.get_selected_bot() {
                chat_panel.poll_response(&bot_name).await?;
            }
        }
        Ok(())
    }
}
