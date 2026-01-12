mod config;
mod error;
mod handlers;
mod models;
mod security;
mod static_files;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "monkeyarch=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Validate root directory exists
    let root = &config.root_directory;
    if !root.exists() {
        let is_relative = !root.is_absolute();
        let cwd = std::env::current_dir().unwrap_or_default();

        eprintln!();
        eprintln!("ERROR: Root directory does not exist: {}", root.display());
        eprintln!();
        if is_relative {
            eprintln!("  The path is relative and resolves to:");
            eprintln!("    {}", cwd.join(root).display());
            eprintln!();
            eprintln!("  Consider using an absolute path in config.toml:");
            eprintln!("    root_directory = \"/path/to/media\"");
        } else {
            eprintln!("  Create it with:");
            eprintln!("    mkdir -p {}", root.display());
        }
        eprintln!();
        eprintln!("  For an external SD card on Raspberry Pi, use the mount point:");
        eprintln!("    root_directory = \"/media/pi/SDCARD\"");
        eprintln!();
        std::process::exit(1);
    }

    if !root.is_dir() {
        eprintln!();
        eprintln!("ERROR: Root path exists but is not a directory: {}", root.display());
        eprintln!();
        std::process::exit(1);
    }

    let canonical_root = root
        .canonicalize()
        .expect("Failed to canonicalize root directory");

    tracing::info!("Root directory: {:?}", canonical_root);
    tracing::info!(
        "Max upload size: {} MB",
        config.max_upload_size / (1024 * 1024)
    );
    tracing::info!("Delete enabled: {}", config.enable_delete);

    if let Some(ref static_dir) = config.static_directory {
        if static_dir.exists() {
            tracing::info!("Static files: {:?} (external)", static_dir);
        } else {
            eprintln!();
            eprintln!("WARNING: static_directory does not exist: {}", static_dir.display());
            eprintln!("         Falling back to embedded assets.");
            eprintln!();
        }
    } else {
        tracing::info!("Static files: embedded");
    }

    let state = AppState {
        config: Arc::new(config.clone()),
    };

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/list", get(handlers::list::list_directory))
        .route("/api/file", get(handlers::serve::serve_file))
        .route("/api/upload", post(handlers::upload::upload_file))
        .route("/api/move", post(handlers::move_file::move_file))
        .route("/api/mkdir", post(handlers::mkdir::create_directory))
        .route("/api/delete", post(handlers::delete::delete_path))
        // Static files (catch-all)
        .fallback(static_files::serve_static)
        // Shared state
        .with_state(state)
        // Middleware
        .layer(RequestBodyLimitLayer::new(config.max_upload_size as usize))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
        );

    let addr = format!("{}:{}", config.bind_address, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    tracing::info!("Shutdown signal received");
}
