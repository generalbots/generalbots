use std::sync::{Arc, Mutex};
use log::{Log, Metadata, LevelFilter, Record, SetLoggerError};
use chrono::Local;

pub struct LogPanel {
 logs: Vec<String>,
 max_logs: usize,
}

impl LogPanel {
 pub fn new() -> Self {
 Self {
 logs: Vec::with_capacity(1000),
 max_logs: 1000,
 }
 }

 pub fn add_log(&mut self, entry: &str) {
 if self.logs.len() >= self.max_logs {
 self.logs.remove(0);
 }
 self.logs.push(entry.to_string());
 }

 pub fn render(&self) -> String {
 let visible_logs = if self.logs.len() > 10 {
 &self.logs[self.logs.len() - 10..]
 } else {
 &self.logs[..]
 };
 visible_logs.join("\n")
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
