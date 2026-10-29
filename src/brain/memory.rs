use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static UNCLEAN_SHUTDOWN: AtomicBool = AtomicBool::new(false);
static SESSION_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Evaluates running.lock exactly once on application startup.
/// Checks if the stored PID in running.lock is still alive via `/proc/{pid}`.
/// If the process is dead and no clean exit was registered, marks unclean_shutdown = true.
/// Then writes the current process PID to running.lock.
pub fn initialize_session() {
    if SESSION_INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);

    let marker = running_marker_path();
    let lock = running_lock_path();

    let mut unclean = false;
    if lock.exists() || marker.exists() {
        let pid_opt = fs::read_to_string(&lock)
            .or_else(|_| fs::read_to_string(&marker))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());

        if let Some(pid) = pid_opt {
            let proc_stat = PathBuf::from(format!("/proc/{}", pid));
            if !proc_stat.exists() {
                // Stored PID is dead and lock was never cleared -> unclean shutdown!
                unclean = true;
            }
        } else {
            unclean = true;
        }
    }

    UNCLEAN_SHUTDOWN.store(unclean, Ordering::SeqCst);

    // Record current process PID in lock files
    let pid_str = format!("{}", std::process::id());
    let _ = fs::write(&marker, &pid_str);
    let _ = fs::write(&lock, &pid_str);
}

pub fn was_unclean_shutdown() -> bool {
    UNCLEAN_SHUTDOWN.load(Ordering::SeqCst)
}

pub fn clear_unclean_shutdown() {
    UNCLEAN_SHUTDOWN.store(false, Ordering::SeqCst);
}

/// Path to local data directory (~/.local/share/kittynotify)
pub fn data_dir() -> PathBuf {
    crate::config::Config::data_dir()
}

/// Path to persistent memory file
pub fn memory_path() -> PathBuf {
    data_dir().join("memory.json")
}

/// Path to clean-exit marker file.
/// If present on startup, the previous session crashed or shut down abruptly.
pub fn running_marker_path() -> PathBuf {
    data_dir().join(".running")
}

/// Path to running lock file (~/.local/share/kittynotify/running.lock).
pub fn running_lock_path() -> PathBuf {
    data_dir().join("running.lock")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatMemory {
    /// Number of consecutive times updates were dismissed without being applied
    #[serde(default)]
    pub dismiss_streak: u32,

    /// Total count of update cycles applied over lifetime
    #[serde(default)]
    pub total_updates_applied: u64,

    /// Frequency count of top applications seen over time
    #[serde(default)]
    pub app_frequencies: HashMap<String, u64>,

    /// Count of sessions active during late-night hours (00:00 - 05:00)
    #[serde(default)]
    pub late_night_sessions: u32,

    /// Last clean shutdown timestamp (ISO-8601 or formatted string)
    #[serde(default)]
    pub last_clean_shutdown: Option<String>,

    /// Timestamp of last pacman cache alert shown (unix seconds, for cooldown)
    #[serde(default)]
    pub last_cache_alert_ts: u64,

    /// Timestamp of last orphan packages alert shown (unix seconds, for cooldown)
    #[serde(default)]
    pub last_orphan_alert_ts: u64,

    /// Timestamp of last failed systemd service alert shown (unix seconds, for cooldown)
    #[serde(default)]
    pub last_failed_service_alert_ts: u64,

    /// In-memory runtime flag indicating whether an unclean shutdown occurred prior to this launch
    #[serde(skip)]
    pub was_unclean_shutdown: bool,

    /// In-memory dirty sample counter for batching disk writes
    #[serde(skip)]
    pub pending_dirty_ticks: u32,
}

impl CatMemory {
    pub fn load_from(path: &std::path::Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return serde_json::from_str::<CatMemory>(&content).unwrap_or_default();
            }
        }
        Self::default()
    }

    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        fs::write(path, json)
    }

    /// Loads memory from ~/.local/share/kittynotify/memory.json and reflects session shutdown status.
    /// Does NOT perform lockfile writes or re-evaluate /proc.
    pub fn load() -> Self {
        let mut memory = Self::load_from(&memory_path());
        memory.was_unclean_shutdown = was_unclean_shutdown();
        memory
    }

    /// Saves the current memory state to ~/.local/share/kittynotify/memory.json.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&memory_path())
    }

    /// Flushes any pending unwritten dirty ticks to disk.
    pub fn flush(&mut self) {
        if self.pending_dirty_ticks > 0 {
            self.pending_dirty_ticks = 0;
            let _ = self.save();
        }
    }

    /// Marks the current session as cleanly exited by removing the running.lock marker
    /// and stamping last_clean_shutdown.
    pub fn mark_clean_exit(&mut self) {
        clear_unclean_shutdown();
        self.was_unclean_shutdown = false;
        let marker = running_marker_path();
        if marker.exists() {
            let _ = fs::remove_file(marker);
        }
        let lock = running_lock_path();
        if lock.exists() {
            let _ = fs::remove_file(lock);
        }
        let now_str = glib::DateTime::now_local()
            .and_then(|dt| dt.format_iso8601())
            .map(|g_str| g_str.to_string())
            .unwrap_or_else(|_| "clean".to_string());
        self.last_clean_shutdown = Some(now_str);
        self.pending_dirty_ticks = 0;
        let _ = self.save();
    }

    /// Wipes memory.json and resets memory tracking completely.
    pub fn reset() -> Self {
        let path = memory_path();
        if path.exists() {
            let _ = fs::remove_file(path);
        }
        let marker = running_marker_path();
        if marker.exists() {
            let _ = fs::remove_file(marker);
        }
        let lock = running_lock_path();
        if lock.exists() {
            let _ = fs::remove_file(lock);
        }
        let fresh = Self::default();
        let _ = fresh.save();
        fresh
    }

    /// Increments the dismiss streak counter when the user skips/dismisses pending updates.
    pub fn record_dismiss(&mut self) {
        self.dismiss_streak += 1;
        let _ = self.save();
    }

    /// Resets dismiss streak to 0 and counts total updates applied.
    pub fn record_update_applied(&mut self) {
        self.dismiss_streak = 0;
        self.total_updates_applied += 1;
        let _ = self.save();
    }

    /// Records an observed active process name into habit frequencies (batched disk writes).
    pub fn record_app_seen(&mut self, app: &str) {
        let clean = app.trim().to_lowercase();
        if clean.is_empty()
            || clean == "system"
            || clean == "unknown"
            || clean.len() < 2
        {
            return;
        }
        *self.app_frequencies.entry(clean).or_insert(0) += 1;
        self.pending_dirty_ticks += 1;
        if self.pending_dirty_ticks >= 20 {
            self.pending_dirty_ticks = 0;
            let _ = self.save();
        }
    }

    /// Returns the most frequently used app if it has been observed at least 3 times.
    pub fn favorite_app(&self) -> Option<String> {
        self.app_frequencies
            .iter()
            .filter(|(_, &count)| count >= 3)
            .max_by_key(|(_, &count)| count)
            .map(|(app, _)| app.clone())
    }

    /// Checks if the current local time is in late-night hours (00:00 - 05:00).
    pub fn is_late_night() -> bool {
        if let Ok(dt) = glib::DateTime::now_local() {
            let hour = dt.hour();
            hour < 5
        } else {
            false
        }
    }

    /// Checks if today is a weekday (Monday - Friday).
    pub fn is_workday() -> bool {
        if let Ok(dt) = glib::DateTime::now_local() {
            let day = dt.day_of_week(); // 1 = Monday, 7 = Sunday in GLib
            (1..=5).contains(&day)
        } else {
            true
        }
    }

    /// Records late-night activity if currently running during late hours.
    pub fn check_and_record_late_night(&mut self) {
        if Self::is_late_night() {
            self.late_night_sessions += 1;
            let _ = self.save();
        }
    }

    /// Returns progressively worried/sassy dialogue if updates have been dismissed 3+ times.
    pub fn get_streak_dialogue(&self) -> Option<String> {
        if self.dismiss_streak < 3 {
            return None;
        }
        match self.dismiss_streak {
            3 => Some("Dismissed again? Day 3 without updates... our Arch is getting dusty! :3".to_string()),
            4 => Some("4 dismissals in a row?! The kernel is weeping gently in the corner ;w;".to_string()),
            5 => Some("Streak of 5 dismissals! At this rate, Debian will feel newer than our Arch!".to_string()),
            n => Some(format!("Streak of {} dismissals! Pacman misses you dearly :3", n)),
        }
    }

    /// Returns crash recovery dialogue if the previous session exited unexpectedly.
    /// Consumes the flag so it only triggers once.
    pub fn get_recovery_dialogue(&mut self) -> Option<String> {
        if self.was_unclean_shutdown {
            self.was_unclean_shutdown = false;
            Some("Puh... that crash earlier was scary. Are we all intact?".to_string())
        } else {
            None
        }
    }

    /// Returns habit-aware dialogue based on time-of-day or top applications.
    pub fn get_habit_dialogue(&self) -> Option<String> {
        if Self::is_late_night() {
            if Self::is_workday() {
                return Some("Gaming past midnight on a workday again? Go get some sleep!".to_string());
            } else {
                return Some("Still hacking past midnight? Don't forget to blink, night owl :3".to_string());
            }
        }

        if let Some(fav) = self.favorite_app() {
            if self.total_updates_applied.is_multiple_of(3) {
                return Some(format!("You spend so much time in {}! Do you love it more than me? :3", fav));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_defaults_and_streak() {
        let mut mem = CatMemory::default();
        assert_eq!(mem.dismiss_streak, 0);
        assert_eq!(mem.get_streak_dialogue(), None);

        mem.dismiss_streak += 1;
        mem.dismiss_streak += 1;
        assert_eq!(mem.dismiss_streak, 2);
        assert_eq!(mem.get_streak_dialogue(), None);

        mem.dismiss_streak += 1;
        assert_eq!(mem.dismiss_streak, 3);
        assert!(mem.get_streak_dialogue().unwrap().contains("getting dusty"));

        mem.dismiss_streak += 1;
        assert_eq!(mem.dismiss_streak, 4);
        assert!(mem.get_streak_dialogue().unwrap().contains("kernel is weeping"));

        mem.dismiss_streak = 0;
        mem.total_updates_applied += 1;
        assert_eq!(mem.dismiss_streak, 0);
        assert_eq!(mem.total_updates_applied, 1);
        assert_eq!(mem.get_streak_dialogue(), None);
    }

    #[test]
    fn test_app_frequency_favorite() {
        let mut mem = CatMemory::default();
        assert_eq!(mem.favorite_app(), None);

        *mem.app_frequencies.entry("firefox".to_string()).or_insert(0) += 1;
        *mem.app_frequencies.entry("firefox".to_string()).or_insert(0) += 1;
        assert_eq!(mem.favorite_app(), None); // needs at least 3

        *mem.app_frequencies.entry("firefox".to_string()).or_insert(0) += 1;
        assert_eq!(mem.favorite_app(), Some("firefox".to_string()));

        *mem.app_frequencies.entry("steam".to_string()).or_insert(0) += 4;
        assert_eq!(mem.favorite_app(), Some("steam".to_string()));
    }

    #[test]
    fn test_recovery_dialogue_consumed() {
        let mut mem = CatMemory {
            was_unclean_shutdown: true,
            ..Default::default()
        };
        assert!(mem.get_recovery_dialogue().is_some());
        assert!(mem.get_recovery_dialogue().is_none());
    }

    #[test]
    fn test_persistence_to_disk() {
        let tmp = std::env::temp_dir().join(format!("kittynotify_test_{}_{}.json", std::process::id(), glib::monotonic_time()));
        let mem = CatMemory {
            total_updates_applied: 7,
            dismiss_streak: 3,
            ..Default::default()
        };
        mem.save_to(&tmp).expect("Failed to save memory to temp file");

        let reloaded = CatMemory::load_from(&tmp);
        assert_eq!(reloaded.total_updates_applied, 7);
        assert_eq!(reloaded.dismiss_streak, 3);

        let _ = fs::remove_file(tmp);
    }
}
