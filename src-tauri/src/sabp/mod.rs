//! Smart AudioBook Player compatibility.
//!
//! SABP stores progress in `!Smart AudioBook Player Statistics/statistics.xml`
//! inside the library root. Format:
//!
//! ```xml
//! <?xml version="1.0" encoding="UTF-8"?><root>
//!     <book>
//!         <path>Folder Name</path>
//!         <time>YYYY-MM POSITION_SECONDS</time>
//!     </book>
//! </root>
//! ```
//!
//! We read this on scan to import progress and, where the statistics folder
//! already exists, write to it on save so SABP on Android stays in sync.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const SABP_STATS_DIR: &str = "!Smart AudioBook Player Statistics";
const SABP_STATS_FILE: &str = "statistics.xml";

/// A single book's progress from SABP.
#[derive(Debug, Clone)]
pub struct SabpBookProgress {
    /// Folder name relative to library root (e.g. "Nicholas Wade - A Troublesome Inheritance")
    pub path: String,
    /// Year-month string (e.g. "2025-12")
    pub month: String,
    /// Position in seconds from start of book
    pub position_secs: u64,
}

/// Find the SABP statistics file for a library root.
pub fn stats_file_path(library_root: &Path) -> PathBuf {
    library_root.join(SABP_STATS_DIR).join(SABP_STATS_FILE)
}

/// Read all book progress from a SABP statistics.xml file.
pub fn read_statistics(library_root: &Path) -> Result<Vec<SabpBookProgress>, String> {
    let stats_path = stats_file_path(library_root);
    let content = match std::fs::read_to_string(&stats_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Failed to read {}: {e}", stats_path.display())),
    };

    parse_statistics_xml(&content)
}

fn parse_statistics_xml(xml: &str) -> Result<Vec<SabpBookProgress>, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().expand_empty_elements = true;
    let mut books = Vec::new();
    let mut saw_root = false;
    let mut depth = 0usize;

    let mut in_book = false;
    let mut current_path: Option<String> = None;
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if depth == 0 {
                    if saw_root || name != "root" {
                        return Err("Expected a single SABP <root> element".into());
                    }
                    saw_root = true;
                }
                depth += 1;
                match name.as_str() {
                    "book" => {
                        in_book = true;
                        current_path = None;
                    }
                    _ if in_book => {
                        current_tag = name;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) if in_book => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                match current_tag.as_str() {
                    "path" => {
                        current_path = Some(text);
                    }
                    "time" => {
                        if let Some(path) = current_path.take() {
                            if let Some(progress) = parse_time_field(&path, &text) {
                                books.push(progress);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                depth -= 1;
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "book" {
                    in_book = false;
                }
                current_tag.clear();
            }
            Ok(Event::Text(e)) if depth == 0 && !e.iter().all(u8::is_ascii_whitespace) => {
                return Err("Unexpected text outside the SABP <root> element".into());
            }
            // The streaming reader can reach EOF with elements still open.
            Ok(Event::Eof) if !saw_root || depth != 0 => {
                return Err("Missing or incomplete SABP <root> element".into());
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
    }

    Ok(books)
}

/// Parse SABP time field: "YYYY-MM SECONDS" -> (month, position_secs)
fn parse_time_field(path: &str, time_str: &str) -> Option<SabpBookProgress> {
    let parts: Vec<&str> = time_str.trim().splitn(2, ' ').collect();
    if parts.len() != 2 {
        log::warn!("SABP: unexpected time format for {path}: {time_str}");
        return None;
    }
    let month = parts[0].to_string();
    let position_secs: u64 = parts[1].parse().ok()?;
    Some(SabpBookProgress {
        path: path.to_string(),
        month,
        position_secs,
    })
}

/// Write book progress to SABP statistics.xml.
/// Merges with existing entries — updates matching paths, keeps others.
/// Does nothing unless the SABP statistics folder already exists, so
/// libraries that SABP does not use are left untouched.
pub fn write_statistics(library_root: &Path, updates: &[SabpBookProgress]) -> Result<(), String> {
    if !library_root.join(SABP_STATS_DIR).is_dir() {
        return Ok(());
    }

    // Read existing entries
    let mut entries: HashMap<String, SabpBookProgress> = HashMap::new();
    for entry in read_statistics(library_root)? {
        entries.insert(entry.path.clone(), entry);
    }

    // Apply updates
    for update in updates {
        entries.insert(update.path.clone(), update.clone());
    }

    // Write XML
    let xml = build_statistics_xml(&entries)?;
    let stats_path = stats_file_path(library_root);
    std::fs::write(&stats_path, &xml)
        .map_err(|e| format!("Failed to write {}: {e}", stats_path.display()))?;

    Ok(())
}

fn build_statistics_xml(entries: &HashMap<String, SabpBookProgress>) -> Result<String, String> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 4);

    // XML declaration
    writer
        .write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            None,
        )))
        .map_err(|e| e.to_string())?;

    // <root>
    writer
        .write_event(Event::Start(BytesStart::new("root")))
        .map_err(|e| e.to_string())?;

    // Sort by path for stable output
    let mut sorted: Vec<_> = entries.values().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));

    for entry in sorted {
        // <book>
        writer
            .write_event(Event::Start(BytesStart::new("book")))
            .map_err(|e| e.to_string())?;

        // <path>...</path>
        writer
            .write_event(Event::Start(BytesStart::new("path")))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::Text(BytesText::new(&entry.path)))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::End(BytesEnd::new("path")))
            .map_err(|e| e.to_string())?;

        // <time>YYYY-MM SECS</time>
        let time_str = format!("{} {}", entry.month, entry.position_secs);
        writer
            .write_event(Event::Start(BytesStart::new("time")))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::Text(BytesText::new(&time_str)))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::End(BytesEnd::new("time")))
            .map_err(|e| e.to_string())?;

        // </book>
        writer
            .write_event(Event::End(BytesEnd::new("book")))
            .map_err(|e| e.to_string())?;
    }

    // </root>
    writer
        .write_event(Event::End(BytesEnd::new("root")))
        .map_err(|e| e.to_string())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Get the current YYYY-MM string.
pub fn current_month() -> String {
    use time::macros::format_description;
    use time::OffsetDateTime;

    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    now.format(&format_description!("[year]-[month]"))
        .unwrap_or_else(|_| "1970-01".to_string())
}

/// Derive the SABP folder name for a book from its folder_path and a library root.
/// Returns the relative folder name, e.g. "Nicholas Wade - A Troublesome Inheritance"
pub fn book_folder_name(folder_path: &str, library_root: &str) -> Option<String> {
    let book = Path::new(folder_path);
    let root = Path::new(library_root);
    book.strip_prefix(root)
        .ok()
        .and_then(|rel| {
            // Take the first component (the top-level folder under the root)
            rel.components().next()
        })
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_statistics_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root>
    <book>
        <path>Nicholas Wade - A Troublesome Inheritance</path>
        <time>2025-12 15202</time>
    </book>
    <book>
        <path>Tomorrow Is Yesterday</path>
        <time>2025-12 29886</time>
    </book>
</root>"#;
        let result = parse_statistics_xml(xml).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "Nicholas Wade - A Troublesome Inheritance");
        assert_eq!(result[0].position_secs, 15202);
        assert_eq!(result[0].month, "2025-12");
        assert_eq!(result[1].path, "Tomorrow Is Yesterday");
        assert_eq!(result[1].position_secs, 29886);
    }

    #[test]
    fn test_book_folder_name() {
        let name = book_folder_name(
            "/library/Nicholas Wade - A Troublesome Inheritance",
            "/library",
        );
        assert_eq!(
            name,
            Some("Nicholas Wade - A Troublesome Inheritance".to_string())
        );

        // Nested subfolder
        let name = book_folder_name("/library/Author/Book Title/disc1", "/library");
        assert_eq!(name, Some("Author".to_string()));
    }

    /// An empty library root under the system temp dir, removed on drop.
    struct TempLibrary(PathBuf);

    impl TempLibrary {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("litsync-sabp-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempLibrary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn progress(path: &str, position_secs: u64) -> SabpBookProgress {
        SabpBookProgress {
            path: path.to_string(),
            month: "2026-11".to_string(),
            position_secs,
        }
    }

    #[test]
    fn write_leaves_library_without_sabp_untouched() {
        // People who don't use SABP must not get its folder in their library.
        let lib = TempLibrary::new("no-sabp");
        write_statistics(&lib.0, &[progress("Book A", 60)]).unwrap();
        assert_eq!(std::fs::read_dir(&lib.0).unwrap().count(), 0);
    }

    #[test]
    fn write_updates_existing_sabp_file_and_keeps_other_books() {
        // SABP's own entries for other books must survive our write.
        let lib = TempLibrary::new("existing-file");
        std::fs::create_dir(lib.0.join(SABP_STATS_DIR)).unwrap();
        std::fs::write(
            stats_file_path(&lib.0),
            r#"<?xml version="1.0" encoding="UTF-8"?><root>
    <book><path>Book A</path><time>2025-12 100</time></book>
    <book><path>Book B</path><time>2025-12 200</time></book>
</root>"#,
        )
        .unwrap();

        write_statistics(&lib.0, &[progress("Book A", 4321)]).unwrap();

        let books = read_statistics(&lib.0).unwrap();
        let saved: Vec<_> = books
            .iter()
            .map(|b| (b.path.as_str(), b.month.as_str(), b.position_secs))
            .collect();
        assert_eq!(
            saved,
            vec![("Book A", "2026-11", 4321), ("Book B", "2025-12", 200)]
        );
    }

    #[test]
    fn write_preserves_corrupt_statistics_file() {
        let lib = TempLibrary::new("corrupt-file");
        std::fs::create_dir(lib.0.join(SABP_STATS_DIR)).unwrap();
        let path = stats_file_path(&lib.0);
        for corrupt in [
            "<root><book><path>Book B</path><time>2025-12 200</time></root>",
            "<root><book><path>Book B</path>",
            "unreadable statistics",
        ] {
            std::fs::write(&path, corrupt).unwrap();

            let result = write_statistics(&lib.0, &[progress("Book A", 60)]);

            assert_eq!(std::fs::read(&path).unwrap(), corrupt.as_bytes());
            assert!(result.is_err());
        }
    }

    #[test]
    fn write_creates_file_in_existing_sabp_folder() {
        // The folder alone means SABP is in use for this library.
        let lib = TempLibrary::new("existing-folder");
        std::fs::create_dir(lib.0.join(SABP_STATS_DIR)).unwrap();
        write_statistics(&lib.0, &[progress("Book A", 60)]).unwrap();
        let books = read_statistics(&lib.0).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].position_secs, 60);
    }
}
