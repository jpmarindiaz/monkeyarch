use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;
use std::path::Path;

use crate::AppState;

#[derive(Embed)]
#[folder = "static/"]
struct EmbeddedAssets;

pub async fn serve_static(State(state): State<AppState>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    // Prevent path traversal
    if path.contains("..") {
        return not_found();
    }

    // Try external static directory first (if configured)
    if let Some(ref static_dir) = state.config.static_directory {
        return serve_from_disk(static_dir, path).await;
    }

    // Fall back to embedded assets
    serve_embedded(path)
}

async fn serve_from_disk(static_dir: &Path, path: &str) -> Response {
    let file_path = static_dir.join(path);

    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                // No caching for development
                .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => {
            // Try index.html for SPA routing
            if !path.starts_with("api/") && path != "index.html" {
                let index_path = static_dir.join("index.html");
                if let Ok(content) = tokio::fs::read(&index_path).await {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .header(header::CACHE_CONTROL, "no-cache")
                        .body(Body::from(content))
                        .unwrap();
                }
            }
            not_found()
        }
    }
}

fn serve_embedded(path: &str) -> Response {
    match EmbeddedAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // SPA fallback
            if !path.starts_with("api/") {
                if let Some(content) = EmbeddedAssets::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(content.data.into_owned()))
                        .unwrap();
                }
            }
            not_found()
        }
    }
}

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("Not Found"))
        .unwrap()
}
