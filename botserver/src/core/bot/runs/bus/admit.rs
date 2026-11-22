//! What happens to a message the person sends to a conversation.
//!
//! Nothing runs: the message starts a turn. A turn runs: the message steers
//! it at its next step (#1677). A turn runs but waits on a permission card:
//! the person's new message wins, so the turn is stopped (its card is
//! withdrawn as cancelled) and the message starts the next turn once the
//! stopped one has ended. The decision and its effect happen under the run's
//! lock, so they cannot interleave with the turn ending, and a stopped turn
//! never takes the queued message as steering (see `take_steering`).

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use botevents::RunState;

use super::{RunBus, ThreadChannel};

/// The most messages queued for a running turn.
const STEERING_CAP: usize = 20;

/// The fate of one inbound message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// It starts a new turn, already marked as the conversation's run.
    Start,
    /// It is queued for the running turn.
    Steer,
    /// The running turn is stopped and the message starts the next one.
    Replace,
}

/// The rule. `running`: a turn runs and was not stopped. `stopping_queued`:
/// a stopped turn has not ended yet and already holds a message, which it
/// starts when it ends (a new message joins that queue instead of starting a
/// second turn beside it). `card_waiting`: the running turn waits on a
/// blocking permission request.
#[must_use]
pub fn admission(running: bool, stopping_queued: bool, card_waiting: bool) -> Admission {
    if running {
        if card_waiting {
            Admission::Replace
        } else {
            Admission::Steer
        }
    } else if stopping_queued {
        Admission::Steer
    } else {
        Admission::Start
    }
}

fn lock_queue(queue: &Mutex<VecDeque<String>>) -> MutexGuard<'_, VecDeque<String>> {
    match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

impl ThreadChannel {
    /// Admits `text`; on [`Admission::Start`] the thread now belongs to
    /// `new_run`, and the caller spawns that turn.
    pub fn admit(&self, text: &str, card_waiting: bool, new_run: &str) -> Admission {
        let mut meta = self.lock_meta();
        let mut queue = lock_queue(&self.steering);
        let running = meta.state == RunState::Running && !meta.cancelled;
        let stopping_queued = meta.cancelled && !meta.state.is_terminal() && !queue.is_empty();
        let admitted = admission(running, stopping_queued, card_waiting);
        match admitted {
            Admission::Start => {
                meta.run_id = new_run.to_string();
                meta.state = RunState::Running;
                meta.cancelled = false;
            }
            Admission::Steer => {
                if queue.len() < STEERING_CAP {
                    queue.push_back(text.to_string());
                }
            }
            Admission::Replace => {
                queue.push_back(text.to_string());
                meta.cancelled = true;
                meta.state = RunState::Cancelling;
            }
        }
        admitted
    }

    /// The message that starts the next turn, taken at a turn boundary: also
    /// after a stop, when [`ThreadChannel::take_steering`] holds it back.
    pub fn take_next_turn(&self) -> Option<String> {
        lock_queue(&self.steering).pop_front()
    }
}

impl RunBus {
    /// See [`ThreadChannel::admit`].
    pub fn admit(&self, thread_id: &str, text: &str, card_waiting: bool, new_run: &str) -> Admission {
        self.thread(thread_id).admit(text, card_waiting, new_run)
    }

    /// See [`ThreadChannel::take_next_turn`].
    pub fn take_next_turn(&self, thread_id: &str) -> Option<String> {
        self.thread(thread_id).take_next_turn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_while_a_card_waits_replaces_the_turn() {
        assert_eq!(admission(true, false, true), Admission::Replace);
    }

    #[test]
    fn a_message_while_a_turn_runs_without_a_card_steers_it() {
        assert_eq!(admission(true, false, false), Admission::Steer);
    }

    #[test]
    fn a_message_to_an_idle_conversation_starts_a_turn() {
        assert_eq!(admission(false, false, false), Admission::Start);
        assert_eq!(admission(false, false, true), Admission::Start);
    }

    #[test]
    fn a_message_while_a_replaced_turn_ends_joins_the_queue() {
        assert_eq!(admission(false, true, false), Admission::Steer);
    }

    #[test]
    fn replacing_stops_the_turn_and_keeps_the_message_for_the_next_one() {
        let channel = ThreadChannel::default();
        assert_eq!(channel.admit("first", false, "run-1"), Admission::Start);
        assert_eq!(channel.current_run_id().as_deref(), Some("run-1"));
        assert_eq!(channel.admit("new question", true, "run-2"), Admission::Replace);
        assert!(channel.is_cancelled());
        assert_eq!(channel.current_run_id().as_deref(), Some("run-1"));
        assert_eq!(channel.take_steering(), None, "the stopped turn must not consume it");
        assert_eq!(channel.admit("and this", false, "run-3"), Admission::Steer);
        channel.finish(RunState::Succeeded);
        assert_eq!(channel.state(), RunState::Cancelled);
        assert_eq!(channel.take_next_turn().as_deref(), Some("new question"));
        assert_eq!(channel.take_next_turn().as_deref(), Some("and this"));
    }

    #[test]
    fn steering_a_running_turn_is_unchanged() {
        let channel = ThreadChannel::default();
        assert_eq!(channel.admit("first", false, "run-1"), Admission::Start);
        assert_eq!(channel.admit("also", false, "run-2"), Admission::Steer);
        assert!(!channel.is_cancelled());
        assert_eq!(channel.take_steering().as_deref(), Some("also"));
    }
}
