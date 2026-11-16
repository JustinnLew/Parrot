use crate::translate::translation_init;
use crate::{ocr::ocr_init, translate::translate};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod ocr;
mod sqlite;
mod translate;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app: &mut tauri::App| {
            sqlite::init(app.handle())?;
            ocr_init(app.handle())?;
            translation_init(app.handle())?;

            #[cfg(desktop)]
            register_hotkeys(app.handle())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            ocr::snip_cancel,
            ocr::snip_process,
            ocr::snip_confirm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn do_capture(app: &AppHandle) {
    match app.clipboard().read_text() {
        Ok(text) => {
            println!("{text}");
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                match translate(&app, vec![text.as_str()], "en", "zh").await {
                    Ok(pairs) => println!("{pairs:?}"),
                    Err(e) => eprintln!("translate failed: {e}"),
                }
            });
        }
        Err(e) => eprintln!("Failed to read clipboard: {e}"),
    }
}

fn register_hotkeys(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let mods = Modifiers::CONTROL | Modifiers::ALT;
    let capture_key = Shortcut::new(Some(mods), Code::KeyC);
    let snip_key = Shortcut::new(Some(mods), Code::KeyS);

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Released {
                    return;
                }

                if shortcut == &capture_key {
                    do_capture(app);
                } else if shortcut == &snip_key {
                    if let Err(e) = ocr::do_snip(app) {
                        eprintln!("snip failed: {e}");
                    }
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(capture_key)?;
    app.global_shortcut().register(snip_key)?;
    Ok(())
}
