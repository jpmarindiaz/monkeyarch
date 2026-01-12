use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::{MoveRequest, StatusResponse};
use crate::security::{validate_existing, validate_path};
use crate::AppState;

pub async fn move_file(
    State(state): State<AppState>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<StatusResponse>, AppError> {
    let from_path = validate_existing(&state.config.root_directory, &req.from)?;
    let to_path = validate_path(&state.config.root_directory, &req.to)?;

    // Prevent moving the root directory
    let canonical_root = state.config.root_directory.canonicalize()?;
    if from_path == canonical_root {
        return Err(AppError::BadRequest(
            "cannot move root directory".to_string(),
        ));
    }

    // Check if destination exists
    if to_path.exists() && !req.overwrite {
        return Err(AppError::Conflict("destination already exists".to_string()));
    }

    // Prevent moving a directory into itself
    if from_path.is_dir() && to_path.starts_with(&from_path) {
        return Err(AppError::BadRequest(
            "cannot move directory into itself".to_string(),
        ));
    }

    tokio::fs::rename(&from_path, &to_path).await?;

    Ok(Json(StatusResponse::ok()))
}
