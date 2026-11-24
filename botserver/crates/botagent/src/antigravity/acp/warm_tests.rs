//! A warm `agy acp` session across prompts, against the scripted agent:
//! each prompt's permission requests reach that prompt's gate, a request
//! sent between prompts is refused, and a dead process never starts one.

use std::time::Duration;

use super::tests::{agy_initialize, start, Script};
use super::turn::{run_prompt_tracked, PromptEnd, PromptInput};
use super::{agent_info, AcpWarm};
use crate::duplex::INTERNAL_ERROR;
use crate::warm::{refuse_idle_rpc, WarmProcess};

fn opened(mode: &str) -> AcpWarm {
    let (client, inbox) = start(mode);
    AcpWarm { client, inbox, session_id: "s1".into(), info: agent_info(&agy_initialize()) }
}

async fn prompt(process: &mut AcpWarm, gate: &Script) -> PromptEnd {
    let input = PromptInput { session_id: "s1", text: "olá", images: &[], resources: &[], info: &process.info };
    tokio::time::timeout(Duration::from_secs(30), run_prompt_tracked(&process.client, &mut process.inbox, input, gate))
        .await
        .expect("the prompt ends in time")
}

#[tokio::test]
async fn each_prompt_on_a_warm_session_asks_its_own_gate() {
    let mut process = opened("ask");
    let first = Script::with(&["allow"]);
    let end = prompt(&mut process, &first).await;
    assert!(end.answered && end.heard);
    assert_eq!(end.result, Ok("escolha yes".to_string()));
    assert!(refuse_idle_rpc(&mut process.inbox, "test"));

    let second = Script::with(&["decline"]);
    let end = prompt(&mut process, &second).await;
    assert!(end.answered);
    assert_eq!(end.result, Ok("escolha no".to_string()));
    assert_eq!(first.log().len(), 2, "the first gate saw its ask and its result only");
    assert_eq!(second.log().len(), 1, "the second gate saw only its own ask");
    process.shut_down();
}

#[tokio::test]
async fn a_permission_request_between_prompts_is_refused_and_reaches_no_gate() {
    let mut process = opened("stray");
    let first = Script::default();
    assert_eq!(prompt(&mut process, &first).await.result, Ok("stray=None".to_string()));
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert!(refuse_idle_rpc(&mut process.inbox, "test"));

    let second = Script::default();
    assert_eq!(prompt(&mut process, &second).await.result, Ok(format!("stray={INTERNAL_ERROR}")));
    assert!(first.log().is_empty() && second.log().is_empty());
    process.shut_down();
}

#[tokio::test]
async fn a_dead_process_never_starts_a_prompt() {
    let mut process = opened("plain");
    process.client.sender().close();
    process.client.sender().wait_ended().await;
    assert!(!process.alive());
    let gate = Script::default();
    let end = prompt(&mut process, &gate).await;
    assert!(end.result.is_err() && !end.heard && !end.answered, "the turn may fall back to a fresh process");
}
