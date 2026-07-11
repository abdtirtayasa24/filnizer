# AGENTS.md

This file is the operating guide for future agents and contributors working on Filnizer.

Before changing code, read:

1. `README.md`
2. `docs/ARCHITECTURE.md`
3. This `AGENTS.md`

## Project Summary

Filnizer is a portable Tauri desktop app for organizing local files, detecting duplicates, safely undoing organizer operations, and converting files locally.

Core stack:

- Frontend: React + TypeScript + Vite.
- Desktop shell: Tauri 2.
- Backend/core: Rust + Tokio.
- Database: SQLite through `rusqlite` with bundled SQLite.
- Portable helpers: FFmpeg and Pdfium bundled in `binaries/` for release packages.
- LibreOffice: detected locally; may be installed through Windows `winget` only after explicit user confirmation.

## Current Safety Model

Filnizer is local-first.

Allowed:

- Local filesystem scanning/organizing/conversion after user chooses files/folders.
- App-local FFmpeg/Pdfium helper execution.
- Local LibreOffice detection.
- User-confirmed LibreOffice installation through Windows `winget`.

Not allowed without explicit product approval:

- Telemetry.
- Update checks.
- Remote conversion APIs.
- Browser opening from the app.
- Unconfirmed downloads or background installs.
- Any new network path beyond user-confirmed LibreOffice installation.

If you change this model, update `docs/ARCHITECTURE.md`, `README.md`, release docs, UI copy, and tests/checklists in the same change.

## Incremental Workflow

Work in small, reversible increments.

For each slice:

1. Inspect relevant files.
2. Make the smallest complete change.
3. Run the smallest relevant verification command.
4. Commit only if the user asks or the current workflow explicitly requires it.
5. Report exactly what changed and what was verified.

Do not mix unrelated refactors with feature changes.

## Git Rules

- Check `git status --short` before editing.
- Do not overwrite user changes.
- Do not commit unless explicitly asked.
- Do not amend, rebase, reset, force-push, or delete branches unless explicitly asked.
- Keep commits atomic when commits are requested.
- Never commit generated `artifacts/`, `dist/`, or `src-tauri/target/` outputs.

## Frontend Rules

### Structure

- App shell: `src/app/App.tsx`.
- Feature screens:
  - `src/features/organizer/OrganizerView.tsx`
  - `src/features/converter/ConverterView.tsx`
  - `src/features/jobs/JobsView.tsx`
  - `src/features/settings/SettingsView.tsx`
- Typed Tauri API wrapper: `src/lib/tauri-client.ts`.
- Global styling: `src/styles.css`.

### Tauri Calls

- Do not call `invoke()` directly from feature components.
- Add typed wrappers in `src/lib/tauri-client.ts`.
- Keep TypeScript DTOs aligned with Rust serde `camelCase` output.
- Use `formatCommandError()` for user-facing command errors.

### UI Style

- The UI should feel like a compact native desktop utility, not a long web page.
- Prefer dense but readable cards, clear controls, and minimal scrolling.
- Keep accessibility basics: labels, button semantics, focus states, readable error text.
- Do not add heavy UI libraries unless explicitly approved.

### Naming

- Components/types: `PascalCase`.
- Functions/variables: `camelCase`.
- CSS classes: descriptive kebab-case grouped by feature, e.g. `converter-*`, `job-*`, `settings-*`.

## Rust Backend Rules

### Structure

- Commands: `src-tauri/src/commands/`.
- Domain DTOs/enums: `src-tauri/src/domain/`.
- DB repositories: `src-tauri/src/db/`.
- Organizer logic: `src-tauri/src/organizer/`.
- Converter logic: `src-tauri/src/converter/`.
- Helper detection/install: `src-tauri/src/tools/`.
- Error serialization: `src-tauri/src/errors.rs`.

### Command Pattern

- Tauri commands return `CommandResult<T>`.
- Successful command payloads are wrapped in `CommandResponse<T>`.
- Frontend receives `{ data: ... }` and unwraps it in `invokeCommand<T>()`.
- Use `AppError` for frontend-safe errors.

### Naming

- Modules/files/functions: `snake_case`.
- Types/enums: `PascalCase`.
- Rust struct fields: `snake_case`.
- Serialized DTOs: `#[serde(rename_all = "camelCase")]`.

### Error Handling

- Do not `unwrap()` in production code.
- Convert external errors into `AppError` at boundaries.
- Keep error messages useful but do not expose secrets or private file contents.

### External Processes

- Use `std::process::Command` or Tokio process APIs with structured `.args([...])`.
- Never interpolate user-controlled data into shell strings.
- Existing external process integrations:
  - FFmpeg media conversion.
  - LibreOffice office conversion.
  - Winget LibreOffice installation after explicit user confirmation.

## Database Rules

- SQLite migrations live in `src-tauri/migrations/` and are included from `AppDatabase::run_migrations()`.
- Use repositories under `src-tauri/src/db/`; do not scatter SQL through commands.
- Use parameterized queries only.
- Existing settings are JSON under a single `app_settings` key; new fields must have serde defaults.
- Preserve backward compatibility for old settings JSON when adding fields.

## Organizer Rules

- Organizer operations must remain preview-first.
- Apply must not overwrite unless the selected conflict policy allows it.
- Undo must refuse unsafe overwrites.
- Duplicate detection must never delete files automatically.
- Keep tests for scan, categories, planner, apply, undo, rules, and duplicates when changing behavior.

## Converter Rules

- Use shared output/conflict planning in `converter/planner.rs`.
- Conversions should return per-file results instead of failing whole batches when possible.
- FFmpeg and Pdfium are expected in app-local `binaries/` for portable releases.
- Markdown conversion must reject remote URLs and keep `markdown2pdf` fetch features disabled.
- LibreOffice conversion depends on local detection; install flow is separate and user-confirmed.

## Packaging and Release Rules

- Portable packaging script: `scripts/package-portable.ps1`.
- `npm run package:portable` requires:
  - `binaries/ffmpeg.exe`
  - `binaries/pdfium.dll`
- Alternative helper input folder: `FILNIZER_HELPER_BINARIES_DIR`.
- Release builds are currently unsigned.
- Keep helper binary provenance/license notes in `docs/release/licenses.md` before publishing.
- Use `docs/release/checklist.md` before release.

## Verification Commands

Frontend/type build:

```powershell
npm run build
```

Rust tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Rust formatting:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Rust linting:

```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Full configured test command:

```powershell
npm run test -- --run
```

Portable packaging:

```powershell
npm run package:portable
```

If helper binaries are missing, packaging should fail early with a clear message.

## Documentation Rules

- Keep `README.md` concise and user/developer facing.
- Keep `docs/ARCHITECTURE.md` as the primary technical orientation.
- Keep this file focused on contributor/agent rules.
- Update release docs under `docs/release/` when packaging, helper, install, or release behavior changes.
- Do not recreate bulky task/spec planning docs unless the user explicitly asks.

## Definition of Done

Before reporting completion:

- Relevant code/docs are updated.
- Changes are limited to the requested scope.
- Formatting/lint/build/tests are run when relevant.
- Any verification that could not run is explained clearly.
- No secrets or generated artifacts are included.
- Final response lists changed files and verification results.
