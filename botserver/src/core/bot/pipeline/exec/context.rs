//! The context a chat turn hands the model besides the question: the
//! workspace contract, the conversation history and the grounding blocks
//! (completed actions, Drive, web pages, web search, front pages).

use serde_json::Value;

use super::super::types::SessionIdentity;

/// Drive listing, workspace contract, Drive files named in the message and
/// the conversation history, in that order.
pub(super) async fn workspace_and_history(
    identity: &SessionIdentity<'_>,
    parsed: &Value,
    user_text: &str,
    messages: &mut Vec<Value>,
) {
    super::super::turn_context::apply_drive_listing(messages, parsed);

    // #1580: Always-on agent VM — auto-provision and inject execution context.
    // #1835 — news/informational turns belong to web grounding, not to the
    // agent VM's code-execution surface: skip the VM contract for them so the
    // model does not route the request into vm_exec (which never shows refs).
    // #2238 — explicit Agent toggle: skip VM provisioning/inject when off.
    // Without web grounding in the build, informational turns keep the VM.
    let web_grounding = cfg!(any(feature = "research", feature = "web-search"));
    if identity.agent_vm_enabled
        && !(web_grounding && super::super::turn_hooks::is_informational_request(user_text))
    {
        super::super::turn_watch::long_stage("workspace");
        super::super::agent_vm_context::inject(identity, messages).await;
    }

    // #1584: files named in the message are copied from Drive into data/
    // before the model writes the program that reads them.
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if identity.agent_vm_enabled {
        super::super::native_drive::fetch_referenced(
            identity.state,
            identity.bot_uuid,
            identity.user_id,
            identity.session_id,
            user_text,
            messages,
        )
        .await;
    }

    super::super::turn_watch::stage("history");
    super::super::history::append_conversation_history(
        identity.state,
        identity.bot_uuid,
        identity.session_id,
        identity.user_id,
        messages,
    )
    .await;
}

/// The grounding blocks, last before the model call so they are the
/// freshest instructions in the prompt.
pub(super) async fn grounding(
    identity: &SessionIdentity<'_>,
    parsed: &Value,
    user_text: &str,
    model_preference: &str,
    messages: &mut Vec<Value>,
) {
    // #1644 — results of approved browser/terminal actions are fed back into
    // the conversation before the model answers, so the next reply can cite
    // the page title (or say the action was denied/failed).
    super::super::turn_context::apply_completed_actions(identity.session_id, identity.user_id, messages).await;

    // #1545/#1547: grounding context goes LAST, immediately before the model
    // call, so it is the freshest instruction in the prompt.
    #[cfg(feature = "drive")]
    super::super::drive_context::inject(identity.state, &identity.bot_uuid, identity.user_id, user_text, parsed, messages).await;

    // A web address in the message (or the Web switch) lets the model read
    // public pages: `web_fetch`, or the named pages read ahead.
    super::super::native_tools::web_prefetch::apply(identity, parsed, model_preference, user_text, messages).await;

    // #1632 item 3 / #2238 — the composer's Web switch is the decision.
    // When explicitly off, do not search (even for informational requests).
    // When on (or unspecified legacy), search as before.
    //
    // #2147 — a front-page request is answered from rendered images, not from
    // search snippets, so it never pays for a web search first.
    #[cfg(all(feature = "browser", feature = "drive"))]
    let front_page_turn = super::super::front_page::wants_front_page(user_text);

    #[cfg(any(feature = "research", feature = "web-search"))]
    {
        #[cfg(all(feature = "browser", feature = "drive"))]
        let web_search_allowed = !front_page_turn;
        #[cfg(not(all(feature = "browser", feature = "drive")))]
        let web_search_allowed = true;

        let web_flag = parsed.get("web_search").and_then(|value| value.as_bool());
        let web_search_enabled = web_search_allowed
            && match web_flag {
                Some(false) => false,
                Some(true) => true,
                // No flag from the client. The government edition searches
                // only when the person asked for it: nothing leaves the
                // machine on the server's own judgement.
                None => {
                    !cfg!(feature = "gbgov")
                        && super::super::turn_hooks::is_informational_request(user_text)
                }
            };
        if web_search_enabled {
            let citations =
                super::super::web_context::search_and_inject(identity, user_text, messages).await;
            if !citations.is_empty() {
                identity
                    .state
                    .pending_citations
                    .lock()
                    .await
                    .insert(identity.session_id.to_string(), citations);
            }
        }
    }

    // #2147 — "as capas dos jornais … foto" must arrive as pictures. The
    // capture runs here, after every other grounding block, so the exact
    // markdown image is the freshest instruction the model reads.
    #[cfg(all(feature = "browser", feature = "drive"))]
    if front_page_turn {
        super::super::front_page::capture_and_inject(identity, user_text, messages).await;
    }
}
