mod app_version;
mod config;
mod llm;
mod proxy;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let version = app_version::get_app_version();
            let version = version.trim_start_matches('v');
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_title(&format!("偶译析 · 翻译解析助手 v{version}"));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::save_config,
            llm::chat_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
