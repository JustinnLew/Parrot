use std::{fs::create_dir_all, sync::Mutex};

use rusqlite::{Connection, params};
use tauri::{AppHandle, Manager};

use crate::translate::TranslationPair;

struct Db(Mutex<Connection>);

pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    create_dir_all(&dir)?;
    let conn = Connection::open(dir.join("parrot.db"))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pairs (
            id              INTEGER PRIMARY KEY,
            source_text     TEXT,
            translations    TEXT NOT NULL,
            source_lang     TEXT NOT NULL,
            target_lang     TEXT NOT NULL,
            created_at      INTEGER NOT NULL,
            note            TEXT
        )",
        [],
    )?;

    app.manage(Db(Mutex::new(conn)));

    Ok(())
}

pub fn add_translation_pairs(app: &AppHandle, pairs: &[TranslationPair]) -> Result<(), String> {
    let db = &app.state::<Db>().0;
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("INSERT INTO pairs VALUES (NULL, ?1, ?2, ?3, ?4, ?5, ?6)")
            .map_err(|e| e.to_string())?;

        for pair in pairs {
            let translations =
                serde_json::to_string(&pair.translations).map_err(|e| e.to_string())?;

            stmt.execute(params![
                pair.source_text,
                translations,
                pair.source_lang,
                pair.target_lang,
                pair.created_at as i64,
                pair.note
            ])
            .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
