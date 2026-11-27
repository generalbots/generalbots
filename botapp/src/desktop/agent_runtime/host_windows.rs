//! Windows-specific host implementations: registry reads, WSL detection,
//! disk space and UAC elevation.

use std::io;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

fn reg_query_sz(key: &str, value: &str) -> Option<String> {
    let output = std::process::Command::new("reg")
        .args(["query", key, "/v", value])
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(value) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix("REG_SZ") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

fn reg_query_dword(key: &str, value: &str) -> Option<u32> {
    let output = std::process::Command::new("reg")
        .args(["query", key, "/v", value])
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(value) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix("REG_DWORD") {
                let hex = rest.trim();
                let hex = hex.strip_prefix("0x").unwrap_or(hex);
                return u32::from_str_radix(hex, 16).ok();
            }
        }
    }
    None
}

fn reg_key_exists(key: &str) -> bool {
    std::process::Command::new("reg")
        .args(["query", key])
        .creation_flags(0x0800_0000)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(super) fn read_os_build() -> u32 {
    reg_query_sz(
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "CurrentBuildNumber",
    )
    .and_then(|s| s.parse().ok())
    .unwrap_or(0)
}

pub(super) fn wsl_policy_blocked() -> bool {
    reg_query_dword(
        r"HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsSubsystemForLinux",
        "AllowWSL",
    )
    .is_some_and(|v| v == 0)
}

pub(super) fn wsl_installed() -> bool {
    // The inbox WSL registers `LxssManager`; the Store/MSI WSL that current
    // Windows 11 installs registers `WslService` and may not have the old key.
    const SERVICES: [&str; 2] = [
        r"HKLM\SYSTEM\CurrentControlSet\Services\WslService",
        r"HKLM\SYSTEM\CurrentControlSet\Services\LxssManager",
    ];
    if !SERVICES.iter().any(|key| reg_key_exists(key)) {
        return false;
    }
    let wsl = wsl_exe_path();
    std::process::Command::new(&wsl)
        .args(["--status"])
        .env("WSL_UTF8", "1")
        .creation_flags(0x0800_0000)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn wsl_exe_path() -> PathBuf {
    let system_root = std::env::var("SystemRoot")
        .unwrap_or_else(|_| r"C:\Windows".to_string());
    PathBuf::from(system_root).join("System32").join("wsl.exe")
}

pub(super) fn virtualization_available() -> bool {
    if reg_key_exists(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Virtualization") {
        return true;
    }
    true
}

pub(super) fn wsl_reboot_pending() -> bool {
    reg_key_exists(
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
    )
}

pub(super) fn distro_registered(name: &str) -> bool {
    distro_base_path(name).is_some()
}

pub(super) fn distro_base_path(name: &str) -> Option<PathBuf> {
    let lxss_key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss";
    let output = std::process::Command::new("reg")
        .args(["query", lxss_key])
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("HKEY_CURRENT_USER") {
            continue;
        }
        let subkey = trimmed.replacen("HKEY_CURRENT_USER", "HKCU", 1);
        if let Some(distro_name) = reg_query_sz(&subkey, "DistributionName") {
            if distro_name.eq_ignore_ascii_case(name) {
                return reg_query_sz(&subkey, "BasePath").map(PathBuf::from);
            }
        }
    }
    None
}

pub(super) fn disk_free_space(path: &Path) -> io::Result<u64> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut free_bytes: u64 = 0;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(free_bytes)
}

/// Runs exactly `<System32>\wsl.exe --install --no-distribution` through
/// a UAC prompt.  Takes no caller-supplied arguments: the elevated command
/// line is fixed so nothing user-controlled can reach it.
pub(super) fn run_wsl_install_elevated() -> io::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wsl = wsl_exe_path();
    let params = "--install --no-distribution";
    let operation: Vec<u16> =
        OsStr::new("runas").encode_wide().chain(Some(0)).collect();
    let file: Vec<u16> =
        wsl.as_os_str().encode_wide().chain(Some(0)).collect();
    let parameters: Vec<u16> =
        OsStr::new(params).encode_wide().chain(Some(0)).collect();

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            std::ptr::null(),
            0,
        )
    };
    if (result as isize) <= 32 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("ShellExecuteW returned {}", result as isize),
        ));
    }
    Ok(())
}

#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        directory: *const u16,
        free_bytes_available: *mut u64,
        total_bytes: *mut u64,
        total_free_bytes: *mut u64,
    ) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: *mut std::ffi::c_void,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_cmd: i32,
    ) -> *mut std::ffi::c_void;
}
