// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod ocr;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app: &mut tauri::App| {
            use crate::ocr::ocr_init;

            ocr_init(app.handle())?;

            #[cfg(desktop)]
            ocr::register_hotkeys(app.handle())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            ocr::snip_cancel,
            ocr::snip_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
