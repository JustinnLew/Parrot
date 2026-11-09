// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::sync::Mutex;

use xcap::image::RgbaImage;

mod capture;

#[derive(Default)]
struct AppState {
}

#[cfg(desktop)]

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::default())
        .setup(|app: &mut tauri::App| {
            #[cfg(desktop)]
            capture::register_hotkeys(app.handle())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            capture::cancel_snip,
            capture::finish_snip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
