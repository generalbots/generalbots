//! Test-rig stand-in for the virtual machine.
//!
//! The test rig has no virtual machine. With `GB_RIG_FAKE_VM=1`, which only
//! the rig's start command sets (the product never does), an allowed action
//! "runs" here: nothing executes and a fixed transcript comes back, so the
//! consent card, the steps block and the result path can be checked end to
//! end. The consent ticket is still required: this sits below the gate.

const ENV: &str = "GB_RIG_FAKE_VM";

pub(crate) fn enabled() -> bool {
    let on = std::env::var(ENV).is_ok_and(|value| value == "1");
    if on {
        botlog::safe_warn!("{ENV}=1: virtual machine actions are simulated (test rig)");
    }
    on
}

pub(crate) fn run(language: &str, code: &str) -> (bool, String, String) {
    let lines = code.lines().count();
    let stdout = format!(
        "[rig] {language}: {lines} line(s), {} byte(s) accepted\n[rig] nothing was executed\n[rig] exit 0\n",
        code.len()
    );
    (true, stdout, String::new())
}

/// The running workspace the rig pretends to have, so the run header shows
/// what the product shows once the Virtual Machine is up.
pub(crate) fn session_row(
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
) -> botagent::models::AgentSessionRow {
    let now = chrono::Utc::now();
    botagent::models::AgentSessionRow {
        id: uuid::Uuid::new_v4(),
        session_id: session_id.to_string(),
        user_id,
        org_id: None,
        branch_id: None,
        bot_id: bot_uuid,
        vm_name: "rig-fake-vm".to_string(),
        status: botagent::vm::STATUS_RUNNING.to_string(),
        last_active_at: now,
        expires_at: None,
        metadata: None,
        created_at: now,
        active_conversation_id: Some(session_id.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_fake_workspace_is_running_in_its_conversation() {
        let session = uuid::Uuid::new_v4();
        let row = super::session_row(session, uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
        assert_eq!(row.status, botagent::vm::STATUS_RUNNING);
        assert_eq!(row.active_conversation_id, Some(session.to_string()));
    }
}
