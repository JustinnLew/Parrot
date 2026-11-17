use std::{fs::create_dir_all, sync::Mutex};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

pub mod pairs;
pub struct Db(Mutex<Connection>);

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
