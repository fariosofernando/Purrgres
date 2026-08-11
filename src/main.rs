use chrono::Local;
use clap::Parser;
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tokio::time;

mod utils;
use utils::process_identifier as PID;

#[tokio::main]
async fn main() {
    let args = utils::args_struct::Args::parse();

    // ── Load config (needed for both modes) ──
    let config_path = args.config.as_ref().map(|p| PathBuf::from(p));
    let config = utils::config::Config::load(config_path.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("Warning: {}", e);
            utils::config::Config::default()
        });

    // ── Server mode ──
    if args.server {
        let server_config = config
            .server
            .expect("Missing [server] section in purrgres.toml");

        if let Err(e) = utils::server::start(server_config, args.port).await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // ── Client mode ──
    let tool_path = utils::path::get_bkp_path();

    if !tool_path.exists() {
        fs::create_dir_all(&tool_path).expect("Failed to create backup directory");
    }

    if args.stats {
        println!("{}", "=== Status purrgres ===".bold().underline());
        match PID::status() {
            Some(pid) => {
                let elapsed_time = PID::get_process_uptime(pid);
                println!("Backup running: {}", format!("PID: {}", pid).green());
                println!("Execution time: {}", elapsed_time.yellow());
            }
            None => {
                println!("{}", "Backup is not running".red());
            }
        }
        println!("{}", "=".repeat(25).bold());
        return;
    }

    if args.stop {
        println!("=== Stopping the backup ===");
        match PID::stop_process() {
            Ok(_) => {
                println!("Backup process stopped");
                PID::kill();
            }
            Err(e) => eprintln!("Error stopping the process: {}", e),
        }
        println!("=========================");
        return;
    }

    if args.list_purrs {
        utils::process::list_backups(&tool_path);
        return;
    }

    if let Some(backup_file) = args.rpurry.as_ref() {
        println!("=== Restoring backup from: {} === ", backup_file);
        utils::process::apply_backup(backup_file, &args);
        println!("=========================");
        return;
    }

    let pid_file_path = utils::path::get_bkp_path().join("purrgres_pid");
    if pid_file_path.exists() {
        let pid = match fs::read_to_string(&pid_file_path) {
            Ok(content) => content
                .trim()
                .parse::<u32>()
                .expect("Failed to parse PID from file"),
            Err(e) => {
                eprintln!("Error reading PID file: {}", e);
                std::process::exit(1);
            }
        };

        if PID::process_exists(pid) {
            eprintln!("A purrgres process is already active with PID: {}", pid);
            std::process::exit(1);
        } else {
            if let Err(e) = fs::remove_file(&pid_file_path) {
                eprintln!("Error removing obsolete PID file: {}", e);
            }
        }
    }

    // ── Check remote config once before loop ──
    let remote_config = config.remote.as_ref().filter(|r| r.enabled);
    if remote_config.is_some() {
        println!("📡 Remote sending enabled → {}", remote_config.unwrap().host);
    }

    let retention_max = config.retention.as_ref().map(|r| r.max_local_backups);

    let schedule = utils::schedule::Schedule::OneDay;
    let mut interval = time::interval(schedule.to_duration());

    loop {
        interval.tick().await;

        let now = Local::now();
        let file_name = format!(
            "{}/{}_backup.sql",
            tool_path.to_str().expect("Failed to get backup folder."),
            now.format("%d_%m_%Y_%H_%M"),
        );

        let output = Command::new("docker")
            .args(&[
                "exec",
                &args.container.clone().expect("Container name required"),
                "pg_dump",
                "-U",
                &args.user.clone().expect("Database user required"),
                &args.database.clone().expect("Database name required"),
            ])
            .output()
            .expect("Failed to execute the pg_dump command");

        let pid = std::process::id();
        PID::save_pid(pid);

        if output.status.success() {
            fs::write(&file_name, &output.stdout).expect("Failed save the backup file");
            println!("=== Backup complete ===");
            println!("Backup saved in: {}", file_name);

            // ── Send to remote if enabled ──
            if let Some(remote) = &config.remote {
                if remote.enabled {
                    let backup_path = std::path::Path::new(&file_name);
                    match utils::remote::send_backup(backup_path, remote).await {
                        Ok(_) => println!("✅ Remote sync complete"),
                        Err(e) => eprintln!("❌ Remote sync failed: {}", e),
                    }
                }
            }

            // ── Cleanup old local backups ──
            if let Some(max) = retention_max {
                utils::process::cleanup_old_backups(&tool_path, max);
            }
        } else {
            eprintln!(
                "Error make backup: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}