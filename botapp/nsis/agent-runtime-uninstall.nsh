; Agent runtime cleanup for the NSIS uninstaller.
;
; The hook calls the app binary with a hidden switch that runs the Rust
; removal code (BasePath verification inside the product's data folder,
; silent, no window, exits with a code).  This avoids inline registry
; parsing in NSIS and reuses the same safety logic as the app.
;
; Reference in botapp/tauri.gbgov.conf.json -> bundle.windows.nsis:
;   "installerHooks": "nsis/agent-runtime-uninstall.nsh"
;
; The app binary must handle --agent-runtime-remove-silent as a hidden
; CLI switch (see botapp/src/desktop/agent_runtime/commands.rs).
; The switch must do nothing unless the argument is exactly that string.

!macro NSIS_HOOK_PREUNINSTALL
  ${If} $DeleteAppDataCheckboxState == 1
    ; Only attempt removal if wsl.exe exists (WSL may never have been
    ; installed on this machine).
    IfFileExists "$SYSDIR\wsl.exe" 0 +3
      ; Call the app binary with the hidden removal switch.
      nsExec::ExecToStack '"$INSTDIR\generalbots.exe" --agent-runtime-remove-silent'
      Pop $0
    ; Remove the agent-runtime directory regardless.
    RMDir /r "$LOCALAPPDATA\generalbots\stack\agent-runtime"
  ${EndIf}
!macroend
