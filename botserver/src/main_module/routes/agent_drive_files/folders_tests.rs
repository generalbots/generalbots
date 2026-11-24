use uuid::Uuid;

use super::{group, owner_of, validate_path, OwnedSession, PathRefusal};

fn session(id: &str, title: &str, last_activity: i64) -> OwnedSession {
    OwnedSession {
        id: Uuid::parse_str(id).unwrap_or_default(),
        bot_id: Uuid::nil(),
        title: title.to_string(),
        last_activity,
    }
}

const MINE: &str = "1a2b3c4d-0000-4000-8000-000000000001";
const OLD: &str = "9f8e7d6c-0000-4000-8000-000000000002";

#[test]
fn a_folder_is_matched_by_the_session_id_suffix() {
    let sessions = vec![session(MINE, "Memorando", 20)];
    let owner = owner_of("Memorando-de-vacinação-1a2b3c4d", &sessions);
    assert_eq!(owner.and_then(|o| o.conversation_id), Uuid::parse_str(MINE).ok());
}

#[test]
fn a_folder_of_someone_else_is_not_owned() {
    let sessions = vec![session(MINE, "Memorando", 20)];
    assert!(owner_of("Relatorio-ffffffff", &sessions).is_none());
    assert!(owner_of("Outra-conversa", &sessions).is_none());
}

#[test]
fn an_old_title_only_folder_is_never_owned() {
    // A title can be shared by several people's conversations on one bot:
    // only the id suffix proves ownership.
    let sessions = vec![session(MINE, "Plano de março", 20)];
    assert!(owner_of("Plano-de-março", &sessions).is_none());
    assert!(owner_of("Plano-de-maro", &sessions).is_none());
}

#[test]
fn grouping_keeps_only_owned_folders_newest_first() {
    let sessions = vec![session(MINE, "Novo", 30), session(OLD, "Antigo", 10)];
    let objects = vec![
        ("Antigo/a.txt".to_string(), 1),
        ("Novo-1a2b3c4d/memo.docx".to_string(), 2),
        ("Novo-1a2b3c4d/sub/plan.xlsx".to_string(), 3),
        ("Alheio-ffffffff/secret.pdf".to_string(), 4),
        ("Novo-1a2b3c4d/".to_string(), 0),
    ];
    let folders = group(&objects, &sessions);
    let names: Vec<&str> = folders.iter().map(|f| f.folder.as_str()).collect();
    assert_eq!(names, vec!["Novo-1a2b3c4d"], "the title-only folder is not listed");
    let first = &folders[0];
    assert_eq!(first.files.len(), 2);
    assert_eq!(first.files[1].name, "plan.xlsx");
    assert_eq!(first.files[1].path, "conversations/Novo-1a2b3c4d/sub/plan.xlsx");
    assert!(folders.iter().all(|f| f.files.iter().all(|file| file.name != "secret.pdf")));
}

#[test]
fn paths_outside_a_conversation_folder_are_refused() {
    assert_eq!(validate_path("conversations/Novo-1a2b3c4d/memo.docx"), Ok("Novo-1a2b3c4d"));
    assert_eq!(validate_path(""), Err(PathRefusal::Empty));
    assert_eq!(validate_path("/conversations/a/b.txt"), Err(PathRefusal::Absolute));
    assert_eq!(validate_path("C:/conversations/a/b.txt"), Err(PathRefusal::Absolute));
    assert_eq!(validate_path("\\\\server\\share"), Err(PathRefusal::Absolute));
    assert_eq!(validate_path("conversations/../secrets/x"), Err(PathRefusal::Traversal));
    assert_eq!(validate_path("conversations/a/../../x"), Err(PathRefusal::Traversal));
    assert_eq!(validate_path("conversations/a/./b"), Err(PathRefusal::Traversal));
    assert_eq!(validate_path("conversations//b"), Err(PathRefusal::Traversal));
    assert_eq!(validate_path("conversations/a\\..\\b"), Err(PathRefusal::Traversal));
    assert_eq!(validate_path("conversations/a/b\u{0}.txt"), Err(PathRefusal::Traversal));
    assert_eq!(validate_path("documents/a/b.txt"), Err(PathRefusal::OutsideConversations));
    assert_eq!(validate_path("conversations/a"), Err(PathRefusal::OutsideConversations));
}
