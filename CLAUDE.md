# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ESLReporter** is a Tauri v2 desktop application that watches `Dawn of War II - Retribution` replay files, parses them, and uploads match results to the Elite Mod server. The frontend is Angular 20 with ag-grid data tables and Material Design; the backend is Rust with file watching, replay parsing, and upload handling.

## Repository Structure

```
├── src/                    # Angular frontend (TypeScript)
│   └── app/
│       ├── core/          # Services (tauri.service.ts, update.service.ts)
│       ├── features/      # Feature modules (match-list, etc.)
│       └── shared/        # Shared components and modules
├── src-tauri/              # Tauri v2 project (Rust + config)
│   ├── src/
│   │   ├── main.rs        # Tauri app bootstrap
│   │   ├── core/          # Core business logic
│   │   │   ├── mod.rs     # File watching and event loop
│   │   │   ├── game.rs    # Replay enrichment and upload
│   │   │   ├── logfile.rs # Log file parsing
│   │   │   └── error.rs   # Error types
│   │   └── logger.rs      # Tracing setup
│   └── tauri.conf.json    # Tauri app config (updater, plugins, window)
├── package.json            # Frontend + Tauri scripts
├── angular.json            # Angular build config
└── README.md              # Full setup and release guide
```

## Core Architecture

### Frontend → Backend Communication

The Angular frontend (`src/app/core/services/tauri.service.ts`) uses Tauri event listeners:
- **Game event channel**: backend emits `new-game` events when a replay is detected and processed
- **Update listener**: checks for app updates via Tauri updater plugin
- **Local store**: persists matches in Tauri store plugin (`store.json`)

### Backend Event Loop (`src-tauri/src/core/mod.rs`)

1. **File watching**: watches `temp.rec` for changes using `notify-debouncer-mini` (5-second debounce)
2. **Replay parsing**: calls `parser_lib` (external crate from ReplayParser repo) to extract match metadata
3. **Logfile enrichment** (`core/logfile.rs`): parses `warnings.txt` to extract player names and match context
4. **Upload flow** (`core/game.rs`):
   - copies replay locally with match-based filename
   - base64-encodes the file
   - POSTs to `https://dow2elite.com/esl-report.php`
   - emits `new-game` event to frontend with result
5. **Logging**: writes errors to Tauri app log directory as `error.log` (via `logger.rs`)

### File Paths

The app watches these hardcoded game directories:
- Replay: `Documents/My Games/Dawn of War II - Retribution/Playback/temp.rec`
- Logfile: `Documents/My Games/Dawn of War II - Retribution/Logfiles/warnings.txt`

If either file is missing, the backend will fail to initialize.

## Commands

### Development

```bash
npm run tauri:serve     # Run in dev mode: Angular dev server + Tauri window
npm run web:serve       # Angular dev server only (frontend-only iteration)
npm run web:build       # Angular production build
npm run web:prod        # Production build used by Tauri packaging
```

### Testing & Quality

```bash
npm test                # Angular unit tests via Karma
npm run build          # Angular production build
```

### Building & Release

```bash
npm run tauri:build     # Build desktop bundle for current platform
```

See **Release Builds And Signing** in [README.md](README.md) for signing instructions using `TAURI_SIGNING_PRIVATE_KEY` env vars.

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/main.rs` | Tauri app initialization and plugin setup |
| `src-tauri/src/core/mod.rs` | File watcher event loop, replay/logfile parsing |
| `src-tauri/src/core/game.rs` | Replay enrichment, copy/upload logic, `new-game` event emission |
| `src/app/core/services/tauri.service.ts` | Frontend event listeners, local store persistence |
| `src/app/features/match-list` | Match list table component (ag-grid) and details dialog |
| `src-tauri/tauri.conf.json` | Tauri config: plugins, updater, window, capabilities |
| `package.json` | Frontend dependencies and build/Tauri scripts |
| `src-tauri/Cargo.toml` | Rust dependencies (including `parser_lib` from git) |

## Frontend Stack

- **Angular 20** (standalone components in later versions, modular in current setup)
- **ag-grid-angular 34**: high-performance match list table
- **Angular Material 20**: navbar, dialogs
- **Tailwind CSS 3.2**: utility-first styling
- **RxJS 7.5**: reactive programming
- **Tauri plugins**: clipboard, dialog, notifications, fs, global shortcuts, http, shell, store, updater

## Backend Stack

- **Tauri v2**: desktop app framework, IPC, plugins
- **Rust 1.65+**: async/sync hybrid (std::thread for core loop)
- **parser_lib**: external crate for replay parsing (git dependency from dowmechanicus/ReplayParser)
- **notify** + **notify-debouncer-mini**: file watching with debounce
- **reqwest**: HTTP client for uploads
- **serde/serde_json**: serialization
- **tracing** + **tracing-appender**: structured logging to file
- **thiserror** + **color-eyre**: error handling

## Testing

- **Angular**: Karma + Jasmine (`npm test`)
- **Rust**: inline tests in src-tauri (run with `cargo test` from `src-tauri/` dir)
- Testing the **replay watcher** or **upload flow** requires:
  - Either actual game files in the expected paths
  - Or mocking the file paths in tests (see `core/mod.rs` `get_input_files()`)

## Common Development Patterns

### Adding a New Event from Backend to Frontend

1. Emit from Rust: `main_window_handle.emit("event-name", &payload)?;` in `core/game.rs`
2. Listen in Angular: `this.tauriService.onEvent("event-name").subscribe(...)` in `tauri.service.ts`
3. Update the match store after receiving the event if persistent storage is needed

### Adding a Frontend Route

1. Create feature module in `src/app/features/`
2. Define routing in feature's `-routing.module.ts`
3. Add lazy-loaded route to `app-routing.module.ts`

### Logging from Rust

Use the `tracing` macros: `tracing::debug!()`, `tracing::info!()`, `tracing::error!()`. Logs appear in Tauri app log directory.

## Release & Distribution

- **Updater**: Tauri v2 updater plugin configured to fetch signatures from a Gist
- **Platform bundles**: MSI/NSIS (Windows), AppImage (Linux), app + DMG (macOS)
- **Signing**: Requires `TAURI_SIGNING_PRIVATE_KEY` and optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` env vars
- **Artifacts**: generated in `src-tauri/target/release/bundle/` with `.sig` files for updates

## Tauri Configuration

Key settings in `src-tauri/tauri.conf.json`:
- **Window**: 800x600, resizable, HTTPS scheme
- **Plugins**: store, fs, updater, clipboard, http, global shortcuts, dialog, process, notifications, os, shell
- **Updater endpoint**: GitHub Gist for fetching update manifests
- **CSP**: null (relies on custom protocol security)

## Notable Constraints

- **Hardcoded game paths**: app is tightly coupled to Windows/Linux/macOS `Documents` folder structure for the game
- **File watching**: uses a 5-second debounce; rapid replay file changes may be coalesced
- **Replay upload**: blocking HTTP call in the backend event loop; long uploads may briefly freeze the watcher
- **Error recovery**: if backend fails, no automatic restart; user must quit and relaunch the app

## Debugging

### Debug Logs

- **Rust errors**: check `error.log` in Tauri app data dir (platform-specific)
- **Frontend**: open DevTools in dev mode via Tauri window

### Dev Server Only

`npm run web:serve` runs the Angular dev server on `http://localhost:4200` without Tauri—useful for styling and component work, but skips replay watching and upload logic.
