pub mod metadata;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Audio file extensions we recognize.
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "m4a", "m4b", "ogg", "opus", "flac", "aac", "wma"];

/// Files to ignore during scanning.
fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// A discovered book from the filesystem, before DB insertion.
#[derive(Debug)]
pub struct ScannedBook {
    pub folder_path: String,
    pub audio_files: Vec<PathBuf>,
}

/// Scan a library root and group audio files into books.
///
/// Grouping rules:
/// - A single .m4b file = one book (the parent folder is the book folder).
/// - A folder containing audio files (with no sub-folders that also have audio) = one book.
/// - Each leaf folder with audio files = one book.
pub fn scan_library_root(root: &Path) -> Vec<ScannedBook> {
    // Collect all audio files grouped by their parent directory.
    let mut folders: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_audio_file(path) {
            if let Some(parent) = path.parent() {
                folders
                    .entry(parent.to_path_buf())
                    .or_default()
                    .push(path.to_path_buf());
            }
        }
    }

    let mut books: Vec<ScannedBook> = Vec::new();

    for (folder, mut files) in folders {
        // Sort files naturally by filename for correct ordering
        files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

        books.push(ScannedBook {
            folder_path: folder.to_string_lossy().into_owned(),
            audio_files: files,
        });
    }

    // Sort books by folder path for consistent ordering
    books.sort_by(|a, b| a.folder_path.cmp(&b.folder_path));
    books
}

/// Generate a sort key that handles embedded numbers naturally.
/// "Disc 2" sorts before "Disc 10".
fn natural_sort_key(path: &Path) -> Vec<NaturalToken> {
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let mut tokens = Vec::new();
    let mut chars = name.chars().peekable();

    while chars.peek().is_some() {
        if chars.peek().map_or(false, |c| c.is_ascii_digit()) {
            let mut num_str = String::new();
            while chars.peek().map_or(false, |c| c.is_ascii_digit()) {
                num_str.push(chars.next().unwrap());
            }
            tokens.push(NaturalToken::Number(num_str.parse::<u64>().unwrap_or(0)));
        } else {
            let mut text = String::new();
            while chars.peek().map_or(false, |c| !c.is_ascii_digit()) {
                text.push(chars.next().unwrap());
            }
            tokens.push(NaturalToken::Text(text.to_lowercase()));
        }
    }

    tokens
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NaturalToken {
    Text(String),
    Number(u64),
}
