//! Centralized dialogue, quotes, system resource evaluator, and autonomous banter engine.

use crate::config::Personality;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::OnceLock;
#[cfg(feature = "smart")]
use sysinfo::{Components, System};

/// Embedded dialogue files categorized by personality and behavior.
pub const DIALOGUE_CUTE_TXT: &str = include_str!("../../assets/dialogue/cute.txt");
pub const DIALOGUE_SASSY_TXT: &str = include_str!("../../assets/dialogue/sassy.txt");
pub const DIALOGUE_LAZY_TXT: &str = include_str!("../../assets/dialogue/lazy.txt");
pub const DIALOGUE_SYSTEM_TXT: &str = include_str!("../../assets/dialogue/system.txt");
pub const DIALOGUE_GENERAL_TXT: &str = include_str!("../../assets/dialogue/general.txt");
#[allow(dead_code)]
pub const DEFAULT_DIALOGUE_TXT: &str = include_str!("../../assets/dialogue.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueCategory {
    /// High load driven by a specific resource-heavy process ({app} / {top_process})
    AppLoad,
    /// Extreme overall system load, gaming, or high CPU temperatures
    ExtremeLoad,
    /// Low memory, high RAM usage, or swapping
    RamPanic,
    /// Relaxed idle banter, fun Arch quips, and status reminders
    GeneralIdle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueLine {
    pub text: String,
    pub category: DialogueCategory,
}

/// Reads a dialogue file from user config directory if present, otherwise returns the embedded default.
pub fn load_dialogue_text(name: &str, embedded: &str) -> String {
    let user_path = crate::config::Config::config_dir()
        .join("dialogue")
        .join(format!("{}.txt", name));
    if user_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&user_path) {
            return content;
        }
    }
    embedded.to_string()
}

/// Parses lines of dialogue, stripping comments, empty lines, and quotation marks.
pub fn parse_dialogue_lines(
    content: &str,
    default_category: DialogueCategory,
) -> Vec<DialogueLine> {
    let mut lines = Vec::new();
    let mut current_category = default_category;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header recognition
        let lower = line.to_lowercase();
        if lower.starts_with("[app_load]")
            || lower.contains("high load by specific apps")
            || lower.starts_with("1.")
        {
            current_category = DialogueCategory::AppLoad;
            continue;
        } else if lower.starts_with("[extreme_load]")
            || lower.contains("extreme system load")
            || lower.starts_with("2.")
        {
            current_category = DialogueCategory::ExtremeLoad;
            continue;
        } else if lower.starts_with("[ram_panic]")
            || lower.contains("ram checks")
            || lower.starts_with("3.")
        {
            current_category = DialogueCategory::RamPanic;
            continue;
        } else if lower.starts_with("[idle]")
            || lower.contains("relaxed & random idle")
            || lower.starts_with("4.")
            || lower.contains("arch & package")
            || lower.contains("system health")
            || lower.contains("helpful")
            || lower.contains("cute")
        {
            current_category = DialogueCategory::GeneralIdle;
            continue;
        }

        // Extract quote from line (e.g. enclosed in quotes or raw)
        let quote = if line.starts_with('"') && line.ends_with('"') && line.len() >= 2 {
            &line[1..line.len() - 1]
        } else {
            line
        }
        .trim();

        if quote.is_empty() {
            continue;
        }

        // Categorize by placeholder tags if ambiguous
        let mut cat = current_category;
        if quote.contains("{app}") || quote.contains("{top_process}") {
            cat = DialogueCategory::AppLoad;
        } else if quote.contains("{cpu_temp}") || quote.contains("{cpu_usage_percent}") {
            cat = DialogueCategory::ExtremeLoad;
        } else if (quote.contains("{ram_usage_percent}")
            || quote.contains("{ram_total}")
            || quote.contains("{ram_used}"))
            && cat != DialogueCategory::AppLoad
            && cat != DialogueCategory::ExtremeLoad
        {
            cat = DialogueCategory::RamPanic;
        }

        lines.push(DialogueLine {
            text: quote.to_string(),
            category: cat,
        });
    }

    lines
}

/// Parses a dialogue text file into structured lines categorized by load requirements.
#[allow(dead_code)]
pub fn parse_dialogue_file(content: &str) -> Vec<DialogueLine> {
    parse_dialogue_lines(content, DialogueCategory::GeneralIdle)
}

static CUTE_POOL: OnceLock<Vec<DialogueLine>> = OnceLock::new();
static SASSY_POOL: OnceLock<Vec<DialogueLine>> = OnceLock::new();
static LAZY_POOL: OnceLock<Vec<DialogueLine>> = OnceLock::new();
static NORMAL_POOL: OnceLock<Vec<DialogueLine>> = OnceLock::new();

/// Pools dialogue lines based on the selected cat personality:
/// - `Cute`: cute.txt + system.txt
/// - `Sassy`: sassy.txt + system.txt
/// - `Lazy`: lazy.txt
/// - `Normal`: cute.txt + sassy.txt + lazy.txt + system.txt + general.txt
pub fn get_personality_dialogue_pool(personality: Personality) -> &'static [DialogueLine] {
    match personality {
        Personality::Cute => CUTE_POOL.get_or_init(|| {
            let cute_str = load_dialogue_text("cute", DIALOGUE_CUTE_TXT);
            let system_str = load_dialogue_text("system", DIALOGUE_SYSTEM_TXT);
            let mut pool = parse_dialogue_lines(&cute_str, DialogueCategory::GeneralIdle);
            pool.extend(parse_dialogue_lines(&system_str, DialogueCategory::GeneralIdle));
            pool
        }),
        Personality::Sassy => SASSY_POOL.get_or_init(|| {
            let sassy_str = load_dialogue_text("sassy", DIALOGUE_SASSY_TXT);
            let system_str = load_dialogue_text("system", DIALOGUE_SYSTEM_TXT);
            let mut pool = parse_dialogue_lines(&sassy_str, DialogueCategory::GeneralIdle);
            pool.extend(parse_dialogue_lines(&system_str, DialogueCategory::GeneralIdle));
            pool
        }),
        Personality::Lazy => LAZY_POOL.get_or_init(|| {
            let lazy_str = load_dialogue_text("lazy", DIALOGUE_LAZY_TXT);
            parse_dialogue_lines(&lazy_str, DialogueCategory::GeneralIdle)
        }),
        Personality::Normal => NORMAL_POOL.get_or_init(|| {
            let cute_str = load_dialogue_text("cute", DIALOGUE_CUTE_TXT);
            let sassy_str = load_dialogue_text("sassy", DIALOGUE_SASSY_TXT);
            let lazy_str = load_dialogue_text("lazy", DIALOGUE_LAZY_TXT);
            let system_str = load_dialogue_text("system", DIALOGUE_SYSTEM_TXT);
            let general_str = load_dialogue_text("general", DIALOGUE_GENERAL_TXT);
            let mut pool = parse_dialogue_lines(&cute_str, DialogueCategory::GeneralIdle);
            pool.extend(parse_dialogue_lines(&sassy_str, DialogueCategory::GeneralIdle));
            pool.extend(parse_dialogue_lines(&lazy_str, DialogueCategory::GeneralIdle));
            pool.extend(parse_dialogue_lines(&system_str, DialogueCategory::GeneralIdle));
            pool.extend(parse_dialogue_lines(&general_str, DialogueCategory::GeneralIdle));
            pool
        }),
    }
}

/// A live snapshot of current hardware metrics used for banter and placeholder interpolation.
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub total_ram_gb: u32,
    pub used_ram_gb: f32,
    pub ram_usage_percent: u32,
    pub top_process_name: String,
    pub top_process_cpu: f32,
    pub top_process_mem_mb: f32,
    pub global_cpu_percent: u32,
    pub cpu_temp_celsius: u32,
    pub uptime_secs: u64,
    pub uptime_formatted: String,
    pub last_update_formatted: String,
}

#[cfg(feature = "smart")]
impl SystemSnapshot {
    /// Inspects live system hardware resources.
    pub fn gather() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_mem_bytes = sys.total_memory();
        let used_mem_bytes = sys.used_memory();
        let total_ram_gb =
            ((total_mem_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32).max(1);
        let used_ram_gb = (used_mem_bytes as f64 / (1024.0 * 1024.0 * 1024.0)) as f32;
        let ram_usage_percent = if total_mem_bytes > 0 {
            ((used_mem_bytes as f64 / total_mem_bytes as f64) * 100.0)
                .round()
                .clamp(0.0, 100.0) as u32
        } else {
            0
        };

        let global_cpu_percent = sys.global_cpu_usage().round().clamp(0.0, 100.0) as u32;

        // Inspect running processes to find the top user application
        let ignore_list = [
            "systemd",
            "kthreadd",
            "dbus-broker",
            "pipewire",
            "wireplumber",
            "kwin_wayland",
            "Xwayland",
            "kitty",
            "konsole",
            "alacritty",
            "bash",
            "zsh",
            "fish",
            "kittynotify",
        ];

        let mut top_name = String::new();
        let mut top_cpu = 0.0f32;
        let mut top_mem_mb = 0.0f32;
        let mut highest_score = 0.0f32;

        for proc in sys.processes().values() {
            let raw_name = proc.name().to_string_lossy().trim().to_string();
            if raw_name.is_empty() {
                continue;
            }
            if ignore_list
                .iter()
                .any(|&ign| raw_name.eq_ignore_ascii_case(ign))
            {
                continue;
            }

            let cpu = proc.cpu_usage();
            let mem_mb = proc.memory() as f32 / (1024.0 * 1024.0);
            let score = cpu + (mem_mb / 80.0);
            if score > highest_score {
                highest_score = score;
                top_name = raw_name;
                top_cpu = cpu;
                top_mem_mb = mem_mb;
            }
        }

        if top_name.is_empty() {
            top_name = "system".to_string();
        }

        let clean_app = clean_process_name(&top_name);

        // Hardware CPU temperature
        let comps = Components::new_with_refreshed_list();
        let mut max_temp = 0.0f32;
        for c in &comps {
            let label = c.label().to_lowercase();
            if label.contains("cpu")
                || label.contains("core")
                || label.contains("tctl")
                || label.contains("tdie")
                || label.contains("k10temp")
            {
                if let Some(t) = c.temperature() {
                    if t > max_temp {
                        max_temp = t;
                    }
                }
            }
        }
        if max_temp == 0.0 {
            for c in &comps {
                if let Some(t) = c.temperature() {
                    if t > max_temp {
                        max_temp = t;
                    }
                }
            }
        }
        let cpu_temp_celsius = if max_temp > 0.0 {
            max_temp.round().clamp(20.0, 115.0) as u32
        } else {
            48
        };

        let uptime_secs = System::uptime();
        let uptime_formatted = format_uptime(uptime_secs);

        let last_update_formatted = get_last_update_elapsed_string()
            .unwrap_or_else(|| "Last update was recently".to_string());

        Self {
            total_ram_gb,
            used_ram_gb,
            ram_usage_percent,
            top_process_name: clean_app,
            top_process_cpu: top_cpu,
            top_process_mem_mb: top_mem_mb,
            global_cpu_percent,
            cpu_temp_celsius,
            uptime_secs,
            uptime_formatted,
            last_update_formatted,
        }
    }

    /// True if an application is using notable resources (>20% CPU or >1.5GB RAM)
    pub fn is_app_heavy(&self) -> bool {
        self.top_process_cpu >= 20.0 || self.top_process_mem_mb >= 1500.0
    }

    /// True if system is under heavy gaming or high stress (>50% CPU or >75°C)
    pub fn is_extreme_load(&self) -> bool {
        self.global_cpu_percent >= 50 || self.cpu_temp_celsius >= 75
    }

    /// True if memory is constrained (>75% occupied)
    pub fn is_ram_panic(&self) -> bool {
        self.ram_usage_percent >= 75
    }
}

#[cfg(not(feature = "smart"))]
impl SystemSnapshot {
    pub fn gather() -> Self {
        let uptime_secs = std::fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| s.split_whitespace().next().and_then(|val| val.parse::<f64>().ok()))
            .map(|s| s as u64)
            .unwrap_or(0);
        let uptime_formatted = format_uptime(uptime_secs);
        let last_update_formatted = get_last_update_elapsed_string()
            .unwrap_or_else(|| "Last update was recently".to_string());
        Self {
            total_ram_gb: 16,
            used_ram_gb: 4.0,
            ram_usage_percent: 25,
            top_process_name: "system".to_string(),
            top_process_cpu: 0.0,
            top_process_mem_mb: 0.0,
            global_cpu_percent: 5,
            cpu_temp_celsius: 45,
            uptime_secs,
            uptime_formatted,
            last_update_formatted,
        }
    }

    pub fn is_app_heavy(&self) -> bool {
        false
    }

    pub fn is_extreme_load(&self) -> bool {
        false
    }

    pub fn is_ram_panic(&self) -> bool {
        false
    }
}

/// Cleans and formats raw process names into friendly display names.
#[allow(dead_code)]
pub fn clean_process_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    if lower.contains("firefox") {
        "Firefox".to_string()
    } else if lower.contains("discord") || lower.contains("vesktop") {
        "Discord".to_string()
    } else if lower.contains("steam") {
        "Steam".to_string()
    } else if lower.contains("rbx") || lower.contains("roblox") {
        "Roblox".to_string()
    } else if lower.contains("minecraft") || lower.contains("javaw") {
        "Minecraft".to_string()
    } else if lower.contains("code") {
        "VS Code".to_string()
    } else if lower.contains("chrome") || lower.contains("chromium") {
        "Chrome".to_string()
    } else if lower.contains("spotify") {
        "Spotify".to_string()
    } else if lower.contains("blender") {
        "Blender".to_string()
    } else if lower.contains("obs") {
        "OBS Studio".to_string()
    } else if lower.contains("antigravity") {
        "Antigravity".to_string()
    } else {
        let mut chars = trimmed.chars();
        match chars.next() {
            None => "App".to_string(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

/// Formats system uptime in seconds to a friendly human-readable string.
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 1 {
        format!("{} days", days)
    } else if days == 1 {
        if hours > 0 {
            format!("1 day, {} hours", hours)
        } else {
            "1 day".to_string()
        }
    } else if hours > 1 {
        format!("{} hours", hours)
    } else if hours == 1 {
        format!("1 hour, {} mins", mins)
    } else if mins > 1 {
        format!("{} minutes", mins)
    } else {
        "just started".to_string()
    }
}

/// Replaces all dynamic hardware placeholders in a quote template.
pub fn interpolate_placeholders(template: &str, snapshot: &SystemSnapshot) -> String {
    let mut result = template.to_string();

    if result.trim() == "{last_update}" {
        return snapshot.last_update_formatted.clone();
    }

    result = result.replace("{app}", &snapshot.top_process_name);
    result = result.replace("{top_process}", &snapshot.top_process_name);
    result = result.replace("{ram_total}", &format!("{}", snapshot.total_ram_gb));
    result = result.replace("{ram_used}", &format!("{:.1} GB", snapshot.used_ram_gb));
    result = result.replace(
        "{ram_usage_percent}",
        &format!("{}", snapshot.ram_usage_percent),
    );
    result = result.replace(
        "{cpu_usage_percent}",
        &format!("{}", snapshot.global_cpu_percent),
    );
    result = result.replace("{cpu_temp}", &format!("{}", snapshot.cpu_temp_celsius));
    result = result.replace("{uptime}", &snapshot.uptime_formatted);
    result = result.replace("{last_update}", &snapshot.last_update_formatted);

    result
}

/// Evaluates current system state and returns a smart, context-aware dialogue line for a given personality using a provided system snapshot.
pub fn get_smart_dialogue_for_personality_with_snapshot(
    personality: Personality,
    snapshot: &SystemSnapshot,
) -> String {
    #[cfg(feature = "smart")]
    {
        let memory = crate::brain::CatMemory::load();
        if let Some(quote) = crate::brain::evaluate_behavioral_dialogue_with_snapshot(&memory, personality, snapshot) {
            return quote;
        }
    }

    let pool = get_personality_dialogue_pool(personality);

    let is_app_heavy = snapshot.is_app_heavy();
    let is_extreme_load = snapshot.is_extreme_load();
    let is_ram_panic = snapshot.is_ram_panic();

    let mut eligible: Vec<&DialogueLine> = Vec::new();

    for line in pool {
        match line.category {
            DialogueCategory::AppLoad => {
                if is_app_heavy {
                    eligible.push(line);
                }
            }
            DialogueCategory::ExtremeLoad => {
                if is_extreme_load {
                    eligible.push(line);
                }
            }
            DialogueCategory::RamPanic => {
                if is_ram_panic {
                    eligible.push(line);
                }
            }
            DialogueCategory::GeneralIdle => {
                eligible.push(line);
            }
        }
    }

    let mut rng = crate::brain::create_seeded_rng();

    if eligible.is_empty() {
        if !pool.is_empty() {
            let chosen = crate::brain::select_unique_candidate(pool, |l| &l.text, &mut rng)
                .unwrap_or(&pool[0]);
            return interpolate_placeholders(&chosen.text, snapshot);
        }
        return "purr... i use arch btw :3".to_string();
    }

    let chosen = crate::brain::select_unique_candidate(&eligible, |l| &l.text, &mut rng)
        .unwrap_or(&eligible[0]);

    interpolate_placeholders(&chosen.text, snapshot)
}

/// Evaluates current system state and returns a smart, context-aware dialogue line for a given personality.
pub fn get_smart_dialogue_for_personality(personality: Personality) -> String {
    let snapshot = SystemSnapshot::gather();
    get_smart_dialogue_for_personality_with_snapshot(personality, &snapshot)
}

/// Evaluates current system state and returns a dialogue line using the user's active personality setting.
pub fn get_smart_dialogue() -> String {
    let personality = crate::config::Config::load().banter.personality;
    get_smart_dialogue_for_personality(personality)
}

/// Legacy entry point: returns a smart, dynamic quip evaluated from current system state.
pub fn get_random_quip() -> String {
    get_smart_dialogue()
}

/// Computes a friendly string describing when the system was last updated.
/// Tries reading `/var/log/pacman.log` first, then falls back to `CacheState`.
pub fn get_last_update_elapsed_string() -> Option<String> {
    if let Some(elapsed) = parse_last_update_pacman() {
        return Some(elapsed);
    }

    let state = crate::checker::CacheState::load();
    if state.last_checked > 0 {
        if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            let now_sec = duration.as_secs();
            if now_sec >= state.last_checked {
                let diff_sec = now_sec - state.last_checked;
                return Some(format_elapsed_seconds(diff_sec));
            }
        }
    }

    None
}

/// Reads the end of `/var/log/pacman.log` to find the last transaction completion timestamp.
fn parse_last_update_pacman() -> Option<String> {
    let path = Path::new("/var/log/pacman.log");
    let mut file = File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    let file_len = meta.len();

    let read_len = file_len.min(64 * 1024) as usize;
    if read_len == 0 {
        return None;
    }

    file.seek(SeekFrom::End(-(read_len as i64))).ok()?;
    let mut buf = vec![0u8; read_len];
    file.read_exact(&mut buf).ok()?;

    let text = String::from_utf8_lossy(&buf);
    let mut last_iso = None;

    for line in text.lines().rev() {
        if line.contains("transaction completed") {
            if let Some(start) = line.find('[') {
                if let Some(end) = line[start..].find(']') {
                    let ts = &line[start + 1..start + end];
                    last_iso = Some(ts.to_string());
                    break;
                }
            }
        }
    }

    let iso_raw = last_iso?;
    let iso_normalized = normalize_iso_tz(&iso_raw);
    let log_time = glib::DateTime::from_iso8601(&iso_normalized, None).ok()?;
    let now = glib::DateTime::now_local().ok()?;
    let diff_us = now.difference(&log_time);
    let diff_sec = (diff_us.0 / 1_000_000).max(0) as u64;

    Some(format_elapsed_seconds(diff_sec))
}

/// Normalizes ISO timezone offset `+0100` -> `+01:00` for strict ISO 8601 parsers.
fn normalize_iso_tz(raw: &str) -> String {
    if raw.len() >= 5 {
        let sign_pos = raw.len() - 5;
        let sign = raw.as_bytes()[sign_pos];
        if (sign == b'+' || sign == b'-') && !raw.ends_with('Z') && !raw[sign_pos..].contains(':') {
            return format!("{}:{}", &raw[..raw.len() - 2], &raw[raw.len() - 2..]);
        }
    }
    raw.to_string()
}

fn format_elapsed_seconds(diff_sec: u64) -> String {
    let diff_mins = diff_sec / 60;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_days > 0 {
        format!(
            "Last update was {} {} ago!",
            diff_days,
            if diff_days == 1 { "day" } else { "days" }
        )
    } else if diff_hours > 0 {
        format!(
            "Last update was {} {} ago!",
            diff_hours,
            if diff_hours == 1 { "hour" } else { "hours" }
        )
    } else if diff_mins > 0 {
        format!(
            "Last update was {} {} ago!",
            diff_mins,
            if diff_mins == 1 { "minute" } else { "minutes" }
        )
    } else {
        "Last update was just moments ago!".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modular_files() {
        let cute = parse_dialogue_lines(DIALOGUE_CUTE_TXT, DialogueCategory::GeneralIdle);
        let sassy = parse_dialogue_lines(DIALOGUE_SASSY_TXT, DialogueCategory::GeneralIdle);
        let lazy = parse_dialogue_lines(DIALOGUE_LAZY_TXT, DialogueCategory::GeneralIdle);
        let system = parse_dialogue_lines(DIALOGUE_SYSTEM_TXT, DialogueCategory::GeneralIdle);
        let general = parse_dialogue_lines(DIALOGUE_GENERAL_TXT, DialogueCategory::GeneralIdle);

        assert!(!cute.is_empty(), "cute.txt must not be empty");
        assert!(!sassy.is_empty(), "sassy.txt must not be empty");
        assert!(!lazy.is_empty(), "lazy.txt must not be empty");
        assert!(!system.is_empty(), "system.txt must not be empty");
        assert!(!general.is_empty(), "general.txt must not be empty");
    }

    #[test]
    fn test_personality_pools() {
        let pool_cute = get_personality_dialogue_pool(Personality::Cute);
        let pool_sassy = get_personality_dialogue_pool(Personality::Sassy);
        let pool_lazy = get_personality_dialogue_pool(Personality::Lazy);
        let pool_normal = get_personality_dialogue_pool(Personality::Normal);

        assert!(!pool_cute.is_empty());
        assert!(!pool_sassy.is_empty());
        assert!(!pool_lazy.is_empty());
        assert!(pool_normal.len() > pool_lazy.len());
    }

    #[test]
    fn test_interpolate_placeholders() {
        let snapshot = SystemSnapshot {
            total_ram_gb: 32,
            used_ram_gb: 12.5,
            ram_usage_percent: 39,
            top_process_name: "Firefox".to_string(),
            top_process_cpu: 28.5,
            top_process_mem_mb: 1800.0,
            global_cpu_percent: 15,
            cpu_temp_celsius: 54,
            uptime_secs: 10800,
            uptime_formatted: "3 hours".to_string(),
            last_update_formatted: "Last update was 2 hours ago!".to_string(),
        };

        let template = "{app} uses {ram_used} out of {ram_total} GB at {cpu_temp}°C";
        let interpolated = interpolate_placeholders(template, &snapshot);
        assert_eq!(interpolated, "Firefox uses 12.5 GB out of 32 GB at 54°C");
    }

    #[test]
    fn test_clean_process_name() {
        assert_eq!(clean_process_name("firefox-bin"), "Firefox");
        assert_eq!(clean_process_name("discord"), "Discord");
        assert_eq!(clean_process_name("steam"), "Steam");
        assert_eq!(clean_process_name("code"), "VS Code");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "just started");
        assert_eq!(format_uptime(300), "5 minutes");
        assert_eq!(format_uptime(7200), "2 hours");
        assert_eq!(format_uptime(90000), "1 day, 1 hours");
        assert_eq!(format_uptime(180000), "2 days");
    }

    #[test]
    fn test_normalize_iso_tz() {
        assert_eq!(
            normalize_iso_tz("2026-10-26T22:37:03+0100"),
            "2026-10-26T22:37:03+01:00"
        );
        assert_eq!(
            normalize_iso_tz("2026-10-26T22:37:03-0500"),
            "2026-10-26T22:37:03-05:00"
        );
        assert_eq!(
            normalize_iso_tz("2026-10-26T22:37:03Z"),
            "2026-10-26T22:37:03Z"
        );
    }

    #[test]
    fn test_format_elapsed_seconds() {
        assert_eq!(format_elapsed_seconds(30), "Last update was just moments ago!");
        assert_eq!(format_elapsed_seconds(120), "Last update was 2 minutes ago!");
        assert_eq!(format_elapsed_seconds(3600), "Last update was 1 hour ago!");
        assert_eq!(format_elapsed_seconds(7200), "Last update was 2 hours ago!");
        assert_eq!(format_elapsed_seconds(86400), "Last update was 1 day ago!");
        assert_eq!(format_elapsed_seconds(172800), "Last update was 2 days ago!");
    }

    #[test]
    fn test_get_smart_dialogue_all_personalities() {
        for p in [
            Personality::Normal,
            Personality::Cute,
            Personality::Sassy,
            Personality::Lazy,
        ] {
            let dialogue = get_smart_dialogue_for_personality(p);
            assert!(!dialogue.is_empty());
            assert!(!dialogue.contains("{app}"));
            assert!(!dialogue.contains("{ram_total}"));
            assert!(!dialogue.contains("{cpu_temp}"));
        }
    }
}
