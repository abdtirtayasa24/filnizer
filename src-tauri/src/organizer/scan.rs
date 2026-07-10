use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

use crate::domain::files::{FileCategory, FileEntry, HashStatus};
use crate::errors::AppError;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub roots: Vec<String>,
    pub recursive: bool,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub job_id: String,
    pub current_path: String,
    pub scanned_files: u64,
}

pub fn scan_folders(
    request: &ScanRequest,
    job_id: &str,
    app: Option<&AppHandle>,
) -> Result<ScanResult, AppError> {
    validate_scan_request(request)?;

    let mut files = Vec::new();
    for root in &request.roots {
        let root_path = PathBuf::from(root);
        let max_depth = if request.recursive { usize::MAX } else { 1 };

        for entry in WalkDir::new(root_path).max_depth(max_depth) {
            let entry = entry.map_err(|error| AppError::Filesystem(error.to_string()))?;
            let path = entry.path();

            if !request.include_hidden && is_hidden(path) {
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let file_entry = file_entry_from_path(path)?;
            files.push(file_entry);

            if let Some(app) = app {
                let _ = app.emit(
                    "organizer://scan-progress",
                    ScanProgressEvent {
                        job_id: job_id.to_string(),
                        current_path: path.to_string_lossy().to_string(),
                        scanned_files: files.len() as u64,
                    },
                );
            }
        }
    }

    Ok(ScanResult { files })
}

fn validate_scan_request(request: &ScanRequest) -> Result<(), AppError> {
    if request.roots.is_empty() {
        return Err(AppError::validation("Select at least one folder to scan"));
    }

    for root in &request.roots {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(AppError::validation(format!(
                "Folder does not exist: {}",
                root_path.display()
            )));
        }

        if !root_path.is_dir() {
            return Err(AppError::validation(format!(
                "Path is not a folder: {}",
                root_path.display()
            )));
        }
    }

    Ok(())
}

fn file_entry_from_path(path: &Path) -> Result<FileEntry, AppError> {
    let metadata = fs::metadata(path)?;
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    Ok(FileEntry {
        path: path.to_string_lossy().to_string(),
        name,
        extension,
        size_bytes: metadata.len(),
        modified_unix_ms,
        category: FileCategory::Other,
        hash_status: HashStatus::NotRequested,
    })
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{scan_folders, ScanRequest};

    #[test]
    fn scan_folders_returns_file_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("photo.jpg"), b"image").unwrap();
        fs::create_dir(temp_dir.path().join("nested")).unwrap();
        fs::write(temp_dir.path().join("nested").join("note.txt"), b"note").unwrap();

        let result = scan_folders(
            &ScanRequest {
                roots: vec![temp_dir.path().to_string_lossy().to_string()],
                recursive: true,
                include_hidden: false,
            },
            "job-1",
            None,
        )
        .unwrap();

        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().any(|file| file.name == "photo.jpg"));
        assert!(result.files.iter().any(|file| file.name == "note.txt"));
    }

    #[test]
    fn scan_folders_respects_top_level_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("root.txt"), b"root").unwrap();
        fs::create_dir(temp_dir.path().join("nested")).unwrap();
        fs::write(temp_dir.path().join("nested").join("nested.txt"), b"nested").unwrap();

        let result = scan_folders(
            &ScanRequest {
                roots: vec![temp_dir.path().to_string_lossy().to_string()],
                recursive: false,
                include_hidden: false,
            },
            "job-1",
            None,
        )
        .unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "root.txt");
    }
}
