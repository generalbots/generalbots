//! Output parsers for the host prerequisite scripts.
//!
//! Linux (`DEPENDENCIES.sh --check`) prints `MISSING <kind> <name>` lines and
//! an `Install with:` hint. Windows (`PROD-DEPENDENCIES.ps1 -Check`) prints
//! `[missing]`/`[warn]` lines under a `--- Desktop shell ---` or
//! `--- Agent mode (WSL2 + Incus) ---` heading and encodes the final verdict in
//! its exit code (0 ready, 1 desktop blocked, 2 agent blocked, 3 not Windows).

use super::{base, classify, default_remediation, summarize, HostCheckItem, HostCheckResult};
use std::path::Path;

/// Removes the colour escapes the shell scripts emit around their summaries.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for marker in chars.by_ref() {
                    if ('@'..='~').contains(&marker) {
                        break;
                    }
                }
            }
            continue;
        }
        if character != '\r' {
            out.push(character);
        }
    }
    out
}

fn linux_feature(kind: &str, name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    match kind {
        "group" | "membership" | "service" | "storage" => "agent",
        "command" => match lower.as_str() {
            "incus" => "agent",
            "pdftotext" => "media",
            _ => "desktop",
        },
        "library" | "package" => {
            if lower.contains("incus") {
                "agent"
            } else if lower.contains("zbar") || lower.contains("poppler") {
                "media"
            } else {
                "desktop"
            }
        }
        _ => "desktop",
    }
}

fn linux_remediation(kind: &str, overall: &str) -> String {
    match kind {
        "membership" => "newgrp incus-admin (then sign out and back in)".to_string(),
        "service" => "sudo systemctl enable --now incus".to_string(),
        "storage" => "sudo incus admin init --minimal".to_string(),
        _ => overall.to_string(),
    }
}

pub(super) fn linux(output: &str, exit_code: i32, script: &Path) -> HostCheckResult {
    let clean = strip_ansi(output);
    let display = script.display();
    let fallback = format!("sudo {display} --install");
    let mut remediation = String::new();
    let mut items = Vec::new();
    for raw in clean.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("Install with:") {
            remediation = rest.trim().to_string();
            continue;
        }
        let Some(rest) = line.strip_prefix("MISSING") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let kind = parts.next().unwrap_or("item");
        let name = parts.collect::<Vec<_>>().join(" ");
        if name.is_empty() {
            continue;
        }
        items.push(HostCheckItem {
            feature: linux_feature(kind, &name).to_string(),
            kind: kind.to_string(),
            name,
            status: "missing".to_string(),
            detail: String::new(),
            remediation: linux_remediation(kind, &fallback),
        });
    }
    let overall = if remediation.is_empty() {
        fallback
    } else {
        remediation
    };
    let mut result = base("linux", script, exit_code, overall, &clean);
    result.items = items;
    classify(&mut result);
    summarize(&mut result);
    result
}

fn windows_remediation(message: &str, overall: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("webview2") {
        "winget install --id Microsoft.EdgeWebView2Runtime -e".to_string()
    } else if lower.contains("vc++") {
        "winget install --id Microsoft.VCRedist.2015+.x64 -e".to_string()
    } else if lower.contains("libpq.dll") {
        "Add the PostgreSQL bin directory to PATH, then reopen the app.".to_string()
    } else if lower.contains("wsl") {
        "wsl --install -d Debian".to_string()
    } else {
        overall.to_string()
    }
}

pub(super) fn windows(output: &str, exit_code: i32, script: &Path) -> HostCheckResult {
    let clean = strip_ansi(output);
    let overall = default_remediation("windows", script);
    let mut items = Vec::new();
    let mut section = "desktop";
    for raw in clean.lines() {
        let line = raw.trim();
        if line.starts_with("--- Desktop shell") {
            section = "desktop";
            continue;
        }
        if line.starts_with("--- Agent mode") {
            section = "agent";
            continue;
        }
        let (status, message) = if let Some(rest) = line.strip_prefix("[missing]") {
            ("missing", rest.trim())
        } else if let Some(rest) = line.strip_prefix("[warn]") {
            ("warn", rest.trim())
        } else {
            continue;
        };
        if message.is_empty() {
            continue;
        }
        // Windows has no published BotModels pythonbox, so media is reported as
        // an optional gap rather than a desktop blocker.
        let is_media = message.to_ascii_lowercase().contains("pythonbox");
        let status = if is_media { "missing" } else { status };
        items.push(HostCheckItem {
            feature: if is_media { "media" } else { section }.to_string(),
            kind: "runtime".to_string(),
            name: message.to_string(),
            status: status.to_string(),
            detail: String::new(),
            remediation: if is_media {
                String::new()
            } else {
                windows_remediation(message, &overall)
            },
        });
    }
    let mut result = base("windows", script, exit_code, overall, &clean);
    result.items = items;
    classify(&mut result);
    if exit_code == 1 {
        result.desktop_ready = false;
    }
    if exit_code == 2 {
        result.agent_ready = false;
    }
    summarize(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINUX_OPTIONAL: &str = "Verifying runtime prerequisites: debian (family: debian)\n\n  MISSING  command    incus\n  MISSING  library    libzbar.so.0*\n  MISSING  package    libzbar0t64 or libzbar0\n  MISSING  group      incus-admin\n  MISSING  service    incus daemon not reachable (sudo systemctl enable --now incus)\n\n2 runtime prerequisite(s) missing.\nInstall with: sudo /opt/general-bots/PROD-DEPENDENCIES.sh --install\n";

    const LINUX_DESKTOP: &str = "  MISSING  command    curl\n  MISSING  library    libgtk-3.so.0*\n  MISSING  group      incus-admin\n";

    const WINDOWS_DESKTOP: &str = "--- Desktop shell ---\n  [ok]      WebView2 runtime 125.0\n  [missing] libpq.dll not found on PATH.\n  [warn]    BotModels pythonbox is Linux x86_64 only; vision/STT/TTS stay unavailable on Windows.\n\n--- Agent mode (WSL2 + Incus) ---\n  [missing] wsl.exe not found: WSL2 is not installed.\n\nResult: DESKTOP BLOCKED - fix the [missing] items above.\n";

    const WINDOWS_AGENT: &str = "--- Desktop shell ---\n  [ok]      WebView2 runtime 125.0\n  [warn]    BotModels pythonbox is Linux x86_64 only; vision/STT/TTS stay unavailable on Windows.\n\n--- Agent mode (WSL2 + Incus) ---\n  [missing] systemd is not running in 'Debian' (state: stopped).\n\nResult: AGENT BLOCKED - desktop Chat/Drive can run; agent mode cannot.\n";

    fn linux_result(output: &str, exit_code: i32) -> HostCheckResult {
        linux(
            output,
            exit_code,
            Path::new("/opt/general-bots/PROD-DEPENDENCIES.sh"),
        )
    }

    fn windows_result(output: &str, exit_code: i32) -> HostCheckResult {
        windows(
            output,
            exit_code,
            Path::new("C:\\GeneralBots\\PROD-DEPENDENCIES.ps1"),
        )
    }

    #[test]
    fn linux_optional_gaps_do_not_block_the_desktop() {
        let result = linux_result(LINUX_OPTIONAL, 1);
        assert!(result.desktop_ready);
        assert!(!result.agent_ready);
        assert!(!result.media_ready);
        assert_eq!(result.status, "limited");
        assert!(result
            .remediation
            .contains("PROD-DEPENDENCIES.sh --install"));
        assert_eq!(result.items.len(), 5);
        assert!(result
            .items
            .iter()
            .any(|item| item.feature == "agent" && item.name == "incus-admin"));
        assert!(result
            .items
            .iter()
            .any(|item| item.kind == "service" && item.remediation.contains("systemctl")));
    }

    #[test]
    fn linux_desktop_gap_blocks_setup() {
        let result = linux_result(LINUX_DESKTOP, 1);
        assert!(!result.desktop_ready);
        assert_eq!(result.status, "desktop-blocked");
        assert!(result.message.contains("desktop shell"));
    }

    #[test]
    fn linux_ready_report_is_clean() {
        let result = linux_result(
            "Verifying runtime prerequisites: debian\n\nAll runtime prerequisites are satisfied.\n",
            0,
        );
        assert!(result.desktop_ready && result.agent_ready && result.media_ready);
        assert_eq!(result.status, "ready");
        assert!(result.items.is_empty());
    }

    #[test]
    fn windows_exit_one_blocks_the_desktop() {
        let result = windows_result(WINDOWS_DESKTOP, 1);
        assert!(!result.desktop_ready);
        assert!(!result.media_ready);
        assert_eq!(result.status, "desktop-blocked");
        assert!(result
            .items
            .iter()
            .any(|item| item.remediation.contains("EdgeWebView2") || item.name.contains("libpq")));
        let media = result.items.iter().find(|item| item.feature == "media");
        assert!(media.is_some());
        assert_eq!(media.map(|item| item.remediation.as_str()), Some(""));
    }

    #[test]
    fn windows_exit_two_keeps_the_desktop_usable() {
        let result = windows_result(WINDOWS_AGENT, 2);
        assert!(result.desktop_ready);
        assert!(!result.agent_ready);
        assert_eq!(result.status, "limited");
        assert!(!result.media_ready);
    }

    #[test]
    fn ansi_sequences_are_stripped() {
        let text = strip_ansi("\u{1b}[0;32m  MISSING  command    incus\u{1b}[0m\r\n");
        assert_eq!(text, "  MISSING  command    incus\n");
    }
}
