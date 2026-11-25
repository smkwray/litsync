use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::models::Progress;
use crate::sabp;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SaveProgressInput {
    pub book_id: i64,
    pub position_ms: i64,
    pub chapter_index: i64,
    pub playback_speed: f64,
    pub status: String,
}

#[tauri::command]
pub fn save_progress(input: SaveProgressInput, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO progress (book_id, position_ms, chapter_index, playback_speed, status, last_played_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
         ON CONFLICT(book_id) DO UPDATE SET
            position_ms = excluded.position_ms,
            chapter_index = excluded.chapter_index,
            playback_speed = excluded.playback_speed,
            status = excluded.status,
            last_played_at = datetime('now'),
            updated_at = datetime('now')",
        params![
            input.book_id,
            input.position_ms,
            input.chapter_index,
            input.playback_speed,
            input.status,
        ],
    )
    .map_err(|e| e.to_string())?;

    // Mirror to SABP statistics.xml if the book's library root uses SABP
    if let Ok(Some((folder_path, library_root))) = get_book_paths(&db, input.book_id) {
        if let Some(folder_name) = sabp::book_folder_name(&folder_path, &library_root) {
            let position_secs = (input.position_ms / 1000) as u64;
            let progress = sabp::SabpBookProgress {
                path: folder_name,
                month: sabp::current_month(),
                position_secs,
            };
            if let Err(e) = sabp::write_statistics(std::path::Path::new(&library_root), &[progress])
            {
                log::warn!("Failed to write SABP statistics: {e}");
            }
        }
    }

    Ok(())
}

fn get_book_paths(
    db: &rusqlite::Connection,
    book_id: i64,
) -> rusqlite::Result<Option<(String, String)>> {
    let folder_path: String = db.query_row(
        "SELECT folder_path FROM books WHERE id = ?1",
        params![book_id],
        |row| row.get(0),
    )?;

    // Find which library root this book belongs to
    let mut stmt = db.prepare("SELECT path FROM library_roots")?;
    let roots: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    for root in roots {
        if folder_path.starts_with(&root) {
            return Ok(Some((folder_path, root)));
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn get_progress(book_id: i64, state: State<AppState>) -> Result<Option<Progress>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.query_row(
        "SELECT book_id, position_ms, chapter_index, playback_speed, status, last_played_at, updated_at
         FROM progress WHERE book_id = ?1",
        params![book_id],
        |row| {
            Ok(Progress {
                book_id: row.get(0)?,
                position_ms: row.get(1)?,
                chapter_index: row.get(2)?,
                playback_speed: row.get(3)?,
                status: row.get(4)?,
                last_played_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    );
    match result {
        Ok(p) => Ok(Some(p)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Serialize)]
pub struct AllProgress {
    pub book_id: i64,
    pub position_ms: i64,
    pub total_duration_ms: i64,
    pub status: String,
    pub playback_speed: f64,
    pub last_played_at: Option<String>,
}

#[tauri::command]
pub fn get_all_progress(state: State<AppState>) -> Result<Vec<AllProgress>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT p.book_id, p.position_ms, b.total_duration_ms, p.status, p.playback_speed, p.last_played_at
         FROM progress p JOIN books b ON p.book_id = b.id"
    ).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AllProgress {
                book_id: row.get(0)?,
                position_ms: row.get(1)?,
                total_duration_ms: row.get(2)?,
                status: row.get(3)?,
                playback_speed: row.get(4)?,
                last_played_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
