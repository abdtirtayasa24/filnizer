use std::fs;
use std::path::Path;

use calamine::{open_workbook_auto, Reader};
use rust_xlsxwriter::Workbook;

use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::errors::AppError;

pub fn convert_spreadsheets(
    request: &ConversionRequest,
) -> Result<Vec<ConversionFileResult>, AppError> {
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

        match convert_single_spreadsheet(&result.input_path, &output_path, &request.output_format) {
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

fn convert_single_spreadsheet(
    input_path: &str,
    output_path: &str,
    output_format: &str,
) -> Result<(), AppError> {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let input_extension = Path::new(input_path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let output_format = output_format.trim().trim_start_matches('.').to_ascii_lowercase();

    match (input_extension.as_str(), output_format.as_str()) {
        ("csv", "xlsx") => csv_to_xlsx(input_path, output_path),
        ("xlsx", "csv") => xlsx_to_csv(input_path, output_path),
        _ => Err(AppError::validation(format!(
            "Unsupported spreadsheet conversion: {input_extension} to {output_format}"
        ))),
    }
}

fn csv_to_xlsx(input_path: &str, output_path: &str) -> Result<(), AppError> {
    let mut reader = csv::Reader::from_path(input_path)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (row_index, record) in reader.records().enumerate() {
        let record = record.map_err(|error| AppError::Unexpected(error.to_string()))?;
        for (column_index, value) in record.iter().enumerate() {
            worksheet
                .write_string(row_index as u32, column_index as u16, value)
                .map_err(|error| AppError::Unexpected(error.to_string()))?;
        }
    }

    workbook
        .save(output_path)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    Ok(())
}

fn xlsx_to_csv(input_path: &str, output_path: &str) -> Result<(), AppError> {
    let mut workbook = open_workbook_auto(input_path)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::validation("Workbook has no worksheets"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|error| AppError::Unexpected(format!("Could not read worksheet: {error:?}")))?;
    let mut writer = csv::Writer::from_path(output_path)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;

    for row in range.rows() {
        let values = row.iter().map(ToString::to_string).collect::<Vec<_>>();
        writer
            .write_record(values)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
    }

    writer
        .flush()
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rust_xlsxwriter::Workbook;

    use crate::converter::spreadsheet::convert_spreadsheets;
    use crate::domain::conversion::{ConversionFileStatus, ConversionRequest};
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn converts_csv_to_xlsx_fixture() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("table.csv");
        fs::write(&input, "name,count\napples,3\n").unwrap();
        let output_dir = temp_dir.path().join("out");

        let results = convert_spreadsheets(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: output_dir.to_string_lossy().to_string(),
            output_format: "xlsx".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Completed);
        assert!(std::path::Path::new(results[0].output_path.as_ref().unwrap()).exists());
    }

    #[test]
    fn converts_xlsx_to_csv_fixture() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("table.xlsx");
        let output_dir = temp_dir.path().join("out");
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.write_string(0, 0, "name").unwrap();
        worksheet.write_string(0, 1, "count").unwrap();
        worksheet.write_string(1, 0, "apples").unwrap();
        worksheet.write_number(1, 1, 3).unwrap();
        workbook.save(&input).unwrap();

        let results = convert_spreadsheets(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: output_dir.to_string_lossy().to_string(),
            output_format: "csv".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Completed);
        let output = fs::read_to_string(results[0].output_path.as_ref().unwrap()).unwrap();
        assert!(output.contains("apples,3"));
    }
}
