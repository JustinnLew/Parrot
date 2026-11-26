use rusqlite::{params};
use serde::Serialize;
use tauri::State;

use crate::sqlite::Db;

#[derive(Serialize)]
pub struct DeckSummary {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub pair_count: i64,
}

#[derive(Serialize)]
pub struct DeckPair {
    pub pair_id: i64,
    pub source_text: Option<String>,
    pub translations: String, // stored text, parse it where you need it
    pub source_lang: String,
    pub target_lang: String,
    pub note: Option<String>,
    pub reversed: bool,
    pub position: i64,
}

const PAGE_SIZE: i64 = 50;

#[tauri::command]
pub async fn deck_create(
    db: State<'_, Db>,
    title: String,
    description: Option<String>,
) -> Result<i64, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Deck title can't be empty".into());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO decks (title, description, created_at) VALUES (?1, ?2, unixepoch())",
        params![title, description],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn deck_delete(db: State<'_, Db>, deck_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    // cascades to deck_pairs; the pairs themselves (and their progress) stay
    conn.execute("DELETE FROM decks WHERE id = ?1", params![deck_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn deck_add_pair(
    db: State<'_, Db>,
    deck_id: i64,
    pair_id: i64,
    reversed: bool,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO deck_pairs (deck_id, pair_id, reversed, position)
         VALUES (?1, ?2, ?3,
                 (SELECT COALESCE(MAX(position) + 1, 0) FROM deck_pairs WHERE deck_id = ?1))
         ON CONFLICT (deck_id, pair_id) DO NOTHING",
        params![deck_id, pair_id, reversed as i64],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn deck_delete_pair(
    db: State<'_, Db>,
    deck_id: i64,
    pair_id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    // removes the card from this deck only, never the card itself
    conn.execute(
        "DELETE FROM deck_pairs WHERE deck_id = ?1 AND pair_id = ?2",
        params![deck_id, pair_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// `pair_ids`: full deck in its new order.
#[tauri::command]
pub async fn deck_rearrange_pairs(
    db: State<'_, Db>,
    deck_id: i64,
    pair_ids: Vec<i64>,
) -> Result<(), String> {
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare(
                "UPDATE deck_pairs SET position = ?1 WHERE deck_id = ?2 AND pair_id = ?3",
            )
            .map_err(|e| e.to_string())?;
        for (i, pair_id) in pair_ids.iter().enumerate() {
            let changed = stmt
                .execute(params![i as i64, deck_id, pair_id])
                .map_err(|e| e.to_string())?;
            if changed == 0 {
                // returning here drops the transaction, which rolls it back
                return Err(format!("Card {pair_id} isn't in this deck"));
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn deck_edit_description(
    db: State<'_, Db>,
    deck_id: i64,
    description: String,
) -> Result<(), String> {
    let description = description.trim();
    let description = if description.is_empty() { None } else { Some(description) };
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let changed = conn
        .execute(
            "UPDATE decks SET description = ?1 WHERE id = ?2",
            params![description, deck_id],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Deck not found".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn deck_list(db: State<'_, Db>, cursor: Option<i64>) -> Result<Vec<DeckSummary>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.title, d.description, d.created_at,
                    (SELECT COUNT(*) FROM deck_pairs dp WHERE dp.deck_id = d.id)
             FROM decks d
             WHERE (?1 IS NULL OR d.id < ?1)
             ORDER BY d.id DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let decks = stmt
        .query_map(params![cursor, PAGE_SIZE], |r| {
            Ok(DeckSummary {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                created_at: r.get(3)?,
                pair_count: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(decks)
}

#[tauri::command]
pub async fn deck_pairs_list(db: State<'_, Db>, deck_id: i64) -> Result<Vec<DeckPair>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.source_text, p.translations, p.source_lang, p.target_lang,
                    p.note, dp.reversed, dp.position
             FROM deck_pairs dp
             JOIN pairs p ON p.id = dp.pair_id
             WHERE dp.deck_id = ?1
             ORDER BY dp.position, p.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![deck_id], |r| {
            Ok(DeckPair {
                pair_id: r.get(0)?,
                source_text: r.get(1)?,
                translations: r.get(2)?,
                source_lang: r.get(3)?,
                target_lang: r.get(4)?,
                note: r.get(5)?,
                reversed: r.get(6)?,
                position: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}