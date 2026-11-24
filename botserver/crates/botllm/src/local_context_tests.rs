use super::*;
use serde_json::json;

fn msg(role: &str, tokens: usize) -> Value {
    json!({ "role": role, "content": "x".repeat(tokens * 3) })
}

fn roles(messages: &Value) -> Vec<String> {
    messages
        .as_array()
        .map(|list| {
            list.iter()
                .map(|m| m["role"].as_str().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_else(Vec::new)
}

#[test]
fn a_request_that_fits_is_left_alone() {
    let messages = json!([msg("system", 100), msg("user", 50)]);
    assert_eq!(fit(&messages, 1000), messages);
}

#[test]
fn the_oldest_turns_go_first() {
    let messages = json!([
        msg("system", 100),
        msg("user", 300),
        msg("assistant", 300),
        msg("user", 300),
        msg("assistant", 300),
        msg("user", 100),
    ]);
    let fitted = fit(&messages, 900);
    assert_eq!(roles(&fitted), ["system", "user", "assistant", "user"]);
    assert_eq!(fitted[3], messages[5], "the last message is untouched");
    assert_eq!(fitted[0], messages[0], "the system message is untouched");
}

#[test]
fn the_conversation_never_opens_with_an_answer() {
    let messages = json!([
        msg("system", 100),
        msg("user", 400),
        msg("assistant", 100),
        msg("user", 100),
    ]);
    // Dropping the first question alone would leave its answer in front.
    let fitted = fit(&messages, 400);
    assert_eq!(roles(&fitted), ["system", "user"]);
}

#[test]
fn a_system_message_too_long_on_its_own_keeps_its_beginning() {
    let system = format!("RULES {}", "d".repeat(30_000));
    let messages = json!([
        { "role": "system", "content": system },
        msg("user", 400),
        msg("assistant", 400),
        { "role": "user", "content": "What does it say?" },
    ]);
    let fitted = fit(&messages, 4000);
    assert_eq!(roles(&fitted), ["system", "user"]);
    let text = fitted[0]["content"].as_str().unwrap_or("");
    assert!(text.starts_with("RULES "));
    assert!(text.ends_with(CUT_NOTE));
    assert_eq!(fitted[1]["content"], json!("What does it say?"));
    let list = fitted.as_array().cloned().unwrap_or_else(Vec::new);
    assert!(total_tokens(&list) <= 4000);
}

#[test]
fn a_cut_lands_on_a_character_boundary() {
    let system = "ação ".repeat(4000);
    let messages = json!([
        { "role": "system", "content": system },
        { "role": "user", "content": "Resuma." },
    ]);
    let fitted = fit(&messages, 1000);
    let text = fitted[0]["content"].as_str().unwrap_or("");
    assert!(text.ends_with(CUT_NOTE));
    assert!(text.len() < system.len());
}

#[test]
fn the_budget_leaves_room_for_the_reply_and_the_tools() {
    assert_eq!(prompt_budget(8192, None), 8192 - 1024);
    assert_eq!(prompt_budget(16384, None), 16384 - 2048);
    assert_eq!(prompt_budget(131_072, None), 131_072 - 2048);
    let tools = vec![json!({ "type": "function", "function": { "name": "n".repeat(300) } })];
    assert!(prompt_budget(8192, Some(&tools)) < 8192 - 1024 - 100);
}

#[test]
fn the_origin_drops_the_path() {
    assert_eq!(origin("http://localhost:8081"), "http://localhost:8081");
    assert_eq!(
        origin("http://127.0.0.1:8081/v1/chat/completions"),
        "http://127.0.0.1:8081"
    );
}
