//! ACP bridge tests: message building and parsing, and prompts against a
//! scripted agent (`test_acp_peer.py`) spawned through `SafeCommand`.

use std::collections::VecDeque;
use std::sync::Mutex;

use botlib::security::command_guard::SafeCommand;
use serde_json::{json, Value};

use crate::codex::app_server::ActionOutcome;
use crate::codex::Verdict;
use crate::duplex::{DuplexOptions, DuplexSession, RpcClient, RpcInbox, RpcOptions};

use super::proto::{self, Choice, Update};
use super::turn::{run_prompt, PromptInput};
use super::*;

/// `initialize` as `agy acp` 1.5.14 answers it before any sign-in.
fn agy_initialize() -> Value {
    json!({"protocolVersion":1,"agentCapabilities":{"loadSession":true,"load_session":true,
        "promptCapabilities":{"image":true,"audio":false,"embeddedContext":true},
        "sessionCapabilities":{"resume":{},"close":{},"load":{},"list":{},"additionalDirectories":{}}},
        "agentInfo":{"name":"Antigravity ACP Server","version":"1.5.14"},
        "authMethods":[{"id":"oauth-personal","name":"Log in with Google"}]})
}

#[test]
fn the_initialize_answer_of_agy_is_understood() {
    let info = agent_info(&agy_initialize());
    assert_eq!(
        info,
        AgentInfo {
            version: "1.5.14".into(),
            protocol: 1,
            load_session: true,
            resume_session: true,
            images: true,
            embedded_context: true,
        }
    );
    assert_eq!(agent_info(&json!({})).version, "unknown");
    let params = proto::initialize_params();
    assert_eq!(params["protocolVersion"], 1);
    assert_eq!(params["clientCapabilities"]["terminal"], false);
    assert_eq!(params["clientCapabilities"]["fs"]["writeTextFile"], false);
}

#[test]
fn the_prompt_carries_text_pictures_and_files_the_agent_accepts() {
    let info = agent_info(&agy_initialize());
    let images = [ImagePart { mime_type: "image/jpeg".into(), data: "QUJD".into() }];
    let files = [TextResource { path: "/opt/vibe/conversations/c/data/a.txt".into(), mime_type: "text/plain".into(), text: "fim".into() }];
    let params = proto::prompt_params("s1", "olá", &images, &files, &info);
    assert_eq!(params["sessionId"], "s1");
    let prompt = params["prompt"].as_array().cloned().unwrap_or_default();
    assert_eq!(prompt[0], json!({"type": "text", "text": "olá"}));
    assert_eq!(prompt[1], json!({"type": "image", "mimeType": "image/jpeg", "data": "QUJD"}));
    assert_eq!(prompt[2]["resource"]["uri"], "file:///opt/vibe/conversations/c/data/a.txt");
    assert_eq!(prompt[2]["resource"]["text"], "fim");
    let blind = AgentInfo { images: false, embedded_context: false, ..info };
    assert_eq!(proto::prompt_params("s1", "olá", &images, &files, &blind)["prompt"].as_array().map(Vec::len), Some(1));
}

#[test]
fn updates_are_read_as_text_reasoning_and_tool_calls() {
    let text = json!({"sessionId":"s","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"oi"}}});
    assert_eq!(proto::update_of(&text), Update::Text("oi".into()));
    let thought = json!({"update":{"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"hm"}}});
    assert_eq!(proto::update_of(&thought), Update::Thought("hm".into()));
    let tool = json!({"update":{"sessionUpdate":"tool_call","toolCallId":"t","kind":"execute","status":"pending"}});
    assert!(matches!(proto::update_of(&tool), Update::Tool { id, kind: Some(kind), .. } if id == "t" && kind == "execute"));
    assert_eq!(proto::update_of(&json!({"update":{"sessionUpdate":"plan"}})), Update::Other);
}

#[test]
fn permission_requests_become_the_actions_the_card_shows() {
    let command = json!({"toolCall":{"toolCallId":"t","kind":"execute","title":"Run ls","rawInput":{"command":"ls -la","cwd":"/home/antigravity"}}});
    assert_eq!(actions_of(&command), vec![AcpAction::Command { command: "ls -la".into(), cwd: Some("/home/antigravity".into()) }]);
    let edit = json!({"toolCall":{"kind":"edit","locations":[{"path":"/a"},{"path":"/b"}]}});
    assert_eq!(actions_of(&edit), vec![AcpAction::Write { path: "/a".into() }, AcpAction::Write { path: "/b".into() }]);
    let read = json!({"toolCall":{"kind":"read","rawInput":{"path":"/opt/vibe/conversations/c/data/t.txt"}}});
    assert_eq!(actions_of(&read), vec![AcpAction::Read { path: "/opt/vibe/conversations/c/data/t.txt".into() }]);
    let delete = json!({"toolCall":{"kind":"delete","locations":[{"path":"/x"}]}});
    assert_eq!(actions_of(&delete), vec![AcpAction::Delete { path: "/x".into() }]);
    let fetch = json!({"toolCall":{"kind":"fetch","rawInput":{"url":"https://example.org/"}}});
    assert_eq!(actions_of(&fetch), vec![AcpAction::Fetch { url: "https://example.org/".into() }]);
    let other = json!({"toolCall":{"kind":"think","title":"Plan the work"}});
    assert_eq!(actions_of(&other), vec![AcpAction::Other { title: "Plan the work".into() }]);
    assert_eq!(actions_of(&json!({})), vec![AcpAction::Other { title: "action".into() }]);
}

#[test]
fn the_answer_picks_the_agents_once_option_never_a_lasting_one() {
    let params = json!({"options":[
        {"optionId":"aa","kind":"allow_always"},{"optionId":"a1","kind":"allow_once"},
        {"optionId":"ra","kind":"reject_always"},{"optionId":"r1","kind":"reject_once"}]});
    assert_eq!(proto::permission_result(&params, Choice::Allow), json!({"outcome":{"outcome":"selected","optionId":"a1"}}));
    assert_eq!(proto::permission_result(&params, Choice::Reject), json!({"outcome":{"outcome":"selected","optionId":"r1"}}));
    assert_eq!(proto::permission_result(&params, Choice::Cancelled), json!({"outcome":{"outcome":"cancelled"}}));
    let only_always = json!({"options":[{"optionId":"aa","kind":"allow_always"}]});
    assert_eq!(proto::permission_result(&only_always, Choice::Allow), json!({"outcome":{"outcome":"cancelled"}}));
}

#[test]
fn modes_that_act_without_asking_are_recognised() {
    let auto = json!({"sessionId":"s","modes":{"currentModeId":"auto","availableModes":[{"id":"auto"},{"id":"plan"},{"id":"default"}]}});
    assert_eq!(proto::auto_mode(&auto).as_deref(), Some("auto"));
    assert_eq!(proto::asking_mode(&auto).as_deref(), Some("default"));
    assert_eq!(proto::auto_mode(&json!({"modes":{"currentModeId":"default"}})), None);
    assert_eq!(proto::auto_mode(&json!({"sessionId":"s"})), None);
}

#[test]
fn a_finished_tool_call_reports_its_output_and_files() {
    let call = json!({"content":[{"type":"content","content":{"type":"text","text":"ok"}}],"rawOutput":{"exitCode":0}});
    let outcome = proto::outcome_of(&call, "completed", vec!["/a".into()]);
    assert!(outcome.ok);
    assert_eq!((outcome.exit_code, outcome.output.as_str(), outcome.files.len()), (Some(0), "ok", 1));
    let failed = proto::outcome_of(&json!({"rawOutput":{"exitCode":2}}), "completed", vec!["/a".into()]);
    assert!(!failed.ok && failed.files.is_empty());
}

#[test]
fn a_command_step_keeps_its_output_without_agys_timestamps() {
    let text = "Created At: 2026-11-24T00:58:09Z\nCompleted At: 2026-11-24T00:58:09Z\nCommand exited with code 0.\n\
                Linux ag 6.6.87.2-microsoft-standard-WSL2 x86_64 GNU/Linux\n";
    let call = json!({"content":[{"type":"content","content":{"type":"text","text":text}}]});
    let outcome = proto::outcome_of(&call, "completed", Vec::new());
    assert_eq!(outcome.output, "Linux ag 6.6.87.2-microsoft-standard-WSL2 x86_64 GNU/Linux");
    assert_eq!((outcome.ok, outcome.exit_code), (true, Some(0)));
    let failed = json!({"rawOutput":{"output":"Created At: x | Completed At: y\nCommand exited with code 2.\nno such file"}});
    let outcome = proto::outcome_of(&failed, "completed", Vec::new());
    assert_eq!((outcome.ok, outcome.exit_code, outcome.output.as_str()), (false, Some(2), "no such file"));
}

#[test]
fn the_launch_script_announces_the_session_and_keeps_the_tag() {
    let script = launch_script("abc-123");
    assert!(script.starts_with("printf 'GB-SESSION '; head -c 130 /home/antigravity/.gemini/gb-sessions/abc-123"));
    assert!(script.contains("exec su antigravity -s /bin/sh -c '"));
    assert!(!script.contains("su -"));
    assert!(script.ends_with("exec agy acp --add-dir /opt/vibe/conversations'"));
    assert!(!script.contains("skip-permissions"));
    let store = store_script("abc", "3f2a-9_b");
    assert!(store.contains("printf '%s' 3f2a-9_b > /home/antigravity/.gemini/gb-sessions/abc"));
    assert!(safe_id("3f2a-9_b") && !safe_id("a;b") && !safe_id(""));
}

#[test]
fn needs_sign_in_matches_the_agents_error() {
    assert!(proto::needs_sign_in("Authentication required"));
    assert!(!proto::needs_sign_in("Rate limited"));
}

const PEER: &str = include_str!("test_acp_peer.py");

fn peer_path() -> &'static std::path::Path {
    static PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("gb-acp-peer-{}.py", std::process::id()));
        std::fs::write(&path, PEER).expect("the scripted agent is written");
        path
    })
}

fn start(mode: &str) -> (RpcClient, RpcInbox) {
    let program = if cfg!(windows) { "python" } else { "python3" };
    let command = SafeCommand::new(program)
        .and_then(|command| command.path_arg(peer_path()))
        .and_then(|command| command.arg(mode))
        .expect("the scripted agent command is accepted by the guard");
    let session = DuplexSession::spawn(&command, DuplexOptions::default()).expect("the scripted agent starts");
    RpcClient::start(session, RpcOptions::accepting(proto::HANDLED_REQUESTS))
}

/// A gate that answers from a script and records what happened.
#[derive(Default)]
struct Script {
    verdicts: Mutex<VecDeque<&'static str>>,
    log: Mutex<Vec<String>>,
    thoughts: Mutex<String>,
}

impl Script {
    fn with(verdicts: &[&'static str]) -> Self {
        Self { verdicts: Mutex::new(verdicts.iter().copied().collect()), ..Self::default() }
    }

    fn log(&self) -> Vec<String> {
        self.log.lock().map(|log| log.clone()).unwrap_or_default()
    }
}

impl AcpGate for Script {
    type Ticket = String;

    async fn decide(&self, action: &AcpAction) -> Verdict<String> {
        self.log.lock().expect("log").push(format!("ask {action:?}"));
        match self.verdicts.lock().expect("verdicts").pop_front() {
            Some("allow") => Verdict::Allow("ticket".into()),
            Some("decline") => Verdict::Decline,
            _ => Verdict::Cancel,
        }
    }

    async fn finished(&self, ticket: String, outcome: ActionOutcome) {
        self.log.lock().expect("log").push(format!("done {ticket} {} {}", outcome.ok, outcome.output));
    }

    async fn text(&self, _chunk: &str) {}

    async fn thought(&self, chunk: &str) {
        self.thoughts.lock().expect("thoughts").push_str(chunk);
    }

    fn stopped(&self) -> bool {
        false
    }
}

async fn prompt(mode: &str, gate: &Script) -> Result<String, AcpError> {
    let (client, mut inbox) = start(mode);
    let info = agent_info(&agy_initialize());
    let images = [ImagePart { mime_type: "image/png".into(), data: "QUJD".into() }];
    let input = PromptInput { session_id: "s1", text: "olá", images: &images, resources: &[], info: &info };
    let result = run_prompt(&client, &mut inbox, input, gate).await;
    client.close().await;
    result
}

#[tokio::test]
async fn the_answer_and_reasoning_stream_and_the_picture_is_sent() {
    let gate = Script::default();
    assert_eq!(prompt("plain", &gate).await, Ok("Olá, blocos text,image".to_string()));
    assert_eq!(gate.thoughts.lock().expect("thoughts").as_str(), "pensando");
}

#[tokio::test]
async fn an_allowed_command_runs_and_its_step_is_closed() {
    let gate = Script::with(&["allow"]);
    assert_eq!(prompt("ask", &gate).await, Ok("escolha yes".to_string()));
    let log = gate.log();
    assert_eq!(log[0], "ask Command { command: \"true\", cwd: None }");
    assert_eq!(log[1], "done ticket true done");
}

#[tokio::test]
async fn a_denied_edit_is_rejected_once_and_the_turn_goes_on() {
    let gate = Script::with(&["decline"]);
    assert_eq!(prompt("edit", &gate).await, Ok("escolha no".to_string()));
    assert_eq!(gate.log(), vec!["ask Write { path: \"/home/antigravity/a.txt\" }".to_string()]);
}

#[tokio::test]
async fn an_action_that_ran_without_asking_ends_the_turn() {
    let gate = Script::default();
    assert_eq!(prompt("unasked", &gate).await, Err(AcpError::Unasked));
    assert!(gate.log().is_empty());
}
