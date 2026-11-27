//! Pure decision logic for window-close and exit-requested handlers.
//!
//! Extracted so the branching can be unit-tested without a Tauri runtime. The
//! actual handlers in `main.rs` read window state, tray state and activity
//! state and feed the results here.

/// What the app should do when a window is destroyed during the first-run
/// wizard phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAction {
    /// First run was not completed: cancel provisioning, stop every stack
    /// process, destroy remaining windows and exit.
    AbortFirstRun,
    /// Nothing to do (handoff already succeeded, or this is not a first-run
    /// window).
    Noop,
}

/// Decide what to do when a window is destroyed.
///
/// * `label` — the Tauri window label ("wizard" or "main").
/// * `handoff_complete` — the wizard finished and the suite was handed off.
/// * `is_first_run` — the first-run marker file has not been written yet.
pub fn on_window_destroyed(
    label: &str,
    handoff_complete: bool,
    is_first_run: bool,
) -> WindowAction {
    // After handoff the wizard closes normally and the suite shows. After
    // second launch is_first_run is false. Either way, nothing to do.
    if handoff_complete || !is_first_run {
        return WindowAction::Noop;
    }
    // During first run, before handoff, destroying either the wizard (user
    // closed it) or the hidden main window (End Task / CloseMainWindow) means
    // the person wants to leave.
    if label == "wizard" || label == "main" {
        return WindowAction::AbortFirstRun;
    }
    WindowAction::Noop
}

/// What the app should do when Tauri fires `ExitRequested` (all OS windows
/// are gone).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitAction {
    /// Stop the sidecar and the stack, then let the process end.
    Exit,
    /// Cancel any running provisioning, stop the sidecar and the stack, then
    /// let the process end (wizard was the last visible window).
    CancelAndExit,
    /// Keep the process alive: the suite window is on screen.
    KeepAlive,
    /// Keep the process alive and recreate the suite window so a running reply
    /// can land.
    KeepAliveAndRecreate,
}

/// Decide what to do when `ExitRequested` fires.
///
/// * `quit_from_tray` — the tray's Quit action was selected.
/// * `main_window_exists` — the main window still exists (tray-quit fires
///   while it does).
/// * `reply_in_progress` — a model reply is still being generated.
pub fn on_exit_requested(
    quit_from_tray: bool,
    main_window_exists: bool,
    reply_in_progress: bool,
) -> ExitAction {
    if quit_from_tray {
        return ExitAction::Exit;
    }
    if main_window_exists {
        return ExitAction::KeepAlive;
    }
    if reply_in_progress {
        return ExitAction::KeepAliveAndRecreate;
    }
    ExitAction::CancelAndExit
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- on_window_destroyed ---

    #[test]
    fn wizard_closed_before_handoff_aborts_first_run() {
        assert_eq!(
            on_window_destroyed("wizard", false, true),
            WindowAction::AbortFirstRun
        );
    }

    #[test]
    fn main_window_closed_before_handoff_aborts_first_run() {
        // End Task or CloseMainWindow hits the hidden main window.
        assert_eq!(
            on_window_destroyed("main", false, true),
            WindowAction::AbortFirstRun
        );
    }

    #[test]
    fn wizard_closed_after_handoff_is_a_noop() {
        // Handoff succeeded, wizard closes normally.
        assert_eq!(
            on_window_destroyed("wizard", true, false),
            WindowAction::Noop
        );
    }

    #[test]
    fn wizard_closed_after_handoff_while_marker_not_yet_flushed() {
        // handoff_complete is set before the marker file is written, so
        // is_first_run may still be true for a moment.
        assert_eq!(
            on_window_destroyed("wizard", true, true),
            WindowAction::Noop
        );
    }

    #[test]
    fn main_window_closed_after_first_run_is_a_noop() {
        // Post-setup: the main window closing is handled by ExitRequested.
        assert_eq!(
            on_window_destroyed("main", false, false),
            WindowAction::Noop
        );
    }

    #[test]
    fn unrelated_window_during_first_run_is_a_noop() {
        assert_eq!(
            on_window_destroyed("splash", false, true),
            WindowAction::Noop
        );
    }

    // --- on_exit_requested ---

    #[test]
    fn tray_quit_always_exits() {
        assert_eq!(on_exit_requested(true, true, true), ExitAction::Exit);
        assert_eq!(on_exit_requested(true, false, false), ExitAction::Exit);
    }

    #[test]
    fn main_window_exists_keeps_the_process_alive() {
        assert_eq!(
            on_exit_requested(false, true, false),
            ExitAction::KeepAlive
        );
        assert_eq!(
            on_exit_requested(false, true, true),
            ExitAction::KeepAlive
        );
    }

    #[test]
    fn busy_reply_recreates_the_suite_window() {
        assert_eq!(
            on_exit_requested(false, false, true),
            ExitAction::KeepAliveAndRecreate
        );
    }

    #[test]
    fn no_windows_and_not_busy_cancels_and_exits() {
        assert_eq!(
            on_exit_requested(false, false, false),
            ExitAction::CancelAndExit
        );
    }
}
