//! Pure tier/model selection tests for the on-device catalogue (#2109).

use super::{
    catalog, effective_budget_gb, find, fits, profiles, recommended, row_status, tier_groups,
    STATUS_ACTIVE, STATUS_AVAILABLE, STATUS_DOWNLOADED, STATUS_DOWNLOADING,
};

#[test]
fn recommends_the_largest_model_that_fits_the_detected_vram() {
    assert_eq!(recommended(8, 16).id, "qwen3.5-9b-q4km");
    assert_eq!(recommended(12, 32).id, "qwen3.5-14b-q4km");
    assert_eq!(recommended(16, 32).id, "qwen3.8-27b-q3kxl");
    assert_eq!(recommended(24, 64).id, "qwen3.8-27b-udq4kxl");
    assert_eq!(recommended(32, 64).id, "qwen3.8-27b-q6");
    assert_eq!(recommended(48, 128).id, "llama-3.3-70b-q4km");
    assert_eq!(recommended(72, 128).id, "llama-3.3-70b-q5km");
    assert_eq!(recommended(96, 128).id, "gpt-oss-120b-mxfp4");
    assert_eq!(recommended(128, 256).id, "gpt-oss-120b-q6");
}

#[test]
fn falls_back_to_ram_when_no_gpu_is_detected() {
    assert_eq!(effective_budget_gb(0, 64), 64);
    assert_eq!(recommended(0, 32).id, "qwen3.8-27b-q6");
    // A GPU always wins over the larger system RAM.
    assert_eq!(effective_budget_gb(8, 128), 8);
}

#[test]
fn a_machine_below_the_smallest_tier_still_gets_one_recommendation() {
    let pick = recommended(0, 4);
    assert_eq!(pick.id, "qwen3.5-9b-q4km");
    assert!(!fits(pick, 0, 4), "the row must warn that it does not fit");
}

#[test]
fn the_bundled_server_model_is_never_recommended() {
    for budget in [0, 8, 24, 96, 128, 512] {
        assert!(!recommended(budget, 512).server_only);
    }
}

#[test]
fn tiers_ascend_and_the_server_profile_comes_last() {
    let groups = tier_groups();
    assert_eq!(groups.len(), 10);
    let tiers: Vec<u16> = groups.iter().filter_map(|group| group.tier_gb).collect();
    assert_eq!(tiers, vec![8, 12, 16, 24, 32, 48, 72, 96, 128]);
    let last = groups.last().expect("server profile group");
    assert_eq!(last.tier_gb, None);
    assert!(last.models.iter().all(|spec| spec.server_only));
}

#[test]
fn every_catalogue_entry_is_addressable_and_labelled() {
    for spec in catalog() {
        assert!(find(spec.id).is_some(), "missing id {}", spec.id);
        assert!(!spec.name.is_empty());
        assert!(!spec.file.ends_with('/'));
        assert!(spec.size_bytes > 0);
    }
    assert!(find("does-not-exist").is_none());
}

#[test]
fn status_precedence_is_active_then_downloaded_then_downloading() {
    assert_eq!(row_status(true, true, true), STATUS_ACTIVE);
    assert_eq!(row_status(false, true, true), STATUS_DOWNLOADED);
    assert_eq!(row_status(false, false, true), STATUS_DOWNLOADING);
    assert_eq!(row_status(false, false, false), STATUS_AVAILABLE);
}

#[test]
fn profiles_are_named_bundles_with_growing_context() {
    let list = profiles();
    assert_eq!(list.len(), 3);
    assert!(list[0].ctx_size < list[1].ctx_size);
    assert!(list[1].ctx_size < list[2].ctx_size);
}
