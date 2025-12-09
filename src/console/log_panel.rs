use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::sync::{Arc, Mutex};

pub struct LogPanel {
    logs: Vec<String>,
    max_logs: usize,
    scroll_offset: usize,
    auto_scroll: bool,
}

impl std::fmt::Debug for LogPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogPanel")
            .field("logs_count", &self.logs.len())
            .field("max_logs", &self.max_logs)
            .field("scroll_offset", &self.scroll_offset)
            .field("auto_scroll", &self.auto_scroll)
            .finish()
    }
}

impl LogPanel {
    pub fn new() -> Self {
        Self {
            logs: Vec::with_capacity(1000),
            max_logs: 1000,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    pub fn add_log(&mut self, entry: &str) {
        if self.logs.len() >= self.max_logs {
            self.logs.remove(0);
            // Adjust scroll offset if we removed a log
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
        self.logs.push(entry.to_string());

        // Auto-scroll to bottom if enabled
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        self.auto_scroll = false;
    }

    pub fn scroll_down(&mut self, lines: usize, visible_lines: usize) {
        let max_scroll = self.logs.len().saturating_sub(visible_lines);
        self.scroll_offset = (self.scroll_offset + lines).min(max_scroll);

        // Re-enable auto-scroll if we're at the bottom
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        // This will be adjusted when rendering based on visible lines
        self.scroll_offset = usize::MAX;
        self.auto_scroll = true;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    pub fn page_up(&mut self, visible_lines: usize) {
        self.scroll_up(visible_lines.saturating_sub(1));
    }

    pub fn page_down(&mut self, visible_lines: usize) {
        self.scroll_down(visible_lines.saturating_sub(1), visible_lines);
    }

    pub fn render(&self, visible_lines: usize) -> String {
        if self.logs.is_empty() {
            return "  Waiting for logs...".to_string();
        }

        let total_logs = self.logs.len();

        // Calculate actual scroll offset
        let max_scroll = total_logs.saturating_sub(visible_lines);
        let actual_offset = if self.scroll_offset == usize::MAX {
            max_scroll
        } else {
            self.scroll_offset.min(max_scroll)
        };

        // Get visible slice
        let end = (actual_offset + visible_lines).min(total_logs);
        let start = actual_offset;

        let visible_logs = &self.logs[start..end];
        visible_logs.join("\n")
    }

    pub fn render_with_scroll_indicator(&self, visible_lines: usize) -> (String, bool, bool) {
        let content = self.render(visible_lines);
        let total_logs = self.logs.len();
        let max_scroll = total_logs.saturating_sub(visible_lines);
        let actual_offset = if self.scroll_offset == usize::MAX {
            max_scroll
        } else {
            self.scroll_offset.min(max_scroll)
        };

        let can_scroll_up = actual_offset > 0;
        let can_scroll_down = actual_offset < max_scroll;

        (content, can_scroll_up, can_scroll_down)
    }

    pub fn logs_count(&self) -> usize {
        self.logs.len()
    }

    pub fn is_auto_scroll(&self) -> bool {
        self.auto_scroll
    }
}

pub struct UiLogger {
    log_panel: Arc<Mutex<LogPanel>>,
    filter: LevelFilter,
}

impl Log for UiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%H:%M:%S");
            let level_icon = match record.level() {
                log::Level::Error => "ERR",
                log::Level::Warn => "WRN",
                log::Level::Info => "INF",
                log::Level::Debug => "DBG",
                log::Level::Trace => "TRC",
            };
            let log_entry = format!("[{}] {} {}", timestamp, level_icon, record.args());
            if let Ok(mut panel) = self.log_panel.lock() {
                panel.add_log(&log_entry);
            }
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(log_panel: Arc<Mutex<LogPanel>>) -> Result<(), SetLoggerError> {
    let logger = Box::new(UiLogger {
        log_panel,
        filter: LevelFilter::Info,
    });
    log::set_boxed_logger(logger)?;
    log::set_max_level(LevelFilter::Trace);
    Ok(())
}
