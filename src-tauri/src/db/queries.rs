use rusqlite::{params, Connection};

use super::models::{Book, Chapter, LibraryRoot};

// -- Library roots --

pub fn get_library_roots(conn: &Connection) -> rusqlite::Result<Vec<LibraryRoot>> {
    let mut stmt = conn.prepare("SELECT id, path, last_scanned_at FROM library_roots")?;
    let rows = stmt.query_map([], |row| {
        Ok(LibraryRoot {
            id: row.get(0)?,
            path: row.get(1)?,
            last_scanned_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn upsert_library_root(conn: &Connection, path: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO library_roots (path) VALUES (?1)
         ON CONFLICT(path) DO UPDATE SET path = path",
        params![path],
    )?;
    Ok(conn.last_insert_rowid())
}

// -- Books --

pub fn get_books(conn: &Connection) -> rusqlite::Result<Vec<Book>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, author, narrator, description, total_duration_ms,
                cover_path, folder_path, file_count, added_at, updated_at
         FROM books ORDER BY title",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Book {
            id: row.get(0)?,
            title: row.get(1)?,
            author: row.get(2)?,
            narrator: row.get(3)?,
            description: row.get(4)?,
            total_duration_ms: row.get(5)?,
            cover_path: row.get(6)?,
            folder_path: row.get(7)?,
            file_count: row.get(8)?,
            added_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    rows.collect()
}

pub fn get_book(conn: &Connection, id: i64) -> rusqlite::Result<Book> {
    conn.query_row(
        "SELECT id, title, author, narrator, description, total_duration_ms,
                cover_path, folder_path, file_count, added_at, updated_at
         FROM books WHERE id = ?1",
        params![id],
        |row| {
            Ok(Book {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                narrator: row.get(3)?,
                description: row.get(4)?,
                total_duration_ms: row.get(5)?,
                cover_path: row.get(6)?,
                folder_path: row.get(7)?,
                file_count: row.get(8)?,
                added_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
}

pub fn get_chapters(conn: &Connection, book_id: i64) -> rusqlite::Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, title, start_ms, end_ms, file_path, file_index
         FROM chapters WHERE book_id = ?1
         ORDER BY file_index, start_ms",
    )?;
    let rows = stmt.query_map(params![book_id], |row| {
        Ok(Chapter {
            id: row.get(0)?,
            book_id: row.get(1)?,
            title: row.get(2)?,
            start_ms: row.get(3)?,
            end_ms: row.get(4)?,
            file_path: row.get(5)?,
            file_index: row.get(6)?,
        })
    })?;
    rows.collect()
}
