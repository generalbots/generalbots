use super::*;

use super::*;

const PRESETS: [&str; 9] = [
    "corporate",
    "creative",
    "minimal",
    "dark",
    "warm",
    "nature",
    "ocean",
    "sunset",
    "midnight",
];

#[test]
fn every_preset_keeps_its_name_and_colours() {
    for preset in PRESETS {
        let theme = theme_from_name(preset);
        assert_eq!(theme.name, preset);
        assert!(theme.colors.primary.starts_with('#'), "{preset} primary");
        assert!(
            theme.colors.background.starts_with('#'),
            "{preset} background"
        );
        assert_eq!(theme.colors.primary.len(), 7, "{preset} primary is #rrggbb");
    }
}

#[test]
fn presets_have_distinct_primary_colours() {
    let mut seen = std::collections::HashSet::new();
    for preset in PRESETS {
        assert!(
            seen.insert(theme_from_name(preset).colors.primary),
            "{preset} must not reuse another preset's primary colour"
        );
    }
}

#[test]
fn an_unknown_preset_falls_back_to_the_default_palette() {
    let theme = theme_from_name("nonexistent");
    assert_eq!(theme.name, "nonexistent");
    let default = create_default_theme();
    assert_eq!(theme.colors.primary, default.colors.primary);
    assert_eq!(theme.colors.background, default.colors.background);
}

#[test]
fn an_empty_name_is_labelled_default() {
    let theme = theme_from_name("");
    assert_eq!(theme.name, "Default");
    assert_eq!(theme.colors.primary, create_default_theme().colors.primary);
}

#[test]
fn a_dark_preset_keeps_a_light_text_colour() {
    let dark = theme_from_name("dark");
    // The text must contrast with the dark background.
    assert_eq!(dark.colors.text, "#f8fafc");
}
