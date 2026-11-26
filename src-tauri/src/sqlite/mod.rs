use std::{fs::create_dir_all, sync::Mutex};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

pub mod pairs;
pub mod decks;
pub struct Db(Mutex<Connection>);

pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    create_dir_all(&dir)?;
    let conn = Connection::open(dir.join("parrot.db"))?;
    conn.execute_batch(
        "
        BEGIN;

        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS pairs (
            id              INTEGER PRIMARY KEY,
            source_text     TEXT,
            translations    TEXT NOT NULL,
            source_lang     TEXT NOT NULL,
            target_lang     TEXT NOT NULL,
            created_at      INTEGER NOT NULL,
            note            TEXT
        );

        CREATE TABLE IF NOT EXISTS decks (
            id              INTEGER PRIMARY KEY,
            title           TEXT NOT NULL,
            description     TEXT,
            created_at      INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS deck_pairs (
            deck_id         INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
            pair_id         INTEGER NOT NULL REFERENCES pairs(id) ON DELETE CASCADE,
            reversed        INTEGER NOT NULL DEFAULT 0,
            position        INTEGER NOT NULL,
            PRIMARY KEY (deck_id, pair_id)
        );
        CREATE INDEX IF NOT EXISTS idx_deck_pairs_pair ON deck_pairs(pair_id);
        COMMIT;"
    )?;

    app.manage(Db(Mutex::new(conn)));

    Ok(())
}
