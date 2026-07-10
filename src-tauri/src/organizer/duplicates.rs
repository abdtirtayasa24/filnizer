use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::domain::files::FileEntry;
use crate::errors::AppError;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindDuplicatesRequest {
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindDuplicatesResponse {
    pub sets: Vec<DuplicateSet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSet {
    pub size_bytes: u64,
    pub blake3: String,
    pub paths: Vec<String>,
}

pub fn find_duplicates(files: &[FileEntry]) -> Result<Vec<DuplicateSet>, AppError> {
    let mut by_size: HashMap<u64, Vec<&FileEntry>> = HashMap::new();
    for file in files {
        by_size.entry(file.size_bytes).or_default().push(file);
    }

    let mut sets = Vec::new();
    for (size_bytes, same_size_files) in by_size {
        if same_size_files.len() < 2 {
            continue;
        }

        let mut by_hash: HashMap<String, Vec<String>> = HashMap::new();
        for file in same_size_files {
            let hash = hash_file(&file.path)?;
            by_hash.entry(hash).or_default().push(file.path.clone());
        }

        for (blake3, mut paths) in by_hash {
            if paths.len() > 1 {
                paths.sort();
                sets.push(DuplicateSet {
                    size_bytes,
                    blake3,
                    paths,
                });
            }
        }
    }

    sets.sort_by(|left, right| left.paths[0].cmp(&right.paths[0]));
    Ok(sets)
}

fn hash_file(path: &str) -> Result<String, AppError> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::domain::files::{FileCategory, FileEntry, HashStatus};
    use crate::organizer::duplicates::find_duplicates;

    #[test]
    fn find_duplicates_groups_by_size_then_blake3() {
        let temp_dir = tempfile::tempdir().unwrap();
        let first = temp_dir.path().join("first.txt");
        let second = temp_dir.path().join("second.txt");
        let same_size_different_content = temp_dir.path().join("third.txt");
        fs::write(&first, b"duplicate").unwrap();
        fs::write(&second, b"duplicate").unwrap();
        fs::write(&same_size_different_content, b"different").unwrap();

        let sets = find_duplicates(&[
            file_entry(first.to_string_lossy().to_string(), 9),
            file_entry(second.to_string_lossy().to_string(), 9),
            file_entry(same_size_different_content.to_string_lossy().to_string(), 9),
        ])
        .unwrap();

        assert_eq!(sets.len(), 1);
        assert_eq!(sets[0].paths.len(), 2);
    }

    fn file_entry(path: String, size_bytes: u64) -> FileEntry {
        FileEntry {
            path,
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size_bytes,
            modified_unix_ms: None,
            category: FileCategory::Documents,
            hash_status: HashStatus::NotRequested,
        }
    }
}
