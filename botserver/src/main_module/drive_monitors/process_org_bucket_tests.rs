//! Unit tests for `super` (process_org_bucket.rs).
//!
//! Split out of `process_org_bucket.rs` to keep the file inside the project
//! size limit; the production code stays beside it. The S3 listing and the
//! database writes need real services (out of scope), so the tests cover the
//! pure object-key and slug parsers that decide which bots a bucket contains.

use super::*;

fn names(keys: &[&str]) -> Vec<String> {
    extract_bot_names(keys.iter().copied(), "acme.gbai/")
        .into_iter()
        .collect()
}

#[test]
fn a_branch_key_names_the_bot_from_its_dialog_directory() {
    assert_eq!(
        names(&["acme.gbai/help.gbdialog/start.bas"]),
        vec!["help".to_string()]
    );
}

#[test]
fn any_bot_directory_identifies_the_bot_without_dialog_sources() {
    // Reform #1501: the scripts live in the bot's repository, so `.gbdialog`
    // is not guaranteed to be present in Drive.
    let mut found = names(&[
        "acme.gbai/help.gbkb/docs/manual.pdf",
        "acme.gbai/sales.gbot/PROMPT-WEB.md",
        "acme.gbai/ops.gbdrive/report.csv",
    ]);
    found.sort();
    assert_eq!(found, vec!["help", "ops", "sales"]);
}

#[test]
fn a_nested_key_is_a_path_not_a_bot() {
    // The caller then inserts the branch slug as the default bot.
    assert!(names(&["acme.gbai/nested/help.gbdialog/x.bas"]).is_empty());
}

#[test]
fn keys_outside_the_branch_prefix_are_ignored() {
    assert!(names(&["other.gbai/help.gbdialog/start.bas"]).is_empty());
}

#[test]
fn bot_names_are_sorted_and_deduplicated() {
    assert_eq!(
        names(&[
            "acme.gbai/beta.gbdialog/a.bas",
            "acme.gbai/alpha.gbdialog/a.bas",
            "acme.gbai/alpha.gbdialog/b.bas",
            "acme.gbai/beta.gbkb/x.md",
        ]),
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn the_marker_nearest_the_start_of_the_key_names_the_bot() {
    // `help.gbkb/docs/x.gbdialog` is a file of bot `help` whose *name* contains
    // a later marker. Slicing at `.gbdialog` (the first entry of
    // `BOT_MARKERS`) returned `help.gbkb/docs/x`, a path, so the bot was
    // dropped and discovery fell back to the branch slug — a real bug fixed by
    // choosing the earliest marker position instead of the array order.
    assert_eq!(
        names(&["acme.gbai/help.gbkb/docs/x.gbdialog"]),
        vec!["help".to_string()]
    );
}

#[test]
fn dots_inside_a_bot_name_are_preserved() {
    assert_eq!(
        names(&["acme.gbai/help.bot.gbdialog/start.bas"]),
        vec!["help.bot".to_string()]
    );
}

#[test]
fn no_objects_yields_no_bots() {
    assert!(names(&[]).is_empty());
}

#[test]
fn a_key_without_any_bot_marker_is_not_a_bot() {
    assert!(names(&["acme.gbai/README.md"]).is_empty());
}

#[test]
fn the_marker_set_is_the_documented_one() {
    // Pins the accepted bot directories: adding one changes what discovery
    // treats as a bot.
    assert_eq!(BOT_MARKERS, [".gbdialog", ".gbot", ".gbkb", ".gbdrive"]);
}

#[test]
fn the_tenant_slug_drops_the_gborg_suffix() {
    assert_eq!(tenant_slug_of("acme.gborg"), "acme");
    assert_eq!(tenant_slug_of("acme"), "acme");
    assert_eq!(tenant_slug_of(""), "");
}

#[test]
fn the_branch_slug_requires_the_gbai_prefix_suffix() {
    assert_eq!(branch_slug_of("acme.gbai/"), Some("acme"));
    assert_eq!(branch_slug_of("acme.gbai"), None);
    assert_eq!(branch_slug_of("acme/"), None);
    assert_eq!(branch_slug_of(""), None);
}
