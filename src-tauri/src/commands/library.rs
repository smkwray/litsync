use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::db::models::{Book, Chapter};
use crate::db::queries;
use crate::scanner;
use crate::scanner::metadata;
use crate::state::AppState;
use std::path::Path;

#[derive(Serialize)]
pub struct ScanResult {
    pub books_found: usize,
    pub books_added: usize,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
pub struct BookWithChapters {
    #[serde(flatten)]
    pub book: Book,
    pub chapters: Vec<Chapter>,
}

#[tauri::command]
pub fn scan_library(
    root_path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<ScanResult, String> {
    let root = Path::new(&root_path);
    if !root.is_dir() {
        return Err(format!("Not a valid directory: {root_path}"));
    }

    // Expand asset protocol scope to include the library root
    let scope = app.asset_protocol_scope();
    scope.allow_directory(root, true).unwrap_or_else(|e| {
        log::warn!("Failed to add library root to asset scope: {e}");
    });

    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Upsert the library root
    queries::upsert_library_root(&db, &root_path).map_err(|e| e.to_string())?;

    // Scan the filesystem
    let scanned_books = scanner::scan_library_root(root);
    let books_found = scanned_books.len();
    let mut books_added = 0;
    let mut errors = Vec::new();

    for scanned in &scanned_books {
        // Skip if we already have this book (by folder path)
        let exists: bool = db
            .query_row(
                "SELECT COUNT(*) > 0 FROM books WHERE folder_path = ?1",
                params![scanned.folder_path],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if exists {
            continue;
        }

        match insert_book(&db, scanned, &state.data_dir) {
            Ok(_) => books_added += 1,
            Err(e) => errors.push(format!("{}: {e}", scanned.folder_path)),
        }
    }

    // Import progress from Smart AudioBook Player
    match crate::sabp::read_statistics(root) {
        Ok(sabp_entries) if !sabp_entries.is_empty() => {
            log::info!("Found {} SABP progress entries", sabp_entries.len());
            import_sabp_progress(&db, &sabp_entries, &root_path);
        }
        Ok(_) => {}
        Err(e) => log::warn!("Failed to read SABP statistics: {e}"),
    }

    // Update last_scanned_at
    let _ = db.execute(
        "UPDATE library_roots SET last_scanned_at = datetime('now') WHERE path = ?1",
        params![root_path],
    );

    Ok(ScanResult {
        books_found,
        books_added,
        errors,
    })
}

fn insert_book(
    db: &rusqlite::Connection,
    scanned: &scanner::ScannedBook,
    data_dir: &Path,
) -> Result<(), String> {
    let (book_meta, file_infos) =
        metadata::build_book_metadata(&scanned.audio_files, &scanned.folder_path);

    // Extract and save cover art
    let cover_path =
        if let Some((cover_data, ext)) = metadata::extract_cover_art(&scanned.audio_files) {
            let covers_dir = data_dir.join("covers");
            std::fs::create_dir_all(&covers_dir).map_err(|e| e.to_string())?;

            // Use a hash of the folder path for a stable filename
            let hash = simple_hash(&scanned.folder_path);
            let cover_filename = format!("{hash}.{ext}");
            let cover_file = covers_dir.join(&cover_filename);
            std::fs::write(&cover_file, &cover_data).map_err(|e| e.to_string())?;
            Some(cover_file.to_string_lossy().into_owned())
        } else {
            None
        };

    // Insert the book
    db.execute(
        "INSERT INTO books (title, author, narrator, total_duration_ms, cover_path, folder_path, file_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            book_meta.title,
            book_meta.author,
            book_meta.narrator,
            book_meta.total_duration_ms as i64,
            cover_path,
            scanned.folder_path,
            scanned.audio_files.len() as i64,
        ],
    )
    .map_err(|e| e.to_string())?;

    let book_id = db.last_insert_rowid();

    // Insert chapters (one per file for multi-file books)
    let mut cumulative_ms: i64 = 0;
    for (idx, info) in file_infos.iter().enumerate() {
        let start_ms = cumulative_ms;
        let end_ms = cumulative_ms + info.duration_ms as i64;

        db.execute(
            "INSERT INTO chapters (book_id, title, start_ms, end_ms, file_path, file_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                book_id,
                info.title,
                start_ms,
                end_ms,
                info.path.to_string_lossy().as_ref(),
                idx as i64,
            ],
        )
        .map_err(|e| e.to_string())?;

        cumulative_ms = end_ms;
    }

    log::info!(
        "Added book: {} ({} files, {}ms)",
        book_meta.title,
        scanned.audio_files.len(),
        book_meta.total_duration_ms
    );

    Ok(())
}

/// Import SABP progress entries into our progress table.
/// Matches by folder name: SABP stores "Folder Name", we match against
/// the first path component after the library root.
fn import_sabp_progress(
    db: &rusqlite::Connection,
    sabp_entries: &[crate::sabp::SabpBookProgress],
    library_root: &str,
) {
    // Build a map of folder_name -> book_id for all books under this root
    let mut stmt = match db.prepare("SELECT id, folder_path FROM books WHERE folder_path LIKE ?1") {
        Ok(s) => s,
        Err(e) => {
            log::warn!("SABP import query failed: {e}");
            return;
        }
    };
    let prefix = format!("{library_root}%");
    let rows = match stmt.query_map(params![prefix], |row| {
        let id: i64 = row.get(0)?;
        let folder_path: String = row.get(1)?;
        Ok((folder_path, id))
    }) {
        Ok(rows) => rows,
        Err(e) => {
            log::warn!("SABP import row query failed: {e}");
            return;
        }
    };
    let book_map: std::collections::HashMap<String, i64> = rows
        .filter_map(|r| r.ok())
        .filter_map(|(fp, id)| {
            crate::sabp::book_folder_name(&fp, library_root).map(|name| (name, id))
        })
        .collect();

    for entry in sabp_entries {
        if let Some(&book_id) = book_map.get(&entry.path) {
            let position_ms = (entry.position_secs * 1000) as i64;

            let existing_ms = db
                .query_row(
                    "SELECT position_ms FROM progress WHERE book_id = ?1",
                    params![book_id],
                    |row| row.get(0),
                )
                .ok();

            if existing_ms.is_none() || position_ms > existing_ms.unwrap_or(0) {
                let status = if position_ms > 0 {
                    "listening"
                } else {
                    "not_started"
                };
                let _ = db.execute(
                    "INSERT INTO progress (book_id, position_ms, chapter_index, playback_speed, status, updated_at)
                     VALUES (?1, ?2, 0, 1.0, ?3, datetime('now'))
                     ON CONFLICT(book_id) DO UPDATE SET
                        position_ms = excluded.position_ms,
                        status = excluded.status,
                        updated_at = datetime('now')",
                    params![book_id, position_ms, status],
                );
                log::info!(
                    "Imported SABP progress for '{}': {}s",
                    entry.path,
                    entry.position_secs
                );
            }
        }
    }
}

/// Simple string hash for cover art filenames.
fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[tauri::command]
pub fn get_books(state: State<AppState>) -> Result<Vec<Book>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_books(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_book(id: i64, state: State<AppState>) -> Result<BookWithChapters, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let book = queries::get_book(&db, id).map_err(|e| e.to_string())?;
    let chapters = queries::get_chapters(&db, id).map_err(|e| e.to_string())?;
    Ok(BookWithChapters { book, chapters })
}

#[tauri::command]
pub fn get_library_roots(
    state: State<AppState>,
) -> Result<Vec<crate::db::models::LibraryRoot>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_library_roots(&db).map_err(|e| e.to_string())
}
