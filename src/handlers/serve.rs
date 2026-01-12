use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio_util::io::ReaderStream;

use crate::error::AppError;
use crate::models::ListQuery;
use crate::security::validate_file;
use crate::AppState;

pub async fn serve_file(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Response, AppError> {
    let file_path = validate_file(&state.config.root_directory, &query.path)?;

    let file = tokio::fs::File::open(&file_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        )
        .body(body)
        .unwrap())
}
