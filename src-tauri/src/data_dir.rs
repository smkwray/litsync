use std::path::PathBuf;
use tauri::Manager;

/// Determines whether we're in portable mode and returns the resolved data directory.
///
/// Portable mode: if a `.litsync-data` directory exists next to the app bundle
/// (on macOS, next to the `.app` directory), use it.
/// Standard mode: use the OS app data directory provided by Tauri.
pub fn resolve_data_dir(app: &tauri::App) -> (PathBuf, bool) {
    // Try portable mode first
    if let Some(portable_dir) = find_portable_dir() {
        if portable_dir.exists() {
            return (portable_dir, true);
        }
    }

    // Fall back to OS standard location
    let standard_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");
    (standard_dir, false)
}

/// Look for `.litsync-data` next to the application bundle.
/// On macOS, current_exe() is inside `Foo.app/Contents/MacOS/binary`,
/// so we walk up to find the `.app` bundle and check its parent.
/// On other platforms, check next to the executable directly.
fn find_portable_dir() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;

    if cfg!(target_os = "macos") {
        // Walk up from the binary to find the .app bundle
        let mut path = exe_path.as_path();
        while let Some(parent) = path.parent() {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.ends_with(".app"))
            {
                // Found the .app — check its parent for .litsync-data
                return Some(parent.join(".litsync-data"));
            }
            path = parent;
        }
        // Not inside a .app bundle (dev mode) — check next to the binary
        exe_path.parent().map(|p| p.join(".litsync-data"))
    } else {
        // Windows / Linux: check next to the executable
        exe_path.parent().map(|p| p.join(".litsync-data"))
    }
}
