use std::path::{Path, PathBuf};

use crate::tools::ffmpeg::ToolStatus;

pub fn webview2_status() -> ToolStatus {
    let path = find_webview2_runtime();
    ToolStatus {
        name: "Microsoft Edge WebView2 Runtime",
        available: path.is_some() || !cfg!(windows),
        path: path.map(|value| value.to_string_lossy().to_string()),
        guidance: Some(
            "Install Microsoft Edge WebView2 Runtime if Filnizer cannot open on Windows 10.",
        ),
    }
}

pub fn find_webview2_runtime() -> Option<PathBuf> {
    common_webview2_locations()
        .into_iter()
        .find_map(|candidate| find_webview2_in(&candidate))
}

pub fn find_webview2_in(base_dir: &Path) -> Option<PathBuf> {
    let direct = base_dir.join("msedgewebview2.exe");
    if direct.is_file() {
        return Some(direct);
    }

    let application_dir = base_dir.join("Application");
    if !application_dir.is_dir() {
        return None;
    }

    std::fs::read_dir(application_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("msedgewebview2.exe"))
        .find(|candidate| candidate.is_file())
}

fn common_webview2_locations() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("Microsoft")
                .join("EdgeWebView"),
        );
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("Microsoft")
                .join("EdgeWebView"),
        );
    }
    candidates
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::tools::webview2::find_webview2_in;

    #[test]
    fn detects_webview2_under_versioned_application_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let version_dir = temp_dir.path().join("Application").join("123.0.0.0");
        fs::create_dir_all(&version_dir).unwrap();
        let runtime = version_dir.join("msedgewebview2.exe");
        fs::write(&runtime, b"placeholder").unwrap();

        assert_eq!(find_webview2_in(temp_dir.path()), Some(runtime));
    }
}
