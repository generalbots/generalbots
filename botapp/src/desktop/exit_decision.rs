//! Pure decision logic for the `ExitRequested` handler.
//!
//! Extracted so the branching can be unit-tested without a Tauri runtime. The
//! actual handler in `main.rs` reads window visibility, tray state and activity
//! state and feeds the results here.

/// What the app should do when Tauri fires `ExitRequested`.
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
/// * `main_window_visible` — the suite window exists and is visible.
/// * `reply_in_progress` — a model reply is still being generated.
pub fn on_exit_requested(
    quit_from_tray: bool,
    main_window_visible: bool,
    reply_in_progress: bool,
) -> ExitAction {
    if quit_from_tray {
        return ExitAction::Exit;
    }
    if main_window_visible {
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

    #[test]
    fn tray_quit_always_exits() {
        assert_eq!(on_exit_requested(true, true, true), ExitAction::Exit);
        assert_eq!(on_exit_requested(true, false, false), ExitAction::Exit);
    }

    #[test]
    fn visible_main_window_keeps_the_process_alive() {
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
    fn wizard_closed_during_first_run_cancels_and_exits() {
        // main window hidden (not visible), no reply in progress, no tray quit
        assert_eq!(
            on_exit_requested(false, false, false),
            ExitAction::CancelAndExit
        );
    }
}
