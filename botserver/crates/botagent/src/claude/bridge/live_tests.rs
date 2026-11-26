//! Live run of the bridge against Claude Code in a throwaway container with
//! a loopback stand-in for the model. Ignored by default. The container
//! needs the pinned CLI, python3, and a scripted model listening on
//! 127.0.0.1:8765 that requests the tools named after `GBSCRIPT ` in the
//! prompt, one per round trip:
//!
//! `GB_CLAUDE_LIVE_VM=<container> AGENT_INCUS_BIN=<incus> cargo test -p botagent --lib claude_bridge_live -- --ignored --nocapture --test-threads 1`

use std::sync::Mutex;
use std::time::Duration;

use botlib::security::command_guard::SafeCommand;

use super::command::{claude_invocation, session_script};
use super::selfcheck::{check_script, CheckGate};
use super::{drive, interpret, BridgeGate, ClaudeAction, DriveReport, GateAnswer, ToolOutcome, ToolRequest};
use crate::duplex::{DuplexOptions, DuplexSession};

const FAKE_MODEL: &str = " ANTHROPIC_BASE_URL=http://127.0.0.1:8765 ANTHROPIC_API_KEY=sk-fake-local \
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 DISABLE_TELEMETRY=1";

fn open(script: &str) -> DuplexSession {
    let bin = std::env::var("AGENT_INCUS_BIN").unwrap_or_else(|_| "incus".to_string());
    let vm = std::env::var("GB_CLAUDE_LIVE_VM").expect("GB_CLAUDE_LIVE_VM names a throwaway container");
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let command = SafeCommand::new(&bin)
        .and_then(|c| c.args(&["exec", &vm, "-T", "--env"]))
        .and_then(|c| c.trusted_arg(&format!("GB_DUPLEX_SESSION={tag}")))
        .and_then(|c| c.args(&["--", "sh", "-c"]))
        .and_then(|c| c.trusted_shell_script_arg(script))
        .expect("the live command is accepted");
    DuplexSession::spawn(&command, DuplexOptions::default()).expect("incus exec starts")
}

#[derive(Default)]
struct Person {
    decided: Mutex<Vec<ClaudeAction>>,
    finished: Mutex<Vec<(String, ToolOutcome)>>,
}

impl BridgeGate for Person {
    async fn decide(&self, _request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        self.decided.lock().expect("lock").push(action.clone());
        match action {
            ClaudeAction::Fetch { .. } => GateAnswer::Deny("PERMISSION DENIED by the person (live test).".into()),
            _ => GateAnswer::Allow,
        }
    }

    async fn finished(&self, tool_use_id: &str, outcome: &ToolOutcome) {
        self.finished.lock().expect("lock").push((tool_use_id.to_string(), outcome.clone()));
    }

    async fn text(&self, chunk: &str) -> bool {
        println!("TEXT {chunk:?}");
        true
    }

    fn stopped(&self) -> bool {
        false
    }
}

async fn turn(prompt: &str, resume: Option<&str>, gate: &Person) -> DriveReport {
    let script = session_script("", FAKE_MODEL, "", &claude_invocation("claude-sonnet-4-5", resume));
    let report = tokio::time::timeout(Duration::from_secs(180), drive(open(&script), prompt, gate))
        .await
        .expect("the live turn ends in time");
    println!(
        "REPORT session={:?} asked={:?} unasked={:?} result={:?} end={:?}",
        report.session_id,
        report.asked,
        report.unasked,
        report.result,
        report.end.as_ref().map(|e| (e.reason, e.exit_code))
    );
    report
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container with Claude Code and the scripted model"]
async fn claude_bridge_live_self_check_sees_the_permission_request() {
    let gate = CheckGate::default();
    let report = drive(open(&check_script(23_456)), "Run the workspace permission self-check.", &gate).await;
    println!("SELFCHECK asked={:?} unasked={:?} result={:?}", report.asked, report.unasked, report.result);
    assert!(gate.saw_the_request());
    assert!(super::selfcheck::verdict(gate.saw_the_request(), &report.unasked));
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container with Claude Code and the scripted model"]
async fn claude_bridge_live_every_tool_asks_and_reports() {
    let script = r#"GBSCRIPT [{"tool":"Bash","input":{"command":"ls /home/claude","description":"list"}},{"tool":"Write","input":{"file_path":"/home/claude/bridge.txt","content":"hello\n"}},{"tool":"Read","input":{"file_path":"/home/claude/bridge.txt"}},{"tool":"Bash","input":{"command":"ls /nonexistent"}},{"tool":"WebFetch","input":{"url":"https://example.com/","prompt":"title"}},{"tool":"CronCreate","input":{"cron":"* * * * *","prompt":"x"}}]"#;
    let gate = Person::default();
    let report = turn(script, None, &gate).await;
    let decided = gate.decided.lock().expect("lock").clone();
    println!("DECIDED {decided:?}");
    println!("FINISHED {:?}", gate.finished.lock().expect("lock"));
    assert_eq!(decided.len(), 5, "Bash, Write, Read, Bash and WebFetch each asked");
    assert!(report.unasked.is_empty());
    let finished = gate.finished.lock().expect("lock").clone();
    assert_eq!(finished.len(), 4);
    assert_eq!(finished[3].1.exit_code, Some(2));
    let session = report.session_id.clone().expect("a session id");
    let answer = interpret(report).expect("a good turn");
    assert!(answer.answer.contains("All done."));

    let again = Person::default();
    let resumed = turn(r#"GBSCRIPT [{"tool":"Bash","input":{"command":"echo resumed"}}]"#, Some(&session), &again).await;
    assert_eq!(resumed.session_id.as_deref(), Some(session.as_str()));
}
