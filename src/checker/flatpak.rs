use super::arch_repo::PackageUpdate;
use super::aur::is_command_in_path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub async fn check_flatpak_updates() -> Vec<PackageUpdate> {
    if !is_command_in_path("flatpak") {
        return Vec::new();
    }

    let check_future = Command::new("flatpak")
        .args(["remote-ls", "--updates", "--columns=application,version"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match timeout(Duration::from_secs(15), check_future).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_flatpak_output(&stdout)
        }
        _ => Vec::new(),
    }
}

pub fn parse_flatpak_output(output: &str) -> Vec<PackageUpdate> {
    let mut updates = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            updates.push(PackageUpdate {
                name: parts[0].to_string(),
                old_version: "current".to_string(),
                new_version: parts.get(1).unwrap_or(&"latest").to_string(),
            });
        }
    }
    updates
}
