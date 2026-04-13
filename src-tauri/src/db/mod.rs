pub mod migrations;
pub mod models;
pub mod queries;

use rusqlite::Connection;
use std::path::Path;

/// Open (or create) the SQLite database and run migrations.
pub fn init(data_dir: &Path) -> rusqlite::Result<Connection> {
    std::fs::create_dir_all(data_dir).expect("failed to create data directory");

    let db_path = data_dir.join("litsync.db");
    let conn = Connection::open(&db_path)?;

    // Performance pragmas for a desktop app
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )?;

    migrations::run(&conn)?;
    Ok(conn)
}
