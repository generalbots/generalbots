use super::*;

const HOOK_TEXT: &str = "veja os anexos\n\n\
[User attached a file stored at inbox/Tusks Poster.jpg; size=9704711; the upload was staged for the bot]\n\
\u{1F4CE} Tusks Poster.jpg\n\n\
[User attached a file stored at inbox/Sentient Pixel - CBD Marketing.pdf; size=812; the upload was staged for the bot]\n\
\u{1F4CE} Sentient Pixel - CBD Marketing.pdf\n\n\
[User attached a file stored at inbox/relatório (1); size=final.txt; size=12; the upload was staged for the bot]";

#[test]
fn attachment_names_are_read_whole_from_the_hook_markers() {
    assert_eq!(
        attached_names(HOOK_TEXT),
        vec![
            "Tusks Poster.jpg".to_string(),
            "Sentient Pixel - CBD Marketing.pdf".to_string(),
            "relatório (1); size=final.txt".to_string(),
        ]
    );
    assert!(attached_names("summarise sales.csv from my Drive").is_empty());
}

#[test]
fn the_drive_guess_no_longer_sees_pieces_of_attachment_names() {
    let rest = without_attachment_lines(HOOK_TEXT);
    assert_eq!(rest.trim(), "veja os anexos");
    assert!(!rest.contains("Poster.jpg"));
}

#[test]
fn names_inside_the_vm_pass_the_workspace_path_check() {
    for (raw, expected) in [
        ("Tusks Poster.jpg", "Tusks Poster.jpg"),
        ("relatório final (1).pdf", "relatorio final _1_.pdf"),
        ("../../etc/passwd", "passwd"),
        (".hidden", "hidden"),
        ("日本.png", "__.png"),
        ("***", "anexo"),
    ] {
        let name = vm_file_name(raw);
        assert_eq!(name, expected, "{raw}");
        assert!(
            botagent::transfer::safe_rel_path(&format!("data/{name}")).is_ok(),
            "{raw} -> {name}"
        );
    }
}

#[test]
fn the_note_names_every_exact_path_and_the_failures() {
    let copied = vec![(
        "Tusks Poster.jpg".to_string(),
        "/opt/vibe/conversations/c/data/Tusks Poster.jpg".to_string(),
        9_704_711,
    )];
    let note = workspace_note(&copied, &["\"x.pdf\"".to_string()]);
    assert!(note.contains("/opt/vibe/conversations/c/data/Tusks Poster.jpg"));
    assert!(note.contains("read the whole file"));
    assert!(note.contains("part of what they asked for"));
    assert!(note.contains("could not be copied"));
    assert!(workspace_note(&[], &[]).is_empty());
}

#[test]
fn staged_files_last_until_the_next_message() {
    let session = Uuid::new_v4();
    let file = StagedFile { vm_path: "/opt/vibe/conversations/c/data/a.png".to_string(), image: true };
    record(session, vec![file.clone()]);
    assert_eq!(staged_for(session), vec![file.clone()]);
    assert_eq!(staged_for(session), vec![file], "a retried attempt still sees them");
    clear(session);
    assert!(staged_for(session).is_empty());
    assert!(is_image("A.JPG") && !is_image("a.pdf"));
}
