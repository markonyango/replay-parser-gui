# ESLReporter

Desktop app for automatically parsing `Dawn of War II - Retribution` replays and reporting them to the Elite Mod replay endpoint.

The project combines:

- an Angular frontend for browsing reported matches
- a Tauri v2 desktop shell
- a Rust backend that watches the local replay/log files, parses them, uploads the replay, and emits updates to the UI

## What It Does

When the app is running, the Rust backend watches the replay file used by the game client:

- replay file: `Documents/My Games/Dawn of War II - Retribution/Playback/temp.rec`
- logfile: `Documents/My Games/Dawn of War II - Retribution/Logfiles/warnings.txt`

On each replay update, the app:

1. parses the latest replay with `parser_lib`
2. parses `warnings.txt` to enrich player and match metadata
3. copies the replay locally using a match-based filename
4. base64-encodes the replay and POSTs it to `https://dow2elite.com/esl-report.php`
5. emits the result to the Angular UI
6. stores the match list locally with the Tauri store plugin

The UI shows a match table with:

- match ID
- players
- map
- duration
- upload status
- played-at timestamp
- match details dialog with messages and parsed actions

## Repository Layout

```text
.
├── src/                 Angular application
├── src-tauri/           Tauri + Rust backend
├── package.json         Frontend and Tauri scripts
└── angular.json         Angular build configuration
```

Relevant files:

- [src-tauri/src/main.rs](src-tauri/src/main.rs): Tauri bootstrap and backend thread startup
- [src-tauri/src/core/mod.rs](src-tauri/src/core/mod.rs): file watching and replay processing flow
- [src-tauri/src/core/game.rs](src-tauri/src/core/game.rs): replay enrichment, copy/upload logic, window events
- [src/app/core/services/tauri.service.ts](src/app/core/services/tauri.service.ts): frontend event handling and local persistence
- [src/app/features/match-list](src/app/features/match-list): match list UI

## Requirements

- Node.js and npm
- Rust toolchain
- Tauri prerequisites for your platform
- access to the `Dawn of War II - Retribution` Documents folder structure above

The app is effectively coupled to the game's default Documents paths. If those files do not exist, the backend will fail to start processing replays.

## Installation

```bash
npm install
```

Cargo dependencies are resolved automatically through Tauri commands. The Rust backend also pulls `parser_lib` from:

- `https://github.com/dowmechanicus/ReplayParser`

## Development

Recommended workflow:

```bash
npm run tauri:serve
```

That starts the Angular dev server and launches the Tauri desktop app against it.

Other useful commands:

```bash
npm run web:serve    # Angular dev server only
npm run build        # Angular production build
npm run web:prod     # Angular production build used by Tauri packaging
npm run tauri:build  # Desktop bundle
npm test             # Angular unit tests via Karma
```

`web:serve` is useful for frontend-only iteration, but the replay watcher, file access, updater, notifications, and other native integrations are only exercised through Tauri.

## Build Output

- Angular output: `dist/eslreporter/browser`
- Tauri config: [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)

The desktop bundle is configured with:

- product name: `ESLReporter`
- identifier: `com.elite-mod.replay-parser`
- updater endpoint via Tauri updater plugin

## Release Builds And Signing

This project is already configured to generate updater artifacts during release builds:

- `src-tauri/tauri.conf.json` sets `"createUpdaterArtifacts": "v1Compatible"`
- `src-tauri/tauri.conf.json` already contains the updater public key used by the app to verify updates

To build a signed release with the existing updater private key, export the signing key into your shell first. Per the Tauri v2 updater documentation, `.env` files are not used for this.

### macOS / Linux

```bash
export TAURI_SIGNING_PRIVATE_KEY="/absolute/path/to/existing/private.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your-key-password"
npm run tauri:build
```

If the existing key has no password, leave `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` empty:

```bash
export TAURI_SIGNING_PRIVATE_KEY="/absolute/path/to/existing/private.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
npm run tauri:build
```

### PowerShell

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY="C:\path\to\existing\private.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your-key-password"
npm run tauri:build
```

Tauri accepts either:

- the private key file path in `TAURI_SIGNING_PRIVATE_KEY`
- or the private key contents directly in `TAURI_SIGNING_PRIVATE_KEY`

After a successful build, Tauri will generate the normal release bundles and matching updater signature files inside `src-tauri/target/release/bundle/`.

Typical outputs are platform-specific:

- Windows: installer bundles under `src-tauri/target/release/bundle/msi/` and/or `src-tauri/target/release/bundle/nsis/`
- Linux: AppImage bundle under `src-tauri/target/release/bundle/appimage/`
- macOS: app bundle and update archive under `src-tauri/target/release/bundle/macos/`

For updater publishing, keep the generated signature file next to the corresponding update artifact.

Note:

- Do not generate a new key if you want existing installed clients to continue trusting updates. Reuse the same private key that matches the public key already embedded in `tauri.conf.json`.
- This README section covers Tauri updater signing. Native platform code-signing for Windows certificates or macOS Apple signing is not configured in this repository and would require additional platform-specific credentials and Tauri settings.

## Runtime Behavior

### Local Storage

The frontend persists matches in a Tauri store file named `store.json`.

### Logging

The backend writes error logs to the Tauri app log directory as `error.log`.

### Auto Updates

On startup, the Angular app checks for updates through the Tauri updater plugin and relaunches after a successful install.

## Notes

- The current README previously contained generic Angular CLI boilerplate. This repository is primarily a Tauri desktop app with an Angular frontend.
- The project includes several Tauri plugins beyond the current core flow, including updater, dialog, notifications, filesystem access, clipboard access, process control, shell integration, and global shortcuts.
- The Rust crate version and bundled app version are currently `1.0.6`.

## License

[MIT](LICENSE)
