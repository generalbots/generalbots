pub fn get_stack_path() -> String {
    let stack_dir = std::path::Path::new("./botserver-stack");
    let has_env = std::path::Path::new("./.env").exists()
        || std::path::Path::new("/opt/gbo/bin/.env").exists();
    let production_base = std::path::Path::new("/opt/gbo/bin/botserver");
    if stack_dir.exists() {
        "./botserver-stack".to_string()
    } else if has_env || production_base.exists() {
        "/opt/gbo".to_string()
    } else {
        "./botserver-stack".to_string()
    }
}
