use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, path::BaseDirectory};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

pub fn register_hotkeys(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
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
                    if let Err(e) = do_snip(app) {
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

fn do_capture(app: &AppHandle) {
    match app.clipboard().read_text() {
        Ok(text) => println!("{text}"),
        Err(e) => eprintln!("Failed to read clipboard: {e}"),
    }
}

fn do_snip(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.get_webview_window("snip").is_some() {
        return Ok(());
    }
    let monitor = xcap::Monitor::all()?.into_iter().next().ok_or("no monitor")?;
    let img = monitor.capture_image()?;
    let path = app.path().resolve("snsq_snip.png", BaseDirectory::Temp)?;
    img.save(&path)?;

    WebviewWindowBuilder::new(app, "snip", WebviewUrl::App("index.html#/snip".into()))
        .title("snsq Snip")
        .fullscreen(true)
        .decorations(false)
        .always_on_top(true)
        .build()?;
    Ok(())
}

#[tauri::command]
pub fn finish_snip(app: tauri::AppHandle, x: u32, y: u32, w: u32, h: u32) -> Result<(), String> {
    let path = app.path().resolve("snsq_snip.png", BaseDirectory::Temp).map_err(|e| e.to_string())?;
    let img = image::open(&path).map_err(|e| e.to_string())?.to_rgba8();
    let cropped = image::imageops::crop_imm(&img, x, y, w, h).to_image();
    // TODO: OCR `cropped`
    let _ = std::fs::remove_file(&path);
    if let Some(win) = app.get_webview_window("snip") { let _ = win.close(); }
    Ok(())
}

#[tauri::command]
pub fn cancel_snip(app: tauri::AppHandle) {
    if let Ok(p) = app.path().resolve("snsq_snip.png", BaseDirectory::Temp) {
        let _ = std::fs::remove_file(p);
    }
    if let Some(win) = app.get_webview_window("snip") { let _ = win.close(); }
}