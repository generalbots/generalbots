use uuid::Uuid;

use super::action::AUDIT_KEY;
use super::tools::{outside_action, program_action};
use super::{AllowSource, VmAction, VmActionKind, VmAgent, VmConsent, VmDenial};

fn command(text: &str, cwd: Option<&str>) -> VmAction {
    VmAction::native(VmActionKind::Command {
        command: text.to_string(),
        cwd: cwd.map(str::to_string),
        language: None,
        packages: Vec::new(),
        background: false,
        repair: false,
    })
}

fn ticket(thread: Uuid) -> VmConsent {
    VmConsent::issue(thread, AllowSource::Once, "h".repeat(64), "tool:vm_action:x".to_string())
}

#[test]
fn in_vm_actions_offer_four_choices_and_outside_ones_never_allow_all() {
    let run = command("ls -la", Some("/work"));
    assert_eq!(run.offers(), vec!["allow_once", "allow_conversation", "allow_all", "deny"]);
    assert!(!run.outside_vm());
    let save = outside_action("drive_save", r#"{"files":["output/a.csv"]}"#).expect("drive_save maps");
    assert!(save.outside_vm());
    assert_eq!(save.class(), "vm_drive_save");
    assert!(!save.offers().contains(&"allow_all"));
    let external = outside_action("save_to_external_drive", r#"{"provider":"x","files":"b.pdf"}"#)
        .expect("external save maps");
    assert_eq!(external.class(), "vm_external_save");
    assert!(!external.offers().contains(&"allow_all"));
    assert!(outside_action("drive_list", "{}").is_none());
}

#[test]
fn credential_paths_are_asked_every_time() {
    let read = command("cat ~/.codex/auth.json", None);
    assert!(read.protected());
    assert_eq!(read.offers(), vec!["allow_once", "deny"]);
    assert!(command("ls", Some("/home/claude/.claude/.credentials.json")).protected());
    assert!(!command("ls output", None).protected());
}

#[test]
fn every_kind_has_a_known_vm_class() {
    let kinds = [
        command("true", None),
        VmAction::native(VmActionKind::FileRead { path: "a".into() }),
        VmAction::native(VmActionKind::FileWrite { path: "a".into(), from: None }),
        VmAction::native(VmActionKind::FileDelete { path: "a".into() }),
        VmAction::native(VmActionKind::OpenUrl { url: "https://example.org".into() }),
        VmAction::native(VmActionKind::NetworkInstall { packages: vec!["pandas".into()], manager: None }),
    ];
    for action in kinds {
        assert!(action.class().starts_with("vm_"), "{}", action.class());
        assert!(!action.outside_vm());
    }
}

#[test]
fn a_file_read_card_names_its_own_kind_and_path() {
    let read = VmAction::native(VmActionKind::FileRead { path: "/home/claude/report.csv".into() });
    let detail = read.card_detail();
    assert_eq!(detail["kind"], "file_read");
    assert_eq!(detail["path"], "/home/claude/report.csv");
    assert_eq!(detail["text"], "/home/claude/report.csv");
    assert_eq!(read.class(), "vm_file_read");
    assert_eq!(read.offers(), vec!["allow_once", "allow_conversation", "allow_all", "deny"]);
    let delete = VmAction::native(VmActionKind::FileDelete { path: "/home/claude/report.csv".into() });
    assert_ne!(read.hash(), delete.hash());
    let secret = VmAction::native(VmActionKind::FileRead { path: "/root/.codex/auth.json".into() });
    assert!(secret.protected());
    assert_eq!(secret.offers(), vec!["allow_once", "deny"]);
}

#[test]
fn the_hash_identifies_the_exact_action() {
    let a = command("rm -rf build", Some("/w"));
    assert_eq!(a.hash(), command("rm -rf build", Some("/w")).hash());
    assert_ne!(a.hash(), command("rm -rf build", Some("/x")).hash());
    assert_ne!(a.hash(), command("rm -rf build2", Some("/w")).hash());
    let first = program_action("python", "print(1)", &[], false, false);
    let repaired = program_action("python", "print(1)", &[], false, true);
    assert_ne!(first.hash(), repaired.hash(), "the corrected program needs its own decision");
    let other_agent = VmAction { agent: VmAgent::ClaudeCode, kind: a.kind.clone() };
    assert_ne!(a.hash(), other_agent.hash());
    assert_eq!(a.hash().len(), 64);
}

#[test]
fn the_card_shows_the_exact_text_and_the_audit_copy_is_redacted() {
    let action = command("curl -H 'x' https://h/?password=hunter2 && echo done", Some("/w"));
    let card = action.card_detail();
    assert_eq!(card["vm"], true);
    assert_eq!(card["text"], action.text());
    assert_eq!(card["cwd"], "/w");
    assert_eq!(card["action_hash"], action.hash());
    let audit = card[AUDIT_KEY].to_string();
    assert!(!audit.contains("hunter2"), "audit must not keep the secret: {audit}");
    assert!(audit.contains(&action.hash()));
}

#[test]
fn a_ticket_covers_only_its_own_conversation() {
    let mine = Uuid::new_v4();
    let t = ticket(mine);
    assert!(t.covers(mine));
    assert!(!t.covers(Uuid::new_v4()));
    let receipt = t.receipt();
    assert_eq!(receipt.thread_id, mine);
    assert_eq!(receipt.source, AllowSource::Once);
}

#[tokio::test]
async fn vm_exec_refuses_a_ticket_from_another_conversation() {
    let user = Uuid::new_v4();
    let session = Uuid::new_v4();
    let foreign = ticket(Uuid::new_v4());
    let run = crate::core::bot::agent_vm_hook::exec_in_user_vm_result(
        &foreign, user, session, "python", "print(1)", 1_000, &[],
    )
    .await;
    assert!(matches!(run, Err((403, _))));
    let write = crate::core::bot::agent_vm_hook::write_session_file(ticket(Uuid::new_v4()), user, session, "a.txt", b"x").await;
    assert!(write.is_err());
}

#[test]
fn denials_tell_the_model_it_did_not_run() {
    let action = command("make deploy", None);
    let denied = VmDenial::ByPerson.model_text(&action);
    assert!(denied.starts_with("PERMISSION DENIED"));
    assert!(denied.contains("make deploy"));
    assert!(denied.contains("did not run"));
    assert!(VmDenial::NoAnswer.model_text(&action).contains("Do not retry"));
    assert!(VmDenial::Unavailable.model_text(&action).contains("did not run"));
}

#[test]
fn grants_are_kept_per_agent_and_conversation() {
    assert_eq!(super::gate::app_id(VmAgent::Native, "t1"), "vm:chat:t1");
    assert_eq!(super::gate::app_id(VmAgent::CodexCli, "t1"), "vm:codex-cli:t1");
}

#[cfg(feature = "consent")]
#[test]
fn only_the_three_allows_let_an_action_run() {
    use botconsent::Decision;
    use super::gate::decision_source;
    assert_eq!(decision_source(Some(Decision::AllowOnce)), Ok(AllowSource::Once));
    assert_eq!(decision_source(Some(Decision::AllowConversation)), Ok(AllowSource::Conversation));
    assert_eq!(decision_source(Some(Decision::AllowAll)), Ok(AllowSource::AllowAll));
    assert_eq!(decision_source(Some(Decision::Deny)), Err(VmDenial::ByPerson));
    assert_eq!(decision_source(Some(Decision::Always)), Err(VmDenial::ByPerson));
    assert_eq!(decision_source(None), Err(VmDenial::Cancelled));
    assert_eq!(botconsent::enforce::AUDIT_KEY, AUDIT_KEY);
}

#[test]
fn a_step_keeps_the_first_three_output_lines() {
    let lines = super::steps::first_lines("a\n\nb\nc\nd\n", 3);
    assert_eq!(lines, vec!["a", "b", "c"]);
}
