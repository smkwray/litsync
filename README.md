<h1>
  <span>LitSync</span>
  <img src="src/lib/assets/logo-transparent.png" alt="LitSync logo" width="84" align="center" />
</h1>

LitSync is a portable audiobook player built with SvelteKit and Tauri.

It keeps track of listening progress in its own local database, and when that data lives in a synced folder it can stay in sync across your devices. It also supports Android Smart AudioBook Player compatibility by reading and writing `!Smart AudioBook Player Statistics/statistics.xml` when that file is present inside a library root.

## Status

- Verified on macOS
- Intended to support Windows and Linux via Tauri
- Windows packaging has not been verified yet

## Features

- Scan a local audiobook library and build a searchable desktop library
- Read embedded metadata and cover art from common audiobook formats
- Display extracted cover art in the library and player
- Import progress from Smart AudioBook Player on scan
- Write progress back to Smart AudioBook Player's `statistics.xml` on save
- Keep LitSync progress in its own SQLite database for cross-device sync when `.litsync-data` is in a synced folder
- Portable mode: store app data next to the app bundle in `.litsync-data`
- Playback controls for chapter navigation, seek, speed, and volume
- Chapter-aware playback for multi-file books
- Adjustable playback speed
- Optional silence-skip analysis and playback skipping
- Light and dark themes

## Smart AudioBook Player Compatibility

LitSync looks for:

```text
<library-root>/!Smart AudioBook Player Statistics/statistics.xml
```

On scan, it imports book progress by matching each SABP folder name to the top-level audiobook folder under the scanned library root.

On playback save, it updates the same XML file so Android and desktop stay aligned.

## Progress Sync

LitSync always stores its own playback progress in its SQLite database.

If the packaged app is using portable mode and the `.litsync-data` directory is inside a synced folder, that LitSync progress database can sync between devices even without Smart AudioBook Player.

Smart AudioBook Player support is an additional compatibility layer, not the only way progress sync works.

## Development

Requirements:

- Node.js 20+
- Rust stable
- Tauri prerequisites for your platform

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

Build a production app:

```sh
npm run tauri:build
```

## Portable Mode

If a `.litsync-data` directory exists next to the packaged app, LitSync stores its database and extracted covers there instead of using the OS application-data directory.

That makes it possible to keep both the app and its state inside a synced folder.

## License

MIT
