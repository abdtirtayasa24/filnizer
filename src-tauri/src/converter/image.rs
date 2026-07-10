use std::fs;
use std::path::Path;

use image::ImageFormat;

use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::errors::AppError;

pub fn convert_images(request: &ConversionRequest) -> Result<Vec<ConversionFileResult>, AppError> {
    let mut results = plan_conversion_outputs(request)?;
    let output_format = image_format(&request.output_format)?;

    for result in &mut results {
        if result.status != ConversionFileStatus::Pending {
            continue;
        }

        let Some(output_path) = result.output_path.clone() else {
            result.status = ConversionFileStatus::Failed;
            result.message = Some("Output path could not be planned".to_string());
            continue;
        };

        match convert_single_image(&result.input_path, &output_path, output_format) {
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

fn convert_single_image(
    input_path: &str,
    output_path: &str,
    output_format: ImageFormat,
) -> Result<(), AppError> {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let image = image::open(input_path).map_err(|error| AppError::Unexpected(error.to_string()))?;
    image
        .save_with_format(output_path, output_format)
        .map_err(|error| AppError::Unexpected(error.to_string()))?;
    Ok(())
}

fn image_format(format: &str) -> Result<ImageFormat, AppError> {
    match format
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::WebP),
        "bmp" => Ok(ImageFormat::Bmp),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        other => Err(AppError::validation(format!(
            "Unsupported image output format: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};

    use crate::converter::image::convert_images;
    use crate::domain::conversion::{ConversionFileStatus, ConversionRequest};
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn converts_png_fixture_to_jpeg() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("sample.png");
        let image = ImageBuffer::from_pixel(2, 2, Rgba([255_u8, 0, 0, 255]));
        image.save(&input).unwrap();

        let output_dir = temp_dir.path().join("out");
        let results = convert_images(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: output_dir.to_string_lossy().to_string(),
            output_format: "jpg".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert_eq!(results[0].status, ConversionFileStatus::Completed);
        assert!(results[0]
            .output_path
            .as_ref()
            .unwrap()
            .ends_with("sample.jpg"));
        assert!(std::path::Path::new(results[0].output_path.as_ref().unwrap()).exists());
    }
}
