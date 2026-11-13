//! Destructive-pattern rules shared by the terminal executor and its tests.

/// Server-side re-check of destructive patterns. The consent card enforces
/// similar checks client-side, but the server must not rely on it.
pub(crate) fn blocked_pattern(command: &str) -> Option<&'static str> {
    let normalized = normalize_command(command);
    let tokens = shell_like_tokens(&normalized);
    let cmd = normalized.as_str();

    if has_fork_bomb(cmd) {
        return Some("fork bomb");
    }
    if has_block_device_redirect(cmd) {
        return Some("redirect to block device");
    }
    if is_system_power_command(&tokens) {
        return Some("system power command");
    }
    if is_mkfs_command(&tokens) {
        return Some("mkfs");
    }
    if is_dangerous_rm(&tokens, cmd) {
        return Some("dangerous recursive delete");
    }
    if is_dd_block_device(&tokens) {
        return Some("dd block device access");
    }
    if is_block_device_wipe(&tokens) {
        return Some("disk wipe utility");
    }
    if is_dangerous_find(&tokens) {
        return Some("dangerous find delete/exec");
    }
    if is_dangerous_permission_change(&tokens) {
        return Some("dangerous recursive permission/owner change");
    }
    if is_truncate_block_device(&tokens) {
        return Some("truncate block device");
    }

    None
}

fn normalize_command(command: &str) -> String {
    let mut out = String::with_capacity(command.len());
    let mut prev_space = false;
    for ch in command.chars() {
        let mapped = match ch {
            '\n' | '\r' | '\t' => ' ',
            '\'' | '"' | '`' => ' ',
            _ => ch,
        };
        if mapped.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(mapped);
            prev_space = false;
        }
    }
    out.trim().to_ascii_lowercase()
}

fn shell_like_tokens(normalized: &str) -> Vec<String> {
    normalized
        .split(|c: char| c.is_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '<' | '>'))
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.' && c != '/' && c != '$' && c != '{' && c != '}' && c != '~' && c != ':' && c != '=' && c != '*' ).to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn command_basename(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

fn is_command_token(token: &str) -> bool {
    let base = command_basename(token);
    matches!(base, "rm" | "dd" | "mkfs" | "shred" | "wipefs" | "find" | "chmod" | "chown" | "truncate" | "shutdown" | "reboot" | "halt" | "poweroff" | "init" | "telinit" | "systemctl")
        || base.starts_with("mkfs.")
}

fn is_wrapper(token: &str) -> bool {
    matches!(
        command_basename(token),
        "sudo" | "env" | "command" | "nohup" | "nice" | "ionice" | "time" | "chrt" | "setsid" | "xargs"
    )
}

fn first_effective_command(tokens: &[String]) -> Option<&str> {
    let mut i = 0;
    while i < tokens.len() {
        let t = command_basename(&tokens[i]);
        if is_wrapper(t) {
            i += 1;
            // skip simple wrapper args like -u user, -n 10
            while i < tokens.len() && tokens[i].starts_with('-') {
                let opt = &tokens[i];
                i += 1;
                // wrappers with value-taking short opts
                if matches!(command_basename(opt), "-u" | "-g" | "-n" | "-p") && i < tokens.len() {
                    i += 1;
                }
            }
            continue;
        }
        if t.contains('=') && !is_command_token(t) {
            i += 1;
            continue;
        }
        return Some(t);
    }
    None
}

fn has_fork_bomb(cmd: &str) -> bool {
    cmd.contains(":(){") || cmd.contains(":() {") || cmd.contains(".() { .|.& };.")
}

fn has_block_device_redirect(cmd: &str) -> bool {
    // match > /dev/sdX, >/dev/nvme, >>/dev/vd, 1>/dev/hd, &>/dev/...
    let compact = cmd.replace(' ', "");
    for dev in ["/dev/sd", "/dev/nvme", "/dev/hd", "/dev/vd", "/dev/xvd", "/dev/mmcblk", "/dev/loop"] {
        if compact.contains(&format!(">{dev}")) || compact.contains(&format!(">>{dev}")) {
            return true;
        }
    }
    false
}

fn is_block_device_path(path: &str) -> bool {
    let p = path.trim();
    p.starts_with("/dev/sd")
        || p.starts_with("/dev/nvme")
        || p.starts_with("/dev/hd")
        || p.starts_with("/dev/vd")
        || p.starts_with("/dev/xvd")
        || p.starts_with("/dev/mmcblk")
        || p.starts_with("/dev/loop")
}

fn is_dangerous_root_target(target: &str) -> bool {
    let t = target.trim();
    matches!(t, "/" | "/*" | "~" | "~/" | "~/*" | "$home" | "${home}" | "$home/" | "${home}/" | "$home/*" | "${home}/*")
}

fn is_system_power_command(tokens: &[String]) -> bool {
    match first_effective_command(tokens) {
        Some("shutdown" | "reboot" | "halt" | "poweroff") => true,
        Some("init" | "telinit") => tokens.iter().any(|t| t == "0" || t == "6"),
        Some("systemctl") => tokens.iter().any(|t| {
            matches!(
                t.as_str(),
                "poweroff" | "reboot" | "halt" | "kexec" | "shutdown"
            )
        }),
        _ => false,
    }
}

fn is_mkfs_command(tokens: &[String]) -> bool {
    tokens.iter().any(|t| {
        let b = command_basename(t);
        b == "mkfs" || b.starts_with("mkfs.")
    })
}

fn is_dangerous_rm(tokens: &[String], cmd: &str) -> bool {
    if !tokens.iter().any(|t| command_basename(t) == "rm") {
        return false;
    }
    let recursive = cmd.contains(" -r")
        || cmd.contains(" -rf")
        || cmd.contains(" -fr")
        || cmd.contains("--recursive")
        || tokens.iter().any(|t| {
            let b = t.as_str();
            b.starts_with('-') && !b.starts_with("--") && b.contains('r')
        });
    let force = cmd.contains(" -f")
        || cmd.contains(" -rf")
        || cmd.contains(" -fr")
        || cmd.contains("--force")
        || cmd.contains("--no-preserve-root")
        || tokens.iter().any(|t| {
            let b = t.as_str();
            b.starts_with('-') && !b.starts_with("--") && b.contains('f')
        });
    if !(recursive && force) {
        return false;
    }
    tokens.iter().any(|t| is_dangerous_root_target(t))
}

fn is_dd_block_device(tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| command_basename(t) == "dd") {
        return false;
    }
    tokens.iter().any(|t| {
        let v = t.strip_prefix("if=").or_else(|| t.strip_prefix("of="));
        v.map(is_block_device_path).unwrap_or(false)
    })
}

fn is_block_device_wipe(tokens: &[String]) -> bool {
    let has_wiper = tokens
        .iter()
        .any(|t| matches!(command_basename(t), "shred" | "wipefs"));
    if !has_wiper {
        return false;
    }
    tokens.iter().any(|t| is_block_device_path(t) || is_dangerous_root_target(t))
}

fn is_dangerous_find(tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| command_basename(t) == "find") {
        return false;
    }
    let has_dangerous_action = tokens
        .iter()
        .any(|t| t == "-delete" || t == "-exec" || t == "-execdir");
    if !has_dangerous_action {
        return false;
    }
    let mentions_rm = tokens.iter().any(|t| command_basename(t) == "rm");
    let targets_danger = tokens.iter().any(|t| is_dangerous_root_target(t));
    // find / -delete or find ~ -delete, or find / -exec rm ... / find with rm on dangerous roots
    has_dangerous_action && (targets_danger || (mentions_rm && tokens.iter().any(|t| t == "/" || t == "~" || t == "$home" || t == "${home}")))
}

fn is_dangerous_permission_change(tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| matches!(command_basename(t), "chmod" | "chown")) {
        return false;
    }
    let recursive = tokens.iter().any(|t| t == "-r" || t == "--recursive" || (t.starts_with('-') && !t.starts_with("--") && t.contains('r')));
    if !recursive {
        return false;
    }
    tokens.iter().any(|t| is_dangerous_root_target(t))
}

fn is_truncate_block_device(tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| command_basename(t) == "truncate") {
        return false;
    }
    tokens.iter().any(|t| is_block_device_path(t))
}

#[cfg(test)]
mod tests {
    use super::blocked_pattern;

    #[test]
    fn destructive_commands_are_refused_server_side() {
        assert!(blocked_pattern("rm -rf /").is_some());
        assert!(blocked_pattern("sudo SHUTDOWN now").is_some());
        assert!(blocked_pattern("dd if=/dev/zero of=/dev/sda").is_some());
        assert!(blocked_pattern(":(){ :|:& };:").is_some());
    }

    #[test]
    fn documented_bypasses_are_blocked() {
        let cases = [
            "rm -rf ~",
            "rm -rf $HOME",
            "rm -rf ${HOME}",
            "rm -rf -- /",
            "rm -rf --no-preserve-root /",
            "rm  -rf   /",
            "rm -rf '/'",
            "rm -rf \"/\"",
            "dd of=/dev/sda if=/dev/zero",
            ">/dev/sda",
            "echo hi >/dev/nvme0n1",
            "shred -u /dev/sda",
            "wipefs -a /dev/sda",
            "find / -delete",
            "find ~ -delete",
            "find / -exec rm -rf {} \\;",
            "chmod -R 000 $HOME",
            "chown -R nobody:nogroup /",
            "truncate -s 0 /dev/sda",
            "mkfs.ext4 /dev/sda",
            "halt",
            "poweroff",
        ];
        for command in cases {
            assert!(
                blocked_pattern(command).is_some(),
                "{command} must be blocked"
            );
        }
    }

    #[test]
    fn ordinary_commands_are_not_blocked() {
        for command in [
            "ls -la",
            "python3 make_pdf.py",
            "echo hi",
            "git status",
            "rm file.txt",
            "find . -name '*.tmp'",
        ] {
            assert_eq!(blocked_pattern(command), None, "{command} must be allowed");
        }
    }
}
