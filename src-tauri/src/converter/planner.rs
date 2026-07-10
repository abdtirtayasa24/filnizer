use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::domain::jobs::JobStatus;
use crate::domain::operations::ConflictPolicy;
use crate::errors::AppError;

pub fn plan_conversion_outputs(
    request: &ConversionRequest,
) -> Result<Vec<ConversionFileResult>, AppError> {
    validate_request(request)?;

    let output_directory = PathBuf::from(&request.output_directory);
    let output_format = normalize_format(&request.output_format)?;
    let mut reserved_outputs = HashSet::new();
    let mut results = Vec::new();

    for input_path in &request.input_paths {
        let input = Path::new(input_path);
        if !input.exists() {
            results.push(ConversionFileResult {
                input_path: input_path.clone(),
                output_path: None,
                status: ConversionFileStatus::Failed,
                message: Some("Input file does not exist".to_string()),
            });
            continue;
        }

        let base_name = input
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("converted");
        let output_path = output_directory.join(format!("{base_name}.{output_format}"));
        let planned_output = plan_output_path(
            &output_path,
            &request.conflict_policy,
            &mut reserved_outputs,
        );

        match planned_output {
            Some(path) => results.push(ConversionFileResult {
                input_path: input_path.clone(),
                output_path: Some(path.to_string_lossy().to_string()),
                status: ConversionFileStatus::Pending,
                message: None,
            }),
            None => results.push(ConversionFileResult {
                input_path: input_path.clone(),
                output_path: Some(output_path.to_string_lossy().to_string()),
                status: ConversionFileStatus::Skipped,
                message: Some("Output exists and conflict policy is skip".to_string()),
            }),
        }
    }

    Ok(results)
}

pub fn conversion_job_status(results: &[ConversionFileResult]) -> JobStatus {
    if results.is_empty() {
        return JobStatus::Failed;
    }

    let completed = results
        .iter()
        .filter(|result| result.status == ConversionFileStatus::Completed)
        .count();
    let pending = results
        .iter()
        .filter(|result| result.status == ConversionFileStatus::Pending)
        .count();
    let failed_or_skipped = results.iter().any(|result| {
        matches!(
            result.status,
            ConversionFileStatus::Failed | ConversionFileStatus::Skipped
        )
    });

    if completed == results.len() {
        JobStatus::Completed
    } else if pending == results.len() {
        JobStatus::Queued
    } else if completed > 0 || failed_or_skipped {
        JobStatus::PartiallyCompleted
    } else {
        JobStatus::Failed
    }
}

fn validate_request(request: &ConversionRequest) -> Result<(), AppError> {
    if request.input_paths.is_empty() {
        return Err(AppError::validation("Select at least one file to convert"));
    }

    if request.output_directory.trim().is_empty() {
        return Err(AppError::validation("Choose an output folder"));
    }

    Ok(())
}

fn normalize_format(format: &str) -> Result<String, AppError> {
    let normalized = format.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(AppError::validation("Choose an output format"));
    }
    Ok(normalized)
}

fn plan_output_path(
    output_path: &Path,
    conflict_policy: &ConflictPolicy,
    reserved_outputs: &mut HashSet<String>,
) -> Option<PathBuf> {
    match conflict_policy {
        ConflictPolicy::Overwrite => {
            reserved_outputs.insert(output_path.to_string_lossy().to_lowercase());
            Some(output_path.to_path_buf())
        }
        ConflictPolicy::Skip if output_path.exists() => None,
        ConflictPolicy::Skip => reserve_if_available(output_path, reserved_outputs),
        ConflictPolicy::Rename => unique_output_path(output_path, reserved_outputs),
    }
}

fn reserve_if_available(path: &Path, reserved_outputs: &mut HashSet<String>) -> Option<PathBuf> {
    let key = path.to_string_lossy().to_lowercase();
    if reserved_outputs.contains(&key) {
        return None;
    }
    reserved_outputs.insert(key);
    Some(path.to_path_buf())
}

fn unique_output_path(path: &Path, reserved_outputs: &mut HashSet<String>) -> Option<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("converted");
    let extension = path.extension().and_then(|value| value.to_str());
    let mut index = 0;

    loop {
        let file_name = if index == 0 {
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("converted")
                .to_string()
        } else if let Some(extension) = extension {
            format!("{stem} ({index}).{extension}")
        } else {
            format!("{stem} ({index})")
        };
        let candidate = parent.join(file_name);
        let key = candidate.to_string_lossy().to_lowercase();

        if !candidate.exists() && !reserved_outputs.contains(&key) {
            reserved_outputs.insert(key);
            return Some(candidate);
        }

        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::converter::planner::{conversion_job_status, plan_conversion_outputs};
    use crate::domain::conversion::{
        ConversionFileResult, ConversionFileStatus, ConversionRequest,
    };
    use crate::domain::jobs::JobStatus;
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn output_planning_renames_conflicts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("photo.jpg");
        fs::write(&input, b"image").unwrap();
        fs::write(temp_dir.path().join("photo.png"), b"existing").unwrap();

        let results = plan_conversion_outputs(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().to_string_lossy().to_string(),
            output_format: "png".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Pending);
        assert!(results[0]
            .output_path
            .as_ref()
            .unwrap()
            .ends_with("photo (1).png"));
    }

    #[test]
    fn output_planning_skips_existing_output() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("photo.jpg");
        fs::write(&input, b"image").unwrap();
        fs::write(temp_dir.path().join("photo.png"), b"existing").unwrap();

        let results = plan_conversion_outputs(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().to_string_lossy().to_string(),
            output_format: "png".to_string(),
            conflict_policy: ConflictPolicy::Skip,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Skipped);
    }

    #[test]
    fn conversion_status_tracks_job_transitions() {
        let results = vec![ConversionFileResult {
            input_path: "input".to_string(),
            output_path: Some("output".to_string()),
            status: ConversionFileStatus::Pending,
            message: None,
        }];
        assert_eq!(conversion_job_status(&results), JobStatus::Queued);

        let results = vec![ConversionFileResult {
            status: ConversionFileStatus::Completed,
            ..results[0].clone()
        }];
        assert_eq!(conversion_job_status(&results), JobStatus::Completed);
    }
}
