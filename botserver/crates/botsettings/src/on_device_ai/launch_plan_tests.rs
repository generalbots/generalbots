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

// ── placement: the third argument is now model_mib (file size / MiB) ───

#[test]
fn placement_compares_engine_vram_to_the_model_size() {
    let model = NINE_B_Q4 / MIB; // 5417 MiB

    // Fully on the card: engine used more than the model size (weights + KV + buffers).
    assert_eq!(
        placement(Some(7600), Some(1200), model),
        Placement::Gpu,
        "6400 MiB used >= 5417 MiB model",
    );

    // Split: engine used less than the model size (some layers on the processor).
    assert_eq!(
        placement(Some(5934), Some(802), model),
        Placement::Split,
        "5132 MiB used < 5417 MiB model — today's measured split",
    );

    // CPU: negligible video memory change.
    assert_eq!(
        placement(Some(6300), Some(6200), model),
        Placement::Cpu,
        "100 MiB used < 256 MiB slack",
    );

    // Unknown when either measurement is absent.
    assert_eq!(placement(None, Some(600), model), Placement::Unknown);
    assert_eq!(placement(Some(6300), None, model), Placement::Unknown);
}

#[test]
fn placement_with_todays_measured_numbers() {
    // RTX 4070 laptop (8188 MiB), Qwen3.5-9B-Q4_K_M (5417 MiB file).
    // With --fit-target 512: some layers stay on the processor.
    let model = 5417;

    // Load 1: before=5934, after=802, engine_vram=5132
    assert_eq!(placement(Some(5934), Some(802), model), Placement::Split);
    // Load 2: before=6158, after=822, engine_vram=5336
    assert_eq!(placement(Some(6158), Some(822), model), Placement::Split);
    // Load 3: before=6084, after=779, engine_vram=5305
    assert_eq!(placement(Some(6084), Some(779), model), Placement::Split);
}

#[test]
fn placement_full_offload_uses_more_vram_than_the_model_file() {
    // When all layers are on the card the engine holds weights + KV cache +
    // compute buffers, so engine_vram > model_mib.
    let model = 5417;
    // A 12 GB card with plenty of room: 7600 - 1000 = 6600 used > 5417.
    assert_eq!(placement(Some(7600), Some(1000), model), Placement::Gpu);
    // Edge case: engine used exactly model size (rounding tolerance).
    assert_eq!(placement(Some(7600), Some(2183), model), Placement::Gpu);
}

#[test]
fn placement_handles_a_small_model_on_a_large_card() {
    let small = ONE_FIVE_B_Q3 / MIB; // ~881 MiB
    // 6100 - 4900 = 1200 used > 881 → GPU.
    assert_eq!(placement(Some(6100), Some(4900), small), Placement::Gpu);
}

#[test]
fn placement_zero_model_mib_is_treated_as_gpu() {
    // Model size unknown: cannot judge split, so assume GPU.
    assert_eq!(placement(Some(6000), Some(1000), 0), Placement::Gpu);
}

#[test]
fn the_engine_opens_one_slot_and_a_smaller_margin_than_its_default() {
    assert_eq!(PARALLEL_SLOTS, 1);
    assert!(FIT_TARGET_MIB < 1024, "the engine's own default margin is 1024 MiB");
    assert!(FIT_TARGET_MIB >= 256, "leave the desktop some room");
}
