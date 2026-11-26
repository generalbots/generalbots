//! Instance locking functions for bootstrap
//!
//! Extracted from mod.rs

use crate::shared::utils::get_stack_path;
use botlib::security::command_guard::SafeCommand;
use log::warn;
use std::fs;
use std::path::PathBuf;

/// Check if another instance is already running
pub fn check_single_instance() -> Result<bool, Box<dyn std::error::Error>> {
    let stack_path = get_stack_path();
    let lock_file = PathBuf::from(&stack_path).join(".lock");
    if lock_file.exists() {
        if let Ok(pid_str) = fs::read_to_string(&lock_file) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                let pid_str = pid.to_string();
                if let Some(output) = SafeCommand::new("kill")
                    .and_then(|c| c.args(&["-0", &pid_str]))
                    .ok()
                    .and_then(|cmd| cmd.execute().ok())
                {
                    if output.status.success() {
                        warn!(
                            "Another botserver process (PID {}) is already running on this stack",
                            pid
                        );
                        return Ok(false);
                    }
                }
            }
        }
    }

    let pid = std::process::id();
    if let Some(parent) = lock_file.parent() {
        if let Err(e) = fs::create_dir_all(parent) { log::debug!("cannot create lock dir: {e}"); }
    }
    if let Err(e) = fs::write(&lock_file, pid.to_string()) { log::debug!("cannot write instance lock: {e}"); }
    Ok(true)
}

/// Release the instance lock
pub fn release_instance_lock() {
    let stack_path = get_stack_path();
    let lock_file = PathBuf::from(&stack_path).join(".lock");
    if lock_file.exists() {
        if let Err(e) = fs::remove_file(&lock_file) { log::debug!("cannot remove instance lock: {e}"); }
    }
}
