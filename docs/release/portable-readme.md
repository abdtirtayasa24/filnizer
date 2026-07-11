# Filnizer Portable

Filnizer is a local/offline Windows desktop utility for organizing and converting files.

## Run

1. Extract the ZIP into a normal folder such as `C:\Tools\Filnizer`.
2. Launch `Filnizer.exe`.
3. Keep the bundled `binaries` folder beside `Filnizer.exe`.

## Offline behavior

Filnizer is designed to run without telemetry, update checks, browser opening, or remote conversion APIs. LibreOffice installation may use network access only after explicit user confirmation.

## Bundled helper tools

- FFmpeg is bundled app-local under `binaries/ffmpeg.exe` for media conversion.
- Pdfium is bundled app-local under `binaries/pdfium.dll` for PDF conversion.
- LibreOffice is detected from your system installation for Office-to-PDF conversion; if missing, Filnizer can offer to install it through Windows winget after explicit user confirmation.

If LibreOffice is missing and the user declines installation, Filnizer shows English guidance in Settings and Office-to-PDF conversion fails safely.
