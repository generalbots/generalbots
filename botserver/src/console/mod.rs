use crate::core::shared::state::AppState;
use std::sync::Arc;
use std::sync::Mutex;

mod chat_panel;
mod editor;
pub mod file_tree;
mod log_panel;
mod status_panel;
pub mod wizard;
mod ui_core;
mod ui_input;
mod ui_panels;
mod ui_render;

use chat_panel::ChatPanel;
use editor::Editor;
use file_tree::FileTree;
use log_panel::LogPanel;
use status_panel::StatusPanel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivePanel {
    FileTree,
    Editor,
    Status,
    Logs,
    Chat,
}

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

impl Default for XtreeUI {
    fn default() -> Self {
        Self::new()
    }
}
