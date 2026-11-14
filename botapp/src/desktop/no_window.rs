//! Starts child processes without a console window on Windows.
//!
//! The desktop shell is a windowed program. A console program it starts — the
//! botserver sidecar, the provisioner, a sync tool — is given a console window
//! of its own unless the parent asks for none, and everything that child
//! starts inherits it. The user then sees a terminal next to the application.

/// `CREATE_NO_WINDOW`: the child gets a console without a window.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) trait NoConsoleWindow {
    /// No-op outside Windows.
    fn no_console_window(&mut self) -> &mut Self;
}

impl NoConsoleWindow for std::process::Command {
    fn no_console_window(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            self.creation_flags(CREATE_NO_WINDOW);
        }
        self
    }
}

impl NoConsoleWindow for tokio::process::Command {
    fn no_console_window(&mut self) -> &mut Self {
        #[cfg(windows)]
        self.creation_flags(CREATE_NO_WINDOW);
        self
    }
}
