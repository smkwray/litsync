mod commands;
mod data_dir;
mod db;
mod sabp;
mod scanner;
mod state;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Logging in debug builds
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Resolve data directory (portable vs standard)
            let (data_dir, is_portable) = data_dir::resolve_data_dir(app);
            std::fs::create_dir_all(&data_dir).expect("failed to create data directory");

            let mode = if is_portable { "portable" } else { "standard" };
            log::info!("Data directory ({mode}): {}", data_dir.display());

            // Initialize SQLite
            let conn = db::init(&data_dir).expect("failed to initialize database");

            // Allow the asset protocol to serve covers and audio files
            let scope = app.asset_protocol_scope();
            if let Err(e) = scope.allow_directory(&data_dir, true) {
                log::warn!("Failed to add data dir to asset scope: {e}");
            }

            // Re-scope any previously saved library roots
            if let Ok(roots) = db::queries::get_library_roots(&conn) {
                for root in roots {
                    let root_path = std::path::Path::new(&root.path);
                    if let Err(e) = scope.allow_directory(root_path, true) {
                        log::warn!("Failed to add library root to asset scope: {e}");
                    }
                }
            }

            // Store global state
            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir,
                is_portable,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_data_dir,
            commands::settings::get_app_mode,
            commands::settings::debug_file,
            commands::library::scan_library,
            commands::library::get_books,
            commands::library::get_book,
            commands::library::get_library_roots,
            commands::progress::save_progress,
            commands::progress::get_progress,
            commands::progress::get_all_progress,
            commands::silence::analyze_silence,
            commands::app_settings::get_settings,
            commands::app_settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
