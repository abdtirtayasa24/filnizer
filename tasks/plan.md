# Implementation Plan: Filnizer

## Overview

Build Filnizer as a Windows 10+ portable/no-install Tauri 2 desktop application with React + TypeScript UI and Rust core logic. The app will provide offline/local file organization, duplicate detection, safe undo for organizer operations, and local file conversion using Rust libraries plus app-local FFmpeg/Pdfium and detected system LibreOffice.

This plan is for review before implementation. No dependency installation or code scaffolding has been performed yet.

## Architecture Decisions

- **Distribution:** Portable ZIP/folder, not a single executable and not an installer. `Filnizer.exe`, app-local helper binaries, licenses, assets, and required runtime files live in one portable folder.
- **UI shell:** Tauri 2 + React + TypeScript. UI calls narrow Rust commands rather than directly performing privileged filesystem operations.
- **Core logic:** Rust modules own scanning, rule evaluation, operation planning, conversion orchestration, jobs, SQLite persistence, and tool detection.
- **Jobs:** Long-running scan/conversion/apply operations run through a job service with progress events, cancellation where safe, and SQLite history.
- **Safety model:** Organizer actions are plan-first. Move/rename actions persist inverse operations for safe undo. Destructive actions require explicit confirmation.
- **Conversion backends:** `image`, `csv`, `calamine`, `rust_xlsxwriter`, `pdfium-render` with app-local Pdfium, `markdown2pdf` with no fetch feature, app-local FFmpeg, and detected system LibreOffice.
- **Network posture:** Strictly network-silent at runtime. No telemetry, update checks, automatic downloads, browser opening, remote docs fetches, or remote conversion APIs.
- **Signing:** Release artifacts are signed via SignPath.io Open Source flow in CI. Signing secrets/certificates are never stored in the repo.

## Recommended Initial Rust Pins

- `tauri = "2.11.5"`
- `tauri-build = "2.6.3"`
- `tauri-plugin-dialog = "2.7.1"`
- `tauri-plugin-fs = "2.5.1"`
- `tokio = "1.52.3"` with `rt-multi-thread`, `macros`, `process`, `fs`, `sync`, `time`
- `rusqlite = "0.40.1"` with `bundled`
- `serde = "1.0.228"` with `derive`
- `serde_json = "1.0.150"`
- `walkdir = "2.5.0"`
- `notify = "8.2.0"`
- `blake3 = "1.8.5"`
- `image = "0.25.10"`
- `csv = "1.4.0"`
- `calamine = "0.35.0"`
- `rust_xlsxwriter = "0.95.0"`
- `pdfium-render = "0.9.0"`
- `markdown2pdf = "1.5.0"` without `fetch`
- `thiserror = "2.0.18"`
- `anyhow = "1.0.103"`
- `uuid = "1.23.4"` with `v7`, `serde`
- `trash = "5.2.6"`
- `tempfile = "3.27.0"`

## Dependency Graph

```text
Project scaffold + pinned toolchain
  ├── Tauri permissions/capabilities
  ├── Rust domain types/errors/settings paths
  │   ├── SQLite migrations/repositories
  │   │   ├── job/history service
  │   │   │   ├── organizer scan/rules/plans/apply/undo
  │   │   │   └── converter adapters/jobs
  │   │   └── settings/tool status UI
  │   └── frontend command clients
  │       ├── Organizer UI
  │       ├── Converter UI
  │       ├── Jobs/History UI
  │       └── Settings UI
  └── Portable packaging + signing
```

## Implementation Strategy

Use vertical slices after a small foundation:

1. Scaffold app and lock toolchain/dependencies.
2. Establish Rust domain, SQLite, job service, and frontend command wrapper patterns.
3. Build organizer end-to-end first because it exercises safety-critical filesystem planning/history/undo.
4. Add conversion adapters one category at a time with small fixtures.
5. Add settings/tool detection and portable packaging/signing last, once helper-binary paths are known.

## Phase 1: Foundation

- [ ] Task 1: Scaffold Tauri 2 + React + TypeScript project
- [ ] Task 2: Pin Rust/JS dependencies and baseline quality commands
- [ ] Task 3: Define Rust app errors, domain DTOs, and command response pattern
- [ ] Task 4: Add SQLite initialization, migrations, and settings/job repositories
- [ ] Task 5: Add frontend shell, navigation, and command-client convention

### Checkpoint: Foundation

- [ ] `npm run build` passes
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passes
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passes
- [ ] Basic app launches in Tauri dev mode

## Phase 2: Organizer Vertical Slice

- [ ] Task 6: Implement scan job with folder selection and progress events
- [ ] Task 7: Implement category inference and simple custom rule storage
- [ ] Task 8: Implement organizer preview plan for move/rename operations
- [ ] Task 9: Implement apply plan with persisted per-file results
- [ ] Task 10: Implement safe undo for move/rename operations
- [ ] Task 11: Implement duplicate detection using size grouping and BLAKE3
- [ ] Task 12: Implement organizer UI workflows end-to-end

### Checkpoint: Organizer

- [ ] User can scan fixture folder and see categories
- [ ] User can create a simple custom rule and see it affect preview
- [ ] User can apply a move/rename plan and safely undo it
- [ ] Duplicate fixture set is detected correctly
- [ ] Safety-critical planner/apply/undo tests pass

## Phase 3: Conversion Vertical Slices

- [ ] Task 13: Add conversion job model and common output/conflict planning
- [ ] Task 14: Implement image conversion adapter and UI flow
- [ ] Task 15: Implement CSV/XLSX spreadsheet conversion adapters and UI flow
- [ ] Task 16: Add FFmpeg app-local detection and media conversion adapter
- [ ] Task 17: Add app-local Pdfium detection and PDF to image/text adapters
- [ ] Task 18: Add Markdown to PDF adapter with network/fetch disabled
- [ ] Task 19: Add LibreOffice detection and office-to-PDF adapter

### Checkpoint: Converters

- [ ] At least one fixture conversion works for image, spreadsheet, media, PDF, Markdown, and LibreOffice-backed office if LibreOffice is available
- [ ] Missing helper tools disable dependent actions with clear English guidance
- [ ] Conversion jobs show per-file success/failure and do not overwrite without explicit choice

## Phase 4: Jobs, Settings, Packaging, Release Readiness

- [ ] Task 20: Complete Jobs/History UI with persisted operation details
- [ ] Task 21: Complete Settings UI for defaults, helper status, and history retention
- [ ] Task 22: Add portable folder/ZIP packaging script
- [ ] Task 23: Add startup validation for app-local FFmpeg/Pdfium and WebView2 guidance
- [ ] Task 24: Add SignPath.io signing workflow documentation/config placeholders
- [ ] Task 25: Add release smoke-test checklist and license inventory

### Checkpoint: Release Candidate

- [ ] Portable ZIP/folder launches on Windows 10+
- [ ] `Filnizer.exe` validates app-local helpers and shows clear missing-tool guidance
- [ ] App runs offline with no intended network behavior
- [ ] Release signing path is documented and ready for SignPath.io onboarding
- [ ] License inventory covers bundled FFmpeg/Pdfium and Rust/JS dependencies

## Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Tauri portable layout differs from expectation | Packaging rework | Prototype packaging immediately after scaffold |
| Pdfium DLL loading fails on clean Windows | PDF features blocked | Validate app-local lookup early with a small PDF fixture |
| LibreOffice not installed | Office conversion unavailable | Detect and clearly disable/guide instead of failing late |
| FFmpeg/Pdfium licensing complexity | Release blocker | Track exact binary sources and licenses before bundling |
| Undo overwrites user changes | Data loss | Compare expected state before undo; refuse unsafe reversal |
| Large scans/conversions freeze UI | Poor UX | Use jobs, bounded concurrency, progress throttling |
| Runtime network call slips in through dependency/plugin | Privacy violation | Avoid Tauri updater/shell URL open; review features; test with network monitor before release |
| SignPath onboarding takes time | Delayed signed release | Start onboarding before public release candidate |

## Parallelization Opportunities

Safe after foundation:

- Organizer UI can proceed in parallel with Rust organizer logic once DTOs are stable.
- Converter adapters can be split by format family after common conversion job model exists.
- Packaging/signing documentation can proceed while core features are implemented.
- Fixture creation and smoke-test checklist can proceed in parallel with feature work.

Must be sequential:

- Scaffold before dependency pinning/commands.
- Domain DTOs before frontend command clients.
- SQLite/job service before history and long-running workflows.
- Common conversion planning before individual converter UI flows.

## Remaining Validation Items

- Confirm exact portable Tauri output paths on Windows.
- Confirm app-local Pdfium loading strategy on clean Windows 10.
- Run `cargo audit` and license review after scaffold.
- Confirm SignPath.io Open Source project requirements and CI integration details.

## Implementation Gate

Do not start implementation until this plan and `tasks/todo.md` are reviewed and approved.
