<h1>
  <span>LitSync</span>
  <img src="src/lib/assets/logo-transparent.png" alt="LitSync logo" width="84" align="center" />
</h1>

LitSync is a portable desktop audiobook player built with SvelteKit and Tauri. It keeps listening progress in its own database, which can sync between computers through a synced folder, and shares progress with Smart AudioBook Player (SABP), an Android audiobook app.

## Status

- Verified on macOS
- Intended to support Windows and Linux via Tauri
- Windows packaging has not been verified yet
- No prebuilt releases are published; build the app from source as described under [Development](#development)

## Features

- Scan a local audiobook library and build a searchable desktop library
- Read embedded metadata and cover art from common audiobook formats (MP3, M4A, M4B, OGG, Opus, FLAC, AAC, WMA)
- Import progress from SABP on scan and write it back on save
- Portable mode that keeps app data next to the app
- Playback controls for chapter navigation, seek, speed, and volume
- Chapter-aware playback for multi-file books
- Optional silence-skip analysis and playback skipping
- Light and dark themes

## Sync and portable mode

LitSync stores progress in a SQLite database, `litsync.db`, and keeps extracted cover art beside it. By default these live in the operating system's application-data directory. If a directory named `.litsync-data` exists next to the app (next to `LitSync.app` on macOS, next to the executable on Windows and Linux), LitSync uses that directory instead. Keep the app and `.litsync-data` in a synced folder to share progress between computers without SABP.

For SABP, LitSync uses this file inside each library root you scan:

```text
<library-root>/!Smart AudioBook Player Statistics/statistics.xml
```

On scan, it imports progress by matching each SABP entry to the top-level audiobook folder with the same name under the library root. Each time it saves progress, it writes the book's position to that file if the `!Smart AudioBook Player Statistics` folder already exists in that library root, so SABP on Android sees the same position once the folder syncs. LitSync never creates that folder: in libraries without it, progress stays only in LitSync's database.

## Development

Requirements:

- Node.js 20.19+ or 22.12+
- Rust stable
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

Install dependencies:

```sh
npm install
```

Run the app in development:

```sh
npm run tauri:dev
```

Run checks:

```sh
npm run check
cd src-tauri && cargo check
```

Build a production app (output goes to `src-tauri/target/release/bundle/`):

```sh
npm run tauri:build
```

## License

MIT. See [LICENSE](LICENSE).
