//! CRM value helpers shared by every handler and HTMX fragment: stage
//! weighting (used by the pipeline forecast), HTML escaping for user data and
//! the compact currency format shown in grids.
use super::*;

#[test]
fn every_pipeline_stage_has_its_weight() {
    let cases = [
        ("new", 10),
        ("qualified", 25),
        ("qualification", 25),
        ("proposal", 50),
        ("negotiation", 75),
        ("won", 100),
        ("converted", 100),
        ("lost", 0),
    ];

    for (stage, expected) in cases {
        assert_eq!(stage_probability(stage), expected, "stage {stage}");
    }
}

#[test]
fn an_unknown_stage_is_treated_as_a_quarter_chance() {
    assert_eq!(stage_probability(""), 25);
    assert_eq!(stage_probability("WON"), 25);
}

#[test]
fn html_escape_covers_every_dangerous_character() {
    assert_eq!(
        html_escape("a & b <script> \"x\" 'y'"),
        "a &amp; b &lt;script&gt; &quot;x&quot; &#x27;y&#x27;"
    );
}

#[test]
fn html_escape_leaves_plain_text_alone() {
    assert_eq!(html_escape("Ana Silva"), "Ana Silva");
}

#[test]
fn currency_switches_to_thousands_and_millions() {
    assert_eq!(format_currency(0.0), "$0");
    assert_eq!(format_currency(999.0), "$999");
    assert_eq!(format_currency(1_000.0), "$1.0K");
    assert_eq!(format_currency(1_500.0), "$1.5K");
    assert_eq!(format_currency(1_000_000.0), "$1.0M");
    assert_eq!(format_currency(2_500_000.0), "$2.5M");
}

#[test]
fn an_amount_just_below_a_million_still_reads_in_thousands() {
    assert_eq!(format_currency(999_999.0), "$1000.0K");
}
