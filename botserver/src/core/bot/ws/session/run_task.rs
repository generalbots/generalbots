use std::sync::Arc;

use botcore::shared::state::AppState;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::bus_sink::RunBusSink;
use crate::core::bot::pipeline::{self, SessionIdentity};
use crate::core::bot::runs::bus;

/// Everything a background turn needs; built by the inbound frame handler.
pub(super) struct RunSpec {
    pub(super) state: Arc<AppState>,
    pub(super) bot_uuid: Uuid,
    pub(super) session_id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) bot_name: String,
    pub(super) thread_id: String,
    pub(super) text: String,
    pub(super) start_bas_ran: bool,
}

pub(super) fn spawn(spec: RunSpec) {
    let RunSpec {
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        thread_id,
        text,
        start_bas_ran,
    } = spec;
    tokio::spawn(async move {
        let sink = RunBusSink::new(session_id, state.clone());
        let identity = SessionIdentity {
            sink: &sink,
            state: &state,
            bot_uuid,
            session_id,
            user_id,
            bot_name: &bot_name,
            // Placeholder only: `process_message_internal` re-resolves the
            // chat Agent/VM switch per turn (payload or stored session value)
            // before any pipeline stage or tool can dispatch a workspace run.
            agent_vm_enabled: true,
        };
        let mut next_text = Some(text);
        let mut first_turn = true;
        let mut task_start_bas_ran = start_bas_ran;
        while let Some(turn_text) = next_text {
            if !first_turn {
                let next_run = Uuid::new_v4().to_string();
                bus::bus().start(&thread_id, &next_run);
            }
            let (closed_tx, mut task_rx) = mpsc::channel::<botlib::models::BotResponse>(1);
            drop(closed_tx);
            let result = pipeline::process_message_internal(
                &identity,
                &mut task_rx,
                &mut task_start_bas_ran,
                &turn_text,
            )
            .await;
            match result {
                Ok(()) => bus::bus().finish(&thread_id, botevents::RunState::Succeeded),
                Err(error) => {
                    log::error!("pipeline task failed: {error}");
                    bus::bus().finish(&thread_id, botevents::RunState::Failed);
                    break;
                }
            }
            // Deliver steering at the turn boundary. The queued text is
            // processed exactly once.
            next_text = bus::bus().take_steering(&thread_id);
            first_turn = false;
        }
    });
}
