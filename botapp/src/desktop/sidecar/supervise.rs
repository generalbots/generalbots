//! What became of the sidecar this app spawned, for the boot watcher.

use super::Sidecar;
use log::warn;

/// State of the child process this handle owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildState {
    /// The spawned sidecar is still running.
    Running,
    /// The spawned sidecar has exited, with its exit code when it had one.
    Exited(Option<i32>),
    /// This handle owns no child: the app attached to an assistant it did not
    /// spawn, or another caller already collected the exit.
    NotOwned,
}

impl Sidecar {
    /// Polls the owned child without blocking. An exit is reported once and
    /// the handle is released, as `is_running` does.
    pub(crate) fn child_state(&self) -> ChildState {
        let mut guard = match self.inner.child.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("sidecar handle mutex was poisoned while polling; recovering it");
                poisoned.into_inner()
            }
        };
        let Some(child) = guard.as_mut() else {
            return ChildState::NotOwned;
        };
        match child.try_wait() {
            Ok(None) => ChildState::Running,
            Ok(Some(status)) => {
                *guard = None;
                ChildState::Exited(status.code())
            }
            Err(e) => {
                warn!("could not poll the sidecar process: {e}");
                *guard = None;
                ChildState::Exited(None)
            }
        }
    }
}
