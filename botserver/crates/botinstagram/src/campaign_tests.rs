//! Offline tests for the Instagram campaign builder (issue #2075, channels
//! wave). The helpers are pure: no Graph API call, no image backend.

use super::*;

#[test]
fn a_campaign_defaults_to_one_image() {
    assert_eq!(campaign_image_count(None), 1);
}

#[test]
fn a_request_for_zero_images_still_produces_one() {
    // `num_images: 0` used to fall through the "publish immediately" branch and
    // return a single image anyway; the count is now explicit.
    assert_eq!(campaign_image_count(Some(0)), 1);
}

#[test]
fn the_requested_image_count_is_honoured_up_to_the_published_ceiling() {
    let cases = [(Some(1), 1), (Some(2), 2), (Some(9), 9), (Some(10), 10)];
    for (requested, expected) in cases {
        assert_eq!(campaign_image_count(requested), expected, "{requested:?}");
    }
}

#[test]
fn an_unbounded_image_count_is_clamped() {
    for requested in [11, 100, u32::MAX] {
        assert_eq!(
            campaign_image_count(Some(requested)),
            MAX_CAMPAIGN_IMAGES,
            "{requested}"
        );
    }
}

#[test]
fn every_image_of_a_campaign_gets_its_own_url() {
    let urls: Vec<String> = (0..campaign_image_count(Some(3)))
        .map(|index| campaign_image_url("summer sale", index))
        .collect();

    assert_eq!(urls.len(), 3);
    assert_eq!(
        urls,
        vec![
            "https://via.placeholder.com/1080x1080.png?text=summer%20sale&n=1".to_string(),
            "https://via.placeholder.com/1080x1080.png?text=summer%20sale&n=2".to_string(),
            "https://via.placeholder.com/1080x1080.png?text=summer%20sale&n=3".to_string(),
        ]
    );
}

#[test]
fn the_prompt_is_url_encoded_in_the_image_url() {
    let url = campaign_image_url("50% off & more", 0);

    assert!(url.contains("50%25%20off%20%26%20more"), "url {url}");
    assert!(!url.contains(' '), "no raw spaces may reach the query string");
}

#[test]
fn the_prompt_is_trimmed_before_it_reaches_the_image_url() {
    let url = campaign_image_url("   sale   ", 0);
    assert!(url.contains("text=sale&n=1"), "url {url}");
}

#[test]
fn a_long_prompt_is_truncated_at_thirty_bytes() {
    let url = campaign_image_url(&"a".repeat(100), 0);
    let text = url
        .split("text=")
        .nth(1)
        .and_then(|rest| rest.split('&').next())
        .expect("text parameter");

    assert_eq!(text.len(), 30);
}

#[test]
fn multibyte_prompts_are_truncated_on_a_char_boundary() {
    // "ção" is 5 bytes per repeat, so a 30-byte cut lands exactly on a
    // boundary and the percent-encoded text must still decode as UTF-8.
    let url = campaign_image_url(&"ção".repeat(20), 0);
    let text = url
        .split("text=")
        .nth(1)
        .and_then(|rest| rest.split('&').next())
        .expect("text parameter");
    let decoded = urlencoding::decode(text).expect("valid percent-encoding");

    assert_eq!(decoded.as_ref(), "ção".repeat(6).as_str());
}

#[test]
fn the_caption_appends_the_campaign_tags() {
    assert_eq!(
        campaign_caption("Summer sale"),
        "Summer sale #generalbots #instagram #campaign"
    );
}

#[test]
fn a_long_caption_is_cut_at_one_hundred_and_twenty_characters() {
    let caption = campaign_caption(&"x".repeat(200));
    let body = caption
        .strip_suffix(" #generalbots #instagram #campaign")
        .expect("tags are appended");

    assert_eq!(body.chars().count(), 120);
}

#[test]
fn caption_truncation_counts_characters_not_bytes() {
    let caption = campaign_caption(&"á".repeat(200));
    let body = caption
        .strip_suffix(" #generalbots #instagram #campaign")
        .expect("tags are appended");

    assert_eq!(body.chars().count(), 120);
}
