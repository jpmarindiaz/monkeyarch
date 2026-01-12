use axum::{
    extract::{Multipart, Query, State},
    Json,
};
use tokio::io::AsyncWriteExt;

use crate::error::AppError;
use crate::models::{StatusResponse, UploadQuery};
use crate::security::{validate_directory, validate_filename};
use crate::AppState;

const ALLOWED_AUDIO: &[&str] = &["audio/mpeg", "audio/mp3"];
const ALLOWED_IMAGE_PREFIX: &str = "image/";

pub async fn upload_file(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<StatusResponse>, AppError> {
    let dest_dir = validate_directory(&state.config.root_directory, &query.path)?;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {}", e)))?
    {
        let filename = field
            .file_name()
            .ok_or_else(|| AppError::BadRequest("missing filename".to_string()))?
            .to_string();

        validate_filename(&filename)?;

        // Check MIME type
        let content_type = field
            .content_type()
            .ok_or_else(|| AppError::BadRequest("missing content type".to_string()))?
            .to_string();

        if !is_allowed_mime(&content_type) {
            return Err(AppError::UnsupportedMediaType(format!(
                "type '{}' not allowed, only MP3 and images accepted",
                content_type
            )));
        }

        let dest_path = dest_dir.join(&filename);

        // Check overwrite
        if dest_path.exists() && !query.overwrite {
            return Err(AppError::Conflict(format!(
                "file '{}' already exists",
                filename
            )));
        }

        // Stream to file using chunk() method
        let mut file = tokio::fs::File::create(&dest_path).await?;
        let mut total_size: u64 = 0;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| AppError::BadRequest(format!("upload error: {}", e)))?
        {
            total_size += chunk.len() as u64;

            // Check size limit during streaming
            if total_size > state.config.max_upload_size {
                // Clean up partial file
                drop(file);
                let _ = tokio::fs::remove_file(&dest_path).await;
                return Err(AppError::PayloadTooLarge);
            }

            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        tracing::info!(
            "Uploaded file: {} ({} bytes)",
            dest_path.display(),
            total_size
        );
    }

    Ok(Json(StatusResponse::ok()))
}

fn is_allowed_mime(mime: &str) -> bool {
    let mime_lower = mime.to_lowercase();

    // Check audio types
    if ALLOWED_AUDIO.iter().any(|&allowed| mime_lower == allowed) {
        return true;
    }

    // Check image types
    if mime_lower.starts_with(ALLOWED_IMAGE_PREFIX) {
        return true;
    }

    false
}
