use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

fn get_redis_connection(cache_client: &Arc<redis::Client>) -> Option<redis::Connection> {
    let timeout = Duration::from_millis(50);
    cache_client.get_connection_with_timeout(timeout).ok()
}

pub fn clear_switchers_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();

    engine
        .register_custom_syntax(["CLEAR", "SWITCHERS"], true, move |_context, _inputs| {
            if let Some(cache_client) = &cache {
                let redis_key = format!("suggestions:{}:{}", user_session.bot_id, user_session.id);
                let mut conn = match get_redis_connection(cache_client) {
                    Some(conn) => conn,
                    None => {
                        trace!("Cache not ready, skipping clear switchers");
                        return Ok(Dynamic::UNIT);
                    }
                };

                let result: Result<i64, redis::RedisError> =
                    redis::cmd("DEL").arg(&redis_key).query(&mut conn);

                match result {
                    Ok(deleted) => {
                        trace!(
                            "Cleared {} switchers from session {}",
                            deleted,
                            user_session.id
                        );
                    }
                    Err(e) => error!("Failed to clear switchers from Redis: {}", e),
                }
            } else {
                trace!("No cache configured, switchers not cleared");
            }

            Ok(Dynamic::UNIT)
        })
        .expect("valid syntax registration");
}

pub fn add_switcher_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();

    // ADD_SWITCHER "switcher_name" as "button text"
    // Note: compiler converts AS -> as (lowercase keywords), so we use lowercase here
    engine
        .register_custom_syntax(
            ["ADD_SWITCHER", "$expr$", "as", "$expr$"],
            true,
            move |context, inputs| {
                let switcher_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_switcher(
                    cache.as_ref(),
                    &user_session,
                    &switcher_name,
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");
}

fn add_switcher(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    switcher_name: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    trace!(
        "ADD_SWITCHER called: switcher={}, button={}",
        switcher_name,
        button_text
    );

    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.bot_id, user_session.id);

        let suggestion = json!({
            "type": "switcher",
            "switcher": switcher_name,
            "text": button_text,
            "action": {
                "type": "switch_context",
                "switcher": switcher_name
            }
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add switcher");
                return Ok(());
            }
        };

        let _: Result<i64, redis::RedisError> = redis::cmd("SADD")
            .arg(&redis_key)
            .arg(suggestion.to_string())
            .query(&mut conn);

        trace!(
            "Added switcher suggestion '{}' to session {}",
            switcher_name,
            user_session.id
        );
    } else {
        trace!("No cache configured, switcher suggestion not added");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_switcher_json() {
        let suggestion = json!({
            "type": "switcher",
            "switcher": "mode_switcher",
            "text": "Switch Mode",
            "action": {
                "type": "switch_context",
                "switcher": "mode_switcher"
            }
        });

        assert_eq!(suggestion["type"], "switcher");
        assert_eq!(suggestion["action"]["type"], "switch_context");
        assert_eq!(suggestion["switcher"], "mode_switcher");
    }
}
