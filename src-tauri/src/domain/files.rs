use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_unix_ms: Option<i64>,
    pub category: FileCategory,
    pub hash_status: HashStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileCategory {
    Images,
    Documents,
    Pdfs,
    Spreadsheets,
    Presentations,
    Videos,
    Audio,
    Archives,
    Code,
    Executables,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HashStatus {
    NotRequested,
    Pending,
    Hashed { blake3: String },
    Failed { message: String },
}
