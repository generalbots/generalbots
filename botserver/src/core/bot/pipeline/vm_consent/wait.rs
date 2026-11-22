//! Waiting on the person's answer to one card.
//!
//! A request lives as long as the turn that raised it
//! (`botconsent::turn_superseded`): a message the person sends while that
//! turn runs is queued for it and leaves the card answerable; a stopped turn,
//! or a new turn started by the person's next message, withdraws the request,
//! closes the card and marks the step cancelled. So does a turn whose future
//! is dropped (its time limit), through [`WithdrawOnDrop`].

use std::sync::Arc;
use std::time::Duration;

use botconsent::{ConsentService, Decision};
use tokio::sync::oneshot;

use super::{AllowSource, VmDenial};

/// How often the turn's state is looked at while a card is open.
const TURN_POLL: Duration = Duration::from_millis(500);

/// Withdraws the request when the waiting future is dropped before it ended
/// (the turn was abandoned), so no card stays answerable for an action
/// nobody waits for.
/// While it lives the card counts as progress of the turn of `thread`.
struct WithdrawOnDrop {
    service: Arc<ConsentService>,
    request_id: String,
    armed: bool,
    thread: String,
}

impl Drop for WithdrawOnDrop {
    fn drop(&mut self) {
        crate::core::bot::pipeline::turn_progress::busy(&self.thread, false);
        if !self.armed {
            return;
        }
        let service = Arc::clone(&self.service);
        let request_id = std::mem::take(&mut self.request_id);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                botconsent::withdraw(&service, &request_id).await;
            });
        }
    }
}

/// Waits for the decision on `request_id`, raised in run `raised_in` of
/// conversation `thread`: up to [`super::DECISION_WAIT`], until the turn is
/// superseded, or until the request is withdrawn elsewhere (the conversation
/// was deleted).
pub(super) async fn wait(
    service: &Arc<ConsentService>,
    request_id: &str,
    thread: &str,
    raised_in: Option<&str>,
    answer: oneshot::Receiver<Decision>,
) -> Result<AllowSource, VmDenial> {
    let mut guard = WithdrawOnDrop {
        service: Arc::clone(service),
        request_id: request_id.to_string(),
        armed: true,
        thread: thread.to_string(),
    };
    crate::core::bot::pipeline::turn_progress::busy(thread, true);
    let bus = crate::core::bot::runs::bus::bus();
    let cancelled_at_raise = bus.is_cancelled(thread);
    let superseded = async {
        loop {
            tokio::time::sleep(TURN_POLL).await;
            let current = bus.current_run_id(thread);
            if botconsent::turn_superseded(raised_in, current.as_deref(), bus.is_cancelled(thread), cancelled_at_raise) {
                break;
            }
        }
    };
    let outcome = tokio::select! {
        decided = answer => super::gate::decision_source(decided.ok()),
        () = tokio::time::sleep(super::DECISION_WAIT) => Err(VmDenial::NoAnswer),
        () = superseded => Err(VmDenial::Cancelled),
    };
    guard.armed = false;
    if matches!(outcome, Err(VmDenial::NoAnswer | VmDenial::Cancelled)) {
        botconsent::withdraw(service, request_id).await;
    }
    outcome
}
