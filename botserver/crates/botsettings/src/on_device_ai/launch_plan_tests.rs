use super::*;

const NINE_B_Q4: u64 = 5_680_522_464;
const ONE_FIVE_B_Q3: u64 = 924_456_032;

#[test]
fn the_context_follows_the_free_memory_not_the_card_size() {
    // An 8 GB laptop card with the desktop and other programs on it.
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, Some(6100)), 8192);
    // The same card nearly empty.
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, Some(7900)), 8192);
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, Some(8600)), 16384);
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, Some(22000)), 32768);
    assert_eq!(ctx_for_free_vram(ONE_FIVE_B_Q3, Some(6100)), 32768);
}

#[test]
fn the_context_falls_back_when_nothing_is_known() {
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, None), BASE_CTX, "no graphics card");
    assert_eq!(ctx_for_free_vram(0, Some(24000)), BASE_CTX, "model size unknown");
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, Some(0)), BASE_CTX, "card full");
}

#[test]
fn full_offload_is_expected_only_with_room_for_weights_context_and_margin() {
    assert!(!expect_full_offload(NINE_B_Q4, 8192, 6100));
    assert!(expect_full_offload(NINE_B_Q4, 8192, 7600));
    assert!(expect_full_offload(ONE_FIVE_B_Q3, 32768, 6100));
    assert!(!expect_full_offload(NINE_B_Q4, 32768, 7600));
}

#[test]
fn a_card_filled_to_the_margin_means_a_split_model() {
    let margin = u64::from(FIT_TARGET_MIB);
    // Measured on the owner's card: the engine left only its margin free.
    assert_eq!(placement(Some(6300), Some(600), margin), Placement::Split);
    assert_eq!(placement(Some(7600), Some(1400), margin), Placement::Gpu);
    assert_eq!(placement(Some(6300), Some(6200), margin), Placement::Cpu);
    assert_eq!(placement(None, Some(600), margin), Placement::Unknown);
    assert_eq!(placement(Some(6300), None, margin), Placement::Unknown);
}

#[test]
fn the_engine_opens_one_slot_and_a_smaller_margin_than_its_default() {
    assert_eq!(PARALLEL_SLOTS, 1);
    assert!(FIT_TARGET_MIB < 1024, "the engine's own default margin is 1024 MiB");
    assert!(FIT_TARGET_MIB >= 256, "leave the desktop some room");
}
