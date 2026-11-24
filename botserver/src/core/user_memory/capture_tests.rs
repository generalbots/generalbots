use super::capture::{compose_note, fact_to_remember, strip_lead_in, MemoryTurn};

#[test]
fn tags_and_greetings_before_the_request_are_ignored() {
    assert_eq!(strip_lead_in("[qa] Lembre que x"), "Lembre que x");
    assert_eq!(strip_lead_in("[teste] Por favor, lembre que x"), "lembre que x");
    assert_eq!(strip_lead_in("Olá! Lembre-se: x"), "Lembre-se: x");
    assert_eq!(strip_lead_in("Oito horas"), "Oito horas");
    assert_eq!(strip_lead_in("ok"), "ok");
}

#[test]
fn a_tagged_remember_request_keeps_the_whole_fact() {
    assert_eq!(
        fact_to_remember("[qa] Lembre que meu setor é a Secretaria de Saúde e prefiro respostas curtas."),
        Some("meu setor é a Secretaria de Saúde e prefiro respostas curtas".to_string())
    );
    assert_eq!(
        fact_to_remember("Quero que você lembre que uso o SEI todos os dias"),
        Some("uso o SEI todos os dias".to_string())
    );
}

#[test]
fn a_lasting_preference_is_kept_without_the_word_remember() {
    assert_eq!(fact_to_remember("Prefiro respostas curtas."), Some("Prefiro respostas curtas".to_string()));
    assert_eq!(fact_to_remember("My name is Ana"), Some("My name is Ana".to_string()));
}

#[test]
fn questions_tasks_and_long_texts_are_not_memories() {
    assert_eq!(fact_to_remember("[qa] Qual é o meu setor?"), None);
    assert_eq!(fact_to_remember("Pode lembrar que horas são?"), None);
    assert_eq!(fact_to_remember("Explique a nova lei de licitações"), None);
    let pasted = format!("Prefiro que você revise este texto:\n{}", "palavra ".repeat(60));
    assert_eq!(fact_to_remember(&pasted), None);
}

#[test]
fn the_note_states_what_is_remembered_and_what_this_turn_did() {
    let saved = compose_note(true, "- prefere respostas curtas\n", &MemoryTurn::Saved("prefere respostas curtas".into()));
    assert!(saved.contains("Stored for this person:\n- prefere respostas curtas"));
    assert!(saved.contains("Vou lembrar disso."));
    let empty = compose_note(true, "", &MemoryTurn::Untouched);
    assert!(empty.contains("Nothing is stored yet"));
    assert!(empty.contains("Never say you cannot remember"));
    assert!(!empty.contains("THIS TURN"));
    let failed = compose_note(true, "", &MemoryTurn::Failed);
    assert!(failed.contains("never say it was saved"));
}

#[test]
fn a_switched_off_memory_shows_nothing_and_claims_nothing() {
    let note = compose_note(false, "- segredo\n", &MemoryTurn::Off);
    assert!(note.contains("switched off"));
    assert!(!note.contains("segredo"));
    assert!(!note.contains("Vou lembrar"));
}
