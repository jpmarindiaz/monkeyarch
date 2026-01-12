use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::{DeleteRequest, StatusResponse};
use crate::security::validate_existing;
use crate::AppState;

pub async fn delete_path(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<StatusResponse>, AppError> {
    if !state.config.enable_delete {
        return Err(AppError::Forbidden);
    }

    let path = validate_existing(&state.config.root_directory, &req.path)?;

    // Prevent deleting the root directory itself
    let canonical_root = state.config.root_directory.canonicalize()?;
    if path == canonical_root {
        return Err(AppError::BadRequest(
            "cannot delete root directory".to_string(),
        ));
    }

    if path.is_dir() {
        if req.recursive {
            tokio::fs::remove_dir_all(&path).await?;
        } else {
            tokio::fs::remove_dir(&path).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::DirectoryNotEmpty {
                    AppError::Conflict("directory not empty".to_string())
                } else {
                    AppError::Io(e)
                }
            })?;
        }
    } else {
        tokio::fs::remove_file(&path).await?;
    }

    Ok(Json(StatusResponse::ok()))
}
