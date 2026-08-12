use std::fs::{self};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use super::args_struct::Args;
use crate::utils::path::get_bkp_path;

pub fn apply_backup(backup_file: &str, args: &Args) {
    let copy_output = Command::new("docker")
        .args(&[
            "cp",
            get_bkp_path()
                .join(backup_file)
                .to_str()
                .expect("Failed to get backup file path"),
            &format!(
                "{}:/tmp/{}",
                args.container.clone().expect("Container name required"),
                backup_file
            ),
        ])
        .output()
        .expect("Failed to copy backup file to containe");

    if !copy_output.status.success() {
        eprintln!(
            "Error copying file to container: {:?}",
            String::from_utf8_lossy(&copy_output.stderr)
        );
        return;
    }

    let output = Command::new("docker")
        .args(&[
            "exec",
            &args.container.clone().expect("Container name required"),
            "psql",
            "-U",
            &args.user.clone().expect("Database user required"),
            "-d",
            &args.database.clone().expect("Database name requierd"),
            "-f",
            &format!("/tmp/{}", backup_file), // isto deve usar o caminho do proprio container.
        ])
        .output()
        .expect("Failed to execute psql command");

    if output.status.success() {
        println!("Backup restored successfully");

        let log_file = get_bkp_path().join(".purrs");
        let mut log = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)
            .expect("Failed to open PID file");

        let now = chrono::Local::now();
        writeln!(
            log,
            "Backup restored form: {} - Date: {}",
            backup_file,
            now.format("%d/%m/%Y %H:%M")
        )
        .expect("Failed to write a PID file");
    } else {
        eprintln!(
            "Error restoring backup: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn list_backups(tool_path: &Path) {
    let entries = fs::read_dir(tool_path).expect("Failed to read backup directory");

    let mut backups = vec![];
    let last_restored = read_last_restored();

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if path.is_file() && (name.ends_with(".sql") || name.ends_with(".sql.gz")) {
                let metadata = fs::metadata(&path).expect("Error getting file metadata");
                let created = metadata
                    .modified()
                    .expect("Error getting modification date");

                if let Ok(duration_since_epoch) = created.duration_since(SystemTime::UNIX_EPOCH) {
                    let created_naive = chrono::DateTime::from_timestamp(
                        duration_since_epoch.as_secs() as i64,
                        duration_since_epoch.subsec_nanos(),
                    )
                    .map(|dt| dt.naive_local());
                    if let Some(created_naive) = created_naive {
                        let formatted_time = created_naive.format("%d/%m/%Y %H:%M").to_string();
                        if let Some(file_name) = path.file_name() {
                            if let Some(file_str) = file_name.to_str() {
                                let is_last_restored = if let Some(last_backup) = &last_restored {
                                    file_str == last_backup
                                } else {
                                    false
                                };
                                backups.push((
                                    file_str.to_string(),
                                    formatted_time,
                                    is_last_restored,
                                ));
                            }
                        }
                    }
                } else {
                    eprintln!(
                        "Error calculating duration since epoch for file: {}",
                        path.display()
                    );
                }
            }
        }
    }

    backups.sort_by(|a, b| b.1.cmp(&a.1));

    println!("======================================================================");
    println!(
        "{:<27} | {:<20} | {:<20}",
        "   backups", "   date", "restore point"
    );
    println!("======================================================================");

    for (file, time, is_last_restored) in backups {
        let last_restored_label = if is_last_restored { "last" } else { "" };
        println!("{:<20} | {:<20} | {:<20}", file, time, last_restored_label);
    }

    println!("======================================================================");
}
fn read_last_restored() -> Option<String> {
    let log_file_path = get_bkp_path().join(".purrs");

    if let Ok(contents) = fs::read_to_string(log_file_path) {
        let last_line = contents.lines().last();

        if let Some(last_entry) = last_line {
            let parts: Vec<&str> = last_entry.split_whitespace().collect();
            if parts.len() >= 4 {
                return Some(parts[3].to_string());
            }
        }
    }

    println!("It was impossible to open the purrs file");

    None
}

/// Removes the oldest backup files when count exceeds max_count.
/// Keeps only .sql and .sql.gz files in scope.
pub fn cleanup_old_backups(dir: &std::path::Path, max_count: u32) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Cleanup: failed to read directory: {}", err);
            return;
        }
    };

    let mut backups: Vec<(std::path::PathBuf, SystemTime)> = entries
        .flatten()
        .filter(|e| {
            let path = e.path();
            if !path.is_file() {
                return false;
            }
            let name = path.to_string_lossy().to_lowercase();
            name.ends_with(".sql") || name.ends_with(".sql.gz")
        })
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((e.path(), modified))
        })
        .collect();

    if backups.len() <= max_count as usize {
        return;
    }

    // Sort newest first
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    // Remove everything beyond max_count
    for (path, _) in backups.iter().skip(max_count as usize) {
        match fs::remove_file(path) {
            Ok(_) => println!("Removed old backup: {}", path.display()),
            Err(e) => eprintln!("Failed to remove {}: {}", path.display(), e),
        }
    }
}
