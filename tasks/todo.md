# Todo: Filnizer Implementation Tasks

## Phase 1: Foundation

- [x] Task 1: Scaffold Tauri 2 + React + TypeScript project
  - Acceptance: App opens in Tauri dev mode with a minimal Filnizer window; generated files follow standard Tauri layout; no runtime network/updater feature is enabled.
  - Verify: `npm run tauri dev` launches; `npm run build` passes.
  - Files: `package.json`, `src/`, `src-tauri/`, `index.html`, `vite.config.*`
  - Dependencies: None

- [x] Task 2: Pin Rust/JS dependencies and baseline quality commands
  - Acceptance: `rust-toolchain.toml` pins Rust 1.88+; `Cargo.toml` includes approved crate pins/features; package scripts include build/test/lint/package placeholders.
  - Verify: `cargo metadata --manifest-path src-tauri/Cargo.toml` succeeds; `npm run build` succeeds.
  - Files: `rust-toolchain.toml`, `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `package.json`
  - Dependencies: Task 1

- [x] Task 3: Define Rust app errors, domain DTOs, and command response pattern
  - Acceptance: Shared Rust request/response types exist for jobs, file entries, operation plans, settings, and conversion requests; errors serialize to frontend-safe messages.
  - Verify: `cargo test --manifest-path src-tauri/Cargo.toml` passes.
  - Files: `src-tauri/src/errors.rs`, `src-tauri/src/domain/*`, `src-tauri/src/commands/*`
  - Dependencies: Task 2

- [x] Task 4: Add SQLite initialization, migrations, and settings/job repositories
  - Acceptance: Fresh app creates local SQLite DB; migrations create settings, jobs, operation plans, and file results tables; repository tests use temp DB.
  - Verify: `cargo test --manifest-path src-tauri/Cargo.toml db` passes.
  - Files: `src-tauri/migrations/*`, `src-tauri/src/db/*`, `src-tauri/src/settings/*`
  - Dependencies: Task 3

- [x] Task 5: Add frontend shell, navigation, and command-client convention
  - Acceptance: UI has Organizer, Converter, Jobs/History, Settings sections; command wrappers are centralized and typed.
  - Verify: `npm run build` passes; manual navigation check in dev mode.
  - Files: `src/app/*`, `src/features/*`, `src/lib/tauri-client.ts`, `src/styles/*`
  - Dependencies: Task 3

## Checkpoint: Foundation

- [x] Run `npm run build`
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] Human review before organizer implementation

## Phase 2: Organizer

- [x] Task 6: Implement scan job with folder selection and progress events
  - Acceptance: User-selected folder scan returns file metadata with recursive option; progress events include current file/count; scan does not block UI.
  - Verify: Rust tests with temp directory; manual scan of fixture folder.
  - Files: `src-tauri/src/organizer/scan.rs`, `src-tauri/src/jobs/*`, `src-tauri/src/commands/organizer.rs`, `src/features/organizer/*`
  - Dependencies: Tasks 4, 5

- [x] Task 7: Implement category inference and simple custom rule storage
  - Acceptance: Default categories work by extension; users can store extension/pattern rules locally; rules affect scan/preview output.
  - Verify: Unit tests for category inference and rule precedence.
  - Files: `src-tauri/src/organizer/categories.rs`, `src-tauri/src/organizer/rules.rs`, `src-tauri/src/settings/*`, `src/features/organizer/*`
  - Dependencies: Task 6

- [x] Task 8: Implement organizer preview plan for move/rename operations
  - Acceptance: Preview produces deterministic target paths, filename cleanup, conflict suffixes, and no filesystem mutations.
  - Verify: Unit tests for rename normalization, conflict handling, and target folder plans.
  - Files: `src-tauri/src/organizer/planner.rs`, `src-tauri/src/organizer/rename.rs`, `src-tauri/src/fs/*`, `tests/fixtures/organizer/*`
  - Dependencies: Task 7

- [x] Task 9: Implement apply plan with persisted per-file results
  - Acceptance: Move/rename plan applies with per-file success/failure; partial failures are persisted; no overwrite occurs unless explicitly selected.
  - Verify: Integration tests using temp directories.
  - Files: `src-tauri/src/organizer/apply.rs`, `src-tauri/src/fs/*`, `src-tauri/src/db/*`, `src-tauri/src/commands/organizer.rs`
  - Dependencies: Task 8

- [x] Task 10: Implement safe undo for move/rename operations
  - Acceptance: Applied move/rename plans can be undone when source/destination state is unchanged; unsafe undo is refused with explanation.
  - Verify: Integration tests for successful undo and refused unsafe undo.
  - Files: `src-tauri/src/organizer/undo.rs`, `src-tauri/src/db/*`, `src-tauri/src/fs/*`, `src/features/jobs/*`
  - Dependencies: Task 9

- [x] Task 11: Implement duplicate detection using size grouping and BLAKE3
  - Acceptance: Duplicate sets are detected by size grouping then BLAKE3 hashing; app never deletes duplicates automatically.
  - Verify: Unit/integration tests with duplicate fixtures.
  - Files: `src-tauri/src/organizer/duplicates.rs`, `src-tauri/src/jobs/*`, `src-tauri/src/commands/organizer.rs`, `tests/fixtures/organizer/duplicates/*`
  - Dependencies: Task 6

- [x] Task 12: Implement organizer UI workflows end-to-end
  - Acceptance: UI supports scan, custom rules, preview, apply, undo, duplicate detection, and clear safety confirmations.
  - Verify: `npm run build`; manual fixture workflow.
  - Files: `src/features/organizer/*`, `src/features/jobs/*`, `src/components/*`, `src/lib/*`
  - Dependencies: Tasks 7-11

## Checkpoint: Organizer

- [x] Run Rust organizer tests
- [x] Run frontend build
- [ ] Manual fixture: scan → preview → apply → undo
- [ ] Manual fixture: duplicate detection
- [ ] Human review before converter implementation

## Phase 3: Converters

- [x] Task 13: Add conversion job model and common output/conflict planning
  - Acceptance: Conversion requests create jobs; output path planning handles skip/rename/overwrite modes; per-file statuses persist.
  - Verify: Unit tests for output conflict planning and job status transitions.
  - Files: `src-tauri/src/converter/mod.rs`, `src-tauri/src/converter/planner.rs`, `src-tauri/src/jobs/*`, `src-tauri/src/db/*`
  - Dependencies: Task 4

- [x] Task 14: Implement image conversion adapter and UI flow
  - Acceptance: Supported image inputs convert to selected output formats locally; errors are per-file and user-readable.
  - Verify: Fixture image conversion tests; manual UI conversion.
  - Files: `src-tauri/src/converter/image.rs`, `src-tauri/src/commands/converter.rs`, `src/features/converter/*`, `tests/fixtures/images/*`
  - Dependencies: Task 13

- [x] Task 15: Implement CSV/XLSX spreadsheet conversion adapters and UI flow
  - Acceptance: CSV to XLSX and XLSX worksheet to CSV work for fixtures; delimiter/worksheet choices are handled for MVP.
  - Verify: Spreadsheet fixture tests; manual UI conversion.
  - Files: `src-tauri/src/converter/spreadsheet.rs`, `src-tauri/src/commands/converter.rs`, `src/features/converter/*`, `tests/fixtures/spreadsheets/*`
  - Dependencies: Task 13

- [x] Task 16: Add FFmpeg app-local detection and media conversion adapter
  - Acceptance: App detects app-local FFmpeg; media actions are disabled if missing; conversions use structured args and report progress where feasible.
  - Verify: Tool detection test; gated media fixture test when FFmpeg exists.
  - Files: `src-tauri/src/converter/media.rs`, `src-tauri/src/tools/ffmpeg.rs`, `src/features/settings/*`, `tests/fixtures/media/*`
  - Dependencies: Task 13

- [x] Task 17: Add app-local Pdfium detection and PDF to image/text adapters
  - Acceptance: App loads app-local Pdfium; PDF to image/text works for fixtures; missing Pdfium disables PDF actions clearly.
  - Verify: Tool detection test; PDF fixture conversion tests.
  - Files: `src-tauri/src/converter/pdf.rs`, `src-tauri/src/tools/pdfium.rs`, `src/features/converter/*`, `tests/fixtures/documents/*`
  - Dependencies: Task 13

- [x] Task 18: Add Markdown to PDF adapter with network/fetch disabled
  - Acceptance: Local Markdown fixture converts to PDF; no URL/fetch feature is enabled; remote image/url inputs are rejected or ignored safely.
  - Verify: Markdown fixture test; inspect `Cargo.toml` features.
  - Files: `src-tauri/src/converter/markdown.rs`, `src-tauri/Cargo.toml`, `src/features/converter/*`, `tests/fixtures/documents/*`
  - Dependencies: Task 13

- [x] Task 19: Add LibreOffice detection and office-to-PDF adapter
  - Acceptance: App detects local LibreOffice; DOC/DOCX to PDF action is enabled only when available; missing state shows English setup guidance without opening network.
  - Verify: Tool detection tests; gated conversion test when LibreOffice exists.
  - Files: `src-tauri/src/tools/libreoffice.rs`, `src-tauri/src/converter/office.rs`, `src/features/settings/*`, `src/features/converter/*`
  - Dependencies: Task 13

## Checkpoint: Converters

- [x] Run converter tests
- [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] Manual fixture conversions for available backends
- [ ] Confirm no conversion overwrites without explicit user choice

## Phase 4: Jobs, Settings, Packaging, Release Readiness

- [x] Task 20: Complete Jobs/History UI with persisted operation details
  - Acceptance: Jobs/History displays scans, organizer applies/undo, duplicate analysis, and conversions with per-file details.
  - Verify: Manual workflow creates visible history entries; frontend build passes.
  - Files: `src/features/jobs/*`, `src-tauri/src/jobs/*`, `src-tauri/src/commands/jobs.rs`, `src-tauri/src/db/*`
  - Dependencies: Tasks 12, 19

- [x] Task 21: Complete Settings UI for defaults, helper status, and history retention
  - Acceptance: Settings shows output/conflict defaults, FFmpeg/Pdfium/LibreOffice status, privacy/network-silent note, and retention settings.
  - Verify: Manual settings check; settings repository tests.
  - Files: `src/features/settings/*`, `src-tauri/src/settings/*`, `src-tauri/src/tools/*`, `src-tauri/src/commands/settings.rs`
  - Dependencies: Tasks 16, 17, 19

- [x] Task 22: Add portable folder/ZIP packaging script
  - Acceptance: Script assembles `Filnizer.exe`, app-local helpers, licenses, README, and required assets into a portable folder/ZIP.
  - Verify: `npm run package:portable` creates expected artifact on Windows build machine.
  - Files: `package.json`, `scripts/package-portable.*`, `src-tauri/tauri.conf.json`, `docs/release/*`
  - Dependencies: Tasks 16, 17

- [ ] Task 23: Add startup validation for app-local FFmpeg/Pdfium and WebView2 guidance
  - Acceptance: Startup/tool status detects missing helper files and WebView2-related failures with clear English guidance and no network calls.
  - Verify: Manual run with helpers temporarily removed; unit tests for path resolution.
  - Files: `src-tauri/src/tools/*`, `src-tauri/src/settings/*`, `src/features/settings/*`, `docs/release/*`
  - Dependencies: Task 22

- [ ] Task 24: Add SignPath.io signing workflow documentation/config placeholders
  - Acceptance: Release docs describe SignPath.io Open Source flow and CI placeholders; no signing secrets are committed.
  - Verify: Static review; secret scan for signing credentials.
  - Files: `docs/release/signing.md`, `.github/workflows/*` or CI docs, `package.json`
  - Dependencies: Task 22

- [ ] Task 25: Add release smoke-test checklist and license inventory
  - Acceptance: Release checklist covers portable launch, offline behavior, organizer, conversions, helper detection, signing, and bundled binary licenses.
  - Verify: Static review; checklist can be followed on Windows 10+.
  - Files: `docs/release/checklist.md`, `docs/release/licenses.md`, `README.md`
  - Dependencies: Tasks 22-24

## Final Checkpoint

- [ ] `npm run build` passes
- [ ] `npm run test -- --run` passes if frontend tests exist
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passes
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passes
- [ ] `cargo audit` reviewed
- [ ] Portable ZIP/folder launches on Windows 10+
- [ ] Network-silent behavior manually checked before release
- [ ] SignPath.io signing path verified for release artifact
