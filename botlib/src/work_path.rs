pub fn get_work_path() -> String {
    if let Ok(path) = std::env::var("GBO_WORK_PATH") {
        if !path.is_empty() {
            return path;
        }
    }

    let stack_work = "./botserver-stack/data/system/work";
    let stack_root = std::path::Path::new("./botserver-stack");
    let prod_env = std::path::Path::new("/opt/gbo/bin/.env").exists();
    let prod_exe = std::env::current_exe()
        .ok()
        .is_some_and(|p| p.starts_with("/opt/gbo/bin"));

    if stack_root.exists() || !(prod_env || prod_exe) {
        stack_work.to_string()
    } else {
        "/opt/gbo/work".to_string()
    }
}
