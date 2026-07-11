# License Inventory

This inventory tracks release-time licensing for Filnizer and bundled helper binaries.

## Application source

- Filnizer app source: repository license to be confirmed before public release.
- Rust dependencies: review with `cargo metadata`, `cargo tree`, and a license/audit tool before release.
- JavaScript dependencies: review `package-lock.json` / npm metadata before release.

## Bundled helper binaries

Portable releases are expected to include app-local FFmpeg and Pdfium under `Filnizer/binaries/`.

Before publishing a release, fill in the exact version, source URL, checksum, and license notes for the actual binaries copied into the package.

| Component | Bundled? | Required for | Version | Source | License notes |
| --- | --- | --- | --- | --- | --- |
| FFmpeg (`binaries/ffmpeg.exe`) | Yes | Media conversion | 8.1.2 | https://www.videohelp.com/software/ffmpeg | Confirm chosen FFmpeg build license/configuration before bundling. LGPL/GPL obligations depend on build options. |
| Pdfium (`binaries/pdfium.dll`) | Yes | PDF conversion | 146.0.7651.0 | https://www.dllme.com/dll/files/pdfium/bb8e602ea2dc9aa192f1fe9af0b359a2 | Confirm binary source, checksum, and redistribution terms before bundling. |
| LibreOffice | No | Office-to-PDF | User-installed or user-confirmed winget install | Local system install | Filnizer detects LibreOffice and may install it through Windows winget only after explicit confirmation. |
| Microsoft Edge WebView2 Runtime | No | Tauri UI runtime on Windows | User/system-installed | Local system install | Filnizer does not bundle or download WebView2. |

## Packaging input

`npm run package:portable` expects helper binaries in `binaries/` by default:

```text
binaries/ffmpeg.exe
binaries/pdfium.dll
```

Alternatively, set `FILNIZER_HELPER_BINARIES_DIR` to a folder containing those files.

## Release gate

Do not publish a portable ZIP until this file includes exact binary provenance and license notes for bundled FFmpeg and Pdfium.
