use super::*;

pub(super) fn safe_pkill(args: &[&str]) {
    if let Ok(cmd) = SafeCommand::new("pkill").and_then(|c| c.args(args)) {
        if let Err(e) = cmd.execute() {
            log::warn!("pkill execution failed: {e}");
        }
    }
}
