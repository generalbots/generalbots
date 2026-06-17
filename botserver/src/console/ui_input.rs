use super::{file_tree::TreeNode, ActivePanel, Editor, XtreeUI};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};

impl XtreeUI {
    async fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key {
                KeyCode::Char('c' | 'q') => {
                    self.should_quit = true;
                    return Ok(());
                }
                KeyCode::Char('s') => {
                    if let Some(editor) = &mut self.editor {
                        if let Some(app_state) = &self.app_state {
                            if let Err(e) = editor.save(app_state).await {
                                if let Ok(mut log_panel) = self.log_panel.lock() {
                                    log_panel.add_log(&format!("Save failed: {e}"));
                                }
                            } else if let Ok(mut log_panel) = self.log_panel.lock() {
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
                        if let Ok(mut log_panel) = self.log_panel.lock() {
                            log_panel.add_log("Closed editor");
                        }
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
                        if let Ok(mut log_panel) = self.log_panel.lock() {
                            log_panel.add_log(&format!("Enter error: {e}"));
                        }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(file_tree) = &mut self.file_tree {
                        if file_tree.go_up() {
                            if let Err(e) = file_tree.refresh_current().await {
                                if let Ok(mut log_panel) = self.log_panel.lock() {
                                    log_panel.add_log(&format!("Navigation error: {e}"));
                                }
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
                            if let Ok(mut log_panel) = self.log_panel.lock() {
                                log_panel.add_log(&format!("Refresh failed: {e}"));
                            }
                        } else if let Ok(mut log_panel) = self.log_panel.lock() {
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
                            if let Ok(mut log_panel) = self.log_panel.lock() {
                                log_panel.add_log("Closed editor");
                            }
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
                        panel.scroll_down(1, 10);
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
                                if let Ok(mut log_panel) = self.log_panel.lock() {
                                    log_panel.add_log(&format!("Chat error: {e}"));
                                }
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
            ActivePanel::Status => {
                if key == KeyCode::Tab {
                    self.active_panel = ActivePanel::Chat;
                }
            }
        }
        Ok(())
    }

    async fn handle_tree_enter(&mut self) -> Result<()> {
        if let (Some(file_tree), Some(app_state)) = (&mut self.file_tree, &self.app_state) {
            if let Some(node) = file_tree.get_selected_node().cloned() {
                match node {
                    TreeNode::Bucket { name, .. } => {
                        file_tree.enter_bucket(name.clone()).await?;
                        if let Ok(mut log_panel) = self.log_panel.lock() {
                            log_panel.add_log(&format!("Opened bucket: {name}"));
                        }
                    }
                    TreeNode::Folder {
                        bucket, path, ..
                    } => {
                        file_tree.enter_folder(bucket.clone(), path.clone()).await?;
                        if let Ok(mut log_panel) = self.log_panel.lock() {
                            log_panel.add_log(&format!("Opened folder: {path}"));
                        }
                    }
                    TreeNode::File {
                        bucket, path, ..
                    } => {
                        match Editor::load(app_state, &bucket, &path).await {
                            Ok(editor) => {
                                self.editor = Some(editor);
                                self.active_panel = ActivePanel::Editor;
                                if let Ok(mut log_panel) = self.log_panel.lock() {
                                    log_panel.add_log(&format!("Editing: {path}"));
                                }
                            }
                            Err(e) => {
                                if let Ok(mut log_panel) = self.log_panel.lock() {
                                    log_panel.add_log(&format!("Failed to load file: {e}"));
                                }
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
            status_panel.update()?;
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
                chat_panel.poll_response(&bot_name)?;
            }
        }
        Ok(())
    }
}
