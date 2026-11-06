//! Golden-transcript replay.
//!
//! The reducer is the reference client: it consumes frames in `seq` order,
//! refuses gaps, ignores replayed `event_id`s and enforces the run state
//! machine. A recorded transcript that replays to the same state twice proves
//! the stream is deterministic.
//!
//! Split into the state machine (`state`), the event-family reducer
//! (`families`) and the JSON-patch subset (`patch`); every public item keeps
//! its `frames::replay::` path through the re-exports below.

use botevents::Envelope;

mod families;
mod patch;
mod state;

#[cfg(test)]
mod tests;

pub use patch::apply_patch;
pub use state::{
    CustomRecord, MessageView, ReplayError, ReplayState, RunView, StepView, ToolView,
};

/// Replays a complete transcript.
pub fn replay(frames: &[Envelope]) -> Result<ReplayState, ReplayError> {
    let mut state = ReplayState::default();
    for frame in frames {
        state.apply(frame)?;
    }
    Ok(state)
}
