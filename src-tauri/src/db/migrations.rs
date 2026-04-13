use rusqlite::Connection;

/// Run all pending migrations in order.
pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        );",
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let migrations: &[(&str, i64)] = &[
        (MIGRATION_001_CORE_SCHEMA, 1),
        (MIGRATION_002_PROGRESS, 2),
        (MIGRATION_003_BOOKMARKS, 3),
        (MIGRATION_004_SETTINGS, 4),
    ];

    for (sql, version) in migrations {
        if *version > current_version {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [version],
            )?;
            log::info!("Applied migration v{version}");
        }
    }

    Ok(())
}

const MIGRATION_001_CORE_SCHEMA: &str = "
    CREATE TABLE library_roots (
        id INTEGER PRIMARY KEY,
        path TEXT NOT NULL UNIQUE,
        last_scanned_at TEXT
    );

    CREATE TABLE books (
        id INTEGER PRIMARY KEY,
        title TEXT NOT NULL,
        author TEXT,
        narrator TEXT,
        description TEXT,
        total_duration_ms INTEGER NOT NULL DEFAULT 0,
        cover_path TEXT,
        folder_path TEXT NOT NULL,
        file_count INTEGER NOT NULL DEFAULT 1,
        added_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE chapters (
        id INTEGER PRIMARY KEY,
        book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
        title TEXT NOT NULL,
        start_ms INTEGER NOT NULL,
        end_ms INTEGER NOT NULL,
        file_path TEXT NOT NULL,
        file_index INTEGER NOT NULL DEFAULT 0,
        UNIQUE(book_id, file_index, start_ms)
    );
";

const MIGRATION_002_PROGRESS: &str = "
    CREATE TABLE progress (
        book_id INTEGER PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
        position_ms INTEGER NOT NULL DEFAULT 0,
        chapter_index INTEGER NOT NULL DEFAULT 0,
        playback_speed REAL NOT NULL DEFAULT 1.0,
        status TEXT NOT NULL DEFAULT 'not_started',
        last_played_at TEXT,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
";

const MIGRATION_003_BOOKMARKS: &str = "
    CREATE TABLE bookmarks (
        id INTEGER PRIMARY KEY,
        book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
        position_ms INTEGER NOT NULL,
        chapter_index INTEGER NOT NULL,
        title TEXT,
        note TEXT,
        color TEXT DEFAULT '#fbbf24',
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
";

const MIGRATION_004_SETTINGS: &str = "
    CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
";
