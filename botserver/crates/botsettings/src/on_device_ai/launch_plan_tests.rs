use super::*;

const NINE_B_Q4: u64 = 5_680_522_464;
const ONE_FIVE_B_Q3: u64 = 924_456_032;

#[test]
fn the_context_follows_the_free_memory_not_the_card_size() {
    assert_eq!(ctx_for_free_vram(NINE_B_Q4, Some(6100)), 8192);
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

// ── placement from the engine's layer report ───────────────────────────

#[test]
fn parse_offloaded_layers_reads_the_real_line() {
    let log = "\
0.01.153.544 I load_tensors: offloading output layer to GPU\n\
0.01.153.548 I load_tensors: offloading 31 repeating layers to GPU\n\
0.01.153.549 I load_tensors: offloaded 33/33 layers to GPU\n\
0.01.153.552 I load_tensors:   CPU_Mapped model buffer size =   545.62 MiB\n";
    assert_eq!(parse_offloaded_layers(log), Some((33, 33)));
}

#[test]
fn parse_offloaded_layers_reads_a_partial_offload() {
    let log = "0.01.200.000 I load_tensors: offloaded 20/33 layers to GPU\n";
    assert_eq!(parse_offloaded_layers(log), Some((20, 33)));
}

#[test]
fn parse_offloaded_layers_returns_none_when_absent() {
    assert_eq!(parse_offloaded_layers(""), None);
    assert_eq!(parse_offloaded_layers("some other log line\n"), None);
    assert_eq!(parse_offloaded_layers("offloaded but no slash"), None);
}

#[test]
fn placement_from_layers_classifies_correctly() {
    assert_eq!(placement_from_layers(Some((33, 33))), Placement::Gpu);
    assert_eq!(placement_from_layers(Some((20, 33))), Placement::Split);
    assert_eq!(placement_from_layers(Some((0, 33))), Placement::Cpu);
    assert_eq!(placement_from_layers(None), Placement::Unknown);
}

#[test]
fn todays_full_offload_is_gpu_not_split() {
    // RTX 4070 laptop, Qwen3.5-9B-Q4_K_M: 33/33 layers offloaded,
    // even though VRAM used (5287 MiB) < file size (5417 MiB) because
    // 546 MiB of weights are CPU-mapped by design.
    let log = "0.01.153.549 I load_tensors: offloaded 33/33 layers to GPU\n";
    assert_eq!(
        placement_from_layers(parse_offloaded_layers(log)),
        Placement::Gpu,
        "a full 33/33 offload must be Gpu, regardless of VRAM arithmetic"
    );
}

#[test]
fn parse_projected_mib_reads_the_real_line() {
    let log = "0.00.461.791 I common_params_fit_impl: projected to use 5279 MiB of device memory vs. 7068 MiB of free device memory\n";
    assert_eq!(parse_projected_mib(log), Some(5279));
}

#[test]
fn parse_projected_mib_returns_none_when_absent() {
    assert_eq!(parse_projected_mib(""), None);
    assert_eq!(parse_projected_mib("no projection here\n"), None);
}

#[test]
fn the_engine_opens_one_slot_and_a_smaller_margin_than_its_default() {
    assert_eq!(PARALLEL_SLOTS, 1);
    assert!(FIT_TARGET_MIB < 1024, "the engine's own default margin is 1024 MiB");
    assert!(FIT_TARGET_MIB >= 256, "leave the desktop some room");
}
