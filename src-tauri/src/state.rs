use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Global application state managed by Tauri.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
    pub is_portable: bool,
}
