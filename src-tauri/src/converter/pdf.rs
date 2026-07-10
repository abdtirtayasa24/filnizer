use std::fs;
use std::path::{Path, PathBuf};

use pdfium_render::prelude::*;

use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::errors::AppError;
use crate::tools::pdfium::find_app_local_pdfium;

pub fn convert_pdfs(request: &ConversionRequest) -> Result<Vec<ConversionFileResult>, AppError> {
    let mut results = plan_conversion_outputs(request)?;

    let Some(pdfium_path) = find_app_local_pdfium() else {
        for result in &mut results {
            if result.status == ConversionFileStatus::Pending {
                result.status = ConversionFileStatus::Failed;
                result.message = Some("Pdfium is not available in the app folder".to_string());
            }
        }
        return Ok(results);
    };

    let output_format = request
        .output_format
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();

    for result in &mut results {
        if result.status != ConversionFileStatus::Pending {
            continue;
        }

        let Some(output_path) = result.output_path.clone() else {
            result.status = ConversionFileStatus::Failed;
            result.message = Some("Output path could not be planned".to_string());
            continue;
        };

        let conversion_result = match output_format.as_str() {
            "txt" => pdf_to_text(&pdfium_path, &result.input_path, &output_path),
            "png" => pdf_to_pngs(&pdfium_path, &result.input_path, &output_path),
            _ => Err(AppError::validation(format!(
                "Unsupported PDF output format: {output_format}"
            ))),
        };

        match conversion_result {
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

fn bind_pdfium(pdfium_path: &Path) -> Result<Pdfium, AppError> {
    Pdfium::bind_to_library(pdfium_path)
        .map(Pdfium::new)
        .map_err(|error| AppError::ExternalTool(error.to_string()))
}

fn pdf_to_text(pdfium_path: &Path, input_path: &str, output_path: &str) -> Result<(), AppError> {
    let pdfium = bind_pdfium(pdfium_path)?;
    let document = pdfium
        .load_pdf_from_file(input_path, None)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    let mut text = String::new();

    for page in document.pages().iter() {
        let page_text = page
            .text()
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
        text.push_str(&page_text.all());
        text.push('\n');
    }

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, text)?;
    Ok(())
}

fn pdf_to_pngs(pdfium_path: &Path, input_path: &str, output_path: &str) -> Result<(), AppError> {
    let pdfium = bind_pdfium(pdfium_path)?;
    let document = pdfium
        .load_pdf_from_file(input_path, None)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    let output_path = Path::new(output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let page_count = document.pages().len();
    for (index, page) in document.pages().iter().enumerate() {
        let page_output = if page_count == 1 {
            output_path.to_path_buf()
        } else {
            page_output_path(output_path, index + 1)
        };
        page.render_with_config(&PdfRenderConfig::new().set_target_width(1600))
            .map_err(|error| AppError::Unexpected(error.to_string()))?
            .as_image()
            .map_err(|error| AppError::Unexpected(error.to_string()))?
            .save_with_format(&page_output, image::ImageFormat::Png)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
    }

    Ok(())
}

fn page_output_path(output_path: &Path, page_number: usize) -> PathBuf {
    let stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("page");
    let extension = output_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    output_path.with_file_name(format!("{stem}-{page_number}.{extension}"))
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
