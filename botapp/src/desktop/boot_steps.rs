//! The splash's boot steps, read from the server's own log.
//!
//! The server prints a line when each phase of its start begins or ends
//! (`lifecycle.rs` for the services, `listen_impl.rs` for the HTTP listener).
//! This module maps those lines to the six steps the splash shows and reads
//! only what the server appended since the app started. It is pure std so it
//! can be tested without a window.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::SystemTime;

/// Index of the last step the splash shows ("preparing the workspace").
pub const LAST_STEP: u8 = 5;

/// Message fragments that mark the start of a step. Step 0 (starting the
/// services) runs from launch and needs no marker. Both the first boot and a
/// restart pass through `start_all`, which prints either the "Starting" or the
/// "already running" form for the vault and the database.
const MARKERS: &[(&str, u8)] = &[
    ("Starting vault...", 1),
    ("vault started and initialized", 2),
    ("vault is already running", 2),
    ("tables started", 3),
    ("tables is already running", 3),
    ("Bootstrap process completed", 4),
    ("HTTP server listening on", 5),
];

/// Most bytes read in one poll; a larger backlog is read on the next polls.
const MAX_READ: u64 = 4 * 1024 * 1024;

/// Longest unfinished line kept between polls. Marker lines are short, so a
/// longer fragment is dropped instead of growing without bound.
const MAX_PARTIAL: usize = 64 * 1024;

/// The step a log line starts, if it is one of the markers.
pub fn step_for_line(line: &str) -> Option<u8> {
    MARKERS
        .iter()
        .find(|(marker, _)| line.contains(marker))
        .map(|(_, step)| *step)
}

/// The furthest step seen so far. Markers can arrive out of order (services
/// start in parallel) or be missing (no local vault), so the step only ever
/// moves forward and a missing marker never blocks a later one.
#[derive(Debug, Default)]
pub struct StepTracker {
    current: u8,
}

impl StepTracker {
    /// Feeds one line; returns true when it moved the step forward.
    pub fn observe(&mut self, line: &str) -> bool {
        match step_for_line(line) {
            Some(step) if step > self.current => {
                self.current = step;
                true
            }
            _ => false,
        }
    }

    pub fn current(&self) -> u8 {
        self.current
    }
}

/// Reads the lines appended to a file since the tail was opened.
///
/// A file that does not exist yet starts at offset 0 once it appears; a file
/// that shrank or was re-created is read again from the start; an unfinished
/// last line is kept until its newline arrives.
#[derive(Debug)]
pub struct LogTail {
    path: PathBuf,
    offset: u64,
    created: Option<SystemTime>,
    partial: Vec<u8>,
}

impl LogTail {
    /// Opens the tail at the current end of the file, so earlier runs are ignored.
    pub fn at_end(path: PathBuf) -> Self {
        let meta = std::fs::metadata(&path).ok();
        Self {
            offset: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            created: meta.and_then(|m| m.created().ok()),
            path,
            partial: Vec::new(),
        }
    }

    /// Returns the complete lines appended since the last call, without their
    /// line endings.
    pub fn read_lines(&mut self) -> Vec<String> {
        let Ok(meta) = std::fs::metadata(&self.path) else {
            self.restart(None);
            return Vec::new();
        };
        let created = meta.created().ok();
        if meta.len() < self.offset || created != self.created {
            self.restart(created);
        }
        if meta.len() == self.offset {
            return Vec::new();
        }
        let mut chunk = Vec::new();
        let read = File::open(&self.path).and_then(|mut file| {
            file.seek(SeekFrom::Start(self.offset))?;
            file.take(MAX_READ).read_to_end(&mut chunk)
        });
        match read {
            Ok(count) => self.offset += count as u64,
            Err(_) => return Vec::new(),
        }
        self.split(&chunk)
    }

    fn restart(&mut self, created: Option<SystemTime>) {
        self.offset = 0;
        self.created = created;
        self.partial.clear();
    }

    fn split(&mut self, chunk: &[u8]) -> Vec<String> {
        self.partial.extend_from_slice(chunk);
        let Some(end) = self.partial.iter().rposition(|b| *b == b'\n') else {
            if self.partial.len() > MAX_PARTIAL {
                self.partial.clear();
            }
            return Vec::new();
        };
        let rest = self.partial.split_off(end + 1);
        let complete = std::mem::replace(&mut self.partial, rest);
        complete
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| String::from_utf8_lossy(line.strip_suffix(b"\r").unwrap_or(line)).into_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn steps(lines: &[&str]) -> Vec<u8> {
        let mut tracker = StepTracker::default();
        let mut seen = Vec::new();
        for line in lines {
            if tracker.observe(line) {
                seen.push(tracker.current());
            }
        }
        seen
    }

    #[test]
    fn real_cold_start_lines_map_to_every_step() {
        let lines = [
            "20261125152802.336 info lifecycle:Starting bootstrap process (parallel services)...",
            "20261125152802.365 info lifecycle:Starting vault...",
            "20261125152805.194 info lifecycle:vault started and initialized",
            "20261125152807.271 info lifecycle:Starting tables...",
            "20261125152807.354 info lifecycle:drive started",
            "20261125152813.833 info lifecycle:tables started",
            "20261125152816.572 info lifecycle:Bootstrap process completed!",
            "20261125152818.903 info init:Database URL obtained from Vault",
            "20261125152831.826 info listen_impl:HTTP server listening on 127.0.0.1:8080",
        ];
        assert_eq!(steps(&lines), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn restart_lines_use_the_already_running_forms() {
        let lines = [
            "x info lifecycle:vault is already running",
            "x info lifecycle:tables is already running",
            "x info lifecycle:Bootstrap process completed!",
        ];
        assert_eq!(steps(&lines), vec![2, 3, 4]);
    }

    #[test]
    fn steps_only_move_forward_and_tolerate_gaps() {
        let lines = [
            "x info lifecycle:tables started",
            "x info lifecycle:Starting vault...",
            "x info lifecycle:vault started and initialized",
            "x info listen_impl:HTTP server listening on 127.0.0.1:8080",
            "x info lifecycle:Bootstrap process completed!",
        ];
        assert_eq!(steps(&lines), vec![3, 5]);
    }

    #[test]
    fn other_service_lines_are_not_markers() {
        for line in [
            "x info lifecycle:Starting tables...",
            "x info lifecycle:vector_db started",
            "x info lifecycle:directory started",
            "x info lifecycle:Starting bootstrap process (parallel services)...",
        ] {
            assert_eq!(step_for_line(line), None, "{line}");
        }
    }

    fn temp_log(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gb-boot-steps-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(std::fs::create_dir_all(&dir).is_ok());
        dir.join("botserver-sidecar.log")
    }

    fn append(path: &PathBuf, text: &str) {
        let file = std::fs::OpenOptions::new().create(true).append(true).open(path);
        assert!(file.and_then(|mut f| f.write_all(text.as_bytes())).is_ok());
    }

    #[test]
    fn tail_skips_earlier_runs_and_joins_partial_crlf_lines() {
        let path = temp_log("grow");
        append(&path, "old run: HTTP server listening on 127.0.0.1:8080\n");
        let mut tail = LogTail::at_end(path.clone());
        assert!(tail.read_lines().is_empty());

        append(&path, "a info lifecycle:Starting vau");
        assert!(tail.read_lines().is_empty());
        append(&path, "lt...\r\nb info lifecycle:vault started and initialized\r\nc par");
        assert_eq!(
            tail.read_lines(),
            vec![
                "a info lifecycle:Starting vault...".to_string(),
                "b info lifecycle:vault started and initialized".to_string(),
            ]
        );
        append(&path, "tial\n");
        assert_eq!(tail.read_lines(), vec!["c partial".to_string()]);
        let _ = std::fs::remove_dir_all(path.parent().map(PathBuf::from).unwrap_or_default());
    }

    #[test]
    fn tail_waits_for_a_missing_file_and_rereads_a_truncated_one() {
        let path = temp_log("truncate");
        let mut tail = LogTail::at_end(path.clone());
        assert!(tail.read_lines().is_empty());

        append(&path, "first line\nsecond line\n");
        assert_eq!(tail.read_lines(), vec!["first line".to_string(), "second line".to_string()]);

        assert!(std::fs::write(&path, "new\n").is_ok());
        assert_eq!(tail.read_lines(), vec!["new".to_string()]);
        append(&path, "after\n");
        assert_eq!(tail.read_lines(), vec!["after".to_string()]);
        let _ = std::fs::remove_dir_all(path.parent().map(PathBuf::from).unwrap_or_default());
    }
}
