//! [`ask`]: stored grant, or a card the tool call waits on.

use super::steps;
use super::{AllowSource, VmAction, VmConsent, VmDecision, VmDenial};
use crate::core::bot::pipeline::types::SessionIdentity;

/// Asks the person (or a stored grant of this conversation) whether `action`
/// may run, and waits for the answer: up to [`super::DECISION_WAIT`], or
/// until the turn is cancelled. Starts the action's step in the steps block;
/// the caller finishes it with [`super::report`] after running the action.
pub async fn ask(identity: &SessionIdentity<'_>, action: &VmAction) -> VmDecision {
    let hash = action.hash();
    let step_id = steps::begin(identity, action, &hash).await;
    let decided = decide(identity, action).await;
    steps::decided(identity, action, &step_id, decided).await;
    let (label, source) = match decided {
        Ok(source) => ("allow", source.as_str()),
        Err(denial) => ("deny", denial.as_str()),
    };
    botlog::safe_info!(
        "vm consent: thread={} agent={} kind={} decision={label} source={source} arg_sha256={}",
        identity.session_id,
        action.agent.as_str(),
        action.kind_key(),
        &hash[..8]
    );
    match decided {
        Ok(source) => VmDecision::Allowed(VmConsent::issue(identity.session_id, source, hash, step_id)),
        Err(denial) => VmDecision::Denied(denial),
    }
}

/// Grants live per agent and conversation: the unique
/// `(user, app_id, action_class)` row of `app_permissions` is then one row
/// per conversation and kind.
pub(super) fn app_id(agent: super::VmAgent, thread_id: &str) -> String {
    format!("vm:{}:{thread_id}", agent.as_str())
}

#[cfg(feature = "consent")]
async fn decide(identity: &SessionIdentity<'_>, action: &VmAction) -> Result<AllowSource, VmDenial> {
    use botconsent::models::VM_ALL_CLASS;
    use botconsent::store;

    let Some(service) = crate::core::bot::consent_gate::shared_service() else {
        botlog::safe_warn!("vm consent: service not initialised; action refused");
        return Err(VmDenial::Unavailable);
    };
    let thread = identity.session_id.to_string();
    let app = app_id(action.agent, &thread);
    let class = action.class();
    let context = botconsent::ConsentContext {
        thread_id: Some(thread.clone()),
        run_id: crate::core::bot::runs::bus::bus().current_run_id(&thread),
        workspace_id: None,
    };
    if !action.protected() {
        match service.pool.get() {
            Ok(mut conn) => {
                let mut classes = vec![(class, AllowSource::Conversation)];
                if !action.outside_vm() {
                    classes.push((VM_ALL_CLASS, AllowSource::AllowAll));
                }
                for (grant_class, source) in classes {
                    if let Ok(Some(row)) =
                        store::effective_grant_for(&mut conn, identity.user_id, &app, grant_class, &context)
                    {
                        let audit = serde_json::json!({
                            "app_id": app, "action_class": class, "thread_id": thread,
                            "detail": action.audit_detail(), "source": source.as_str(),
                        });
                        store::audit(&mut conn, Some(row.id), Some(identity.user_id), &audit, store::OUTCOME_GRANTED, None);
                        return Ok(source);
                    }
                }
            }
            Err(e) => botlog::safe_warn!("vm consent: grant lookup unavailable: {e}"),
        }
    }
    let (request, answer) = match botconsent::raise_blocking(
        &service,
        identity.user_id,
        &app,
        class,
        action.card_detail(),
        &context,
    )
    .await
    {
        Ok(raised) => raised,
        Err(e) => {
            botlog::safe_warn!("vm consent: could not raise the request: {e}");
            return Err(VmDenial::Unavailable);
        }
    };
    let card = steps::card_shown(identity, action, &request.request_id).await;
    let outcome =
        super::wait::wait(&service, &request.request_id, &thread, context.run_id.as_deref(), answer).await;
    steps::card_closed(identity, card, action, &request.request_id, outcome).await;
    outcome
}

#[cfg(not(feature = "consent"))]
async fn decide(identity: &SessionIdentity<'_>, action: &VmAction) -> Result<AllowSource, VmDenial> {
    botlog::safe_warn!(
        "vm consent: built without the consent service; {} refused for {}",
        action.kind_key(),
        identity.session_id
    );
    Err(VmDenial::Unavailable)
}

/// Maps the person's decision; anything else than the three allows denies.
/// A waiter dropped without an answer means the request was withdrawn (its
/// conversation was deleted): the action is cancelled.
#[cfg(feature = "consent")]
pub(super) fn decision_source(decided: Option<botconsent::Decision>) -> Result<AllowSource, VmDenial> {
    use botconsent::Decision;
    match decided {
        Some(Decision::AllowOnce) => Ok(AllowSource::Once),
        Some(Decision::AllowConversation) => Ok(AllowSource::Conversation),
        Some(Decision::AllowAll) => Ok(AllowSource::AllowAll),
        Some(_) => Err(VmDenial::ByPerson),
        None => Err(VmDenial::Cancelled),
    }
}
