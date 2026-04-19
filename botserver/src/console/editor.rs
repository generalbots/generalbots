use crate::core::shared::state::AppState;
use color_eyre::Result;
use std::sync::Arc;

pub struct Editor {
    file_path: String,
    bucket: String,
    key: String,
    content: String,
    cursor_pos: usize,
    scroll_offset: usize,
    visible_lines: usize,
    modified: bool,
}

impl std::fmt::Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Editor")
            .field("file_path", &self.file_path)
            .field("bucket", &self.bucket)
            .field("key", &self.key)
            .field("content_len", &self.content.len())
            .field("cursor_pos", &self.cursor_pos)
            .field("scroll_offset", &self.scroll_offset)
            .field("visible_lines", &self.visible_lines)
            .field("modified", &self.modified)
            .finish()
    }
}
impl Editor {
    pub async fn load(app_state: &Arc<AppState>, bucket: &str, path: &str) -> Result<Self> {
        let content = if let Some(drive) = &app_state.drive {
            match drive.get_object().bucket(bucket).key(path).send().await {
                Ok(response) => {
                    let bytes = response.body.collect().await?.into_bytes();
                    String::from_utf8_lossy(&bytes).to_string()
                }
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };
        Ok(Self {
            file_path: format!("{}/{}", bucket, path),
            bucket: bucket.to_string(),
            key: path.to_string(),
            content,
            cursor_pos: 0,
            scroll_offset: 0,
            visible_lines: 20,
            modified: false,
        })
    }
    pub async fn save(&mut self, app_state: &Arc<AppState>) -> Result<()> {
        if let Some(drive) = &app_state.drive {
            drive
                .put_object()
                .bucket(&self.bucket)
                .key(&self.key)
                .body(self.content.as_bytes().to_vec().into())
                .send()
                .await?;
            self.modified = false;
        }
        Ok(())
    }
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn set_visible_lines(&mut self, lines: usize) {
        self.visible_lines = lines.max(5);
    }

    pub fn visible_lines(&self) -> usize {
        self.visible_lines
    }

    fn get_cursor_line(&self) -> usize {
        self.content[..self.cursor_pos].lines().count()
    }

    fn ensure_cursor_visible(&mut self) {
        let cursor_line = self.get_cursor_line();

        if cursor_line < self.scroll_offset {
            self.scroll_offset = cursor_line;
        }

        let visible = self.visible_lines.saturating_sub(3);
        if cursor_line >= self.scroll_offset + visible {
            self.scroll_offset = cursor_line.saturating_sub(visible) + 1;
        }
    }

    pub fn render(&self, cursor_blink: bool) -> String {
        let lines: Vec<&str> = self.content.lines().collect();
        let total_lines = lines.len().max(1);
        let visible_lines = self.visible_lines();
        let cursor_line = self.get_cursor_line();
        let cursor_col = self.content[..self.cursor_pos]
            .lines()
            .last()
            .map(|line| line.len())
            .unwrap_or(0);
        let start = self.scroll_offset;
        let end = (start + visible_lines).min(total_lines);
        let mut display_lines = Vec::new();
        for i in start..end {
            let line_num = i + 1;
            let line_content = if i < lines.len() { lines[i] } else { "" };
            let is_cursor_line = i == cursor_line;
            let cursor_indicator = if is_cursor_line && cursor_blink {
                let spaces = " ".repeat(cursor_col);
                format!("{}█", spaces)
            } else {
                String::new()
            };
            display_lines.push(format!(
                " {:4} │ {}{}",
                line_num, line_content, cursor_indicator
            ));
        }
        if display_lines.is_empty() {
            let cursor_indicator = if cursor_blink { "█" } else { "" };
            display_lines.push(format!("    1 │ {}", cursor_indicator));
        }
        display_lines.push("".to_string());
        display_lines
            .push("─────────────────────────────────────────────────────────────".to_string());
        let status = if self.modified { "MODIFIED" } else { "SAVED" };
        display_lines.push(format!(
            " {} {} │ Line: {}, Col: {}",
            status,
            self.file_path,
            cursor_line + 1,
            cursor_col + 1
        ));
        display_lines.push(" Ctrl+S: Save │ Ctrl+W: Close │ Esc: Close without saving".to_string());
        display_lines.join("\n")
    }
    pub fn move_up(&mut self) {
        if let Some(prev_line_end) = self.content[..self.cursor_pos].rfind('\n') {
            if let Some(prev_prev_line_end) = self.content[..prev_line_end].rfind('\n') {
                let target_pos = prev_prev_line_end
                    + 1
                    + (self.cursor_pos - prev_line_end - 1)
                        .min(self.content[prev_prev_line_end + 1..prev_line_end].len());
                self.cursor_pos = target_pos;
            } else {
                self.cursor_pos = (self.cursor_pos - prev_line_end - 1).min(prev_line_end);
            }
        }
        self.ensure_cursor_visible();
    }
    pub fn move_down(&mut self) {
        if let Some(next_line_start) = self.content[self.cursor_pos..].find('\n') {
            let current_line_start = self.content[..self.cursor_pos]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let next_line_absolute = self.cursor_pos + next_line_start + 1;
            if let Some(next_next_line_start) = self.content[next_line_absolute..].find('\n') {
                let target_pos = next_line_absolute
                    + (self.cursor_pos - current_line_start).min(next_next_line_start);
                self.cursor_pos = target_pos;
            } else {
                let target_pos = next_line_absolute
                    + (self.cursor_pos - current_line_start)
                        .min(self.content[next_line_absolute..].len());
                self.cursor_pos = target_pos;
            }
        }
        self.ensure_cursor_visible();
    }

    pub fn page_up(&mut self) {
        for _ in 0..self.visible_lines.saturating_sub(2) {
            if self.content[..self.cursor_pos].rfind('\n').is_none() {
                break;
            }
            self.move_up();
        }
    }

    pub fn page_down(&mut self) {
        for _ in 0..self.visible_lines.saturating_sub(2) {
            if self.content[self.cursor_pos..].find('\n').is_none() {
                break;
            }
            self.move_down();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.content.len() {
            self.cursor_pos += 1;
        }
    }
    pub fn insert_char(&mut self, c: char) {
        self.modified = true;
        self.content.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.modified = true;
            self.content.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }
    pub fn insert_newline(&mut self) {
        self.modified = true;
        self.content.insert(self.cursor_pos, '\n');
        self.cursor_pos += 1;
        self.ensure_cursor_visible();
    }

    pub fn goto_line(&mut self, line: usize) {
        let lines: Vec<&str> = self.content.lines().collect();
        let target_line = line.saturating_sub(1).min(lines.len().saturating_sub(1));

        self.cursor_pos = 0;
        for (i, line_content) in lines.iter().enumerate() {
            if i == target_line {
                break;
            }
            self.cursor_pos += line_content.len() + 1;
        }
        self.ensure_cursor_visible();
    }
}
