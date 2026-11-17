use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::{ShellExt, process::CommandEvent};

use crate::sqlite::pairs::add_translation_pairs;

#[derive(Serialize)]
struct TranslationReq<'a> {
    q: &'a [&'a str],
    source: &'a str,
    target: &'a str,
    format: &'a str,
    alternatives: u16,
}

#[derive(Deserialize)]
struct TranslationRes {
    #[serde(rename = "translatedText")]
    translated_text: Vec<String>,
    #[serde(default)]
    alternatives: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslationPair {
    pub source_text: String,
    pub translations: Vec<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: u64,
    pub note: Option<String>,
}

pub fn translation_init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let libre_translation_sidecar = app.shell().sidecar("lt")?;
    let (mut rx, child) = libre_translation_sidecar
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to spawn libretranslate");
    app.manage(child);

    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Terminated(p) => {
                    eprintln!("[lt] exited: {:?}", p);
                    break;
                }
                _ => {}
            }
        }
    });
    Ok(())
}

pub async fn translate(
    app: &AppHandle,
    lines: Vec<&str>,
    source_lang: &str,
    target_lang: &str,
) -> Result<Vec<TranslationPair>, Box<dyn std::error::Error>> {
    let res: TranslationRes = reqwest::Client::new()
        .post("http://127.0.0.1:8888/translate")
        .json(&TranslationReq {
            q: &lines,
            source: source_lang,
            target: target_lang,
            format: "text",
            alternatives: 1,
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let TranslationRes {
        translated_text,
        alternatives,
    } = res;

    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let pairs: Vec<TranslationPair> = lines
        .iter()
        .zip(translated_text)
        .enumerate()
        .map(|(i, (source, best))| {
            let mut translations = vec![best];
            if let Some(alts) = alternatives.get(i) {
                translations.extend(alts.iter().cloned());
            }
            TranslationPair {
                source_text: source.to_string(),
                translations,
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                created_at,
                note: None,
            }
        })
        .collect();

    add_translation_pairs(app, &pairs)?;
    Ok(pairs)
}
