use rusqlite::{Row, params, types::Type};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{sqlite::Db, translate::TranslationPair};

const PAGE_SIZE: i64 = 5;

#[derive(Serialize, Clone, Debug)]
pub struct StoredPair {
    pub id: i64,
    #[serde(flatten)]
    pub pair: TranslationPair,
}

pub fn add_translation_pairs(app: &AppHandle, pairs: &[TranslationPair]) -> Result<(), String> {
    let db = &&app.state::<Db>().0;
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let mut stored = Vec::with_capacity(pairs.len());

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

            stored.push(StoredPair {
                id: tx.last_insert_rowid(),
                pair: pair.clone(),
            });
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    println!("Stored: {:?}", stored);

    if !stored.is_empty() {
        app.emit("translations-added", &stored)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_translation_pairs(db: State<'_, Db>, pair_ids: Vec<i64>) -> Result<(), String> {
    if pair_ids.is_empty() {
        return Ok(())
    }
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    {
        let mut stmt = tx
            .prepare_cached("DELETE FROM pairs WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        for id in &pair_ids {
            stmt.execute(params![id]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

fn row_to_stored_pair(row: &Row) -> rusqlite::Result<StoredPair> {
    let translations_json: String = row.get(2)?;
    let translations = serde_json::from_str(&translations_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(e)))?;

    Ok(StoredPair {
        id: row.get(0)?,
        pair: TranslationPair {
            source_text: row.get(1)?,
            translations,
            source_lang: row.get(3)?,
            target_lang: row.get(4)?,
            created_at: row.get::<_, i64>(5)? as u64,
            note: row.get(6)?,
        },
    })
}

#[tauri::command]
pub async fn get_translation_pairs(
    db: State<'_, Db>,
    cursor: Option<i64>,
) -> Result<Vec<StoredPair>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, source_text, translations, source_lang,
                    target_lang, created_at, note
             FROM pairs
             WHERE (?1 IS NULL OR id < ?1)
             ORDER BY id DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let pairs = stmt
        .query_map(params![cursor, PAGE_SIZE], row_to_stored_pair)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(pairs)
}
