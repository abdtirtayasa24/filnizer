# Filnizer Architecture

This document is the primary technical orientation for future contributors and agents. Read it together with `README.md` and root `AGENTS.md` before making changes.

## Product and Runtime Model

### Distribution

- Filnizer is distributed as a portable folder/ZIP, not an installer.
- The user launches `Filnizer.exe` from the extracted portable folder.
- Tauri bundling is disabled in `src-tauri/tauri.conf.json`; portable assembly is handled by `scripts/package-portable.ps1`.
- Portable packages are currently unsigned. Signing is not part of the active release flow.
- Portable packages are expected to include app-local helper binaries:
  - `binaries/ffmpeg.exe`
  - `binaries/pdfium.dll`

### Network and install policy

Filnizer is local-first:

- No telemetry.
- No update checks.
- No remote conversion APIs.
- No browser opening from the app.
- No unconfirmed downloads or background installs.

The only approved runtime network path is **user-confirmed LibreOffice installation** through Windows `winget` when LibreOffice is missing. This is exposed through:

- A first-open prompt in `src/app/App.tsx`.
- A manual Settings action in `src/features/settings/SettingsView.tsx`.
- Backend command `install_libreoffice` in `src-tauri/src/commands/settings.rs`.
- Structured `Command::new("winget").args([...])` invocation in `src-tauri/src/tools/libreoffice.rs`.

Do not add any other runtime network behavior without explicitly updating this architecture document, user-facing wording, tests/checklists, and release docs.

## Technology Stack

### Frontend

- React + TypeScript.
- Vite build pipeline.
- Tauri JS APIs only through typed wrappers in `src/lib/tauri-client.ts`.
- Styling is centralized in `src/styles.css`.
- UI sections:
  - `src/features/organizer/OrganizerView.tsx`
  - `src/features/converter/ConverterView.tsx`
  - `src/features/jobs/JobsView.tsx`
  - `src/features/settings/SettingsView.tsx`

### Backend

- Rust 2021, pinned by `rust-toolchain.toml` and `src-tauri/Cargo.toml` (`rust-version = "1.88"`).
- Tauri 2 command bridge.
- Tokio for blocking/background work orchestration.
- SQLite via `rusqlite` with bundled SQLite.
- Core libraries:
  - `walkdir` for scans.
  - `blake3` for duplicate hashing.
  - `image` for raster image conversion.
  - `csv`, `calamine`, `rust_xlsxwriter` for spreadsheet conversion.
  - `pdfium-render` with app-local Pdfium for PDF conversion.
  - `markdown2pdf` with default features disabled for Markdown-to-PDF.
  - FFmpeg as app-local child process for media conversion.
  - LibreOffice detected locally or installed via user-confirmed `winget` for Office-to-PDF.

## High-Level Architecture

```text
React UI
  └── src/lib/tauri-client.ts typed command wrappers
      └── Tauri commands in src-tauri/src/commands/
          ├── domain DTOs in src-tauri/src/domain/
          ├── repositories in src-tauri/src/db/
          ├── organizer logic in src-tauri/src/organizer/
          ├── converter logic in src-tauri/src/converter/
          └── helper detection/install in src-tauri/src/tools/
```

The frontend does not perform privileged filesystem mutations directly. It asks Rust commands to scan, plan, apply, undo, convert, and persist history.

## Repository Layout

```text
src/
  app/                    React app shell and navigation
  features/               User-facing feature screens
  lib/
    tauri-client.ts       Typed frontend command API
  styles.css              Shared UI styling

src-tauri/
  capabilities/           Tauri permission/capability config
  migrations/             SQLite migrations included at compile time
  src/
    commands/             Tauri command handlers and response shape
    converter/            Conversion planners and adapters
    db/                   SQLite connection and repositories
    domain/               Shared DTOs/enums serialized to frontend
    organizer/            Scan/category/plan/apply/undo/duplicate logic
    tools/                FFmpeg/Pdfium/LibreOffice/WebView2 detection/install
    errors.rs             Frontend-safe app error serialization

docs/
  release/                Release checklist, helper licenses, portable notes
scripts/                  Portable packaging script
```

## Command and DTO Pattern

All Tauri commands return `CommandResult<T>`, which wraps successful data as:

```json
{ "data": ... }
```

The shared wrapper lives in `src-tauri/src/commands/mod.rs`:

- `CommandResponse<T>`
- `CommandResult<T>`

Frontend calls must go through `invokeCommand<T>()` in `src/lib/tauri-client.ts`. Do not scatter raw `invoke()` calls across feature components.

Errors use `AppError` in `src-tauri/src/errors.rs`, serialized with safe `code` and `message` fields. Do not expose internal stack traces or sensitive command output.

## Persistence Model

SQLite is opened from Tauri app-data via `AppDatabase::open_app_data()` in `src-tauri/src/db/mod.rs`.

Migrations are compile-time included:

- `001_initial.sql`
  - `settings`
  - `jobs`
  - `operation_plans`
  - `file_results`
- `002_organizer_rules.sql`
  - `organizer_rules`

Repositories:

- `JobsRepository`
  - Insert/update/list/get jobs.
- `OperationRepository`
  - Persist organizer plans, file results, conversion results, scan results, duplicate results.
- `SettingsRepository`
  - Persist `AppSettings` as JSON under a single settings key.

Settings JSON is serde-defaulted to preserve compatibility when fields are added. New settings fields must have safe defaults in `src-tauri/src/domain/settings.rs`.

## Jobs and History

Job history is persisted for:

- Organizer scans.
- Organizer apply operations.
- Organizer undo operations.
- Duplicate analysis.
- Conversion jobs.

The Jobs UI loads history through:

- `list_jobs`
- `get_job_details`

Backend command file: `src-tauri/src/commands/jobs.rs`.
Frontend screen: `src/features/jobs/JobsView.tsx`.

Per-file details are stored in `file_results`. Status values may come from different domains (`success`, `completed`, `scanned`, `duplicate`, etc.), so Jobs UI treats status as a display label instead of a strict enum.

## Organizer Architecture

Organizer code lives in `src-tauri/src/organizer/`.

Core modules:

- `scan.rs`
  - Walks selected folders and returns `FileEntry` metadata.
  - Emits scan progress events.
- `categories.rs`
  - Infers default file categories.
  - Applies custom rules precedence.
- `rules.rs`
  - Stores user custom organizer rules.
- `rename.rs`
  - Normalizes unsafe Windows filename characters and spacing.
- `planner.rs`
  - Creates preview-only move/rename plans with conflict suffixing.
- `apply.rs`
  - Applies move/rename operations and records per-file results.
- `undo.rs`
  - Reverses applied plans only when original/destination state is safe.
- `duplicates.rs`
  - Groups by size, then hashes with BLAKE3.

Safety rules:

- Organizer actions are preview-first.
- Apply does not overwrite unless explicitly requested by conflict policy.
- Undo refuses to overwrite user-created files at original paths.
- Duplicate detection never deletes files automatically.

Frontend screen: `src/features/organizer/OrganizerView.tsx`.
Commands: `src-tauri/src/commands/organizer.rs`.

## Converter Architecture

Converter code lives in `src-tauri/src/converter/`.

Core modules:

- `planner.rs`
  - Shared output path planning and conflict handling.
  - Tracks conversion job status from per-file results.
- `image.rs`
  - Local image conversion through `image`.
- `spreadsheet.rs`
  - CSV to XLSX and XLSX to CSV.
- `media.rs`
  - FFmpeg child process conversion using structured args.
- `pdf.rs`
  - Pdfium-backed PDF to text/PNG when `pdfium.dll` is app-local.
- `markdown.rs`
  - Markdown to PDF with remote URL rejection.
- `office.rs`
  - LibreOffice headless Office-to-PDF when LibreOffice is detected.

Helper tools:

- `src-tauri/src/tools/ffmpeg.rs`
  - Detects `ffmpeg.exe` beside the executable or under `binaries/`.
- `src-tauri/src/tools/pdfium.rs`
  - Detects `pdfium.dll` beside the executable or under `binaries/`.
- `src-tauri/src/tools/libreoffice.rs`
  - Detects local LibreOffice common install paths or PATH.
  - Offers user-confirmed install through `winget`.
- `src-tauri/src/tools/webview2.rs`
  - Reports common WebView2 runtime detection/guidance.

Frontend screen: `src/features/converter/ConverterView.tsx`.
Commands: `src-tauri/src/commands/converter.rs`.

Converter UI uses one selected conversion mode at a time to avoid excessive scrolling.

## Settings Architecture

Settings are persisted as `AppSettings`:

- `default_output_directory`
- `default_conflict_policy`
- `history_retention_days`
- `show_privacy_note`
- `allow_network_installs`
- `libreoffice_install_prompted`

Frontend screen: `src/features/settings/SettingsView.tsx`.
Commands: `src-tauri/src/commands/settings.rs`.
Repository: `src-tauri/src/db/settings_repository.rs`.

Settings also display helper tool status from `get_converter_tool_status`.

## Portable Packaging

Script: `scripts/package-portable.ps1`.
NPM command: `npm run package:portable`.

The packaging script:

1. Validates helper binaries are present before building.
2. Builds frontend.
3. Builds Tauri executable with `tauri build --no-bundle`.
4. Creates `artifacts/portable/Filnizer`.
5. Copies `Filnizer.exe`.
6. Copies required helpers from `binaries/` or `FILNIZER_HELPER_BINARIES_DIR`:
   - `ffmpeg.exe`
   - `pdfium.dll`
7. Copies release docs/licenses.
8. Creates `artifacts/Filnizer-<version>-portable-windows-x64.zip`.

Do not commit generated `artifacts/`. They are ignored by `.gitignore`.

## Tauri Permissions

Capability file: `src-tauri/capabilities/default.json`.

Currently allows dialog open permission. Filesystem mutations happen in Rust backend commands, not via broad frontend filesystem permissions.

When adding new plugins/permissions:

- Add the narrowest permission possible.
- Explain why it is needed in the PR/commit summary.
- Avoid shell/browser-opening permissions unless explicitly approved.

## Security and Safety Rules

- Use structured process arguments for FFmpeg, LibreOffice, and winget. Never build shell command strings from user input.
- Do not log private file contents, raw tokens, installer secrets, or full sensitive payloads.
- Do not add telemetry, update checks, external browser opening, or remote conversion APIs.
- Any network access beyond user-confirmed LibreOffice installation requires explicit product approval and documentation updates.
- Remote URLs in Markdown conversion are rejected before rendering.
- Database operations must use parameterized queries.
- File operations must preserve current safe-preview/undo behavior.

## Naming and Serialization Conventions

Rust:

- Files/modules: `snake_case`.
- Types/enums: `PascalCase`.
- Fields: `snake_case` in Rust, serialized to `camelCase` with `#[serde(rename_all = "camelCase")]`.
- Tauri command functions: descriptive `snake_case`, typically matching frontend command names.
- Errors: return `Result<T, AppError>` or `CommandResult<T>` at command boundaries.

TypeScript:

- Components: `PascalCase`.
- Functions/variables: `camelCase`.
- Types: `PascalCase`.
- Frontend DTO field names: `camelCase`, matching serde output.
- Tauri command wrappers should live in `src/lib/tauri-client.ts`.

CSS:

- Global styles live in `src/styles.css`.
- Use descriptive class names scoped by feature intent (`converter-*`, `job-*`, `settings-*`).
- Keep layout compact and native-desktop-like rather than web-page-like.

## Verification Commands

Use the smallest relevant command after each change, then broader checks before handing off.

```powershell
npm run build
npm run test -- --run
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

For packaging:

```powershell
npm run package:portable
```

Packaging requires real `ffmpeg.exe` and `pdfium.dll` inputs.

## Known Release Gates

Before publishing a public portable ZIP:

- Fill exact FFmpeg/Pdfium version, source, checksum, and license notes in `docs/release/licenses.md`.
- Run `docs/release/checklist.md` on Windows 10+.
- Confirm WebView2 behavior on a clean Windows machine.
- Confirm LibreOffice install prompt works and does not start without explicit confirmation.
- Decide whether unsigned release warnings are acceptable for the target audience.
