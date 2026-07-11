# WebView2 Runtime Guidance

Filnizer uses Tauri on Windows, which requires Microsoft Edge WebView2 Runtime to display the desktop UI.

## Runtime behavior

- Filnizer does not download WebView2.
- Filnizer does not open a browser or installer URL.
- When the app is running, Settings reports whether a common local WebView2 Runtime path was detected.

## User guidance

If `Filnizer.exe` does not open on Windows 10, install Microsoft Edge WebView2 Runtime using your normal trusted software deployment process, then launch Filnizer again.

For portable distribution, do not place WebView2 installers or download links inside the app unless a release maintainer has reviewed licensing and distribution requirements.
