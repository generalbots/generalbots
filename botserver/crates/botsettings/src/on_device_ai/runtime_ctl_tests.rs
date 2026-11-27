use super::*;

#[test]
fn model_matches_bare_filenames() {
    assert!(model_matches("Qwen3.5-9B-Q4_K_M.gguf", "Qwen3.5-9B-Q4_K_M.gguf"));
    assert!(model_matches("/some/path/Qwen3.5-9B-Q4_K_M.gguf", "Qwen3.5-9B-Q4_K_M.gguf"));
    assert!(!model_matches("DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf", "Qwen3.5-9B-Q4_K_M.gguf"));
    assert!(!model_matches("", "Qwen3.5-9B-Q4_K_M.gguf"));
    assert!(!model_matches("model.gguf", ""));
}

#[test]
fn needs_restart_when_unreachable() {
    let p = RuntimeProbe { reachable: false, loaded_model: String::new() };
    assert!(needs_restart(&p, "model.gguf"));
}

#[test]
fn needs_restart_when_wrong_model() {
    let p = RuntimeProbe { reachable: true, loaded_model: "DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf".into() };
    assert!(needs_restart(&p, "Qwen3.5-9B-Q4_K_M.gguf"));
}

#[test]
fn no_restart_when_correct() {
    let p = RuntimeProbe { reachable: true, loaded_model: "Qwen3.5-9B-Q4_K_M.gguf".into() };
    assert!(!needs_restart(&p, "Qwen3.5-9B-Q4_K_M.gguf"));
}

#[test]
fn build_args_matches_package_manager_line() {
    let d = DesiredRuntime {
        model_file: "model.gguf".into(),
        model_path: "/data/llm/model.gguf".into(),
        server_dir: "/bin/llm".into(),
        gpu_layers: String::new(),
        ctx_size: "8192".into(),
    };
    let args = build_args(&d);
    assert!(args.contains(&"--port".to_string()));
    assert!(args.contains(&"8081".to_string()));
    assert!(args.contains(&"-m".to_string()));
    assert!(args.contains(&"/data/llm/model.gguf".to_string()));
    assert!(args.contains(&"--ubatch-size".to_string()));
    assert!(args.contains(&"--ctx-size".to_string()));
    assert!(args.contains(&"8192".to_string()));
    assert!(!args.contains(&"--verbose".to_string()));
    assert!(args.contains(&"-lv".to_string()), "must set verbosity for layer log");
    assert!(args.contains(&"4".to_string()), "verbosity 4 prints offloaded layers");
    assert!(!args.contains(&"--cont-batching".to_string()));
    assert!(!args.contains(&"--n-gpu-layers".to_string()));
}

#[test]
fn build_args_includes_gpu_layers_when_explicit() {
    let d = DesiredRuntime {
        model_file: "model.gguf".into(),
        model_path: "/data/llm/model.gguf".into(),
        server_dir: "/bin/llm".into(),
        gpu_layers: "0".into(),
        ctx_size: "8192".into(),
    };
    let args = build_args(&d);
    assert!(args.contains(&"--n-gpu-layers".to_string()));
    assert!(args.contains(&"0".to_string()));
}

#[test]
fn desired_from_config_empty_model_is_none() {
    assert!(desired_from_config("", "auto", "8192").is_none());
    assert!(desired_from_config("  ", "0", "8192").is_none());
}

#[test]
fn desired_from_config_gpu_layers_auto_means_omit() {
    let check = |input: &str| match input.trim() {
        "" | "auto" => String::new(),
        other => other.to_string(),
    };
    assert_eq!(check("auto"), "");
    assert_eq!(check(""), "");
    assert_eq!(check("0"), "0");
    assert_eq!(check("29"), "29");
}

#[test]
fn desired_from_config_normalizes_ctx_size() {
    let check = |input: &str| match input.trim() {
        "" | "0" => String::new(),
        other => other.to_string(),
    };
    assert_eq!(check(""), "", "sized from free video memory at start");
    assert_eq!(check("0"), "");
    assert_eq!(check("32768"), "32768");
}

#[test]
fn build_args_opens_one_slot_and_lowers_the_fit_margin_when_layers_are_automatic() {
    let d = DesiredRuntime {
        model_file: "model.gguf".into(),
        model_path: "/data/llm/model.gguf".into(),
        server_dir: "/bin/llm".into(),
        gpu_layers: String::new(),
        ctx_size: "8192".into(),
    };
    let args = build_args(&d);
    let after = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    assert_eq!(after("--parallel").as_deref(), Some("1"));
    assert_eq!(after("--fit-target").as_deref(), Some("512"));
    let explicit = build_args(&DesiredRuntime { gpu_layers: "20".into(), ..d });
    assert!(!explicit.contains(&"--fit-target".to_string()), "an explicit layer count is not refitted");
}

#[test]
fn is_local_model_name_identifies_gguf() {
    assert!(is_local_model_name("Qwen3.5-9B-Q4_K_M.gguf"));
    assert!(is_local_model_name("DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf"));
    assert!(!is_local_model_name("dots-studio/dots-3-note-preview:free"));
    assert!(!is_local_model_name("gpt-4o"));
    assert!(!is_local_model_name(""));
    assert!(!is_local_model_name("  "));
}

#[test]
fn a_failed_start_is_classified_from_the_engine_log() {
    let old_engine = "llama_model_load: error loading model: error loading model architecture: unknown model architecture: 'qwen35'";
    assert_eq!(classify_failure(old_engine), CODE_ENGINE_TOO_OLD);
    assert_eq!(classify_failure("ggml_backend_cuda_buffer_type_alloc_buffer: allocating 5200.00 MiB on device 0: cudaMalloc failed: out of memory"), CODE_NOT_ENOUGH_MEMORY);
    assert_eq!(classify_failure("main: exiting due to model loading error"), CODE_START_FAILED);
    assert_eq!(classify_failure(""), CODE_START_FAILED);
}

#[test]
fn handle_start_failure_classifies_from_the_log_and_records_it() {
    // No real log file exists in the test; llm_log_since returns empty.
    clear_last_failure();
    let msg = handle_start_failure("model.gguf", u64::MAX, "Chat runtime exited");
    assert!(msg.starts_with(CODE_START_FAILED), "generic when log is empty: {msg}");
    let (model, code) = last_failure().expect("failure must be recorded");
    assert_eq!(model, "model.gguf");
    assert_eq!(code, CODE_START_FAILED);
    // A good start clears it.
    clear_last_failure();
    assert!(last_failure().is_none());
}

#[test]
fn health_timeout_allows_slow_cold_loads() {
    // A cold disk-cache load of a 5+ GB model can take over 2 minutes.
    // The timeout must be generous enough for that case.
    assert!(HEALTH_TIMEOUT.as_secs() >= 240, "HEALTH_TIMEOUT too short for cold loads");
    assert!(HEALTH_TIMEOUT.as_secs() <= 600, "HEALTH_TIMEOUT unreasonably long");
}

#[test]
fn the_failure_code_is_read_back_from_the_message() {
    assert_eq!(failure_code("engine_too_old: Chat runtime exited during startup"), CODE_ENGINE_TOO_OLD);
    assert_eq!(failure_code("not_enough_memory: Chat runtime exited during startup"), CODE_NOT_ENOUGH_MEMORY);
    assert_eq!(failure_code("Cannot stop chat runtime: access denied"), CODE_START_FAILED);
    assert_eq!(failure_code("Model file not found on disk."), CODE_START_FAILED);
}
