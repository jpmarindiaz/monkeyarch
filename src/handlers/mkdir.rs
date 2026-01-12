use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::{MkdirRequest, StatusResponse};
use crate::security::validate_path;
use crate::AppState;

pub async fn create_directory(
    State(state): State<AppState>,
    Json(req): Json<MkdirRequest>,
) -> Result<Json<StatusResponse>, AppError> {
    let dir_path = validate_path(&state.config.root_directory, &req.path)?;

    if dir_path.exists() {
        return Err(AppError::Conflict("path already exists".to_string()));
    }

    tokio::fs::create_dir(&dir_path).await?;

    Ok(Json(StatusResponse::ok()))
}
