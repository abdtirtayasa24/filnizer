# Filnizer

Filnizer is a Windows 10+ portable desktop app for local file organization, duplicate detection, safe undo, and offline file conversion.

## Goals

- Portable folder/ZIP distribution, no installer required.
- Strictly local runtime behavior: no telemetry, updater, automatic downloads, browser opening, or remote conversion APIs.
- Safe organizer workflow: scan, preview, apply, and undo.
- Local conversion workflows for images, spreadsheets, media, PDF, Markdown-to-PDF, and Office-to-PDF.

## Development

```powershell
npm install
npm run build
npm run test:rust
npm run lint:rust
```

Run the app in development mode:

```powershell
npm run tauri dev
```

## Portable package

Build a portable Windows ZIP/folder:

```powershell
npm run package:portable
```

The package script writes output under `artifacts/`.

## Release docs

- Portable README: `docs/release/portable-readme.md`
- Smoke-test checklist: `docs/release/checklist.md`
- License inventory: `docs/release/licenses.md`
- WebView2 guidance: `docs/release/webview2.md`
- SignPath.io signing notes: `docs/release/signing.md`
