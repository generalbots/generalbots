// Tests for the episodic-memory prompt assembler and response parser, split
// out of `from_config.rs` to keep that file under the AGENTS.md size limit.

use super::*;

fn message(role: &str, content: &str, seconds: u32) -> ConversationMessage {
    ConversationMessage {
        id: Uuid::new_v4(),
        role: role.to_string(),
        content: content.to_string(),
        timestamp: chrono::DateTime::from_timestamp(1_760_000_000 + i64::from(seconds), 0)
            .expect("valid timestamp"),
    }
}

fn manager() -> EpisodicMemoryManager {
    EpisodicMemoryManager::new(EpisodicMemoryConfig::default())
}

#[test]
fn defaults_apply_when_config_map_is_empty() {
    let manager = EpisodicMemoryManager::from_config(&std::collections::HashMap::new());
    assert!(manager.should_summarize(4));
    assert_eq!(manager.get_history_to_keep(), 2);
    assert_eq!(manager.get_threshold(), 4);
}

#[test]
fn config_map_overrides_every_setting() {
    let mut map = std::collections::HashMap::new();
    map.insert("episodic-memory-enabled".to_string(), "false".to_string());
    map.insert("episodic-memory-threshold".to_string(), "9".to_string());
    map.insert("episodic-memory-history".to_string(), "5".to_string());

    let manager = EpisodicMemoryManager::from_config(&map);
    assert!(!manager.should_summarize(100));
    assert_eq!(manager.get_threshold(), 9);
    assert_eq!(manager.get_history_to_keep(), 5);
}

#[test]
fn malformed_numbers_fall_back_to_defaults() {
    let mut map = std::collections::HashMap::new();
    map.insert("episodic-memory-threshold".to_string(), "many".to_string());
    map.insert("episodic-memory-history".to_string(), "lots".to_string());

    let manager = EpisodicMemoryManager::from_config(&map);
    assert_eq!(manager.get_threshold(), 4);
    assert_eq!(manager.get_history_to_keep(), 2);
}

#[test]
fn summarization_respects_the_threshold_boundary() {
    let manager = manager();
    assert!(!manager.should_summarize(3));
    assert!(manager.should_summarize(4));
    assert!(manager.should_summarize(50));
}

#[test]
fn summary_prompt_lists_every_message_in_order() {
    let messages = vec![
        message("user", "Quero um boleto", 0),
        message("bot", "Claro, gerando", 60),
    ];
    let prompt = manager().generate_summary_prompt(&messages);

    assert!(prompt.contains("Analyze and summarize this conversation"));
    assert!(prompt.contains("user: Quero um boleto"));
    assert!(prompt.contains("bot: Claro, gerando"));
    assert!(prompt.contains("\"key_topics\""));
    assert!(prompt.contains("\"action_items\""));
    assert!(prompt.contains("\"resolution\""));
    let user_pos = prompt.find("Quero um boleto").expect("user line");
    let bot_pos = prompt.find("Claro, gerando").expect("bot line");
    assert!(user_pos < bot_pos);
}

#[test]
fn summary_prompt_works_with_no_messages() {
    let prompt = manager().generate_summary_prompt(&[]);
    assert!(prompt.contains("CONVERSATION:"));
    assert!(prompt.contains("\"summary\""));
}

#[test]
fn extract_json_reads_a_json_fence() {
    let response = "Here you go:\n```json\n{\"summary\":\"ok\"}\n```\nDone.";
    assert_eq!(extract_json(response).expect("json"), "{\"summary\":\"ok\"}");
}

#[test]
fn extract_json_reads_a_plain_fence() {
    let response = "```\n{\"a\":1}\n```";
    assert_eq!(extract_json(response).expect("json"), "{\"a\":1}");
}

#[test]
fn extract_json_falls_back_to_the_outer_braces() {
    let response = "prefix {\"a\":1} suffix";
    assert_eq!(extract_json(response).expect("json"), "{\"a\":1}");
}

#[test]
fn extract_json_errors_when_absent() {
    assert!(extract_json("no json here").is_err());
    assert!(extract_json("").is_err());
}

#[test]
fn parse_summary_response_maps_every_field() {
    let response = r#"{
        "summary": "Customer asked for a boleto.",
        "key_topics": ["billing", "boleto"],
        "decisions": ["issue second copy"],
        "action_items": [
            {"description": "send boleto", "assignee": "bot", "priority": "high"},
            {"description": "confirm receipt", "assignee": null, "priority": "critical"}
        ],
        "sentiment": {"score": 0.6, "label": "positive", "confidence": 0.9},
        "resolution": "pending"
    }"#;
    let messages = vec![message("user", "boleto?", 0), message("bot", "ok", 30)];

    let episode = manager()
        .parse_summary_response(response, &messages, Uuid::nil(), Uuid::nil(), Uuid::nil())
        .expect("episode");

    assert_eq!(episode.summary, "Customer asked for a boleto.");
    assert_eq!(episode.key_topics, vec!["billing", "boleto"]);
    assert_eq!(episode.decisions, vec!["issue second copy"]);
    assert_eq!(episode.action_items.len(), 2);
    assert_eq!(episode.action_items[0].priority, Priority::High);
    assert_eq!(episode.action_items[0].assignee.as_deref(), Some("bot"));
    assert_eq!(episode.action_items[1].priority, Priority::Critical);
    assert_eq!(episode.action_items[1].assignee, None);
    assert_eq!(episode.sentiment.label, SentimentLabel::Positive);
    assert!((episode.sentiment.score - 0.6).abs() < f64::EPSILON);
    assert_eq!(episode.resolution, ResolutionStatus::Pending);
    assert_eq!(episode.message_count, 2);
    assert_eq!(episode.message_ids.len(), 2);
    assert_eq!(episode.conversation_start, messages[0].timestamp);
    assert_eq!(episode.conversation_end, messages[1].timestamp);
}

#[test]
fn parse_summary_response_uses_defaults_for_missing_keys() {
    let episode = manager()
        .parse_summary_response("{\"summary\":\"only\"}", &[], Uuid::nil(), Uuid::nil(), Uuid::nil())
        .expect("episode");

    assert_eq!(episode.summary, "only");
    assert!(episode.key_topics.is_empty());
    assert!(episode.decisions.is_empty());
    assert!(episode.action_items.is_empty());
    assert_eq!(episode.sentiment.label, SentimentLabel::Neutral);
    assert_eq!(episode.resolution, ResolutionStatus::Unknown);
    assert_eq!(episode.message_count, 0);
}

#[test]
fn parse_summary_response_handles_unknown_enum_labels() {
    let response = r#"{"summary":"x","sentiment":{"label":"ecstatic"},"resolution":"in_limbo"}"#;
    let episode = manager()
        .parse_summary_response(response, &[], Uuid::nil(), Uuid::nil(), Uuid::nil())
        .expect("episode");

    assert_eq!(episode.sentiment.label, SentimentLabel::Neutral);
    assert_eq!(episode.resolution, ResolutionStatus::Unknown);
}

#[test]
fn parse_summary_response_rejects_non_json() {
    assert!(manager()
        .parse_summary_response("not json", &[], Uuid::nil(), Uuid::nil(), Uuid::nil())
        .is_err());
}

#[test]
fn retention_cutoff_is_in_the_past() {
    assert!(manager().get_retention_cutoff() < Utc::now());
}
