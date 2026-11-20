use std::sync::Mutex;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_store::StoreExt;

#[derive(Serialize, Clone, Debug)]
pub struct AppSettings {
    pub source_lang: String,
    pub target_lang: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
        }
    }
}

pub fn settings_init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = AppSettings::default();
    let settings_file = app.store("settings.json")?;
    match settings_file.get("source_lang") {
        Some(v) => {
            settings.source_lang = v
                .as_str()
                .ok_or("Invalid source_lang setting, expected a JSON string")?
                .to_string();
        }
        None => settings_file.set("source_lang", json!("en")),
    }
    match settings_file.get("target_lang") {
        Some(v) => {
            settings.target_lang = v
                .as_str()
                .ok_or("Invalid target_lang setting, expected a JSON string")?
                .to_string();
        }
        None => settings_file.set("target_lang", json!("zh")),
    }
    println!("{:?}", settings.clone());
    app.manage(Mutex::new(settings));
    Ok(())
}

fn apply_setting(
    app: &AppHandle,
    state: &Mutex<AppSettings>,
    key: &str,
    value: String,
    update: impl FnOnce(&mut AppSettings, String),
) -> Result<AppSettings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set(key, json!(value));

    let mut s = state.lock().unwrap();
    update(&mut s, value);
    Ok(s.clone())
}

#[tauri::command]
pub async fn settings_get(settings: State<'_, Mutex<AppSettings>>) -> Result<AppSettings, String> {
    Ok(settings.lock().unwrap().clone())
}

#[tauri::command]
pub async fn settings_set_source_lang(
    app: AppHandle,
    settings: State<'_, Mutex<AppSettings>>,
    source_lang: String,
) -> Result<AppSettings, String> {
    apply_setting(&app, &settings, "source_lang", source_lang, |s, v| {
        s.source_lang = v
    })
}

#[tauri::command]
pub async fn settings_set_target_lang(
    app: AppHandle,
    settings: State<'_, Mutex<AppSettings>>,
    target_lang: String,
) -> Result<AppSettings, String> {
    apply_setting(&app, &settings, "target_lang", target_lang, |s, v| {
        s.target_lang = v
    })
}
