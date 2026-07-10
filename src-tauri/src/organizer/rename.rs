pub fn clean_filename(name: &str) -> String {
    let (stem, extension) = split_extension(name.trim());
    let cleaned_stem = stem
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => ' ',
            '_' | '-' => ' ',
            value => value,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let fallback_stem = if cleaned_stem.is_empty() {
        "untitled".to_string()
    } else {
        cleaned_stem
    };

    match extension {
        Some(extension) => format!("{}.{}", fallback_stem, extension.to_ascii_lowercase()),
        None => fallback_stem,
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

#[cfg(test)]
mod tests {
    use super::clean_filename;

    #[test]
    fn clean_filename_normalizes_spaces_separators_and_extension() {
        assert_eq!(
            clean_filename("  Q1___Report--Final.XLSX  "),
            "Q1 Report Final.xlsx"
        );
    }

    #[test]
    fn clean_filename_removes_windows_unsafe_characters() {
        assert_eq!(clean_filename("bad:name?.txt"), "bad name.txt");
    }
}
