use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub name: &'static str,
    pub available: bool,
    pub path: Option<String>,
    pub guidance: Option<&'static str>,
}

pub fn ffmpeg_status() -> ToolStatus {
    let path = find_app_local_ffmpeg();
    ToolStatus {
        name: "FFmpeg",
        available: path.is_some(),
        path: path.map(|value| value.to_string_lossy().to_string()),
        guidance: Some("Place ffmpeg.exe beside Filnizer.exe or under the app binaries folder."),
    }
}

pub fn find_app_local_ffmpeg() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))?;
    find_ffmpeg_in(&exe_dir)
}

pub fn find_ffmpeg_in(base_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        base_dir.join("ffmpeg.exe"),
        base_dir.join("binaries").join("ffmpeg.exe"),
        base_dir.join("ffmpeg"),
        base_dir.join("binaries").join("ffmpeg"),
    ];

    candidates.into_iter().find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::tools::ffmpeg::find_ffmpeg_in;

    #[test]
    fn detects_ffmpeg_under_binaries_folder() {
        let temp_dir = tempfile::tempdir().unwrap();
        let binaries = temp_dir.path().join("binaries");
        fs::create_dir(&binaries).unwrap();
        let ffmpeg = binaries.join(if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        });
        fs::write(&ffmpeg, b"placeholder").unwrap();

        assert_eq!(find_ffmpeg_in(temp_dir.path()), Some(ffmpeg));
    }
}
