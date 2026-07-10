use crate::domain::files::FileCategory;
use crate::organizer::rules::{OrganizerRule, RuleKind};

pub fn infer_category(
    name: &str,
    extension: Option<&str>,
    rules: &[OrganizerRule],
) -> FileCategory {
    for rule in rules {
        match rule.kind {
            RuleKind::Extension => {
                if extension.is_some_and(|value| value.eq_ignore_ascii_case(&rule.value)) {
                    return rule.category.clone();
                }
            }
            RuleKind::FilenameContains => {
                if name.to_lowercase().contains(&rule.value.to_lowercase()) {
                    return rule.category.clone();
                }
            }
        }
    }

    default_category(extension)
}

pub fn default_category(extension: Option<&str>) -> FileCategory {
    let Some(extension) = extension else {
        return FileCategory::Other;
    };

    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "heic" => {
            FileCategory::Images
        }
        "pdf" => FileCategory::Pdfs,
        "doc" | "docx" | "txt" | "rtf" | "odt" | "md" => FileCategory::Documents,
        "csv" | "xls" | "xlsx" | "ods" => FileCategory::Spreadsheets,
        "ppt" | "pptx" | "odp" => FileCategory::Presentations,
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "wmv" => FileCategory::Videos,
        "mp3" | "wav" | "flac" | "aac" | "m4a" | "ogg" => FileCategory::Audio,
        "zip" | "7z" | "rar" | "tar" | "gz" => FileCategory::Archives,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "cs" | "html" | "css"
        | "json" | "toml" | "yaml" | "yml" => FileCategory::Code,
        "exe" | "msi" | "bat" | "cmd" | "ps1" => FileCategory::Executables,
        _ => FileCategory::Other,
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::files::FileCategory;
    use crate::organizer::categories::infer_category;
    use crate::organizer::rules::{OrganizerRule, RuleKind};

    #[test]
    fn default_category_infers_from_extension() {
        assert_eq!(
            infer_category("photo.JPG", Some("jpg"), &[]),
            FileCategory::Images
        );
        assert_eq!(
            infer_category("budget.xlsx", Some("xlsx"), &[]),
            FileCategory::Spreadsheets
        );
    }

    #[test]
    fn custom_rules_take_precedence() {
        let rules = vec![OrganizerRule {
            id: "rule-1".to_string(),
            kind: RuleKind::Extension,
            value: "jpg".to_string(),
            category: FileCategory::Documents,
        }];

        assert_eq!(
            infer_category("photo.jpg", Some("jpg"), &rules),
            FileCategory::Documents
        );
    }
}
