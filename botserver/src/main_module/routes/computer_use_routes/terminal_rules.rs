//! Destructive-pattern rules for the terminal executor. The original string
//! goes to `sh -c` inside the user's VM, so we inspect a normalized view:
//! quotes are removed (rejoining `rm -r'f' /`), `${IFS}` expands to whitespace,
//! separators/wrappers are peeled, and operators are kept for redirect checks.

/// Server-side re-check of destructive patterns. The consent card enforces
/// similar checks client-side, but the server must not rely on it.
pub(crate) fn blocked_pattern(command: &str) -> Option<&'static str> {
    let normalized = normalize_command(command);
    let tokens = effective_tokens(&normalized);
    let checks = [
        (has_fork_bomb(command), "fork bomb"),
        (
            has_block_device_redirect(&normalized),
            "redirect to block device",
        ),
        (is_system_power_command(&tokens), "system power command"),
        (is_mkfs_command(&tokens), "mkfs"),
        (is_dangerous_rm(&tokens), "dangerous recursive delete"),
        (is_dd_block_device(&tokens), "dd block device access"),
        (is_block_device_wipe(&tokens), "disk wipe utility"),
        (is_dangerous_find(&tokens), "dangerous find delete/exec"),
        (
            is_dangerous_permission_change(&tokens),
            "dangerous recursive permission/owner change",
        ),
        (is_truncate_block_device(&tokens), "truncate block device"),
    ];
    checks
        .into_iter()
        .find(|(blocked, _)| *blocked)
        .map(|(_, label)| label)
}

/// Removes quote characters so fragmented tokens are rejoined (`rm -r'f' /`
/// becomes `rm -rf /`), expands `${IFS}` back to whitespace, and collapses
/// whitespace / `;` into one separator. Operators `|`, `&`, `(`, `)` and `>`
/// are preserved: the tokenizer splits on them and the fork-bomb / redirect
/// checks read them directly.
fn normalize_command(command: &str) -> String {
    let mut out = String::with_capacity(command.len());
    let mut prev_sep = true;
    for ch in command.chars() {
        match ch {
            '\'' | '"' | '`' => {}
            '\n' | '\r' | '\t' | ';' => {
                if !prev_sep {
                    out.push(' ');
                    prev_sep = true;
                }
            }
            c if c.is_whitespace() => {
                if !prev_sep {
                    out.push(' ');
                    prev_sep = true;
                }
            }
            c => {
                out.push(c);
                prev_sep = false;
            }
        }
    }
    let lower = out.trim().to_ascii_lowercase();
    lower.replace("${ifs}", " ").replace("$ifs", " ")
}

fn shell_like_tokens(normalized: &str) -> Vec<String> {
    normalized
        .split(|c: char| c.is_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '<' | '>'))
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.trim_matches(|c: char| {
                !c.is_ascii_alphanumeric()
                    && !matches!(
                        c,
                        '_' | '-' | '.' | '/' | '$' | '{' | '}' | '~' | ':' | '=' | '*'
                    )
            })
            .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn command_basename(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

fn is_command_token(token: &str) -> bool {
    let base = command_basename(token);
    COMMANDS.contains(&base) || base.starts_with("mkfs.")
}

fn is_wrapper(token: &str) -> bool {
    WRAPPERS.contains(&command_basename(token))
}

fn is_shell(token: &str) -> bool {
    SHELLS.contains(&command_basename(token))
}

const COMMANDS: &[&str] = &[
    "rm",
    "dd",
    "mkfs",
    "shred",
    "wipefs",
    "find",
    "chmod",
    "chown",
    "truncate",
    "shutdown",
    "reboot",
    "halt",
    "poweroff",
    "init",
    "telinit",
    "systemctl",
];
const WRAPPERS: &[&str] = &[
    "sudo", "env", "command", "nohup", "nice", "ionice", "time", "chrt", "setsid", "xargs",
];
const SHELLS: &[&str] = &["sh", "bash", "dash", "zsh", "ksh", "ash", "busybox"];

/// Peels leading wrappers so that `sh -c 'rm -r /'`, `sudo …` and
/// `env VAR=x …` are inspected as the command they actually run. Without
/// this, checks keyed on the first command (power-off, `init 0`) are trivially
/// bypassed by wrapping them in a shell.
fn effective_tokens(normalized: &str) -> Vec<String> {
    let mut tokens = shell_like_tokens(normalized);
    for _ in 0..8 {
        let before = tokens.len();
        let mut i = 0;
        while i < tokens.len() {
            let base = command_basename(&tokens[i]);
            if is_shell(base) {
                tokens.remove(i);
                // `sh -c <script>`: with quoting removed the script is already
                // split into tokens; drop the `-c` marker as well.
                if i < tokens.len() && tokens[i].starts_with('-') && tokens[i].contains('c') {
                    tokens.remove(i);
                }
                continue;
            }
            if is_wrapper(base) {
                tokens.remove(i);
                while i < tokens.len() && tokens[i].starts_with('-') {
                    let opt = tokens[i].clone();
                    tokens.remove(i);
                    if matches!(opt.as_str(), "-u" | "-g" | "-n" | "-p" | "-c") && i < tokens.len()
                    {
                        tokens.remove(i);
                    }
                }
                continue;
            }
            if base.contains('=') && !base.starts_with('-') && !is_command_token(base) {
                // `VAR=value` assignment prefix.
                tokens.remove(i);
                continue;
            }
            break;
        }
        if tokens.len() == before {
            break;
        }
    }
    tokens
}

fn first_effective_command(tokens: &[String]) -> Option<&str> {
    tokens.first().map(|t| command_basename(t))
}

fn has_fork_bomb(command: &str) -> bool {
    let compact: String = command.chars().filter(|c| !c.is_whitespace()).collect();
    compact.contains(":(){")
        || compact.contains(".(){")
        || compact.contains(":|:&")
        || compact.contains(".|.&")
}

const BLOCK_DEVICES: &[&str] = &[
    "/dev/sd",
    "/dev/nvme",
    "/dev/hd",
    "/dev/vd",
    "/dev/xvd",
    "/dev/mmcblk",
    "/dev/loop",
];
const ROOT_TARGETS: &[&str] = &[
    "/",
    "//",
    "/*",
    "/.",
    "/..",
    "~",
    "~/",
    "~/*",
    "$home",
    "${home}",
    "$home/",
    "${home}/",
    "$home/*",
    "${home}/*",
    "$pwd",
    "${pwd}",
];

fn has_block_device_redirect(cmd: &str) -> bool {
    let compact = cmd.replace(' ', "");
    BLOCK_DEVICES
        .iter()
        .any(|dev| compact.contains(&format!(">{dev}")) || compact.contains(&format!(">>{dev}")))
}

fn is_block_device_path(path: &str) -> bool {
    let p = path.trim();
    BLOCK_DEVICES.iter().any(|dev| p.starts_with(dev))
}

fn is_dangerous_root_target(target: &str) -> bool {
    let t = target.trim();
    // `${HOME:0:1}` expands to `/`; substring expansions of root vars are unsafe.
    ROOT_TARGETS.contains(&t) || t.starts_with("${home:") || t.starts_with("${pwd:")
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

/// A recursive delete is dangerous when it targets a dangerous root, with or
/// without `-f`. Requiring both flags (the old behaviour) let `rm -r /` and
/// `rm -r ~` through.
fn is_dangerous_rm(tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| command_basename(t) == "rm") {
        return false;
    }
    let recursive = tokens.iter().any(|t| {
        let b = t.as_str();
        b == "--recursive"
            || (b.starts_with('-')
                && !b.starts_with("--")
                && b.trim_start_matches('-').chars().any(|c| c == 'r'))
    });
    if !recursive {
        return false;
    }
    tokens
        .iter()
        .any(|t| is_dangerous_root_target(t) || is_block_device_path(t))
}

fn is_dd_block_device(tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| command_basename(t) == "dd") {
        return false;
    }
    tokens.iter().any(|t| {
        let v = t.strip_prefix("if=").or_else(|| t.strip_prefix("of="));
        v.is_some_and(is_block_device_path)
    })
}

fn is_block_device_wipe(tokens: &[String]) -> bool {
    let has_wiper = tokens
        .iter()
        .any(|t| matches!(command_basename(t), "shred" | "wipefs"));
    if !has_wiper {
        return false;
    }
    tokens
        .iter()
        .any(|t| is_block_device_path(t) || is_dangerous_root_target(t))
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
    tokens.iter().any(|t| is_dangerous_root_target(t))
}

fn is_dangerous_permission_change(tokens: &[String]) -> bool {
    if !tokens
        .iter()
        .any(|t| matches!(command_basename(t), "chmod" | "chown"))
    {
        return false;
    }
    let recursive = tokens.iter().any(|t| {
        t == "-r"
            || t == "--recursive"
            || (t.starts_with('-') && !t.starts_with("--") && t.contains('r'))
    });
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

    fn assert_blocked(cases: &[&str]) {
        for command in cases {
            assert!(
                blocked_pattern(command).is_some(),
                "{command} must be blocked"
            );
        }
    }

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
        assert_blocked(&cases);
    }

    #[test]
    fn issue_2738_recursive_delete_without_force_is_blocked() {
        assert_blocked(&[
            "rm -r /",
            "rm -r ~",
            "rm -r ~/",
            "rm -r $HOME",
            "rm -r ${HOME}",
            "rm -r ${HOME}/",
            "rm -R /",
            "rm --recursive /",
            "rm -r /*",
            "rm -r /dev/sda",
        ]);
    }

    #[test]
    fn shell_wrappers_are_peeled() {
        assert_blocked(&[
            "sh -c 'rm -r /'",
            "bash -c \"rm -rf ~\"",
            "/bin/sh -c \"rm -rf $HOME\"",
            "sudo -u root sh -c 'rm -r /'",
            "env VAR=1 rm -r /",
            "sh -c 'shutdown now'",
            "bash -c 'init 0'",
            "sh -c 'mkfs.ext4 /dev/sda'",
            "sh -c ':(){ :|:& };:'",
        ]);
    }

    #[test]
    fn quoting_and_expansion_bypasses_are_blocked() {
        assert_blocked(&[
            "rm -r'f' /",
            "rm -'r'f /",
            "r''m -rf /",
            "rm -rf' '/'",
            "rm -rf ${IFS}/",
            "rm -rf $IFS/",
            "rm -r ${HOME:0:1}",
            "sh -c 'rm -r' -- /",
        ]);
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
            "rm -r /tmp/build",
            "dd if=/dev/zero of=/tmp/file bs=1M",
            "find /tmp -name '*.log' -delete",
        ] {
            assert_eq!(blocked_pattern(command), None, "{command} must be allowed");
        }
    }
}
