use std::fs;
use std::path::Path;
use std::process::Command;

use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::errors::AppError;
use crate::tools::libreoffice::find_libreoffice;

pub fn convert_office(request: &ConversionRequest) -> Result<Vec<ConversionFileResult>, AppError> {
    if request.output_format.trim().trim_start_matches('.') != "pdf" {
        return Err(AppError::validation("Office output format must be PDF"));
    }

    let mut results = plan_conversion_outputs(request)?;
    let Some(soffice_path) = find_libreoffice() else {
        for result in &mut results {
            if result.status == ConversionFileStatus::Pending {
                result.status = ConversionFileStatus::Failed;
                result.message = Some("LibreOffice is not installed or was not found".to_string());
            }
        }
        return Ok(results);
    };

    for result in &mut results {
        if result.status != ConversionFileStatus::Pending {
            continue;
        }

        let Some(output_path) = result.output_path.clone() else {
            result.status = ConversionFileStatus::Failed;
            result.message = Some("Output path could not be planned".to_string());
            continue;
        };

        match convert_single_office(&soffice_path, &result.input_path, &output_path) {
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

fn convert_single_office(
    soffice_path: &Path,
    input_path: &str,
    output_path: &str,
) -> Result<(), AppError> {
    let output_path = Path::new(output_path);
    let output_dir = output_path
        .parent()
        .ok_or_else(|| AppError::validation("Output path has no parent folder"))?;
    fs::create_dir_all(output_dir)?;

    let output = Command::new(soffice_path)
        .args([
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            &output_dir.to_string_lossy(),
            input_path,
        ])
        .output()?;

    if !output.status.success() {
        return Err(AppError::Unexpected(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let generated = output_dir
        .join(
            Path::new(input_path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("converted"),
        )
        .with_extension("pdf");

    if generated != output_path && generated.exists() {
        if output_path.exists() {
            fs::remove_file(output_path)?;
        }
        fs::rename(generated, output_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::converter::office::convert_office;
    use crate::domain::conversion::{ConversionFileStatus, ConversionRequest};
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn office_conversion_reports_missing_libreoffice_without_crashing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("document.docx");
        fs::write(&input, b"not a real docx").unwrap();

        let results = convert_office(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().join("out").to_string_lossy().to_string(),
            output_format: "pdf".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert!(matches!(
            results[0].status,
            ConversionFileStatus::Failed | ConversionFileStatus::Completed
        ));
    }
}
