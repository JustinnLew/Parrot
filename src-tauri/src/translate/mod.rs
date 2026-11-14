use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::{ShellExt, process::CommandEvent};

#[derive(Serialize)]
struct TranslationReq<'a> {
    q: &'a [String],
    source: &'a str,
    target: &'a str,
    format: &'a str,
    alternatives: u16,
}

#[derive(Deserialize)]
struct TranslationRes {
    #[serde(rename = "translatedText")]
    translated_text: Vec<String>,
    alternatives: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TranslationPair {
    source: String,
    translations: Vec<String>,
}

pub fn translation_init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let libre_translataion_sidecar = app.shell().sidecar("lt")?;
    let (mut rx, child) = libre_translataion_sidecar
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
    lines: &[String],
) -> Result<Vec<TranslationPair>, Box<dyn std::error::Error>> {
    let res: TranslationRes = reqwest::Client::new()
        .post("http://127.0.0.1:8888/translate")
        .json(&TranslationReq {
            q: lines,
            source: "en",
            target: "zh",
            format: "text",
            alternatives: 1,
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let pairs = lines
        .iter()
        .zip(res.translated_text)
        .enumerate()
        .map(|(i, (source, best))| {
            let mut translations = vec![best];
            if let Some(alts) = res.alternatives.get(i) {
                translations.extend(alts.iter().cloned());
            }
            TranslationPair {
                source: source.clone(),
                translations,
            }
        })
        .collect();
    println!("{:?}", pairs);
    Ok(pairs)
}
