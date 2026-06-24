#[cfg(target_os = "android")]
fn init_logger() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("BotOS"),
    );
}

#[cfg(not(target_os = "android"))]
fn init_logger() {
    env_logger::init();
}

#[tauri::command]
fn get_device_info() -> serde_json::Value {
    serde_json::json!({
        "platform": "android",
        "app": "BotOS",
        "version": env!("CARGO_PKG_VERSION")
    })
}

#[tauri::command]
async fn send_to_bot(message: String, server_url: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{server_url}/api/messages"))
        .json(&serde_json::json!({ "text": message }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.text().await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logger();
    log::info!("BotOS starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_geolocation::init())
        .invoke_handler(tauri::generate_handler![get_device_info, send_to_bot])
        .setup(|_app| {
            log::info!("BotOS initialized, loading botui...");

            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(window) = _app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| log::error!("BotOS failed to start: {e}"));
}
