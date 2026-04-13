use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub description: Option<String>,
    pub total_duration_ms: i64,
    pub cover_path: Option<String>,
    pub folder_path: String,
    pub file_count: i64,
    pub added_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub id: i64,
    pub book_id: i64,
    pub title: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub file_path: String,
    pub file_index: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Progress {
    pub book_id: i64,
    pub position_ms: i64,
    pub chapter_index: i64,
    pub playback_speed: f64,
    pub status: String,
    pub last_played_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub book_id: i64,
    pub position_ms: i64,
    pub chapter_index: i64,
    pub title: Option<String>,
    pub note: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryRoot {
    pub id: i64,
    pub path: String,
    pub last_scanned_at: Option<String>,
}
