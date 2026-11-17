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
        server_dir: "/bin/llm/bin".into(),
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
    assert!(!args.contains(&"--cont-batching".to_string()));
    assert!(!args.contains(&"--n-gpu-layers".to_string()));
}

#[test]
fn build_args_includes_gpu_layers_when_explicit() {
    let d = DesiredRuntime {
        model_file: "model.gguf".into(),
        model_path: "/data/llm/model.gguf".into(),
        server_dir: "/bin/llm/bin".into(),
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
        "" | "0" => "8192".to_string(),
        other => other.to_string(),
    };
    assert_eq!(check(""), "8192");
    assert_eq!(check("0"), "8192");
    assert_eq!(check("32768"), "32768");
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
