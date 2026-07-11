# Filnizer Release Smoke-Test Checklist

Run this checklist on a clean Windows 10+ machine or VM before publishing a release.

## Portable launch

- [ ] Extract the portable ZIP to a normal user-writable folder.
- [ ] Confirm `Filnizer.exe` launches without an installer.
- [ ] Confirm the app opens without requiring administrator privileges.
- [ ] Confirm the sidebar navigation works: Organizer, Converter, Jobs / History, Settings.

## Offline behavior

- [ ] Start the app with network disconnected and confirm it still opens.
- [ ] Confirm the app does not open a browser automatically.
- [ ] Confirm there are no updater, telemetry, or download prompts.
- [ ] Confirm Settings reports runtime network as disabled.

## Organizer

- [ ] Scan a fixture folder.
- [ ] Add a custom extension rule.
- [ ] Preview an organize plan.
- [ ] Apply the plan.
- [ ] Undo the plan.
- [ ] Confirm unsafe undo is refused if the original path is occupied.
- [ ] Run duplicate detection and confirm no files are deleted automatically.

## Converters

- [ ] Convert a PNG/JPG/WebP image fixture.
- [ ] Convert CSV to XLSX and XLSX to CSV fixtures.
- [ ] With FFmpeg present in `binaries/`, convert a small media fixture.
- [ ] With Pdfium present in `binaries/`, convert a PDF to text or PNG.
- [ ] Convert local Markdown to PDF.
- [ ] Confirm Markdown containing `http://` or `https://` is rejected.
- [ ] With LibreOffice installed locally, convert DOC/DOCX/ODT to PDF.
- [ ] Confirm missing helpers show English guidance and do not trigger downloads.
- [ ] Confirm existing outputs are skipped/renamed/overwritten only according to the selected conflict policy.

## Jobs and Settings

- [ ] Confirm Jobs / History shows scans, organizer actions, duplicate analysis, and conversions.
- [ ] Confirm per-file result rows appear for scan/apply/undo/duplicate/conversion jobs.
- [ ] Save Settings defaults and relaunch; confirm values persist.
- [ ] Confirm helper statuses are shown for FFmpeg, Pdfium, LibreOffice, and WebView2 guidance.

## Signing and artifact review

- [ ] Confirm the release artifact has been signed through the approved SignPath.io flow.
- [ ] Confirm the ZIP contains `Filnizer.exe`, `README.md`, `docs/`, and expected `binaries/` helper files.
- [ ] Confirm license inventory has exact source/version/license entries for bundled helper binaries.
