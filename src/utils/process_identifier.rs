use super::path::get_bkp_path;

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn get_pid() -> PathBuf {
    get_bkp_path().join("purrgres_pid")
}

pub fn status() -> Option<u32> {
    if let Ok(pid_str) = fs::read_to_string(get_pid()) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            return Some(pid);
        }
    }
    None
}

pub fn save_pid(pid: u32) {
    let pid_file_path = get_bkp_path().join("purrgres_pid");
    let mut file = File::create(&pid_file_path)
        .map_err(|e| format!("Failed to create a PID file: {}", e))
        .expect("Failed to create a PID file");

    write!(file, "{}", pid).expect("Failed to write PID to file");

    println!("Saving PID: {}", pid);
}

pub fn kill() {
    fs::remove_file(get_pid()).expect("Failed to remove a PID file");
}

pub fn process_exists(pid: u32) -> bool {
    let path = format!("/proc/{}", pid);
    std::path::Path::new(&path).exists()
}

pub fn stop_process() -> Result<(), String> {
    if let Some(pid) = status() {
        let output = Command::new("kill")
            .arg(pid.to_string())
            .output()
            .map_err(|e| format!("Failed to stop process: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err("Failed to kill process.".to_string())
        }
    } else {
        Err("No backup processes found.".to_string())
    }
}

pub fn get_process_uptime(pid: u32) -> String {
    let start_time = Command::new("ps")
        .args(&["-p", &pid.to_string(), "-o", "lstart="])
        .output()
        .expect("Failed to get process start time");

    if start_time.status.success() {
        let start_time_str = String::from_utf8_lossy(&start_time.stdout);

        let months = [
            ("jan", "Jan"),
            ("fev", "Feb"),
            ("mar", "Mar"),
            ("abr", "Apr"),
            ("mai", "May"),
            ("jun", "Jun"),
            ("jul", "Jul"),
            ("ago", "Aug"),
            ("set", "Sep"),
            ("out", "Oct"),
            ("nov", "Nov"),
            ("dez", "Dec"),
        ];

        let days_of_week = [
            ("dom", "Sun"),
            ("seg", "Mon"),
            ("ter", "Tue"),
            ("qua", "Wed"),
            ("qui", "Thu"),
            ("sex", "Fri"),
            ("sab", "Sat"),
        ];

        let mut start_time_str = start_time_str.to_string();
        for (pt, en) in &days_of_week {
            start_time_str = start_time_str.replace(*pt, *en);
        }
        for (pt, en) in &months {
            start_time_str = start_time_str.replace(*pt, *en);
        }

        let start_time =
            chrono::NaiveDateTime::parse_from_str(&start_time_str.trim(), "%a %b %d %H:%M:%S %Y");

        match start_time {
            Ok(start_time) => {
                let elapsed_time = chrono::Local::now().naive_local() - start_time;

                let total_seconds = elapsed_time.num_seconds();
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                format!("{}h {}m {}s", hours, minutes, seconds)
            }
            Err(_) => "Unable to calculate time".to_string(),
        }
    } else {
        "Unable to calculate time".to_string()
    }
}
