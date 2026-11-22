//! A deleted conversation takes its virtual machine permissions with it: its
//! "this conversation" and "allow all" grants of every agent are deleted and
//! its open cards are withdrawn (each waiting action ends as cancelled).

use uuid::Uuid;

/// Called by the conversation delete route after the conversation is gone.
/// Failures are logged; the conversation stays deleted either way.
pub async fn forget_conversation(session_id: Uuid) {
    let Some(service) = crate::core::bot::consent_gate::shared_service() else {
        return;
    };
    let thread = crate::core::bot::runs::bus::thread_id(session_id);
    let withdrawn = botconsent::withdraw_thread(&service, &thread).await;
    let pool = service.pool.clone();
    let revoked = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("pool unavailable: {e}"))?;
        botconsent::store::revoke_conversation_vm_grants(&mut conn, session_id)
    })
    .await;
    match revoked {
        Ok(Ok(grants)) => botlog::safe_info!(
            "vm consent: conversation {session_id} deleted; grants_revoked={grants} requests_withdrawn={withdrawn}"
        ),
        Ok(Err(e)) => botlog::safe_warn!("vm consent: grants of deleted conversation {session_id} not revoked: {e}"),
        Err(e) => botlog::safe_warn!("vm consent: grant cleanup task for {session_id} failed: {e}"),
    }
}
