# License Inventory

This inventory tracks release-time licensing for Filnizer and bundled helper binaries.

## Application source

- Filnizer app source: repository license to be confirmed before public release.
- Rust dependencies: review with `cargo metadata`, `cargo tree`, and a license/audit tool before release.
- JavaScript dependencies: review `package-lock.json` / npm metadata before release.

## Bundled helper binaries

No helper binaries are committed in this repository at this time. When release maintainers add app-local helper binaries to the portable package, fill in the exact version, source URL, checksum, and license below before publishing.

| Component | Bundled? | Required for | Version | Source | License notes |
| --- | --- | --- | --- | --- | --- |
| FFmpeg | Planned app-local | Media conversion | TBD | TBD | Confirm chosen FFmpeg build license/configuration before bundling. |
| Pdfium | Planned app-local | PDF conversion | TBD | TBD | Confirm binary source and redistribution terms before bundling. |
| LibreOffice | No | Office-to-PDF | User-installed | Local system install | Filnizer detects LibreOffice; it does not bundle or download it. |
| Microsoft Edge WebView2 Runtime | No | Tauri UI runtime on Windows | User/system-installed | Local system install | Filnizer does not bundle or download WebView2. |

## Release gate

Do not publish a portable ZIP with bundled FFmpeg or Pdfium until this file includes exact binary provenance and license notes for the bundled files.
