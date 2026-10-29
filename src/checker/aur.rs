use super::arch_repo::PackageUpdate;
use crate::config::AurHelper;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub fn is_command_in_path(cmd: &str) -> bool {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            if Path::new(&dir).join(cmd).is_file() {
                return true;
            }
        }
    }
    false
}

pub async fn check_aur_updates(preferred: AurHelper) -> Vec<PackageUpdate> {
    let helper = match preferred {
        AurHelper::None => return Vec::new(),
        AurHelper::Paru => "paru",
        AurHelper::Yay => "yay",
        AurHelper::Auto => {
            if is_command_in_path("paru") {
                "paru"
            } else if is_command_in_path("yay") {
                "yay"
            } else {
                return Vec::new();
            }
        }
    };

    let check_future = Command::new(helper)
        .arg("-Qua")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match timeout(Duration::from_secs(15), check_future).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_aur_output(&stdout)
        }
        _ => Vec::new(),
    }
}

pub fn parse_aur_output(output: &str) -> Vec<PackageUpdate> {
    let mut updates = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[2] == "->" {
            updates.push(PackageUpdate {
                name: parts[0].to_string(),
                old_version: parts[1].to_string(),
                new_version: parts[3].to_string(),
            });
        } else if !parts.is_empty() {
            updates.push(PackageUpdate {
                name: parts[0].to_string(),
                old_version: "unknown".to_string(),
                new_version: parts.get(1).unwrap_or(&"latest").to_string(),
            });
        }
    }
    updates
}
