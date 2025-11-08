use crate::shared::state::AppState;
use color_eyre::Result;
use crossterm::{
 event::{self, Event, KeyCode, KeyModifiers},
 execute,
 terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;
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
mod editor;
mod file_tree;
mod log_panel;
mod status_panel;
use editor::Editor;
use file_tree::{FileTree, TreeNode};
use log_panel::{init_logger, LogPanel};
use status_panel::StatusPanel;

pub struct XtreeUI {
 app_state: Option<Arc<AppState>>,
 file_tree: Option<FileTree>,
 status_panel: Option<StatusPanel>,
 log_panel: Arc<Mutex<LogPanel>>,
 editor: Option<Editor>,
 active_panel: ActivePanel,
 should_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActivePanel {
 FileTree,
 Editor,
 Status,
 Logs,
}

impl XtreeUI {
 pub fn new() -> Self {
 let log_panel = Arc::new(Mutex::new(LogPanel::new()));
 Self {
 app_state: None,
 file_tree: None,
 status_panel: None,
 log_panel: log_panel.clone(),
 editor: None,
 active_panel: ActivePanel::Logs,
 should_quit: false,
 }
 }

 pub fn set_app_state(&mut self, app_state: Arc<AppState>) {
 self.file_tree = Some(FileTree::new(app_state.clone()));
 self.status_panel = Some(StatusPanel::new(app_state.clone()));
 self.app_state = Some(app_state);
 self.active_panel = ActivePanel::FileTree;
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
 init_logger(self.log_panel.clone())?;
 log::set_max_level(LevelFilter::Trace);
 let result = self.run_event_loop(&mut terminal);
 disable_raw_mode()?;
 execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
 terminal.show_cursor()?;
 result
 }

 fn run_event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
 let mut last_update = std::time::Instant::now();
 let update_interval = std::time::Duration::from_millis(500);
 let rt = tokio::runtime::Runtime::new()?;
 loop {
 terminal.draw(|f| self.render(f))?;
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

 fn render(&self, f: &mut Frame) {
 let bg = Color::Rgb(15, 15, 25);
 let border_active = Color::Rgb(120, 220, 255);
 let border_inactive = Color::Rgb(70, 70, 90);
 let text = Color::Rgb(240, 240, 245);
 let highlight = Color::Rgb(90, 180, 255);
 let title = Color::Rgb(255, 230, 140);
 if self.app_state.is_none() {
 self.render_loading(f, bg, text, border_active, title);
 return;
 }
 let main_chunks = Layout::default()
 .direction(Direction::Vertical)
 .constraints([Constraint::Min(0), Constraint::Length(12)])
 .split(f.area());
 if self.editor.is_some() {
 let editor_chunks = Layout::default()
 .direction(Direction::Horizontal)
 .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
 .split(main_chunks[0]);
 self.render_file_tree(f, editor_chunks[0], bg, text, border_active, border_inactive, highlight, title);
 if let Some(editor) = &self.editor {
 self.render_editor(f, editor_chunks[1], editor, bg, text, border_active, border_inactive, highlight, title);
 }
 } else {
 let top_chunks = Layout::default()
 .direction(Direction::Horizontal)
 .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
 .split(main_chunks[0]);
 self.render_file_tree(f, top_chunks[0], bg, text, border_active, border_inactive, highlight, title);
 self.render_status(f, top_chunks[1], bg, text, border_active, border_inactive, highlight, title);
 }
 self.render_logs(f, main_chunks[1], bg, text, border_active, border_inactive, highlight, title);
 }

 fn render_loading(&self, f: &mut Frame, bg: Color, text: Color, border: Color, title: Color) {
 let chunks = Layout::default()
 .direction(Direction::Vertical)
 .constraints([Constraint::Percentage(40), Constraint::Percentage(20), Constraint::Percentage(40)])
 .split(f.area());
 let center = Layout::default()
 .direction(Direction::Horizontal)
 .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)])
 .split(chunks[1])[1];
 let block = Block::default()
 .title(Span::styled(" 🚀 BOTSERVER ", Style::default().fg(title).add_modifier(Modifier::BOLD)))
 .borders(Borders::ALL)
 .border_style(Style::default().fg(border))
 .style(Style::default().bg(bg));
 let loading_text = vec![
 "",
 " ╔════════════════════════════════╗",
 " ║                                ║",
 " ║    ⚡ Initializing System...  ║",
 " ║                                ║",
 " ║    Loading components...       ║",
 " ║    Connecting to services...   ║",
 " ║    Preparing interface...      ║",
 " ║                                ║",
 " ╚════════════════════════════════╝",
 "",
 ].join("\n");
 let paragraph = Paragraph::new(loading_text)
 .block(block)
 .style(Style::default().fg(text))
 .wrap(Wrap { trim: false });
 f.render_widget(paragraph, center);
 }

 fn render_file_tree(&self, f: &mut Frame, area: Rect, bg: Color, text: Color, border_active: Color, border_inactive: Color, highlight: Color, title: Color) {
 if let Some(file_tree) = &self.file_tree {
 let items = file_tree.render_items();
 let selected = file_tree.selected_index();
 let list_items: Vec<ListItem> = items.iter().enumerate().map(|(idx, (display, _))| {
 let style = if idx == selected {
 Style::default().bg(highlight).fg(Color::Black).add_modifier(Modifier::BOLD)
 } else {
 Style::default().fg(text)
 };
 ListItem::new(Line::from(Span::styled(display.clone(), style)))
 }).collect();
 let is_active = self.active_panel == ActivePanel::FileTree;
 let border_color = if is_active { border_active } else { border_inactive };
 let title_style = if is_active {
 Style::default().fg(title).add_modifier(Modifier::BOLD)
 } else {
 Style::default().fg(text)
 };
 let block = Block::default()
 .title(Span::styled(" 📁 FILE EXPLORER ", title_style))
 .borders(Borders::ALL)
 .border_style(Style::default().fg(border_color))
 .style(Style::default().bg(bg));
 let list = List::new(list_items).block(block);
 f.render_widget(list, area);
 } else {
 let block = Block::default()
 .title(Span::styled(" 📁 FILE EXPLORER ", Style::default().fg(text)))
 .borders(Borders::ALL)
 .border_style(Style::default().fg(border_inactive))
 .style(Style::default().bg(bg));
 f.render_widget(block, area);
 }
 }

 fn render_status(&self, f: &mut Frame, area: Rect, bg: Color, text: Color, border_active: Color, border_inactive: Color, _highlight: Color, title: Color) {
 let status_text = if let Some(status_panel) = &self.status_panel {
 status_panel.render()
 } else {
 "Waiting for initialization...".to_string()
 };
 let is_active = self.active_panel == ActivePanel::Status;
 let border_color = if is_active { border_active } else { border_inactive };
 let title_style = if is_active {
 Style::default().fg(title).add_modifier(Modifier::BOLD)
 } else {
 Style::default().fg(text)
 };
 let block = Block::default()
 .title(Span::styled(" 📊 SYSTEM STATUS ", title_style))
 .borders(Borders::ALL)
 .border_style(Style::default().fg(border_color))
 .style(Style::default().bg(bg));
 let paragraph = Paragraph::new(status_text)
 .block(block)
 .style(Style::default().fg(text))
 .wrap(Wrap { trim: false });
 f.render_widget(paragraph, area);
 }

 fn render_editor(&self, f: &mut Frame, area: Rect, editor: &Editor, bg: Color, text: Color, border_active: Color, border_inactive: Color, _highlight: Color, title: Color) {
 let is_active = self.active_panel == ActivePanel::Editor;
 let border_color = if is_active { border_active } else { border_inactive };
 let title_style = if is_active {
 Style::default().fg(title).add_modifier(Modifier::BOLD)
 } else {
 Style::default().fg(text)
 };
 let title_text = format!(" ✏️ EDITOR: {} ", editor.file_path());
 let block = Block::default()
 .title(Span::styled(title_text, title_style))
 .borders(Borders::ALL)
 .border_style(Style::default().fg(border_color))
 .style(Style::default().bg(bg));
 let content = editor.render();
 let paragraph = Paragraph::new(content)
 .block(block)
 .style(Style::default().fg(text))
 .wrap(Wrap { trim: false });
 f.render_widget(paragraph, area);
 }

 fn render_logs(&self, f: &mut Frame, area: Rect, bg: Color, text: Color, border_active: Color, border_inactive: Color, _highlight: Color, title: Color) {
 let log_panel = self.log_panel.try_lock();
 let log_lines = if let Ok(panel) = log_panel {
 panel.render()
 } else {
 "Loading logs...".to_string()
 };
 let is_active = self.active_panel == ActivePanel::Logs;
 let border_color = if is_active { border_active } else { border_inactive };
 let title_style = if is_active {
 Style::default().fg(title).add_modifier(Modifier::BOLD)
 } else {
 Style::default().fg(text)
 };
 let block = Block::default()
 .title(Span::styled(" 📜 SYSTEM LOGS ", title_style))
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
 log_panel.add_log(&format!("✓ Saved: {}", editor.file_path()));
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
 log_panel.add_log("✓ Closed editor");
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
 log_panel.add_log(&format!("✗ Enter error: {}", e));
 }
 }
 KeyCode::Backspace => {
 if let Some(file_tree) = &mut self.file_tree {
 if file_tree.go_up() {
 if let Err(e) = file_tree.refresh_current().await {
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log(&format!("✗ Navigation error: {}", e));
 }
 }
 }
 }
 KeyCode::Tab => {
 self.active_panel = ActivePanel::Status;
 }
 KeyCode::Char('q') => {
 self.should_quit = true;
 }
 KeyCode::F(5) => {
 if let Some(file_tree) = &mut self.file_tree {
 if let Err(e) = file_tree.refresh_current().await {
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log(&format!("✗ Refresh failed: {}", e));
 } else {
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log("✓ Refreshed");
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
 KeyCode::Char(c) => editor.insert_char(c),
 KeyCode::Backspace => editor.backspace(),
 KeyCode::Enter => editor.insert_newline(),
 KeyCode::Tab => {
 self.active_panel = ActivePanel::FileTree;
 }
 KeyCode::Esc => {
 self.editor = None;
 self.active_panel = ActivePanel::FileTree;
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log("✓ Closed editor");
 }
 _ => {}
 }
 }
 }
 ActivePanel::Status => match key {
 KeyCode::Tab => {
 self.active_panel = ActivePanel::Logs;
 }
 _ => {}
 },
 ActivePanel::Logs => match key {
 KeyCode::Tab => {
 self.active_panel = ActivePanel::FileTree;
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
 log_panel.add_log(&format!("📂 Opened bucket: {}", name));
 }
 TreeNode::Folder { bucket, path, .. } => {
 file_tree.enter_folder(bucket.clone(), path.clone()).await?;
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log(&format!("📂 Opened folder: {}", path));
 }
 TreeNode::File { bucket, path, .. } => {
 match Editor::load(app_state, &bucket, &path).await {
 Ok(editor) => {
 self.editor = Some(editor);
 self.active_panel = ActivePanel::Editor;
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log(&format!("✏️ Editing: {}", path));
 }
 Err(e) => {
 let mut log_panel = self.log_panel.lock().unwrap();
 log_panel.add_log(&format!("✗ Failed to load file: {}", e));
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
 Ok(())
 }
}
