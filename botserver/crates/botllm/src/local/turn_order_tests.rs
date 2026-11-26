use super::*;

fn message(role: &str, content: &str) -> Value {
    serde_json::json!({ "role": role, "content": content })
}

fn roles(messages: &Value) -> Vec<String> {
    messages
        .as_array()
        .map(|list| list.iter().filter_map(|m| role(m).map(str::to_string)).collect())
        .unwrap_or_default()
}

fn turn(memory: &str, web: &str, history: &[(&str, &str)], question: &str) -> Value {
    let mut list = vec![
        message("system", "You are the assistant."),
        message("system", "THIS TURN: answer directly."),
        message("system", memory),
    ];
    list.extend(history.iter().map(|(role, text)| message(role, text)));
    list.push(message("system", web));
    list.push(message("user", question));
    Value::Array(list)
}

#[test]
fn turn_blocks_follow_the_history_inside_the_last_user_message() {
    let ordered = order_for_prompt_cache(&turn(
        "Memory: nothing saved.",
        "Web results: none.",
        &[("user", "first question"), ("assistant", "first answer")],
        "second question",
    ));
    assert_eq!(roles(&ordered), ["system", "user", "assistant", "user"]);
    assert_eq!(ordered[0]["content"], "You are the assistant.");
    assert_eq!(ordered[1]["content"], "first question");
    assert_eq!(
        ordered[3]["content"],
        "THIS TURN: answer directly.\n\nMemory: nothing saved.\n\nWeb results: none.\n\nsecond question"
    );
}

#[test]
fn the_prefix_stays_identical_when_only_the_turn_blocks_change() {
    let history = [("user", "first question"), ("assistant", "first answer")];
    let one = order_for_prompt_cache(&turn("Memory: A.", "Web: X.", &history, "next"));
    let two = order_for_prompt_cache(&turn("Memory: B saved.", "Web: Y and Z.", &history, "next"));
    let prefix = |v: &Value| v.as_array().map(|l| l[..3].to_vec()).unwrap_or_default();
    assert_eq!(prefix(&one), prefix(&two), "system prompt and history must not change");
    assert_ne!(one[3], two[3]);
    // The old order changed the very first message.
    assert_ne!(
        merge_system_first(&turn("Memory: A.", "Web: X.", &history, "next"))[0],
        merge_system_first(&turn("Memory: B saved.", "Web: Y and Z.", &history, "next"))[0]
    );
}

#[test]
fn requests_that_do_not_end_with_a_plain_user_message_keep_the_merged_order() {
    let tool_round = serde_json::json!([
        message("system", "You are the assistant."),
        message("system", "Turn note."),
        message("user", "question"),
        message("assistant", "calling a tool"),
    ]);
    assert_eq!(order_for_prompt_cache(&tool_round), merge_system_first(&tool_round));

    let with_files = serde_json::json!([
        message("system", "You are the assistant."),
        { "role": "system", "content": "Attached file.", "media": [{ "kind": "image" }] },
        message("user", "what is in it?"),
    ]);
    assert_eq!(order_for_prompt_cache(&with_files), merge_system_first(&with_files));

    let parts = serde_json::json!([
        message("system", "You are the assistant."),
        { "role": "user", "content": [{ "type": "text", "text": "hi" }] },
    ]);
    assert_eq!(order_for_prompt_cache(&parts), merge_system_first(&parts));
}

#[test]
fn a_request_with_only_the_system_prompt_is_unchanged() {
    let plain = serde_json::json!([message("system", "You are the assistant."), message("user", "hi")]);
    assert_eq!(order_for_prompt_cache(&plain), plain);
    let blank_blocks = serde_json::json!([
        message("system", "You are the assistant."),
        message("system", "   "),
        message("user", "hi"),
    ]);
    assert_eq!(order_for_prompt_cache(&blank_blocks), plain);
    let no_system = serde_json::json!([message("user", "hi"), message("assistant", "hello")]);
    assert_eq!(order_for_prompt_cache(&no_system), no_system);
}
