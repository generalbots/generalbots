use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::trace;
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};
use std::sync::Arc;

/// Microsoft 365 / SharePoint / Teams BASIC keywords for issue #624.
///
/// Provides: GET SHAREPOINT LISTS, CREATE SHAREPOINT ITEM, SEND TEAMS MESSAGE.
pub fn register_m365_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_get_sharepoint_lists(state.clone(), user.clone(), engine);
    register_create_sharepoint_item(state.clone(), user.clone(), engine);
    register_send_teams_message(state, user, engine);
}

fn register_get_sharepoint_lists(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["GET", "SHAREPOINT", "LISTS", "$expr$"],
            false,
            move |context, inputs| {
                let site_id = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("GET SHAREPOINT LISTS: site={site_id}");
                let result = json!({
                    "kind": "sharepoint_lists",
                    "site_id": site_id,
                    "endpoint": format!("/v1.0/sites/{}/lists", site_id),
                    "method": "GET",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid GET SHAREPOINT LISTS syntax");
}

fn register_create_sharepoint_item(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            [
                "CREATE", "SHAREPOINT", "ITEM", "$expr$", ",", "$expr$", ",", "$expr$",
            ],
            false,
            move |context, inputs| {
                let site_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let list_id = context.eval_expression_tree(&inputs[1])?.to_string();
                let fields_json = context.eval_expression_tree(&inputs[2])?.to_string();
                trace!("CREATE SHAREPOINT ITEM: site={site_id} list={list_id}");
                let result = json!({
                    "kind": "sharepoint_item",
                    "action": "create",
                    "site_id": site_id,
                    "list_id": list_id,
                    "fields": fields_json,
                    "endpoint": format!("/v1.0/sites/{}/lists/{}/items", site_id, list_id),
                    "method": "POST",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid CREATE SHAREPOINT ITEM syntax");
}

fn register_send_teams_message(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["SEND", "TEAMS", "MESSAGE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let chat_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("SEND TEAMS MESSAGE: chat={chat_id}");
                let result = json!({
                    "kind": "teams_message",
                    "action": "send",
                    "chat_id": chat_id,
                    "body": { "contentType": "html", "content": message },
                    "endpoint": format!("/v1.0/chats/{}/messages", chat_id),
                    "method": "POST",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid SEND TEAMS MESSAGE syntax");
}

fn serde_json_to_dynamic(v: &Value) -> Dynamic {
    Dynamic::from(v.to_string())
}
