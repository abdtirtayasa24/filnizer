# Spec: Filnizer Windows Native Offline File Organizer and Converter

## Assumptions

1. Filnizer is a Windows-first desktop application distributed as a portable/no-install executable experience.
2. The application must work fully offline after launch, including media and office conversions.
3. The first release targets Windows 10 and newer.
4. The first release targets local files and folders only; cloud drives are treated as normal filesystem paths when mounted locally.
5. The first release is single-user and stores all history/settings in a local SQLite database.
6. FFmpeg should be bundled with the portable app. LibreOffice should not be bundled; Filnizer should detect a user's local LibreOffice installation and show installation guidance if it is missing.
7. The app must remain strictly network-silent: no telemetry, update checks, remote conversion APIs, automatic downloads, or background network requests.
8. The product language for MVP is English only.
9. The visual direction is a friendly consumer utility, not a dense enterprise/admin tool.
10. All destructive file operations require preview, explicit user confirmation, and MVP undo support where technically safe.
11. This spec defines the product and engineering target before implementation; exact crate/package versions should be pinned when the project is scaffolded.

## Objective

Build **Filnizer**, a Windows native desktop app that helps users clean, organize, rename, deduplicate, and convert local files without uploading them to online tools.

Filnizer should serve users who have messy folders, duplicate downloads, mixed file formats, or recurring conversion tasks and want a privacy-preserving local workflow.

### Primary Users

- Students, office workers, creators, and power users with cluttered Downloads/Desktop/Documents folders.
- Users who need common file conversions but do not want to upload private files to web tools.
- Users who need batch operations with clear previews, progress, history, and undo/safety affordances.

### Core Capabilities

#### 1. File Organizer

Filnizer should help users reorganize messy folders through:

- Folder scanning with recursive and non-recursive modes.
- Automatic categorization into configurable folders such as Images, Documents, Videos, Audio, Archives, Spreadsheets, Presentations, Code, PDFs, and Other.
- Filename cleanup and batch renaming.
- Duplicate detection using content hashing.
- Preview-before-apply for move/rename/delete operations.
- Operation history and undo support for recoverable file operations.

#### 2. File Converter

Filnizer should convert files locally through:

- Image conversions, such as PNG, JPG/JPEG, WEBP, BMP, TIFF, and GIF where supported by selected Rust image crates.
- Spreadsheet conversions, such as CSV to XLSX and XLSX/worksheet export to CSV where supported by selected Rust crates.
- Media conversions using bundled FFmpeg, such as video to audio and common audio/video format transformations.
- Document conversions using detected local LibreOffice headless and other local tools where needed, such as DOC/DOCX to PDF, PDF to image, PDF to text, Markdown to PDF, and related document workflows.
- Batch conversion queues with progress, cancellation, error reporting, and output conflict handling.

### Product Principles

- **Offline-first:** no network calls for normal app behavior.
- **Privacy-first:** files remain on the user's machine.
- **Safe by default:** preview, dry run, confirmation, and non-destructive defaults.
- **Transparent:** show exactly what will happen before changing files.
- **Recoverable:** maintain operation history and provide undo for supported organizer operations.
- **Performant:** handle large folder scans and long conversions without freezing the UI.

## Tech Stack

### Application Shell

- Tauri 2 for Windows desktop shell and secure frontend/backend bridge.
- Tauri command permissions should be scoped to explicit commands needed by the app.

### Frontend

- React with TypeScript.
- Vite for frontend development/build if using the standard Tauri React template.
- CSS modules, Tailwind CSS, or another lightweight styling approach may be chosen during scaffolding; avoid heavy UI dependencies unless approved.

### Backend/Core

- Rust stable.
- Tokio for async job execution, cancellation, and non-blocking orchestration.
- SQLite for local database/history.
- `serde` for serialization between frontend and backend.
- `walkdir` for filesystem walking.
- `notify` for optional file monitoring/watch mode.
- `blake3` for content hashing and duplicate detection.
- `image` crate for common image decoding/encoding.
- `csv`, `calamine`, and `rust_xlsxwriter` for spreadsheet conversions.
- Bundled FFmpeg for media conversions.
- Detected local LibreOffice headless for office document conversions.
- `pdfium-render` with app-local Pdfium files for PDF rendering/text extraction.
- `markdown2pdf` for Markdown to PDF, with network/fetch features disabled.

### Recommended Conversion Backend Crates

The following crate choices and versions are the recommended initial pins based on ecosystem fit, offline operation, Windows viability, and maintenance signals. Use `Cargo.lock` for reproducible application builds; if fully exact manifest pins are desired, use `=x.y.z` in `Cargo.toml`.

| Area | Recommended backend | Initial version | Use | Notes |
| --- | --- | --- | --- | --- |
| Image conversion | `image` | `0.25.10` | PNG/JPEG/WEBP/BMP/TIFF/GIF and related raster conversion | Official docs show broad common-format support and `DynamicImage` save APIs. Prefer explicit feature flags to keep binary size controlled. |
| CSV parsing/writing | `csv` | `1.4.0` | CSV read/write and delimiter handling | Mature standard Rust CSV crate; pair with `calamine`/`rust_xlsxwriter`. |
| Spreadsheet read | `calamine` | `0.35.0` | XLS/XLSX/XLSM/XLSB/ODS to CSV/export workflows | Read-only pure Rust spreadsheet reader; good fit for worksheet export. |
| Spreadsheet write | `rust_xlsxwriter` | `0.95.0` | CSV to XLSX and generated XLSX workbooks | Write-only XLSX library. Consider `constant_memory` feature for large generated workbooks. |
| PDF render/text | `pdfium-render` | `0.9.0` | PDF to image and PDF text extraction | Ship Pdfium app-local beside `Filnizer.exe` for portable PDF workflows. Use the thread-safe/default image integration unless prototype results require narrower features. |
| Markdown to PDF | `markdown2pdf` | `1.5.0` | MD to PDF | Pure Rust Markdown-to-PDF pipeline. Do not enable the optional `fetch` feature. SVG support may be enabled only if local SVG rendering is required. |
| Media conversion | Bundled FFmpeg | Pin exact FFmpeg release during packaging | Video/audio conversion and extraction | Ship app-local beside `Filnizer.exe`; invoke as child process with structured args and no shell interpolation. |
| Office conversion | Detected LibreOffice headless | Detect installed LibreOffice version at runtime | DOC/DOCX/PPT/PPTX/ODT/ODS to PDF where supported | Do not bundle in MVP. Detect installation and show English setup guidance if missing. |

### Recommended Core Rust Crate Pins

| Purpose | Crate | Initial version / features | Notes |
| --- | --- | --- | --- |
| Tauri shell | `tauri` | `2.11.5` | Match JS Tauri packages to the same Tauri v2 generation. Disable unused capabilities. |
| Tauri build | `tauri-build` | `2.6.3` | Build dependency aligned with Tauri 2.11.x. |
| Native file/folder dialogs | `tauri-plugin-dialog` | `2.7.1` | Use for user-selected roots/destinations. |
| Frontend filesystem API | `tauri-plugin-fs` | `2.5.1` | Keep permissions narrow. Prefer Rust backend for privileged file operations. |
| Async runtime/processes | `tokio` | `1.52.3` with `rt-multi-thread`, `macros`, `process`, `fs`, `sync`, `time` | Avoid enabling `net` unless explicitly needed; the app should stay network-silent. |
| SQLite | `rusqlite` | `0.40.1` with `bundled` | Bundled SQLite avoids system SQLite dependency for portable Windows builds. |
| Serialization | `serde` / `serde_json` | `serde 1.0.228` with `derive`; `serde_json 1.0.150` | DTOs, persisted job metadata, and settings. |
| Directory walking | `walkdir` | `2.5.0` | Recursive scans. |
| Optional watch mode | `notify` | `8.2.0` | Use stable release, not `9.0.0-rc.*`, unless a later stable exists during implementation. |
| Hashing | `blake3` | `1.8.5` | Enable `rayon` only if duplicate hashing benchmarks justify it. |
| Errors | `thiserror` / `anyhow` | `thiserror 2.0.18`; `anyhow 1.0.103` | Use `thiserror` for app/domain errors; `anyhow` for internal orchestration where useful. |
| Job IDs | `uuid` | `1.23.4` with `v7`, `serde` | Sortable UUIDv7 IDs are useful for job/history records. |
| Recycle Bin | `trash` | `5.2.6` | Use for optional move-to-Recycle-Bin behavior; permanent delete remains separately confirmed. |
| Tests/temp dirs | `tempfile` | `3.27.0` | Test fixtures and safe integration tests. |

Recommended Rust toolchain: pin `rust-toolchain.toml` to stable Rust `1.88.0` or newer because selected crates such as `image` require Rust 1.88.

Research sources checked: crates.io/docs.rs/GitHub pages for `tauri`, `tauri-build`, Tauri plugins, `tokio`, `rusqlite`, `image`, `csv`, `calamine`, `rust_xlsxwriter`, `pdfium-render`, `markdown2pdf`, `notify`, `blake3`, `serde`, `serde_json`, `thiserror`, `anyhow`, `uuid`, `trash`, and `tempfile`.

### Packaging and Distribution

- Primary distribution target is portable/no-install Windows usage.
- The release artifact should be a portable ZIP/folder, not a literal single `.exe`, because Tauri WebView2 and helper-binary constraints make app-local files necessary.
- User-facing goal: user extracts the ZIP/folder, double-clicks `Filnizer.exe`, and the app opens without an installer.
- `Filnizer.exe`, bundled FFmpeg, app-local Pdfium files, frontend assets, licenses, and required helper files should live in the portable folder.
- FFmpeg should be shipped app-local beside the executable or in an app-local `binaries/` directory.
- Pdfium should be shipped app-local beside the executable or in an app-local `binaries/pdfium/` directory for PDF workflows.
- LibreOffice should be detected from the user's system and never bundled in MVP.
- If LibreOffice is missing, Filnizer should show English instructions and a visible official LibreOffice URL, but must not automatically download, install, open a browser, or make network requests.
- Windows 10 WebView2 runtime availability must be validated. If Tauri requires WebView2 and it is missing, Filnizer should fail gracefully with clear guidance rather than silently making network requests.
- Release builds should be code-signed through SignPath.io using its Open Source project flow.
- Signing should happen in the release CI pipeline; do not store signing certificates/private keys in the repository or local developer machines.
- Sign at minimum `Filnizer.exe`; also sign bundled executable helper binaries when legally/technically appropriate.

## Commands

These commands are the expected developer interface after scaffolding the project.

```bash
# Install JavaScript dependencies
npm install

# Run app in development mode
npm run tauri dev

# Build frontend only
npm run build

# Run frontend linting
npm run lint

# Run frontend tests
npm run test -- --run

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run Rust formatting check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Run Rust linting
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings

# Build Windows portable desktop app artifact
npm run tauri build

# Assemble portable ZIP/folder after Tauri build
npm run package:portable

# Submit release artifact to SignPath.io signing workflow
npm run sign:signpath
```

If the project adopts `pnpm`, `yarn`, `bun`, a Rust workspace, a different Tauri layout, or a dedicated portable packaging script, this section must be updated before implementation.

## Project Structure

Proposed repository layout:

```text
.
├── docs/
│   ├── spec.md                         # This product/engineering specification
│   ├── architecture.md                 # Architecture notes after initial scaffolding
│   └── adr/                            # Architectural decision records
├── src/                                # React + TypeScript frontend
│   ├── app/                            # App shell, routing, global providers
│   ├── components/                     # Reusable UI components
│   ├── features/
│   │   ├── organizer/                  # Organizer screens/state/API wrappers
│   │   ├── converter/                  # Converter screens/state/API wrappers
│   │   ├── jobs/                       # Job progress/history UI
│   │   └── settings/                   # App settings UI
│   ├── lib/                            # Shared frontend utilities
│   ├── styles/                         # Global styles/theme tokens
│   └── test/                           # Frontend test utilities
├── src-tauri/                          # Tauri/Rust application
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── binaries/                       # Portable app-local helper binaries such as FFmpeg/Pdfium if licensing permits
│   ├── migrations/                     # SQLite migrations
│   ├── capabilities/                   # Tauri 2 capability definitions
│   └── src/
│       ├── main.rs                     # App bootstrap
│       ├── commands/                   # Tauri command handlers
│       ├── db/                         # SQLite connection, migrations, repositories
│       ├── domain/                     # Shared domain types and rules
│       ├── organizer/                  # Scan, categorize, rename, duplicate logic
│       ├── converter/                  # Conversion orchestration and adapters
│       ├── jobs/                       # Async job queue, progress, cancellation
│       ├── fs/                         # Safe filesystem helpers
│       ├── settings/                   # Local configuration handling
│       └── errors.rs                   # App error types and serialization
├── tests/                              # Cross-layer or integration test fixtures
├── tasks/
│   ├── plan.md                         # Implementation plan after spec approval
│   └── todo.md                         # Ordered implementation tasks after planning
├── package.json
├── tsconfig.json
└── README.md
```

## Functional Requirements

### App Shell and Navigation

- The app must expose primary sections for Organizer, Converter, Jobs/History, and Settings.
- The UI must remain responsive while scans and conversions run.
- Long-running operations must show progress, current file, elapsed time, and final status.
- Users must be able to cancel queued/running jobs where the underlying operation supports safe cancellation.

### File Organizer Requirements

#### Folder Scan

- User can select one or more local folders.
- User can choose recursive or top-level-only scanning.
- User can exclude hidden files, system files, folders, glob patterns, or file extensions.
- Scan results should include file path, filename, extension, size, modified timestamp, inferred category, and hash status when hashing is enabled.
- Scanning must avoid blocking the UI.

#### Categorization

- Filnizer should infer categories using extension and, where feasible, MIME/content sniffing.
- Default categories should include:
  - Images
  - Documents
  - PDFs
  - Spreadsheets
  - Presentations
  - Videos
  - Audio
  - Archives
  - Code
  - Executables
  - Other
- Users should be able to preview a target folder structure before applying organization.
- Custom category rules are included in MVP.
- MVP custom rules should allow users to map extensions and simple filename patterns to destination categories/folders.
- Custom rules must be previewed before apply and stored locally in SQLite/settings.

#### Rename Cleanup

- User can preview cleaned filenames before applying changes.
- Basic filename cleanup should support:
  - trimming leading/trailing whitespace,
  - replacing repeated spaces/separators,
  - normalizing separator style,
  - removing unsafe characters,
  - optionally adding date or sequence suffixes,
  - preserving file extension casing consistently.
- Renaming must handle conflicts deterministically, for example with `filename (1).ext` suffixes.

#### Duplicate Detection

- Duplicate detection should use a staged approach:
  1. group by file size,
  2. hash candidate groups with BLAKE3,
  3. report duplicate sets.
- The app must not delete duplicates automatically.
- User can select duplicates to move to a review folder, recycle bin, or delete only after explicit confirmation.
- Deletion behavior must prefer Windows Recycle Bin if implemented; permanent deletion requires a separate confirmation.

#### Apply Operations

- File move/rename operations must run from a generated operation plan.
- The operation plan must be displayed before execution.
- The operation plan should be persisted in history before or during execution so failures can be audited.
- Operation plans for move/rename actions should include enough inverse information to support undo where the destination/source state is still safe.
- Partial failures should not abort unrelated operations unless continuing would risk data loss.
- Undo must be previewed before execution and must refuse unsafe reversal if files have changed, disappeared, or would overwrite newer user data.

### File Converter Requirements

#### Common Conversion Behavior

- User can select one or more input files.
- User can choose output format and destination folder.
- User can preview output filenames and conflict behavior.
- Conversion jobs must report status per file.
- Conversion failures must include user-readable error messages and technical details in local logs/history.
- Output files must not overwrite existing files unless the user explicitly chooses overwrite.
- Supported formats should be visible in the UI based on available conversion backends.

#### Image Conversion

- Supported MVP conversions should include common raster formats supported by the chosen Rust image crate.
- Options may include quality, resize, and preserve metadata only if implementation support is reliable.
- Unsupported color profiles, animated formats, and metadata preservation limitations must be communicated clearly.

#### Spreadsheet Conversion

- CSV to XLSX should be supported in MVP if crate support is stable.
- XLSX worksheet export to CSV should be supported if selected library permits reliable parsing.
- The UI should allow selecting delimiter/encoding options for CSV only if required for correctness.
- Large spreadsheets should be processed in a memory-conscious way where library support permits.

#### Media Conversion

- Media conversion should use FFmpeg as a local child process.
- The app must validate that FFmpeg is present and usable at startup or when media conversion is first used.
- MVP media conversions should include:
  - video to MP3 or AAC audio,
  - video container conversion where safe,
  - audio format conversion.
- FFmpeg invocation must avoid shell string interpolation; pass arguments as structured process arguments.
- Progress should be parsed from FFmpeg output when feasible.

#### Document Conversion

- Office conversions should use a detected local LibreOffice headless installation where needed.
- The app must validate LibreOffice availability and show setup/status in Settings.
- If LibreOffice is missing, the app must show English installation guidance without automatically downloading or opening network connections.
- MVP document conversions should prioritize:
  - DOC/DOCX to PDF,
  - Markdown to PDF,
  - PDF to images,
  - PDF to text.
- Not all document conversions need to be implemented through LibreOffice; choose the safest local backend per format.
- Conversion limitations must be explicit, especially layout differences and font availability.

### Jobs, History, and Database

- Each scan, organization apply, duplicate analysis, and conversion should be represented as a local job.
- Jobs should have statuses such as queued, running, completed, failed, canceled, and partially completed.
- SQLite should store:
  - job metadata,
  - operation plans,
  - per-file results,
  - user settings,
  - tool availability status where useful.
- History UI should allow users to inspect past operations.
- Sensitive file contents must never be stored in SQLite.
- File paths are local personal data and should be treated carefully in logs and exports.

### Settings

- Settings should include:
  - default output directory behavior,
  - conflict handling defaults,
  - FFmpeg availability/status,
  - LibreOffice availability/status,
  - database/history retention preferences,
  - privacy/offline status.
- Settings should not require account login.

## Non-Functional Requirements

### Offline and Privacy

- The app must not upload files to any remote service.
- The app must not make network requests during organizer/converter/settings/history operations.
- No update checks, telemetry, remote documentation fetches, external link opening, or automatic downloads are allowed in MVP.

### Safety

- Destructive actions require explicit confirmation.
- Move/rename/delete operations require preview.
- Default behavior should create new outputs rather than overwriting existing files.
- File operation code must protect against invalid paths, path traversal from generated names, and accidental writes outside selected destinations.

### Performance

- Large folder scans must stream progress and avoid loading unnecessary file contents.
- Hashing should be limited to duplicate candidate groups when possible.
- Conversion jobs should run with controlled concurrency to avoid exhausting CPU, memory, or disk I/O.
- UI updates should be throttled if high-frequency backend events cause rendering pressure.

### Reliability

- Jobs should handle partial failure and persist enough information to explain what happened.
- The app should recover gracefully after restart and show jobs that were interrupted.
- External tool failures should not crash the app.

### Accessibility and UX

- Primary workflows should be usable by keyboard.
- Buttons and controls should have accessible labels.
- Error states must be clear and actionable.
- The app should support readable contrast and scalable text.
- The visual design should feel like a friendly consumer utility: approachable language, clear empty states, simple guided workflows, and non-alarming safety prompts.

### Security

- Tauri command handlers must validate all frontend-provided input.
- Backend code should canonicalize paths where appropriate before filesystem operations.
- Child processes must be invoked with argument arrays, not shell-concatenated commands.
- Logs must not include file contents or secrets.
- Bundled binaries must have licensing reviewed and versions tracked.

## Code Style

### TypeScript/React Conventions

- Use TypeScript for all frontend code.
- Prefer function components and explicit props types.
- Keep Tauri command wrappers in feature-specific API modules.
- Avoid passing raw `invoke` calls throughout UI components.
- Use clear domain names such as `OrganizerScan`, `ConversionJob`, and `DuplicateSet`.

Example frontend style:

```tsx
type ConversionJobCardProps = {
  job: ConversionJobSummary;
  onCancel: (jobId: string) => void;
};

export function ConversionJobCard({ job, onCancel }: ConversionJobCardProps) {
  const isRunning = job.status === "running";

  return (
    <article aria-label={`Conversion job ${job.name}`}>
      <header>
        <h3>{job.name}</h3>
        <span>{job.status}</span>
      </header>

      <progress value={job.completedFiles} max={job.totalFiles} />

      {isRunning ? (
        <button type="button" onClick={() => onCancel(job.id)}>
          Cancel
        </button>
      ) : null}
    </article>
  );
}
```

### Rust Conventions

- Use typed domain structs for operation plans and results.
- Return `Result<T, AppError>` from fallible backend functions.
- Keep Tauri command handlers thin; place business logic in domain modules/services.
- Use `serde::{Serialize, Deserialize}` for command request/response DTOs.
- Avoid panics in production paths.

Example Rust style:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct StartScanRequest {
    pub roots: Vec<PathBuf>,
    pub recursive: bool,
    pub include_hidden: bool,
}

#[derive(Debug, Serialize)]
pub struct StartScanResponse {
    pub job_id: String,
}

#[tauri::command]
pub async fn start_organizer_scan(
    request: StartScanRequest,
    app_state: tauri::State<'_, AppState>,
) -> Result<StartScanResponse, AppError> {
    let job_id = app_state.organizer.start_scan(request).await?;
    Ok(StartScanResponse { job_id })
}
```

### Naming

- Frontend files: `kebab-case.tsx` for components/modules unless the scaffold chooses another convention consistently.
- React components: `PascalCase`.
- TypeScript variables/functions: `camelCase`.
- Rust modules/files: `snake_case`.
- Rust types/enums/traits: `PascalCase`.
- Rust functions/variables: `snake_case`.

## Testing Strategy

### Frontend Tests

- Use a React-compatible test runner such as Vitest with React Testing Library.
- Test UI behavior, forms, command wrapper error handling, and accessibility basics.
- Keep tests near feature code or in a consistent `__tests__`/`test` layout once the project is scaffolded.

### Rust Unit Tests

- Unit test organizer logic without touching real user files by using temporary directories and fixtures.
- Unit test filename normalization, conflict resolution, category inference, duplicate grouping, and operation plan generation.
- Unit test command input validation where practical.

### Rust Integration Tests

- Use temporary directories for filesystem operation tests.
- Test SQLite repositories/migrations against temporary databases.
- Test conversion adapters with small fixture files.
- External tool tests for FFmpeg/LibreOffice should be skippable or clearly gated when tools are unavailable in CI.

### End-to-End Tests

- Add E2E tests after the core workflows stabilize.
- Prioritize smoke tests for launching the app, selecting fixtures, previewing operations, and completing a small conversion.

### Fixtures

Fixtures should be small, synthetic, and safe to commit:

```text
tests/fixtures/
├── organizer/
│   ├── messy-names/
│   └── duplicates/
├── images/
├── spreadsheets/
├── media/
└── documents/
```

### Coverage Expectations

- High coverage for pure organizer logic and file operation planning.
- Moderate coverage for conversion orchestration.
- UI tests should cover primary user flows rather than implementation details.
- Safety-critical code, such as delete/move/overwrite logic, must have regression tests before release.

## Boundaries

### Always Do

- Keep all workflows offline/local and network-silent.
- Validate frontend-provided paths and options in Rust before use.
- Preview move/rename/delete/overwrite plans before execution.
- Use explicit user confirmation for destructive operations.
- Run relevant frontend and Rust tests before marking implementation complete.
- Prefer structured process arguments for FFmpeg/LibreOffice invocations.
- Store operation history without storing file contents.
- Provide undo support for MVP move/rename operations where safe.
- Preserve user files unless the user explicitly confirms a destructive operation.

### Ask First

- Adding new dependencies beyond the agreed stack.
- Changing database schema after data may exist for users.
- Introducing telemetry, crash reporting, update checks, automatic downloads, or any network behavior.
- Bundling large third-party binaries that substantially increase portable artifact size.
- Switching away from portable/no-install distribution.
- Adding cloud storage integrations.
- Implementing permanent deletion as a default action.
- Supporting non-Windows platforms in the first release.

### Never Do

- Upload user files to remote services for conversion or analysis.
- Make automatic network requests, including update checks or dependency downloads.
- Log raw file contents, secrets, tokens, or private document text.
- Delete files without explicit user confirmation.
- Overwrite existing files without explicit user-selected overwrite behavior.
- Execute shell-concatenated commands containing user-controlled paths.
- Store real user files inside the app database.
- Commit proprietary or license-incompatible binaries without review.
- Modify files outside user-selected roots/destinations except app-owned config/cache/history directories.

## Success Criteria

### Product Success Criteria

- A user can scan a messy folder and see categorized file results without the UI freezing.
- A user can preview and apply a safe organization plan that moves files into category folders.
- A user can preview filename cleanup and apply deterministic renames with conflict handling.
- A user can detect duplicate files using BLAKE3-based content hashing.
- A user can convert supported images locally.
- A user can convert supported spreadsheets locally.
- A user can convert supported media files locally through FFmpeg.
- A user can convert supported documents locally through detected LibreOffice headless or another local backend.
- A user can define basic custom organizer rules and see them applied in previews.
- A user can undo supported organizer move/rename operations when reversal is safe.
- A user can view job status/history for scans, organization operations, duplicate analysis, and conversions.
- The app can be distributed as a Windows 10+ portable/no-install executable experience.

### Engineering Success Criteria

- Core filesystem operation planning is covered by automated tests.
- Destructive operations are impossible without preview and confirmation in the implemented flow.
- Backend Tauri commands validate inputs and return structured errors.
- Long-running operations use async jobs and emit progress events.
- SQLite migrations can initialize a fresh local database.
- FFmpeg availability is validated from the bundled/app-local copy before dependent conversions run.
- LibreOffice availability checks are implemented before dependent conversions run.
- Build, lint, formatting, and relevant tests pass before release.

### Release Readiness Criteria

- Portable Windows ZIP/folder artifact builds successfully and launches on Windows 10+ without an installer.
- `Filnizer.exe` is signed through SignPath.io Open Source signing for release builds.
- License obligations for bundled dependencies/binaries are documented.
- The app works without internet access for all MVP workflows.
- Basic accessibility checks are completed for primary workflows.
- Manual smoke tests pass for organizer, duplicate detection, and at least one conversion in each supported category.

## MVP Scope

### Included in MVP

- Windows 10+ portable desktop app shell with Organizer, Converter, Jobs/History, and Settings.
- Folder scan with recursive option and category inference.
- Basic custom organizer rules for extensions and filename patterns.
- Organization plan preview, apply, and safe undo.
- Basic filename cleanup preview, apply, and safe undo.
- Duplicate detection with BLAKE3.
- Image conversion for common formats supported by selected Rust crates.
- CSV/XLSX conversion for supported spreadsheet workflows.
- FFmpeg-backed media conversion for selected audio/video workflows.
- Detected-LibreOffice-backed office-to-PDF conversion where feasible.
- Missing-LibreOffice guidance without automatic download.
- SQLite job/history database.
- Portable Windows ZIP/folder packaging proof of concept.
- SignPath.io Open Source signing workflow for release builds.

### Deferred Until After MVP

- Cloud drive APIs beyond locally mounted folders.
- AI-based file classification.
- OCR.
- Advanced custom rule builder beyond simple extension/pattern rules.
- Cross-platform packaging for macOS/Linux.
- Automatic background folder monitoring as a default workflow.
- Any telemetry, update checks, automatic downloads, or runtime network behavior.
- Collaborative or multi-user features.

## Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Portable folder/ZIP has more files than a single `.exe` | Users may move/delete helper files accidentally | Keep folder structure simple, include a clear `README.txt`, and make `Filnizer.exe` validate required app-local helpers at startup |
| LibreOffice may be missing from user systems | Office conversions unavailable | Detect local install, show clear English setup guidance, and disable dependent actions until available |
| FFmpeg/Pdfium licensing and redistribution complexity | Legal/release blocker | Track exact binaries, licenses, and redistribution terms early |
| SignPath.io Open Source signing setup may require project approval/review | Release signing delay | Start SignPath onboarding before first public release and keep unsigned dev builds separate from signed release artifacts |
| Document conversion output differs from original layout | User dissatisfaction | Communicate limitations, test fixtures, preserve source files |
| Large scans/hashes consume CPU and disk I/O | App feels slow | Stage duplicate hashing, throttle concurrency, stream progress |
| File operations fail due to permissions/locks | Partial completion | Persist operation plan/results, show actionable errors, continue safely where possible |
| Tauri command surface too broad | Security issue | Keep commands narrow and validate all inputs in Rust |
| User accidentally deletes important files | Data loss | No automatic deletion, confirmation, recycle bin preference, history |

## Resolved Decisions

1. FFmpeg is bundled/app-local in the portable artifact.
2. LibreOffice is detected from the user's system and not bundled in MVP.
3. The distribution model is a portable ZIP/folder, not a literal single-file `.exe` and not MSI/NSIS/MSIX installer-first.
4. Pdfium is shipped app-local beside the executable or under the app-local `binaries/` folder for PDF workflows.
5. Custom organizer rules are included in MVP.
6. Undo for safe move/rename organizer operations is included in MVP.
7. Recommended conversion crates/backends are `image 0.25.10`, `csv 1.4.0`, `calamine 0.35.0`, `rust_xlsxwriter 0.95.0`, `pdfium-render 0.9.0`, `markdown2pdf 1.5.0`, bundled FFmpeg, and detected LibreOffice.
8. Recommended core Rust pins include `tauri 2.11.5`, `tokio 1.52.3`, `rusqlite 0.40.1`, `serde 1.0.228`, `serde_json 1.0.150`, `walkdir 2.5.0`, `notify 8.2.0`, `blake3 1.8.5`, `thiserror 2.0.18`, `anyhow 1.0.103`, `uuid 1.23.4`, `trash 5.2.6`, and `tempfile 3.27.0`.
9. Minimum supported OS is Windows 10.
10. The app remains strictly network-silent.
11. The design direction is a friendly consumer utility.
12. MVP language is English.
13. Release code signing uses SignPath.io through its Open Source project flow.

## Remaining Validation Items

1. Prototype the portable ZIP/folder build to confirm exact Tauri output paths and app-local helper lookup paths.
2. Validate app-local Pdfium loading on a clean Windows 10 machine.
3. Run `cargo audit` and license review after dependency scaffolding; adjust versions if any selected crate introduces unacceptable advisories or licenses.
4. Complete SignPath.io Open Source onboarding before first public signed release.

## Future Planning Notes

After this spec is approved, create:

- `tasks/plan.md` with the implementation strategy, dependency order, risks, and verification checkpoints.
- `tasks/todo.md` with small vertical-slice tasks, each with acceptance criteria and verification commands.
- ADRs for portable distribution strategy, FFmpeg/LibreOffice/Pdfium distribution, SQLite schema approach, and conversion backend selection.
