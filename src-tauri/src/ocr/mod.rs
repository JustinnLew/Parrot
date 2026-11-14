use std::{error::Error, sync::Mutex};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, path::BaseDirectory};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const TEMP_PATH: &str = "snsq_snip.png";

use image::RgbaImage;
use ort::session::{Session, builder::GraphOptimizationLevel};
use serde::Serialize;

use crate::translate::{TranslationPair, translate};

mod det;
mod rec;
pub struct OcrState {
    pub detection: Mutex<Session>,
    pub recognition: Mutex<Session>,
    pub rec_charset: Vec<String>,
}

pub type Pt = (f32, f32);

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextCapture {
    bounding_box: [Pt; 4],
    text: String,
}

fn load_charset(app: &AppHandle, rel: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let path = app.path().resolve(rel, BaseDirectory::Resource)?;
    let text = std::fs::read_to_string(path)?;
    let mut set = vec!["blank".to_string()];
    set.extend(text.lines().map(|l| l.to_string()));
    Ok(set)
}

pub fn ocr_init(app: &AppHandle) -> Result<(), Box<dyn Error>> {
    let env = ort::init().build()?;
    let detection_model_path = app
        .path()
        .resolve("src/ocr/detection/inference.onnx", BaseDirectory::Resource)?;
    let detection_model = Session::builder(&env)?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(4)?
        .commit_from_file(detection_model_path)?;

    let recognition_model_path = app.path().resolve(
        "src/ocr/recognition/inference.onnx",
        BaseDirectory::Resource,
    )?;
    let recognition_model = Session::builder(&env)?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(4)?
        .commit_from_file(recognition_model_path)?;

    let charset = load_charset(app, "src/ocr/recognition/dict.txt")?;

    app.manage(OcrState {
        detection: Mutex::new(detection_model),
        recognition: Mutex::new(recognition_model),
        rec_charset: charset,
    });
    Ok(())
}

pub fn run_ocr(app: &AppHandle, img: &RgbaImage) -> Result<Vec<TextCapture>, Box<dyn Error>> {
    let state = app.state::<OcrState>();
    let boxes = det::run_detection(&state, img)?;
    rec::run_recognition(&state, img, &boxes)
}

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
    let monitor = xcap::Monitor::all()?
        .into_iter()
        .next()
        .ok_or("no monitor")?;
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

// TODOS HANDLE ALL THE ERRORS HERE
fn snip_close(app: &AppHandle) {
    if let Ok(p) = app.path().resolve(TEMP_PATH, BaseDirectory::Temp) {
        let _ = std::fs::remove_file(p);
    }
    if let Some(win) = app.get_webview_window("snip") {
        let _ = win.close();
    }
}

#[tauri::command]
pub async fn snip_process(
    app: AppHandle,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<Vec<TextCapture>, String> {
    let path = app
        .path()
        .resolve(TEMP_PATH, BaseDirectory::Temp)
        .map_err(|e| e.to_string())?;
    let img = image::open(&path).map_err(|e| e.to_string())?.to_rgba8();
    let cropped = image::imageops::crop_imm(&img, x, y, w, h).to_image();
    let text = run_ocr(&app, &cropped).map_err(|_| "Encountered an error while reading text")?;
    println!("{:?}", text);
    Ok(text)
}

#[tauri::command]
pub async fn snip_confirm(
    app: AppHandle,
    lines: Vec<String>,
) -> Result<Vec<TranslationPair>, String> {
    // Process via translataion model
    println!("{:?}", lines);
    snip_close(&app);
    translate(&lines)
        .await
        .map_err(|e| format!("Error: {e}").to_string())
}

#[tauri::command]
pub fn snip_cancel(app: AppHandle) {
    snip_close(&app);
}
