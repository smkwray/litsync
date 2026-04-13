use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_data_dir(state: State<AppState>) -> String {
    state.data_dir.to_string_lossy().into_owned()
}

#[tauri::command]
pub fn get_app_mode(state: State<AppState>) -> String {
    if state.is_portable {
        "portable".to_string()
    } else {
        "standard".to_string()
    }
}

/// Debug command: check if a file is readable and return its size
#[tauri::command]
pub fn debug_file(path: String) -> Result<String, String> {
    let metadata = std::fs::metadata(&path).map_err(|e| format!("Can't stat {path}: {e}"))?;
    Ok(format!("exists, {} bytes", metadata.len()))
}
