//! Destructive-pattern rules shared by the terminal executor and its tests.

const DENIED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "dd if=",
    "shutdown",
    "reboot",
    ":(){",
    "> /dev/sda",
    "chmod -R 000 /",
];

/// Server-side re-check of destructive patterns. The consent card enforces the
/// same list client-side, but the server must not rely on it.
pub(crate) fn blocked_pattern(command: &str) -> Option<&'static str> {
    let lowered = command.to_ascii_lowercase();
    DENIED_PATTERNS
        .iter()
        .find(|pattern| lowered.contains(&pattern.to_ascii_lowercase()))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::blocked_pattern;

    #[test]
    fn destructive_commands_are_refused_server_side() {
        assert_eq!(blocked_pattern("rm -rf /"), Some("rm -rf /"));
        assert_eq!(blocked_pattern("sudo SHUTDOWN now"), Some("shutdown"));
        assert_eq!(blocked_pattern("dd if=/dev/zero of=/dev/sda"), Some("dd if="));
        assert_eq!(blocked_pattern(":(){ :|:& };:"), Some(":(){"));
    }

    #[test]
    fn ordinary_commands_are_not_blocked() {
        for command in ["ls -la", "python3 make_pdf.py", "echo hi", "git status"] {
            assert_eq!(blocked_pattern(command), None, "{command} must be allowed");
        }
    }
}
