# Filnizer

Filnizer is a portable desktop app for local file organization, duplicate detection, safe undo, and offline file conversion.

## Goals

- Portable folder/ZIP distribution, no installer required.
- Local-first runtime behavior: no telemetry, updater, browser opening, or remote conversion APIs. LibreOffice may be downloaded/installed only after explicit user confirmation.
- Safe organizer workflow: scan, preview, apply, and undo.
- Local conversion workflows for images, spreadsheets, media, PDF, Markdown-to-PDF, and Office-to-PDF.
- Portable releases bundle FFmpeg and Pdfium app-local for media/PDF conversion.

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

The package script writes output under `artifacts/` and expects helper binaries in `binaries/` by default:

```text
binaries/ffmpeg.exe
binaries/pdfium.dll
```

Set `FILNIZER_HELPER_BINARIES_DIR` to use a different local helper-binary source folder.

## Contributor orientation

Before changing code, read:

1. `AGENTS.md`
2. `docs/ARCHITECTURE.md`
3. This `README.md`

## Release docs

- Portable README: `docs/release/portable-readme.md`
- Smoke-test checklist: `docs/release/checklist.md`
- License inventory: `docs/release/licenses.md`
- WebView2 guidance: `docs/release/webview2.md`
