use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use uuid::Uuid;

use crate::domain::files::{FileCategory, FileEntry};
use crate::domain::operations::{
    ConflictPolicy, OperationKind, OperationPlan, OperationPlanStatus, PlannedOperation,
};
use crate::errors::AppError;
use crate::organizer::rename::clean_filename;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewOrganizerPlanRequest {
    pub files: Vec<FileEntry>,
    pub destination_root: String,
    pub clean_filenames: bool,
}

pub fn preview_organizer_plan(
    request: &PreviewOrganizerPlanRequest,
) -> Result<OperationPlan, AppError> {
    validate_request(request)?;

    let destination_root = PathBuf::from(&request.destination_root);
    let mut reserved_targets = HashSet::new();
    let mut operations = Vec::new();

    for file in &request.files {
        let source_path = PathBuf::from(&file.path);
        let original_name = source_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(&file.name);
        let file_name = if request.clean_filenames {
            clean_filename(original_name)
        } else {
            original_name.to_string()
        };
        let category_folder = category_folder_name(&file.category);
        let target_directory = destination_root.join(category_folder);
        let target_path = unique_target_path(&target_directory, &file_name, &mut reserved_targets);

        operations.push(PlannedOperation {
            id: Uuid::now_v7().to_string(),
            kind: if file_name == original_name {
                OperationKind::Move
            } else {
                OperationKind::Rename
            },
            source_path: source_path.to_string_lossy().to_string(),
            target_path: target_path.to_string_lossy().to_string(),
            conflict_policy: ConflictPolicy::Rename,
        });
    }

    Ok(OperationPlan {
        id: Uuid::now_v7().to_string(),
        job_id: None,
        status: OperationPlanStatus::Preview,
        operations,
    })
}

fn validate_request(request: &PreviewOrganizerPlanRequest) -> Result<(), AppError> {
    if request.files.is_empty() {
        return Err(AppError::validation(
            "Scan files before previewing an organizer plan",
        ));
    }

    if request.destination_root.trim().is_empty() {
        return Err(AppError::validation(
            "Choose a destination folder for the plan",
        ));
    }

    Ok(())
}

fn unique_target_path(
    target_directory: &Path,
    file_name: &str,
    reserved_targets: &mut HashSet<String>,
) -> PathBuf {
    let (stem, extension) = split_extension(file_name);
    let mut index = 0;

    loop {
        let candidate_name = if index == 0 {
            file_name.to_string()
        } else if let Some(extension) = extension {
            format!("{} ({}).{}", stem, index, extension)
        } else {
            format!("{} ({})", stem, index)
        };
        let candidate = target_directory.join(candidate_name);
        let candidate_key = candidate.to_string_lossy().to_lowercase();

        if !reserved_targets.contains(&candidate_key) && !candidate.exists() {
            reserved_targets.insert(candidate_key);
            return candidate;
        }

        index += 1;
    }
}

fn split_extension(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && !extension.is_empty() => {
            (stem, Some(extension))
        }
        _ => (name, None),
    }
}

fn category_folder_name(category: &FileCategory) -> &'static str {
    match category {
        FileCategory::Images => "Images",
        FileCategory::Documents => "Documents",
        FileCategory::Pdfs => "PDFs",
        FileCategory::Spreadsheets => "Spreadsheets",
        FileCategory::Presentations => "Presentations",
        FileCategory::Videos => "Videos",
        FileCategory::Audio => "Audio",
        FileCategory::Archives => "Archives",
        FileCategory::Code => "Code",
        FileCategory::Executables => "Executables",
        FileCategory::Other => "Other",
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::files::{FileCategory, FileEntry, HashStatus};
    use crate::domain::operations::{OperationKind, OperationPlanStatus};
    use crate::organizer::planner::{preview_organizer_plan, PreviewOrganizerPlanRequest};

    #[test]
    fn preview_plan_groups_by_category_and_cleans_names() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plan = preview_organizer_plan(&PreviewOrganizerPlanRequest {
            files: vec![file_entry(
                "C:/Input/Q1___Report.PDF",
                "Q1___Report.PDF",
                FileCategory::Pdfs,
            )],
            destination_root: temp_dir.path().to_string_lossy().to_string(),
            clean_filenames: true,
        })
        .unwrap();

        assert_eq!(plan.status, OperationPlanStatus::Preview);
        assert_eq!(plan.operations.len(), 1);
        assert_eq!(plan.operations[0].kind, OperationKind::Rename);
        assert!(
            plan.operations[0]
                .target_path
                .ends_with("PDFs\\Q1 Report.pdf")
                || plan.operations[0]
                    .target_path
                    .ends_with("PDFs/Q1 Report.pdf")
        );
    }

    #[test]
    fn preview_plan_adds_conflict_suffixes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plan = preview_organizer_plan(&PreviewOrganizerPlanRequest {
            files: vec![
                file_entry("C:/Input/photo.jpg", "photo.jpg", FileCategory::Images),
                file_entry("C:/Other/photo.jpg", "photo.jpg", FileCategory::Images),
            ],
            destination_root: temp_dir.path().to_string_lossy().to_string(),
            clean_filenames: true,
        })
        .unwrap();

        assert!(plan.operations[0].target_path.ends_with("photo.jpg"));
        assert!(plan.operations[1].target_path.ends_with("photo (1).jpg"));
    }

    fn file_entry(path: &str, name: &str, category: FileCategory) -> FileEntry {
        FileEntry {
            path: path.to_string(),
            name: name.to_string(),
            extension: None,
            size_bytes: 1,
            modified_unix_ms: None,
            category,
            hash_status: HashStatus::NotRequested,
        }
    }
}
