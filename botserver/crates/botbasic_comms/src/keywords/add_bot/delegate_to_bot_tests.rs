// Tests for the sub-bot trigger matcher (#2065), split out of
// `delegate_to_bot.rs` to keep that file under the AGENTS.md size limit.

use super::{match_bot_triggers, match_tool_triggers, BotTrigger, SessionBot, TriggerType};
use uuid::Uuid;

fn bot(name: &str, trigger: BotTrigger, priority: i32, is_active: bool) -> SessionBot {
    SessionBot {
        id: Uuid::new_v4(),
        session_id: Uuid::nil(),
        bot_id: Uuid::new_v4(),
        bot_name: name.to_string(),
        trigger,
        priority,
        is_active,
    }
}

fn keyword_trigger(keywords: &[&str]) -> BotTrigger {
    BotTrigger {
        trigger_type: TriggerType::Keyword,
        keywords: Some(keywords.iter().map(|k| k.to_string()).collect()),
        tools: None,
        schedule: None,
        event_name: None,
    }
}

fn tool_trigger(tools: &[&str]) -> BotTrigger {
    BotTrigger {
        trigger_type: TriggerType::Tool,
        keywords: None,
        tools: Some(tools.iter().map(|t| t.to_string()).collect()),
        schedule: None,
        event_name: None,
    }
}

fn always_trigger() -> BotTrigger {
    BotTrigger {
        trigger_type: TriggerType::Always,
        keywords: None,
        tools: None,
        schedule: None,
        event_name: None,
    }
}

#[test]
fn keyword_triggers_match_case_insensitively() {
    let bots = vec![bot("billing", keyword_trigger(&["invoice", "boleto"]), 0, true)];
    assert_eq!(match_bot_triggers("I need an INVOICE", &bots).len(), 1);
    assert_eq!(match_bot_triggers("quero um boleto", &bots).len(), 1);
    assert!(match_bot_triggers("hello there", &bots).is_empty());
}

#[test]
fn inactive_bots_never_match() {
    let bots = vec![bot("billing", keyword_trigger(&["invoice"]), 0, false)];
    assert!(match_bot_triggers("invoice", &bots).is_empty());
    let tools = vec![bot("ops", tool_trigger(&["DEPLOY"]), 0, false)];
    assert!(match_tool_triggers("DEPLOY", &tools).is_empty());
}

#[test]
fn always_triggers_match_every_message() {
    let bots = vec![bot("router", always_trigger(), 0, true)];
    assert_eq!(match_bot_triggers("anything at all", &bots).len(), 1);
    assert_eq!(match_bot_triggers("", &bots).len(), 1);
}

#[test]
fn tool_schedule_and_event_triggers_do_not_fire_on_messages() {
    for trigger in [
        tool_trigger(&["DEPLOY"]),
        BotTrigger {
            trigger_type: TriggerType::Schedule,
            keywords: Some(vec!["cron".to_string()]),
            tools: None,
            schedule: Some("0 * * * *".to_string()),
            event_name: None,
        },
        BotTrigger {
            trigger_type: TriggerType::Event,
            keywords: Some(vec!["cron".to_string()]),
            tools: None,
            schedule: None,
            event_name: Some("on_insert".to_string()),
        },
    ] {
        let bots = vec![bot("x", trigger, 0, true)];
        assert!(match_bot_triggers("cron", &bots).is_empty());
    }
}

#[test]
fn results_are_ordered_by_priority_descending() {
    let bots = vec![
        bot("low", always_trigger(), 1, true),
        bot("high", always_trigger(), 10, true),
        bot("mid", always_trigger(), 5, true),
    ];
    let names: Vec<String> = match_bot_triggers("hi", &bots)
        .into_iter()
        .map(|b| b.bot_name)
        .collect();
    assert_eq!(names, vec!["high", "mid", "low"]);
}

#[test]
fn tool_triggers_match_the_exact_tool_name_only() {
    let bots = vec![bot("ops", tool_trigger(&["DEPLOY", "rollback"]), 0, true)];
    assert_eq!(match_tool_triggers("deploy", &bots).len(), 1);
    assert_eq!(match_tool_triggers("ROLLBACK", &bots).len(), 1);
    assert!(match_tool_triggers("deployment", &bots).is_empty());
    assert!(match_tool_triggers("roll", &bots).is_empty());
}

#[test]
fn tool_triggers_ignore_other_trigger_types() {
    let bots = vec![bot("billing", keyword_trigger(&["DEPLOY"]), 0, true)];
    assert!(match_tool_triggers("DEPLOY", &bots).is_empty());
}

#[test]
fn a_bot_without_keywords_or_tools_never_matches() {
    let empty_keywords = BotTrigger {
        trigger_type: TriggerType::Keyword,
        keywords: None,
        tools: None,
        schedule: None,
        event_name: None,
    };
    let empty_tools = BotTrigger {
        trigger_type: TriggerType::Tool,
        keywords: None,
        tools: None,
        schedule: None,
        event_name: None,
    };
    assert!(match_bot_triggers("anything", &[bot("a", empty_keywords, 0, true)]).is_empty());
    assert!(match_tool_triggers("ANY", &[bot("b", empty_tools, 0, true)]).is_empty());
}

#[test]
fn an_empty_bot_list_yields_no_matches() {
    assert!(match_bot_triggers("hello", &[]).is_empty());
    assert!(match_tool_triggers("DEPLOY", &[]).is_empty());
}
