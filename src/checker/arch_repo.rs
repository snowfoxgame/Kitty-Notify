use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageUpdate {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
}

pub async fn check_arch_updates() -> Vec<PackageUpdate> {
    let check_future = Command::new("checkupdates")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let output = match timeout(Duration::from_secs(20), check_future).await {
        Ok(Ok(out)) => out,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_checkupdates_output(&stdout)
}

pub fn parse_checkupdates_output(output: &str) -> Vec<PackageUpdate> {
    let mut updates = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[2] == "->" {
            updates.push(PackageUpdate {
                name: parts[0].to_string(),
                old_version: parts[1].to_string(),
                new_version: parts[3].to_string(),
            });
        }
    }
    updates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checkupdates() {
        let sample = "linux 6.12.1.arch1-1 -> 6.12.2.arch1-1\nmesa 1:24.3.0-1 -> 1:24.3.1-1\n";
        let parsed = parse_checkupdates_output(sample);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "linux");
        assert_eq!(parsed[0].old_version, "6.12.1.arch1-1");
        assert_eq!(parsed[0].new_version, "6.12.2.arch1-1");
    }
}
