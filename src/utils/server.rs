use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Local;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;
use std::time::SystemTime;
use tower_http::limit::RequestBodyLimitLayer;

use super::config::ServerConfig;
use super::process::cleanup_old_backups;

pub struct AppState {
    pub config: ServerConfig,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct UploadResponse {
    success: bool,
    filename: String,
    size_bytes: u64,
    checksum_sha256: String,
    checksum_verified: bool,
    timestamp: String,
}

#[derive(Serialize)]
struct BackupEntry {
    filename: String,
    size_bytes: u64,
    created_at: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: msg.to_string(),
        }),
    )
}

pub async fn start(config: ServerConfig, port_override: Option<u16>) -> Result<(), String> {
    let port = port_override.unwrap_or(config.port);
    let max_body = config.max_upload_bytes() as usize;

    fs::create_dir_all(&config.storage_path)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;

    let storage_display =
        fs::canonicalize(&config.storage_path).unwrap_or(config.storage_path.clone());

    let state = Arc::new(AppState { config });

    let app = Router::new()
        .route("/api/upload", post(upload))
        .route("/api/backups", get(list_backups))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .route("/api/health", get(health))
        .layer(DefaultBodyLimit::max(max_body))
        .layer(RequestBodyLimitLayer::new(max_body))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("🐱 Purrgres server listening on {}", addr);
    println!("Storage: {}", storage_display.display());
    println!("Max upload: {} MB", max_body / (1024 * 1024));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let api_key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if key == state.config.api_key => Ok(next.run(req).await),
        Some(_) => Err(error_response(StatusCode::UNAUTHORIZED, "Invalid API key")),
        None => Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header",
        )),
    }
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn upload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let expected_checksum = headers
        .get("X-Checksum-SHA256")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase());

    let field = multipart
        .next_field()
        .await
        .map_err(|e| {
            error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid multipart: {}", e),
            )
        })?
        .ok_or_else(|| error_response(StatusCode::BAD_REQUEST, "No file field provided"))?;

    let file_name = field
        .file_name()
        .map(|s| sanitize_filename(s))
        .unwrap_or_else(|| {
            let now = Local::now();
            format!("{}_backup.sql.gz", now.format("%d_%m_%Y_%H_%M"))
        });

    let data = field.bytes().await.map_err(|e| {
        error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("Failed to read upload: {}", e),
        )
    })?;

    if data.is_empty() {
        return Err(error_response(StatusCode::BAD_REQUEST, "Empty file"));
    }

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_checksum = hex::encode(hasher.finalize());

    let checksum_verified = match &expected_checksum {
        Some(expected) => {
            if expected != &computed_checksum {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    &format!(
                        "Checksum mismatch: expected {} got {}",
                        expected, computed_checksum
                    ),
                ));
            }
            true
        }
        None => false,
    };

    let size = data.len() as u64;
    let dest_path = state.config.storage_path.join(&file_name);

    let dest_path = unique_path(dest_path);
    let final_name = dest_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    tokio::fs::write(&dest_path, &data).await.map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to save: {}", e),
        )
    })?;

    if let Some(max) = state.config.max_remote_backups {
        cleanup_old_backups(&state.config.storage_path, max);
    }

    let now = Local::now();
    println!(
        "[{}] * Received: {} ({} bytes, checksum ok: {})",
        now.format("%d/%m/%Y %H:%M"),
        final_name,
        size,
        checksum_verified
    );

    Ok(Json(UploadResponse {
        success: true,
        filename: final_name,
        size_bytes: size,
        checksum_sha256: computed_checksum,
        checksum_verified,
        timestamp: now.format("%d/%m/%Y %H:%M:%S").to_string(),
    }))
}

async fn list_backups(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BackupEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let entries = fs::read_dir(&state.config.storage_path).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to read storage: {}", e),
        )
    })?;

    let mut backups: Vec<(BackupEntry, SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Ok(metadata) = fs::metadata(&path) {
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let datetime = chrono::DateTime::<Local>::from(modified);

            backups.push((
                BackupEntry {
                    filename: entry.file_name().to_string_lossy().to_string(),
                    size_bytes: metadata.len(),
                    created_at: datetime.format("%d/%m/%Y %H:%M").to_string(),
                },
                modified,
            ));
        }
    }

    // Sort newest first
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    let result: Vec<BackupEntry> = backups.into_iter().map(|(entry, _)| entry).collect();
    Ok(Json(result))
}

fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\'], "")
        .replace("..", "")
        .trim()
        .to_string()
}

fn unique_path(path: std::path::PathBuf) -> std::path::PathBuf {
    if !path.exists() {
        return path;
    }

    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(std::path::Path::new("."));

    let mut counter = 1u32;
    loop {
        let candidate = parent.join(format!("{}_{}{}", stem, counter, ext));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}
