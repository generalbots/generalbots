//! Strengthened tests for the pure `graphics_support_status()` classifier (#2481).
//!
//! Loaded from `os_appliance.rs` via `#[cfg(test)] #[path = "..."]` because that
//! module already exceeds the 450-line limit. As a child module it still reaches
//! the private classifier through `super::`, so the tests and the support bundle
//! exercise exactly the same code. Every case here is pure: no host probe, no I/O.

use super::graphics_support_status;

/// Read a string field, returning a sentinel so an absent field fails loudly.
fn field_str<'a>(status: &'a serde_json::Value, key: &str) -> &'a str {
    match status.get(key).and_then(serde_json::Value::as_str) {
        Some(value) => value,
        None => "<missing>",
    }
}

/// Read a boolean field without inventing a value when it is absent.
fn field_bool(status: &serde_json::Value, key: &str) -> Option<bool> {
    status.get(key).and_then(serde_json::Value::as_bool)
}

/// Read an array-of-strings field; absent or malformed arrays become empty.
fn field_strings<'a>(status: &'a serde_json::Value, key: &str) -> Vec<&'a str> {
    match status.get(key).and_then(serde_json::Value::as_array) {
        Some(array) => array.iter().filter_map(serde_json::Value::as_str).collect(),
        None => Vec::new(),
    }
}

#[test]
fn software_rasterizer_markers_are_detected_case_insensitively() {
    // All four markers from the classifier, including mixed/upper case and a
    // marker buried in a multi-line glxinfo -B block.
    let cases = [
        (
            "llvmpipe",
            "OpenGL renderer: llvmpipe (LLVM 17.0, 256 bits)",
        ),
        ("LLVMPIPE", "OpenGL renderer: LLVMPIPE (LLVM 17.0)"),
        ("LlvmPipe", "OpenGL renderer: LlvmPipe"),
        ("softpipe", "OpenGL renderer: softpipe"),
        ("SOFTPIPE", "OpenGL renderer: SOFTPIPE"),
        ("swrast", "OpenGL renderer: swrast"),
        ("SWRAST", "OpenGL renderer: SWRAST"),
        (
            "Software Rasterizer",
            "OpenGL renderer: Software Rasterizer (Mesa)",
        ),
        (
            "SOFTWARE RASTERIZER",
            "OpenGL renderer: SOFTWARE RASTERIZER",
        ),
        (
            "llvmpipe in multiline",
            "OpenGL vendor: Mesa\nOpenGL renderer: llvmpipe\nOpenGL version: 4.5",
        ),
    ];
    for (label, glx) in cases {
        let status = graphics_support_status(None, None, Some(glx));
        assert_eq!(
            field_str(&status, "compositing"),
            "software",
            "marker {label}"
        );
        assert_eq!(
            field_bool(&status, "software_rendering"),
            Some(true),
            "marker {label}"
        );
    }
}

#[test]
fn hardware_renderer_classifies_as_hardware() {
    let glx = "OpenGL vendor string: Intel\n\
               OpenGL renderer string: Mesa Intel(R) UHD Graphics 620 (KBL GT2)\n\
               OpenGL core profile version string: 4.6";
    let status = graphics_support_status(
        Some("00:02.0 VGA compatible controller: Intel UHD Graphics"),
        None,
        Some(glx),
    );
    assert_eq!(field_str(&status, "compositing"), "hardware");
    assert_eq!(field_bool(&status, "software_rendering"), Some(false));
    assert_eq!(field_bool(&status, "gpu_detected"), Some(true));
}

#[test]
fn compositing_is_unknown_when_glx_probe_missing_or_empty() {
    for glx in [None, Some("")] {
        let status = graphics_support_status(None, None, glx);
        assert_eq!(field_str(&status, "compositing"), "unknown");
        assert_eq!(field_bool(&status, "software_rendering"), Some(false));
    }
}

#[test]
fn compositing_classification_is_independent_of_gpu_and_vaapi_probes() {
    let gpu = Some("01:00.0 VGA compatible controller: NVIDIA Corporation TU106");
    let vaapi = Some("VAProfileH264Main : VAEntrypointVLD");

    let missing_glx = graphics_support_status(gpu, vaapi, None);
    assert_eq!(field_str(&missing_glx, "compositing"), "unknown");
    assert_eq!(field_bool(&missing_glx, "gpu_detected"), Some(true));
    assert_eq!(field_bool(&missing_glx["vaapi"], "available"), Some(true));

    let empty_glx = graphics_support_status(gpu, vaapi, Some(""));
    assert_eq!(field_str(&empty_glx, "compositing"), "unknown");
    assert_eq!(field_bool(&empty_glx, "gpu_detected"), Some(true));
}

#[test]
fn vaapi_profiles_are_split_into_decode_and_encode() {
    let vainfo = "\
VAProfileH264Main : VAEntrypointVLD
VAProfileH264High : VAEntrypointEncSlice
VAProfileNone : VAEntrypointVideoProc";
    let status = graphics_support_status(None, Some(vainfo), None);
    assert_eq!(
        field_strings(&status["vaapi"], "decode_profiles"),
        vec!["VAProfileH264Main"]
    );
    assert_eq!(
        field_strings(&status["vaapi"], "encode_profiles"),
        vec!["VAProfileH264High"]
    );
    assert_eq!(field_bool(&status["vaapi"], "available"), Some(true));
}

#[test]
fn vaapi_profiles_are_sorted_and_deduplicated() {
    // Deliberately unsorted and with duplicates in both lists.
    let vainfo = "\
VAProfileH264High : VAEntrypointVLD
VAProfileH264Main : VAEntrypointVLD
VAProfileH264High : VAEntrypointVLD
VAProfileHEVCMain : VAEntrypointEncSlice
VAProfileH264Main : VAEntrypointEncSlice
VAProfileHEVCMain : VAEntrypointEncSlice";
    let status = graphics_support_status(None, Some(vainfo), None);
    assert_eq!(
        field_strings(&status["vaapi"], "decode_profiles"),
        vec!["VAProfileH264High", "VAProfileH264Main"]
    );
    assert_eq!(
        field_strings(&status["vaapi"], "encode_profiles"),
        vec!["VAProfileH264Main", "VAProfileHEVCMain"]
    );
}

#[test]
fn vaapi_available_reflects_usable_profiles() {
    let missing = graphics_support_status(None, None, None);
    assert_eq!(field_bool(&missing["vaapi"], "available"), Some(false));
    assert!(field_strings(&missing["vaapi"], "decode_profiles").is_empty());
    assert!(field_strings(&missing["vaapi"], "encode_profiles").is_empty());

    let video_proc_only = graphics_support_status(
        None,
        Some("VAProfileNone : VAEntrypointVideoProc\nnot a profile : VAEntrypointVLD"),
        None,
    );
    assert_eq!(
        field_bool(&video_proc_only["vaapi"], "available"),
        Some(false)
    );
    assert!(field_strings(&video_proc_only["vaapi"], "decode_profiles").is_empty());
    assert!(field_strings(&video_proc_only["vaapi"], "encode_profiles").is_empty());

    let encode_only = graphics_support_status(
        None,
        Some("VAProfileVP9Profile0 : VAEntrypointEncSlice"),
        None,
    );
    assert_eq!(field_bool(&encode_only["vaapi"], "available"), Some(true));
    assert!(field_strings(&encode_only["vaapi"], "decode_profiles").is_empty());
    assert_eq!(
        field_strings(&encode_only["vaapi"], "encode_profiles"),
        vec!["VAProfileVP9Profile0"]
    );
}

#[test]
fn vaapi_parser_ignores_non_profile_lines_and_prefers_vld_over_enc() {
    let vainfo = "\
vainfo: Supported profile and entrypoints
      VAProfileNone                   : VAEntrypointVideoProc
no colon here
VAProfileVP9Profile0 : VAEntrypointVLD VAEntrypointEncSlice
XVAProfileFake : VAEntrypointVLD
VAProfileH264Main : VAEntrypointEncSlice FB";
    let status = graphics_support_status(None, Some(vainfo), None);
    // No-colon line is skipped; "XVAProfileFake" does not start with VAProfile.
    // The mixed entrypoint contains VLD, and the `if VLD ... else if ENC` branch
    // therefore classifies it as decode.
    assert_eq!(
        field_strings(&status["vaapi"], "decode_profiles"),
        vec!["VAProfileVP9Profile0"]
    );
    assert_eq!(
        field_strings(&status["vaapi"], "encode_profiles"),
        vec!["VAProfileH264Main"]
    );
}

#[test]
fn gpu_detected_matches_vga_display_and_3d_lines() {
    let cases = [
        ("VGA", "00:02.0 VGA compatible controller: Intel", true),
        ("Display", "01:00.0 Display controller: AMD", true),
        ("3D", "02:00.0 3D controller: NVIDIA", true),
        ("none", "00:00.0 Host bridge: Intel", false),
    ];
    for (label, gpu, expected) in cases {
        let status = graphics_support_status(Some(gpu), None, None);
        assert_eq!(
            field_bool(&status, "gpu_detected"),
            Some(expected),
            "gpu line {label}"
        );
    }
}
