use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use std::time::SystemTime;

use crate::error::AppError;
use crate::models::{EntryType, FileEntry, ListQuery, ListResponse};
use crate::security::validate_directory;
use crate::AppState;

pub async fn list_directory(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    let dir_path = validate_directory(&state.config.root_directory, &query.path)?;

    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(&dir_path).await?;

    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();

        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        let (size, modified) = if metadata.is_file() {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| system_time_to_datetime(t));
            (Some(metadata.len()), modified)
        } else {
            (None, None)
        };

        entries.push(FileEntry {
            name,
            entry_type,
            size,
            modified,
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        match (&a.entry_type, &b.entry_type) {
            (EntryType::Directory, EntryType::File) => std::cmp::Ordering::Less,
            (EntryType::File, EntryType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(Json(ListResponse {
        path: query.path,
        entries,
    }))
}

fn system_time_to_datetime(time: SystemTime) -> Option<DateTime<Utc>> {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()))
        .flatten()
}
