use std::process::Command;

use crate::converter::planner::plan_conversion_outputs;
use crate::domain::conversion::{ConversionFileResult, ConversionFileStatus, ConversionRequest};
use crate::domain::operations::ConflictPolicy;
use crate::errors::AppError;
use crate::tools::ffmpeg::find_app_local_ffmpeg;

pub fn convert_media(request: &ConversionRequest) -> Result<Vec<ConversionFileResult>, AppError> {
    let mut results = plan_conversion_outputs(request)?;
    let Some(ffmpeg_path) = find_app_local_ffmpeg() else {
        for result in &mut results {
            if result.status == ConversionFileStatus::Pending {
                result.status = ConversionFileStatus::Failed;
                result.message = Some("FFmpeg is not available in the app folder".to_string());
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

        let overwrite_arg = match request.conflict_policy {
            ConflictPolicy::Overwrite => "-y",
            ConflictPolicy::Skip | ConflictPolicy::Rename => "-n",
        };
        let output = Command::new(&ffmpeg_path)
            .args([overwrite_arg, "-i", &result.input_path, &output_path])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                result.status = ConversionFileStatus::Completed;
                result.message = None;
            }
            Ok(output) => {
                result.status = ConversionFileStatus::Failed;
                result.message = Some(String::from_utf8_lossy(&output.stderr).trim().to_string());
            }
            Err(error) => {
                result.status = ConversionFileStatus::Failed;
                result.message = Some(error.to_string());
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::converter::media::convert_media;
    use crate::domain::conversion::{ConversionFileStatus, ConversionRequest};
    use crate::domain::operations::ConflictPolicy;

    #[test]
    fn media_conversion_reports_missing_ffmpeg_without_crashing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("clip.mp4");
        fs::write(&input, b"not real media").unwrap();

        let results = convert_media(&ConversionRequest {
            input_paths: vec![input.to_string_lossy().to_string()],
            output_directory: temp_dir.path().join("out").to_string_lossy().to_string(),
            output_format: "mp3".to_string(),
            conflict_policy: ConflictPolicy::Rename,
        })
        .unwrap();

        assert!(matches!(
            results[0].status,
            ConversionFileStatus::Failed | ConversionFileStatus::Completed
        ));
    }
}
