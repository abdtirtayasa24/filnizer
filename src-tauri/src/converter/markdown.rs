use std::fs;
use std::path::Path;

use markdown2pdf::config::ConfigSource;

use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::errors::AppError;

pub fn convert_markdown(
    request: &ConversionRequest,
) -> Result<Vec<ConversionFileResult>, AppError> {
    if request.output_format.trim().trim_start_matches('.') != "pdf" {
        return Err(AppError::validation("Markdown output format must be PDF"));
    }

    let mut results = plan_conversion_outputs(request)?;

    for result in &mut results {
        if result.status != ConversionFileStatus::Pending {
            continue;
        }

        let Some(output_path) = result.output_path.clone() else {
            result.status = ConversionFileStatus::Failed;
            result.message = Some("Output path could not be planned".to_string());
            continue;
        };

        match convert_single_markdown(&result.input_path, &output_path) {
            Ok(()) => {
                result.status = ConversionFileStatus::Completed;
                result.message = None;
            }
            Err(error) => {
                result.status = ConversionFileStatus::Failed;
                result.message = Some(error.to_string());
            }
        }
    }

    Ok(results)
}

fn convert_single_markdown(input_path: &str, output_path: &str) -> Result<(), AppError> {
    let markdown = fs::read_to_string(input_path)?;
    reject_remote_references(&markdown)?;
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    markdown2pdf::parse_into_file(markdown, output_path, ConfigSource::Default, None)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    Ok(())
}

fn reject_remote_references(markdown: &str) -> Result<(), AppError> {
    let lower = markdown.to_ascii_lowercase();
    if lower.contains("http://") || lower.contains("https://") {
        return Err(AppError::validation(
            "Remote URLs are not allowed in offline Markdown conversion",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::converter::markdown::convert_markdown;
    use crate::domain::conversion::{ConversionFileStatus, ConversionRequest};
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn converts_markdown_fixture_to_pdf() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("note.md");
        fs::write(&input, "# Hello\n\nLocal-only markdown.").unwrap();

        let results = convert_markdown(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().join("out").to_string_lossy().to_string(),
            output_format: "pdf".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Completed);
        assert!(std::path::Path::new(results[0].output_path.as_ref().unwrap()).exists());
    }

    #[test]
    fn rejects_remote_markdown_references() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("remote.md");
        fs::write(&input, "![remote](https://example.com/image.png)").unwrap();

        let results = convert_markdown(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().join("out").to_string_lossy().to_string(),
            output_format: "pdf".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Failed);
        assert!(results[0]
            .message
            .as_ref()
            .unwrap()
            .contains("Remote URLs are not allowed"));
    }
}
