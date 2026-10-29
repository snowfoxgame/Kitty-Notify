pub mod arch_repo;
pub mod aur;
pub mod flatpak;

use crate::config::Config;
use arch_repo::PackageUpdate;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSummary {
    pub repo: Vec<PackageUpdate>,
    pub aur: Vec<PackageUpdate>,
    pub flatpak: Vec<PackageUpdate>,
}

impl UpdateSummary {
    pub fn total(&self) -> usize {
        self.repo.len() + self.aur.len() + self.flatpak.len()
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheState {
    pub last_checked: u64,
    pub repo_count: usize,
    pub aur_count: usize,
    pub flatpak_count: usize,
    pub dismissed: bool,
}

impl CacheState {
    pub fn state_file_path() -> PathBuf {
        Config::cache_dir().join("state.json")
    }

    pub fn load() -> Self {
        let path = Self::state_file_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&data) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Config::cache_dir();
        fs::create_dir_all(&dir)?;
        let data = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        fs::write(Self::state_file_path(), data)
    }

    pub fn should_notify(&self, summary: &UpdateSummary, force: bool) -> bool {
        if force {
            return true;
        }
        if summary.is_empty() {
            return false;
        }
        if summary.repo.len() != self.repo_count
            || summary.aur.len() != self.aur_count
            || summary.flatpak.len() != self.flatpak_count
        {
            return true;
        }
        !self.dismissed
    }

    pub fn record_check(&mut self, summary: &UpdateSummary) {
        self.last_checked = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.repo_count = summary.repo.len();
        self.aur_count = summary.aur.len();
        self.flatpak_count = summary.flatpak.len();
        self.dismissed = false;
        let _ = self.save();
    }
}

pub async fn run_update_check(config: &Config) -> UpdateSummary {
    let (raw_repo, raw_aur, raw_flatpak) = tokio::join!(
        arch_repo::check_arch_updates(),
        aur::check_aur_updates(config.updates.preferred_aur_helper),
        async {
            if config.updates.check_flatpaks {
                flatpak::check_flatpak_updates().await
            } else {
                Vec::new()
            }
        }
    );

    let repo: Vec<PackageUpdate> = raw_repo
        .into_iter()
        .filter(|p| !config.is_package_ignored(&p.name))
        .collect();

    let aur: Vec<PackageUpdate> = raw_aur
        .into_iter()
        .filter(|p| !config.is_package_ignored(&p.name))
        .collect();

    let flatpak: Vec<PackageUpdate> = raw_flatpak
        .into_iter()
        .filter(|p| !config.is_package_ignored(&p.name))
        .collect();

    UpdateSummary {
        repo,
        aur,
        flatpak,
    }
}
