use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppSettings {
    pub theme: String,
    pub default_speed: f64,
    pub skip_short_sec: i64,
    pub skip_long_sec: i64,
    pub auto_rewind_enabled: bool,
    pub auto_rewind_short_pause_sec: i64,  // pause threshold
    pub auto_rewind_short_rewind_sec: i64, // rewind amount
    pub auto_rewind_long_pause_sec: i64,
    pub auto_rewind_long_rewind_sec: i64,
    pub sleep_timer_default_min: i64,
    pub silence_skip_threshold: f64,
    pub silence_skip_min_gap_ms: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            default_speed: 1.0,
            skip_short_sec: 10,
            skip_long_sec: 30,
            auto_rewind_enabled: true,
            auto_rewind_short_pause_sec: 30,
            auto_rewind_short_rewind_sec: 5,
            auto_rewind_long_pause_sec: 300,
            auto_rewind_long_rewind_sec: 30,
            sleep_timer_default_min: 30,
            silence_skip_threshold: -50.0,
            silence_skip_min_gap_ms: 700,
        }
    }
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let json: Option<String> = db
        .query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get(0),
        )
        .ok();

    match json {
        Some(j) => serde_json::from_str(&j).map_err(|e| e.to_string()),
        None => Ok(AppSettings::default()),
    }
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO settings (key, value) VALUES ('app_settings', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
