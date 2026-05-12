pub fn get_work_path() -> String {
    let stack_work = std::path::Path::new("./botserver-stack/data/system/work");
    let production_work = std::path::Path::new("/opt/gbo/work");
    if stack_work.exists() {
        stack_work
            .to_str()
            .unwrap_or("./botserver-stack/data/system/work")
            .to_string()
    } else if production_work.exists()
        || std::path::Path::new("./.env").exists()
        || std::path::Path::new("/opt/gbo/bin/.env").exists()
    {
        "/opt/gbo/work".to_string()
    } else {
        "./botserver-stack/data/system/work".to_string()
    }
}
