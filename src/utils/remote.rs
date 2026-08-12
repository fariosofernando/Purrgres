use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;

use super::config::RemoteConfig;

pub async fn send_backup(file_path: &Path, config: &RemoteConfig) -> Result<String, String> {
    let raw_data = fs::read(file_path).map_err(|e| format!("Failed to read backup file: {}", e))?;

    let original_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Compress if enabled
    let (data, filename) = if config.compress {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&raw_data)
            .map_err(|e| format!("Compression failed: {}", e))?;
        let compressed = encoder
            .finish()
            .map_err(|e| format!("Compression finish failed: {}", e))?;

        let ratio = 100 - (compressed.len() * 100 / raw_data.len().max(1));
        println!(
            "Compressed: {} → {} bytes ({}% reduction)",
            raw_data.len(),
            compressed.len(),
            ratio
        );

        (compressed, format!("{}.gz", original_name))
    } else {
        (raw_data, original_name)
    };

    // Compute SHA256
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let checksum = hex::encode(hasher.finalize());

    let url = format!("http://{}:{}/api/upload", config.host, config.port);
    let client = reqwest::Client::new();
    let mut last_error = String::new();

    for attempt in 1..=3u64 {
        // Rebuild form each attempt (Form is not Clone)
        let part = reqwest::multipart::Part::bytes(data.clone())
            .file_name(filename.clone())
            .mime_str("application/octet-stream")
            .map_err(|e| format!("Failed to build multipart: {}", e))?;

        let form = reqwest::multipart::Form::new().part("file", part);

        match client
            .post(&url)
            .header("X-API-Key", &config.api_key)
            .header("X-Checksum-SHA256", &checksum)
            .multipart(form)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!("📤 Backup sent to remote: {} ({})", config.host, filename);
                return Ok(filename);
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                last_error = format!("Server returned {}: {}", status, body);
            }
            Err(e) => {
                last_error = format!("Connection failed: {}", e);
            }
        }

        eprintln!("Attempt {}/3 failed: {}", attempt, last_error);

        if attempt < 3 {
            tokio::time::sleep(std::time::Duration::from_secs(5 * attempt)).await;
        }
    }

    Err(format!(
        "Failed to send backup after 3 attempts: {}",
        last_error
    ))
}
