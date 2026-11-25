use super::lasting::{fact_to_remember, sentences};

fn kept(text: &str) -> Option<String> {
    fact_to_remember(text)
}

#[test]
fn a_preference_after_a_greeting_is_kept() {
    assert_eq!(
        kept("Oi! Prefiro respostas curtas e objetivas."),
        Some("Prefiro respostas curtas e objetivas".to_string())
    );
    assert_eq!(
        kept("Oi, eu prefiro respostas curtas e objetivas"),
        Some("eu prefiro respostas curtas e objetivas".to_string())
    );
    assert_eq!(
        kept("Bom dia! Eu prefiro respostas em tópicos."),
        Some("Eu prefiro respostas em tópicos".to_string())
    );
}

#[test]
fn a_preference_inside_a_longer_message_is_kept() {
    assert_eq!(
        kept("Oi! Tudo bem? Prefiro respostas curtas e objetivas."),
        Some("Prefiro respostas curtas e objetivas".to_string())
    );
    assert_eq!(
        kept("Oi! Só pra avisar, prefiro respostas curtas e objetivas. Pode me ajudar com um ofício?"),
        Some("prefiro respostas curtas e objetivas".to_string())
    );
    assert_eq!(
        kept("Oi! Prefiro respostas curtas e objetivas, ok?"),
        Some("Prefiro respostas curtas e objetivas".to_string())
    );
}

#[test]
fn from_now_on_is_a_lasting_preference() {
    assert_eq!(
        kept("a partir de agora, respostas curtas"),
        Some("a partir de agora, respostas curtas".to_string())
    );
    assert_eq!(
        kept("Oi! A partir de agora respostas curtas."),
        Some("A partir de agora respostas curtas".to_string())
    );
    assert_eq!(kept("From now on, answer in English"), Some("From now on, answer in English".to_string()));
}

#[test]
fn a_remember_request_after_small_talk_is_kept() {
    assert_eq!(
        kept("Oi! Tudo bem? Lembre que meu setor é a Ouvidoria."),
        Some("meu setor é a Ouvidoria".to_string())
    );
}

#[test]
fn questions_are_never_kept() {
    assert_eq!(kept("Oi! Prefiro respostas curtas ou longas?"), None);
    assert_eq!(kept("Oi, tudo bem?"), None);
    assert_eq!(kept("Oi! Você sabe do que eu prefiro?"), None);
    assert_eq!(kept("A partir de agora, como devo enviar o ofício?"), None);
    assert_eq!(kept("Oi! Tudo bem? Pode lembrar que horas são?"), None);
    assert_eq!(kept("Oi! Explique a nova lei de licitações."), None);
}

#[test]
fn sentences_end_only_before_a_space() {
    assert_eq!(
        sentences("Oi! Use a versão 2.0. Certo?"),
        vec![("Oi", Some('!')), ("Use a versão 2.0", Some('.')), ("Certo", Some('?'))]
    );
    assert_eq!(sentences("sem ponto"), vec![("sem ponto", None)]);
}
