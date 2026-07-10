use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::errors::AppError;
use crate::tools::pdfium::find_app_local_pdfium;

pub fn convert_pdfs(request: &ConversionRequest) -> Result<Vec<ConversionFileResult>, AppError> {
    let mut results = plan_conversion_outputs(request)?;

    if find_app_local_pdfium().is_none() {
        for result in &mut results {
            if result.status == ConversionFileStatus::Pending {
                result.status = ConversionFileStatus::Failed;
                result.message = Some("Pdfium is not available in the app folder".to_string());
            }
        }
        return Ok(results);
    }

    for result in &mut results {
        if result.status == ConversionFileStatus::Pending {
            result.status = ConversionFileStatus::Failed;
            result.message = Some(
                "PDF conversion is gated until the bundled Pdfium runtime is added".to_string(),
            );
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::converter::pdf::convert_pdfs;
    use crate::domain::conversion::{ConversionFileStatus, ConversionRequest};
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn pdf_conversion_reports_missing_pdfium_without_crashing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("sample.pdf");
        fs::write(&input, b"%PDF-1.7\n").unwrap();

        let results = convert_pdfs(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().join("out").to_string_lossy().to_string(),
            output_format: "txt".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Failed);
    }
}
