use std::path::{Path, PathBuf};

use crate::tools::ffmpeg::ToolStatus;

pub fn pdfium_status() -> ToolStatus {
    let path = find_app_local_pdfium();
    ToolStatus {
        name: "Pdfium",
        available: path.is_some(),
        path: path.map(|value| value.to_string_lossy().to_string()),
        guidance: Some("Portable releases should include Pdfium at binaries/pdfium.dll."),
    }
}

pub fn find_app_local_pdfium() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))?;
    find_pdfium_in(&exe_dir)
}

pub fn find_pdfium_in(base_dir: &Path) -> Option<PathBuf> {
    let library_name = if cfg!(windows) {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };
    let candidates = [
        base_dir.join(library_name),
        base_dir.join("binaries").join(library_name),
    ];

    candidates.into_iter().find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::tools::pdfium::find_pdfium_in;

    #[test]
    fn detects_pdfium_under_binaries_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let binaries = temp_dir.path().join("binaries");
        fs::create_dir(&binaries).unwrap();
        let library = binaries.join(if cfg!(windows) {
            "pdfium.dll"
        } else if cfg!(target_os = "macos") {
            "libpdfium.dylib"
        } else {
            "libpdfium.so"
        });
        fs::write(&library, b"placeholder").unwrap();

        assert_eq!(find_pdfium_in(temp_dir.path()), Some(library));
    }
}
