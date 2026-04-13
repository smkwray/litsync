use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::{Path, PathBuf};

/// Extracted metadata for a single audio file.
#[derive(Debug)]
pub struct FileMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub disc: Option<u32>,
    pub disc_total: Option<u32>,
    pub track: Option<u32>,
    pub year: Option<u32>,
    pub duration_ms: u64,
}

/// Metadata aggregated for a whole book (from its first/representative file).
#[derive(Debug)]
pub struct BookMetadata {
    pub title: String,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub year: Option<u32>,
    pub total_duration_ms: u64,
}

/// Per-file info needed to build chapters for multi-file books.
#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub title: String,
    pub duration_ms: u64,
}

/// Extract metadata from a single audio file using lofty.
pub fn extract_file_metadata(path: &Path) -> Option<FileMetadata> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;

    let duration_ms = tagged_file.properties().duration().as_millis() as u64;

    // Try to get the primary tag, or any tag
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let (title, artist, album, composer, genre, disc, disc_total, track, year) =
        if let Some(tag) = tag {
            let disc_info = parse_disc_tag(tag);
            (
                tag.title().map(|s| s.to_string()),
                tag.artist().map(|s| s.to_string()),
                tag.album().map(|s| s.to_string()),
                tag.get_string(&lofty::tag::ItemKey::Composer)
                    .map(|s| s.to_string()),
                tag.genre().map(|s| s.to_string()),
                disc_info.0,
                disc_info.1,
                tag.track(),
                tag.year(),
            )
        } else {
            (None, None, None, None, None, None, None, None, None)
        };

    Some(FileMetadata {
        title,
        artist,
        album,
        composer,
        genre,
        disc,
        disc_total,
        track,
        year,
        duration_ms,
    })
}

/// Parse disc number from tag, handling "1/12" format.
fn parse_disc_tag(tag: &lofty::tag::Tag) -> (Option<u32>, Option<u32>) {
    // First try the native disc number
    if let Some(disc) = tag.disk() {
        return (Some(disc), tag.disk_total());
    }

    // Try parsing from the disc string (e.g., "1/12")
    if let Some(disc_str) = tag.get_string(&lofty::tag::ItemKey::DiscNumber) {
        if let Some((num, total)) = disc_str.split_once('/') {
            return (num.trim().parse().ok(), total.trim().parse().ok());
        }
        return (disc_str.trim().parse().ok(), None);
    }

    (None, None)
}

/// Build book-level metadata from a set of audio files.
/// Uses the first file for title/author/narrator, sums durations across all files.
pub fn build_book_metadata(files: &[PathBuf], folder_path: &str) -> (BookMetadata, Vec<FileInfo>) {
    let mut file_infos = Vec::new();
    let mut total_duration_ms: u64 = 0;

    // Extract metadata from each file
    for file_path in files {
        let meta = extract_file_metadata(file_path);
        let duration_ms = meta.as_ref().map(|m| m.duration_ms).unwrap_or(0);
        let title = meta
            .as_ref()
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        total_duration_ms += duration_ms;
        file_infos.push(FileInfo {
            path: file_path.clone(),
            title,
            duration_ms,
        });
    }

    // Use first file for book-level metadata
    let first_meta = files.first().and_then(|f| extract_file_metadata(f));

    let book_title = first_meta
        .as_ref()
        .and_then(|m| m.album.clone())
        .unwrap_or_else(|| {
            // Fall back to folder name
            Path::new(folder_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown Book")
                .to_string()
        });

    let author = first_meta.as_ref().and_then(|m| m.artist.clone());

    // Narrator is often in the "composer" field for audiobooks
    let narrator = first_meta.as_ref().and_then(|m| {
        m.composer.as_ref().and_then(|c| {
            // Strip "Read By " prefix if present
            let cleaned = c
                .strip_prefix("Read By ")
                .or_else(|| c.strip_prefix("Read by "))
                .or_else(|| c.strip_prefix("Narrated By "))
                .or_else(|| c.strip_prefix("Narrated by "))
                .unwrap_or(c);
            Some(cleaned.to_string())
        })
    });

    let year = first_meta.as_ref().and_then(|m| m.year);

    (
        BookMetadata {
            title: book_title,
            author,
            narrator,
            year,
            total_duration_ms,
        },
        file_infos,
    )
}

/// Extract cover art from the first file that has one.
/// Returns the image bytes and a guessed file extension.
pub fn extract_cover_art(files: &[PathBuf]) -> Option<(Vec<u8>, String)> {
    for file_path in files {
        let tagged_file = match Probe::open(file_path).and_then(|p| p.read()) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let tag = match tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
        {
            Some(t) => t,
            None => continue,
        };

        // Try front cover first, then any picture
        let picture = tag
            .pictures()
            .iter()
            .find(|p| p.pic_type() == PictureType::CoverFront)
            .or_else(|| tag.pictures().first());

        if let Some(pic) = picture {
            let ext = match pic.mime_type() {
                Some(lofty::picture::MimeType::Png) => "png",
                Some(lofty::picture::MimeType::Jpeg) => "jpg",
                Some(lofty::picture::MimeType::Bmp) => "bmp",
                _ => "jpg",
            };
            return Some((pic.data().to_vec(), ext.to_string()));
        }
    }
    None
}
