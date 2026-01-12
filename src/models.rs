use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// === List Directory ===

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub path: String,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    File,
    Directory,
}

// === Upload ===

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub overwrite: bool,
}

// === Move/Rename ===

#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub overwrite: bool,
}

// === Create Directory ===

#[derive(Debug, Deserialize)]
pub struct MkdirRequest {
    pub path: String,
}

// === Delete ===

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
}

// === Common Responses ===

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
}

impl StatusResponse {
    pub fn ok() -> Self {
        Self { status: "ok" }
    }
}
