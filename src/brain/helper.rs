use crate::brain::memory::CatMemory;
use crate::config::Config;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperAlert {
    pub title: String,
    pub message: String,
    pub action_label: String,
    pub command: String,
}

/// Helper cooldown in seconds (1 hour between same alert types to prevent nagging)
pub const ALERT_COOLDOWN_SECS: u64 = 3600;

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Computes the disk usage of /var/cache/pacman/pkg in Gigabytes.
pub fn get_pacman_cache_size_gb() -> f32 {
    let path = Path::new("/var/cache/pacman/pkg");
    if !path.exists() {
        return 0.0;
    }

    let mut total_bytes: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                total_bytes += meta.len();
            }
        }
    }

    total_bytes as f32 / (1024.0 * 1024.0 * 1024.0)
}

/// Checks the number of orphaned packages via pacman -Qtdq.
pub fn get_orphan_packages_count() -> usize {
    let output = Command::new("pacman").args(["-Qtdq"]).output();
    if let Ok(out) = output {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.lines().filter(|l| !l.trim().is_empty()).count();
        }
    }
    0
}

/// Checks whether any systemd units are currently in a failed state.
pub fn has_failed_systemd_units() -> bool {
    let status = Command::new("systemctl")
        .args(["--failed", "--quiet"])
        .status();
    match status {
        Ok(s) => !s.success(), // exit code non-zero indicates failed units
        Err(_) => false,
    }
}

/// Evaluates system maintenance conditions and returns a proactive HelperAlert if an issue is found.
pub fn check_system_helpers(config: &Config, memory: &mut CatMemory) -> Option<HelperAlert> {
    if !config.smart.enabled {
        return None;
    }

    let now = current_unix_timestamp();

    // 1. Pacman cache overflow alert
    if config.smart.pacman_cache_alert && now.saturating_sub(memory.last_cache_alert_ts) >= ALERT_COOLDOWN_SECS {
        let cache_gb = get_pacman_cache_size_gb();
        if cache_gb >= config.smart.pacman_cache_limit_gb {
            memory.last_cache_alert_ts = now;
            let _ = memory.save();
            return Some(HelperAlert {
                title: "Nya! Cache Alert".to_string(),
                message: format!(
                    "Hey! The pacman cache is eating up {:.1} GB of disk space. Clean it up?",
                    cache_gb
                ),
                action_label: "CLEAN CACHE".to_string(),
                command: "paccache -rk2".to_string(),
            });
        }
    }

    // 2. Orphan packages alert
    if config.smart.orphan_alert && now.saturating_sub(memory.last_orphan_alert_ts) >= ALERT_COOLDOWN_SECS {
        let orphans = get_orphan_packages_count();
        if orphans >= config.smart.orphan_limit {
            memory.last_orphan_alert_ts = now;
            let _ = memory.save();
            return Some(HelperAlert {
                title: "Nya! Stray Packages".to_string(),
                message: format!(
                    "Found {} orphaned packages lingering around like stray hairs!",
                    orphans
                ),
                action_label: "REMOVE ORPHANS".to_string(),
                command: "sudo pacman -Rns $(pacman -Qtdq)".to_string(),
            });
        }
    }

    // 3. Failed systemd service alert
    if config.smart.failed_services_alert
        && now.saturating_sub(memory.last_failed_service_alert_ts) >= ALERT_COOLDOWN_SECS
        && has_failed_systemd_units()
    {
        memory.last_failed_service_alert_ts = now;
        let _ = memory.save();
        return Some(HelperAlert {
            title: "Nya! Service Alert".to_string(),
            message: "Warning! A background systemd unit crashed recently :o".to_string(),
            action_label: "INSPECT".to_string(),
            command: "systemctl --failed".to_string(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_cache_size_non_negative() {
        let size = get_pacman_cache_size_gb();
        assert!(size >= 0.0);
    }
}
