//! Server-side command allowlist and Windows-only supplement.
//!
//! Split from `command_guard.rs` to keep both files inside the 450-line limit.

use std::collections::HashSet;
use std::sync::LazyLock;

// Server-side allowlist. Desktop-only binaries (`rclone`, `osascript`, `rm`)
// are deliberately absent: the desktop shell carries its own guard, and
// `nc`/`netcat` would turn any caller into a reverse shell.
//
// On Windows the lookup uses `normalise_command_name()` which lower-cases and
// strips a single trailing `.exe`, so entries here are bare lower-case stems
// (e.g. `vault`, not `vault.exe`).
pub static ALLOWED_COMMANDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "pdftotext",
        "pandoc",
        "nvidia-smi",
        "powershell",
        "clamscan",
        "freshclam",
        "mc",
        "ffmpeg",
        "ffprobe",
        "convert",
        "gs",
        "tesseract",
        "which",
        "where",
        "sh",
        "bash",
        "cmd",
        "pkill",
        "pgrep",
        "kill",
        "fuser",
        "curl",
        "tar",
        "unzip",
        "pg_dump",
        "pg_isready",
        "docker",
        "apt-get",
        "brew",
        "rustc",
        "nvcc",
        "notify-send",
        "true",
        "find",
        "ss",
        "cargo",
        "redis-server",
        "redis-cli",
        "valkey-cli",
        "valkey-server",
        "minio",
        "chromedriver",
        "chrome",
        "chromium",
        "brave",
        "diesel",
        "psql",
        "forgejo",
        "forgejo-runner",
        "incus",
        "lynis",
        "rkhunter",
        "chkrootkit",
        "suricata",
        "suricata-update",
        "maldet",
        "sudo",
        "visudo",
        "id",
        "netsh",
        "llama-server",
        "ollama",
        "vault",
        "python",
        "python3",
        "python3.11",
        "python3.12",
        "tasklist",
        "taskkill",
        "git",
        // #1955 — the Drive sync manager runs rsync with an explicit argument
        // list (never a shell) for local-folder mirrors.
        "rsync",
        // First-party call sites: filesystem/disk readout (`df`), host
        // addressing (`ip`), Windows ACL/identity (`icacls`, `whoami`) and the
        // PostgreSQL dump client that ships with the allowlisted `psql`.
        "df",
        "ip",
        "icacls",
        "whoami",
    ])
});

/// Binaries allowed only on Windows. These are dangerous on Linux (they can
/// initialise or destroy a PostgreSQL cluster, or only existed as `.exe`
/// entries before normalisation was introduced) but are required by the
/// Windows bootstrap path (`with_base_path.rs`, `botapp::desktop::wizard`).
#[cfg(windows)]
pub static WINDOWS_ONLY_COMMANDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "pg_ctl",
        "initdb",
        // These had `.exe` entries before normalisation; they are not in the
        // cross-platform allowlist because the corresponding commands on Linux
        // reach the binary via the stack path (pkill pattern), not via
        // SafeCommand::new(<bare name>).
        "zitadel",
    ])
});
