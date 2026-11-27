//! Pure tier/model selection tests for the on-device catalogue (#2109).

use super::{
    capabilities, catalog, effective_budget_gb, find, fit_verdict, fits, profiles, recommended,
    row_status,
    tier_groups, STATUS_ACTIVE, STATUS_AVAILABLE, STATUS_DOWNLOADED, STATUS_DOWNLOADING,
};

#[test]
fn recommends_the_largest_model_that_fits_the_detected_vram() {
    assert_eq!(recommended(8, 16).id, "qwen3.5-9b-q4km");
    assert_eq!(recommended(16, 32).id, "qwen3.8-27b-udq3kxl");
    assert_eq!(recommended(24, 64).id, "qwen3.8-27b-udq4kxl");
    assert_eq!(recommended(32, 64).id, "qwen3.8-27b-udq6k");
    assert_eq!(recommended(48, 128).id, "qwen3.8-27b-udq6k");
    assert_eq!(recommended(72, 128).id, "qwen3.8-27b-udq6k");
    assert_eq!(recommended(96, 128).id, "qwen3.8-27b-udq6k");
    assert_eq!(recommended(128, 256).id, "qwen3.8-27b-udq6k");
}

#[test]
fn falls_back_to_ram_when_no_gpu_is_detected() {
    assert_eq!(effective_budget_gb(0, 64), 64);
    assert_eq!(recommended(0, 32).id, "qwen3.8-27b-udq6k");
    // A GPU always wins over the larger system RAM.
    assert_eq!(effective_budget_gb(8, 128), 8);
}

#[test]
fn a_machine_below_the_smallest_tier_still_gets_one_recommendation() {
    let pick = recommended(0, 4);
    assert_eq!(pick.id, "deepseek-r1-distill-qwen-7b-q4km");
    assert!(!fits(pick, 0, 4), "the row must warn that it does not fit");
}

#[test]
fn the_bundled_server_model_is_never_recommended() {
    for budget in [0, 8, 24, 96, 128, 512] {
        assert!(!recommended(budget, 512).server_only);
    }
}

#[test]
fn tiers_ascend_with_no_server_profile() {
    let groups = tier_groups();
    assert_eq!(groups.len(), 5);
    let tiers: Vec<u16> = groups.iter().filter_map(|group| group.tier_gb).collect();
    assert_eq!(tiers, vec![6, 8, 16, 24, 32]);
    // No server-only entries remain after the licence clean-up.
    assert!(groups.iter().all(|group| group.tier_gb.is_some()));
}

#[test]
fn every_catalogue_entry_is_addressable_and_labelled() {
    for spec in catalog() {
        assert!(find(spec.id).is_some(), "missing id {}", spec.id);
        assert!(!spec.name.is_empty());
        assert!(!spec.file.ends_with('/'));
        assert!(spec.size_bytes > 0);
        assert!(!spec.licence.is_empty(), "missing licence for {}", spec.id);
        // server_only entries may have no source_host
        if !spec.server_only {
            assert!(!spec.source_host.is_empty(), "missing source_host for {}", spec.id);
        }
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
fn fit_verdict_three_tiers() {
    let spec = find("qwen3.5-9b-q4km").expect("catalogue entry");
    // 8 GB min, comfortable threshold = 10 GB
    assert_eq!(fit_verdict(spec, 12, 16), "comfortable");
    assert_eq!(fit_verdict(spec, 8, 16), "tight");
    assert_eq!(fit_verdict(spec, 0, 4), "wont_run");
    // CPU-only with enough RAM is comfortable
    assert_eq!(fit_verdict(spec, 0, 32), "comfortable");
}

#[test]
fn profiles_are_named_bundles_with_growing_context() {
    let list = profiles();
    assert_eq!(list.len(), 3);
    assert!(list[0].ctx_size < list[1].ctx_size);
    assert!(list[1].ctx_size < list[2].ctx_size);
}

#[test]
fn every_model_says_what_it_is_good_at() {
    for spec in catalog() {
        assert!(!capabilities(spec).is_empty(), "no capability for {}", spec.id);
    }
    let r1 = find("deepseek-r1-distill-qwen-7b-q4km").expect("catalogue entry");
    assert_eq!(capabilities(r1).first(), Some(&"reasoning"));
    let qwen = find("qwen3.5-9b-q4km").expect("catalogue entry");
    assert!(capabilities(qwen).contains(&"multilingual"));
}

#[test]
fn every_catalogue_entry_has_a_size_a_sha256_and_a_gguf_name() {
    for spec in catalog() {
        assert!(spec.size_bytes > 0, "missing size for {}", spec.id);
        assert!(
            !spec.sha256.is_empty(),
            "missing SHA-256 pin for {}",
            spec.id
        );
        assert!(
            spec.file.ends_with(".gguf"),
            "file name does not end in .gguf for {}: {}",
            spec.id,
            spec.file
        );
    }
}

#[test]
fn recommended_model_exists_for_every_vram_tier() {
    // A tier is a min_vram_gb value present in the catalogue. The recommended
    // model for each tier must exist in the catalogue after the clean-up.
    let tiers: Vec<u16> = catalog()
        .iter()
        .filter(|spec| !spec.server_only)
        .map(|spec| spec.min_vram_gb)
        .collect();
    for tier in tiers {
        let pick = recommended(tier, 64);
        assert!(
            find(pick.id).is_some(),
            "recommended model for {tier} GB tier ({}) is not in the catalogue",
            pick.id
        );
        assert!(
            !pick.server_only,
            "recommended model for {tier} GB tier ({}) is server-only",
            pick.id
        );
    }
}
