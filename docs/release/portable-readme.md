# Filnizer Portable

Filnizer is a local/offline Windows desktop utility for organizing and converting files.

## Run

1. Extract the ZIP into a normal folder such as `C:\Tools\Filnizer`.
2. Launch `Filnizer.exe`.
3. Keep the `binaries` folder beside `Filnizer.exe` when helper tools are included.

## Offline behavior

Filnizer is designed to run without telemetry, update checks, automatic downloads, browser opening, or remote conversion APIs.

## Optional helper tools

- FFmpeg: enables media conversion when bundled app-local.
- Pdfium: enables PDF conversion when bundled app-local.
- LibreOffice: detected from your system installation for Office-to-PDF conversion; it is not downloaded by Filnizer.

If a helper is missing, Filnizer shows English guidance in Settings and dependent conversions fail safely.
