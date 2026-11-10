//! Creation contract for the chat `slides.create` verb (#2143).
//!
//! The endpoint must answer with a deck that already carries the requested
//! title and slides, plus the id the deep link opens.

use crate::storage::create_presentation_with;
use crate::types::DeckSlideInput;

#[test]
fn created_deck_carries_an_id_title_and_slides() {
    let slides = [
        DeckSlideInput {
            title: Some("Agenda".to_string()),
            body: Some("Goals & risks".to_string()),
        },
        DeckSlideInput {
            title: Some("Next steps".to_string()),
            body: None,
        },
    ];
    let deck = create_presentation_with("owner-1", Some("  Kickoff "), Some(&slides));

    assert!(!deck.id.is_empty(), "the created deck needs an id");
    assert_eq!(deck.name, "Kickoff");
    assert_eq!(deck.owner_id, "owner-1");
    assert_eq!(deck.slides.len(), 2);
    assert_eq!(
        deck.slides[0].elements[0].content.text.as_deref(),
        Some("Agenda")
    );
    assert_eq!(
        deck.slides[0].elements[1].content.text.as_deref(),
        Some("Goals & risks")
    );
    assert_eq!(
        deck.slides[1].elements[0].content.text.as_deref(),
        Some("Next steps")
    );
}

#[test]
fn a_deck_without_slides_keeps_the_default_title_slide() {
    let deck = create_presentation_with("owner-1", None, None);
    assert_eq!(deck.name, "Untitled Presentation");
    assert_eq!(deck.slides.len(), 1);
    assert_eq!(deck.slides[0].layout, "title");
}
