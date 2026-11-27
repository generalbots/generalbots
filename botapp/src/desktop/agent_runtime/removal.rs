//! Removal of the agent runtime: unregister the WSL distro, delete the
//! marker and state files, and optionally export the distro first.
//!
//! Safety: the distro is unregistered ONLY when its registered `BasePath`
//! is inside `<stack>/agent-runtime/distro`.

use std::path::{Path, PathBuf};

use super::host::{self, RuntimeHost};
use super::state::DISTRO_NAME;

#[derive(Debug)]
pub(crate) enum RemovalResult {
    Ok,
    NotRegistered,
    ForeignDistro { base_path: PathBuf },
    CommandFailed(String),
    IoError(String),
}

pub(crate) fn remove(
    h: &dyn RuntimeHost,
    stack: &Path,
    keep_export: bool,
) -> RemovalResult {
    let rt_dir = stack.join("agent-runtime");
    let distro_dir = rt_dir.join("distro");

    if !h.distro_registered(DISTRO_NAME) {
        cleanup_files(h, &rt_dir);
        return RemovalResult::NotRegistered;
    }

    if let Some(base) = h.distro_base_path(DISTRO_NAME) {
        if !host::path_is_inside(&base, &distro_dir) {
            return RemovalResult::ForeignDistro { base_path: base };
        }
    } else {
        return RemovalResult::ForeignDistro {
            base_path: PathBuf::from("(unknown)"),
        };
    }

    let wsl = wsl_exe();
    let ws = wsl.to_string_lossy();

    if keep_export {
        let export_path = rt_dir.join("GeneralBotsLite-export.tar");
        match h.run_command(&ws, &["--export", DISTRO_NAME, &export_path.to_string_lossy()]) {
            Ok((0, _, _)) => {
                log::info!("agent_runtime: exported distro to {}", export_path.display());
            }
            Ok((code, _, stderr)) => {
                log::warn!("agent_runtime: wsl --export failed ({code}): {stderr}");
                return RemovalResult::CommandFailed("wsl --export failed".into());
            }
            Err(e) => {
                return RemovalResult::CommandFailed(format!("cannot run wsl --export: {e}"));
            }
        }
    }

    match h.run_command(&ws, &["--unregister", DISTRO_NAME]) {
        Ok((0, _, _)) => log::info!("agent_runtime: distro unregistered"),
        Ok((code, _, stderr)) => {
            log::warn!("agent_runtime: wsl --unregister failed ({code}): {stderr}");
            return RemovalResult::CommandFailed("wsl --unregister failed".into());
        }
        Err(e) => {
            return RemovalResult::CommandFailed(format!("cannot run wsl --unregister: {e}"));
        }
    }

    cleanup_files(h, &rt_dir);
    RemovalResult::Ok
}

fn cleanup_files(h: &dyn RuntimeHost, rt_dir: &Path) {
    let _ = h.remove_file(&rt_dir.join("ready.json"));
    let _ = h.remove_file(&rt_dir.join("state.json"));
    let _ = h.remove_dir_all(rt_dir);
}

fn wsl_exe() -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var("SystemRoot")
            .unwrap_or_else(|_| r"C:\Windows".to_string());
        PathBuf::from(root).join("System32").join("wsl.exe")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("wsl")
    }
}

/// NSIS uninstall hook.  Instead of inline registry parsing in NSIS, the
/// hook calls the app's own executable with a hidden switch that runs the
/// Rust removal code (silent, no window, exits with a code).
///
/// The installer must `!include` this file and the app binary must handle
/// `--agent-runtime-remove-silent` as a hidden CLI switch.
pub(crate) const NSIS_HOOK: &str = r#"; Agent runtime cleanup for the NSIS uninstaller.
; The hook calls the app binary with a hidden switch that runs the
; Rust removal code (BasePath verification, etc.).
; Include in botapp/tauri.gblite.conf.json -> bundle.windows.nsis:
;   "installerHooks": "nsis/agent-runtime-uninstall.nsh"

!macro customUninstall
  ${If} $DeleteAppData == 1
    ; Check whether wsl.exe exists before trying anything.
    IfFileExists "$SYSDIR\wsl.exe" 0 +3
      ; Call the app binary with the hidden removal switch.
      ; It verifies the BasePath is inside this product's data folder
      ; before unregistering anything.  Exits silently with 0 or non-zero.
      nsExec::ExecToStack '"$INSTDIR\generalbots.exe" --agent-runtime-remove-silent'
      Pop $0
    ; Remove the agent-runtime directory regardless (the removal may have
    ; already cleaned it, but RMDir /r on a missing dir is harmless).
    RMDir /r "$LOCALAPPDATA\generalbots\stack\agent-runtime"
  ${EndIf}
!macroend
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::agent_runtime::tests::FakeHost;

    #[test]
    fn removal_refuses_foreign_distro() {
        let host = FakeHost::new()
            .with_distro("GeneralBotsLite", "/other/path");
        let result = remove(&host, Path::new("/stack"), false);
        assert!(matches!(result, RemovalResult::ForeignDistro { .. }));
    }

    #[test]
    fn removal_succeeds_for_our_distro() {
        let host = FakeHost::new()
            .with_distro("GeneralBotsLite", "/stack/agent-runtime/distro")
            .with_command_result("wsl", 0, "", "");
        let result = remove(&host, Path::new("/stack"), false);
        assert!(matches!(result, RemovalResult::Ok));
    }

    #[test]
    fn removal_succeeds_with_extended_prefix() {
        let host = FakeHost::new()
            .with_distro("GeneralBotsLite", r"\\?\C:\stack\agent-runtime\distro")
            .with_command_result("wsl", 0, "", "");
        let result = remove(&host, Path::new(r"C:\stack"), false);
        assert!(matches!(result, RemovalResult::Ok), "got: {:?}", result);
    }

    #[test]
    fn removal_reports_not_registered() {
        let host = FakeHost::new();
        let result = remove(&host, Path::new("/stack"), false);
        assert!(matches!(result, RemovalResult::NotRegistered));
    }

    #[test]
    fn nsis_hook_calls_app_binary() {
        assert!(NSIS_HOOK.contains("--agent-runtime-remove-silent"));
        assert!(NSIS_HOOK.contains("wsl.exe"));
        assert!(NSIS_HOOK.contains("generalbots.exe"));
    }
}
