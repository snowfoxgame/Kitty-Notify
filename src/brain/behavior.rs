//! Context-aware Behavioral Dialogue & Habit Learning Engine for Desktop Pet mode.

use crate::brain::memory::CatMemory;
use crate::config::Personality;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

pub const BEHAVIORAL_JSON: &str = include_str!("../../assets/dialogue/behavioral.json");
pub const GENERAL_TXT: &str = include_str!("../../assets/dialogue/general.txt");

/// Initializes a deterministic, per-machine unique StdRng seeded by /etc/machine-id
/// combined with current high-resolution timestamp entropy.
pub fn create_seeded_rng() -> StdRng {
    let mut hasher = DefaultHasher::new();
    if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
        machine_id.trim().hash(&mut hasher);
    } else if let Ok(dbus_id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        dbus_id.trim().hash(&mut hasher);
    } else {
        "kittynotify-fallback-seed".hash(&mut hasher);
    }

    let machine_hash = hasher.finish();
    let time_entropy = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let combined_seed = machine_hash ^ time_entropy;
    StdRng::seed_from_u64(combined_seed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    Cute,
    Sassy,
    Worried,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct QuoteCondition {
    #[serde(default)]
    pub min_dismiss_streak: Option<u32>,
    #[serde(default)]
    pub max_dismiss_streak: Option<u32>,
    #[serde(default)]
    pub min_hour: Option<u32>,
    #[serde(default)]
    pub max_hour: Option<u32>,
    #[serde(default)]
    pub min_uptime_secs: Option<u64>,
    #[serde(default)]
    pub max_uptime_secs: Option<u64>,
    #[serde(default)]
    pub top_process_in: Option<Vec<String>>,
    #[serde(default)]
    pub min_ram_percent: Option<f32>,
    #[serde(default)]
    pub unclean_shutdown: Option<bool>,
    #[serde(default)]
    pub min_cache_size_gb: Option<f32>,
    #[serde(default)]
    pub min_orphan_count: Option<usize>,
}

impl QuoteCondition {
    pub fn matches(&self, ctx: &BehaviorContext) -> bool {
        if let Some(min) = self.min_dismiss_streak {
            if ctx.dismiss_streak < min {
                return false;
            }
        }
        if let Some(max) = self.max_dismiss_streak {
            if ctx.dismiss_streak > max {
                return false;
            }
        }
        if let Some(min) = self.min_hour {
            if ctx.hour < min {
                return false;
            }
        }
        if let Some(max) = self.max_hour {
            if ctx.hour > max {
                return false;
            }
        }
        if let Some(min) = self.min_uptime_secs {
            if ctx.uptime_secs < min {
                return false;
            }
        }
        if let Some(max) = self.max_uptime_secs {
            if ctx.uptime_secs > max {
                return false;
            }
        }
        if let Some(ref procs) = self.top_process_in {
            let proc_lower = ctx.top_process.to_lowercase();
            let matches_any = procs.iter().any(|p| {
                let p_lower = p.to_lowercase();
                proc_lower == p_lower || proc_lower.contains(&p_lower)
            });
            if !matches_any {
                return false;
            }
        }
        if let Some(min_ram) = self.min_ram_percent {
            if (ctx.ram_used_percent as f32) <= min_ram {
                return false;
            }
        }
        if let Some(unclean) = self.unclean_shutdown {
            if ctx.unclean_shutdown != unclean {
                return false;
            }
        }
        if let Some(min_cache) = self.min_cache_size_gb {
            if ctx.cache_size <= min_cache {
                return false;
            }
        }
        if let Some(min_orphans) = self.min_orphan_count {
            if ctx.orphan_count <= min_orphans {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralQuote {
    pub category: String,
    pub quote: String,
    pub mood: Mood,
    #[serde(default)]
    pub condition: QuoteCondition,
}

#[derive(Debug, Clone)]
pub struct BehaviorContext {
    pub dismiss_streak: u32,
    pub uptime_secs: u64,
    pub uptime_formatted: String,
    pub hour: u32,
    pub top_process: String,
    pub ram_used_percent: u32,
    pub ram_used_gb: f32,
    pub ram_total_gb: u32,
    pub unclean_shutdown: bool,
    pub cache_size: f32,
    pub orphan_count: usize,
}

// Asynchronous non-blocking maintenance cache
static CACHED_CACHE_SIZE_X10: AtomicU32 = AtomicU32::new(0);
static CACHED_ORPHANS: AtomicUsize = AtomicUsize::new(0);
static LAST_MAINTENANCE_REFRESH: AtomicU64 = AtomicU64::new(0);

pub fn refresh_maintenance_async() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let last = LAST_MAINTENANCE_REFRESH.load(Ordering::Relaxed);
    if now.saturating_sub(last) < 60 && last > 0 {
        return;
    }
    LAST_MAINTENANCE_REFRESH.store(now, Ordering::Relaxed);

    std::thread::spawn(move || {
        let cache_gb = crate::brain::helper::get_pacman_cache_size_gb();
        let orphans = crate::brain::helper::get_orphan_packages_count();
        CACHED_CACHE_SIZE_X10.store((cache_gb * 10.0).round() as u32, Ordering::Relaxed);
        CACHED_ORPHANS.store(orphans, Ordering::Relaxed);
    });
}

pub fn get_cached_maintenance() -> (f32, usize) {
    refresh_maintenance_async();
    let cache_x10 = CACHED_CACHE_SIZE_X10.load(Ordering::Relaxed);
    let orphans = CACHED_ORPHANS.load(Ordering::Relaxed);
    (cache_x10 as f32 / 10.0, orphans)
}

pub fn format_uptime_hm(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

impl BehaviorContext {
    pub fn from_snapshot(memory: &CatMemory, snapshot: &crate::ui::dialogue::SystemSnapshot) -> Self {
        let hour = glib::DateTime::now_local()
            .map(|dt| dt.hour() as u32)
            .unwrap_or(12);

        let unclean_shutdown = crate::brain::was_unclean_shutdown();
        let (cache_size, orphan_count) = get_cached_maintenance();

        Self {
            dismiss_streak: memory.dismiss_streak,
            uptime_secs: snapshot.uptime_secs,
            uptime_formatted: format_uptime_hm(snapshot.uptime_secs),
            hour,
            top_process: snapshot.top_process_name.clone(),
            ram_used_percent: snapshot.ram_usage_percent,
            ram_used_gb: snapshot.used_ram_gb,
            ram_total_gb: snapshot.total_ram_gb,
            unclean_shutdown,
            cache_size,
            orphan_count,
        }
    }

    pub fn sample(memory: &CatMemory) -> Self {
        let snapshot = crate::ui::dialogue::SystemSnapshot::gather();
        Self::from_snapshot(memory, &snapshot)
    }
}

/// Matches quote mood against the active cat personality:
/// - Cute: allow cute and worried
/// - Sassy: allow sassy and worried
/// - Normal: allow all moods
/// - Lazy: fall back to lazy pool
pub fn mood_matches_personality(mood: Mood, personality: Personality) -> bool {
    match personality {
        Personality::Cute => mood == Mood::Cute || mood == Mood::Worried,
        Personality::Sassy => mood == Mood::Sassy || mood == Mood::Worried,
        Personality::Normal => true,
        Personality::Lazy => false,
    }
}

/// Replaces placeholders in behavioral quotes:
/// {dismiss_streak}, {top_process}, {hour}, {uptime}, {ram_used}, {ram_total}, {cache_size}, {orphan_count}
pub fn interpolate_behavioral_placeholders(template: &str, ctx: &BehaviorContext) -> String {
    let mut s = template.to_string();
    s = s.replace("{dismiss_streak}", &format!("{}", ctx.dismiss_streak));
    s = s.replace("{top_process}", &ctx.top_process);
    s = s.replace("{hour}", &format!("{}", ctx.hour));
    s = s.replace("{uptime}", &ctx.uptime_formatted);
    s = s.replace("{ram_used}", &format!("{:.1} GB", ctx.ram_used_gb));
    s = s.replace("{ram_total}", &format!("{} GB", ctx.ram_total_gb));
    s = s.replace("{cache_size}", &format!("{:.1}", ctx.cache_size));
    s = s.replace("{orphan_count}", &format!("{}", ctx.orphan_count));
    s
}

static CACHED_BEHAVIORAL_DATASET: OnceLock<Vec<BehavioralQuote>> = OnceLock::new();

/// Loads behavioral dataset from ~/.config/kittynotify/dialogue/behavioral.json or embedded asset.
pub fn load_behavioral_dataset() -> &'static [BehavioralQuote] {
    CACHED_BEHAVIORAL_DATASET.get_or_init(|| {
        let user_path = crate::config::Config::config_dir()
            .join("dialogue")
            .join("behavioral.json");
        if user_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&user_path) {
                if let Ok(quotes) = serde_json::from_str::<Vec<BehavioralQuote>>(&content) {
                    return quotes;
                }
            }
        }
        serde_json::from_str::<Vec<BehavioralQuote>>(BEHAVIORAL_JSON).unwrap_or_default()
    })
}

static RECENT_QUOTES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
const RECENT_QUOTES_CAPACITY: usize = 7;

/// Selects a quote from candidates avoiding quotes in the recent history buffer.
/// If all candidates have been seen recently, clears the history buffer and picks fresh.
pub fn select_unique_candidate<'a, T, F>(
    candidates: &'a [T],
    get_key: F,
    rng: &mut StdRng,
) -> Option<&'a T>
where
    F: Fn(&T) -> &str,
{
    if candidates.is_empty() {
        return None;
    }

    let mut recent = RECENT_QUOTES.lock().unwrap_or_else(|e| e.into_inner());

    // Filter out quotes that are currently in last_quotes
    let mut available: Vec<&'a T> = candidates
        .iter()
        .filter(|c| !recent.iter().any(|r| r == get_key(c)))
        .collect();

    // If all matching quotes have been shown recently, clear the history buffer
    if available.is_empty() {
        recent.clear();
        available = candidates.iter().collect();
    }

    let idx = rng.random_range(0..available.len());
    let chosen = available[idx];

    let key = get_key(chosen).to_string();
    if recent.len() >= RECENT_QUOTES_CAPACITY {
        recent.pop_front();
    }
    recent.push_back(key);

    Some(chosen)
}

/// Evaluates conditions, filters by personality mood, and selects a behavioral quote.
/// Returns None if no behavioral conditions match.
pub fn evaluate_behavioral_dialogue_with_context(
    quotes: &[BehavioralQuote],
    ctx: &BehaviorContext,
    personality: Personality,
) -> Option<String> {
    let eligible: Vec<&BehavioralQuote> = quotes
        .iter()
        .filter(|q| q.condition.matches(ctx))
        .filter(|q| mood_matches_personality(q.mood, personality))
        .collect();

    if eligible.is_empty() {
        return None;
    }

    let mut rng = create_seeded_rng();
    let chosen = select_unique_candidate(&eligible, |q| &q.quote, &mut rng)?;
    Some(interpolate_behavioral_placeholders(&chosen.quote, ctx))
}

/// Evaluates matching behavioral quotes using an existing SystemSnapshot.
pub fn evaluate_behavioral_dialogue_with_snapshot(
    memory: &CatMemory,
    personality: Personality,
    snapshot: &crate::ui::dialogue::SystemSnapshot,
) -> Option<String> {
    let quotes = load_behavioral_dataset();
    let ctx = BehaviorContext::from_snapshot(memory, snapshot);
    evaluate_behavioral_dialogue_with_context(quotes, &ctx, personality)
}

/// Samples live system context and evaluates matching behavioral quotes.
pub fn evaluate_behavioral_dialogue(
    memory: &CatMemory,
    personality: Personality,
) -> Option<String> {
    let snapshot = crate::ui::dialogue::SystemSnapshot::gather();
    evaluate_behavioral_dialogue_with_snapshot(memory, personality, &snapshot)
}

static CACHED_GENERAL_FALLBACK: OnceLock<Vec<String>> = OnceLock::new();

/// Returns a random fallback quote from general.txt.
pub fn get_general_fallback_dialogue() -> String {
    let lines = CACHED_GENERAL_FALLBACK.get_or_init(|| {
        let general_str = crate::ui::dialogue::load_dialogue_text("general", GENERAL_TXT);
        general_str
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    });

    if lines.is_empty() {
        return "purr... i use arch btw :3".to_string();
    }

    let mut rng = create_seeded_rng();
    let chosen = select_unique_candidate(lines, |l| l.as_str(), &mut rng)
        .unwrap_or(&lines[0]);

    let quote = if chosen.starts_with('"') && chosen.ends_with('"') && chosen.len() >= 2 {
        &chosen[1..chosen.len() - 1]
    } else {
        chosen.as_str()
    };
    quote.to_string()
}

/// Evaluates behavioral quotes, falling back to general.txt if candidate pool is empty.
pub fn get_behavioral_or_general_dialogue(
    memory: &CatMemory,
    personality: Personality,
) -> String {
    if let Some(quote) = evaluate_behavioral_dialogue(memory, personality) {
        quote
    } else {
        get_general_fallback_dialogue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_context() -> BehaviorContext {
        BehaviorContext {
            dismiss_streak: 0,
            uptime_secs: 3600,
            uptime_formatted: "1h 0m".to_string(),
            hour: 12,
            top_process: "Firefox".to_string(),
            ram_used_percent: 50,
            ram_used_gb: 8.0,
            ram_total_gb: 16,
            unclean_shutdown: false,
            cache_size: 2.0,
            orphan_count: 0,
        }
    }

    #[test]
    fn test_parse_behavioral_dataset() {
        let dataset = load_behavioral_dataset();
        assert_eq!(dataset.len(), 37, "Expected exactly 37 behavioral quotes in dataset");
    }

    #[test]
    fn test_dismiss_streak_evaluation() {
        let dataset = load_behavioral_dataset();
        let mut ctx = dummy_context();

        // Streak 3: should match Category 1 cute & sassy
        ctx.dismiss_streak = 3;
        let cute_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Cute);
        assert!(cute_quote.is_some());
        let cute_text = cute_quote.unwrap();
        assert!(cute_text.contains("3"));
        assert!(cute_text.contains("brushed me off") || cute_text.contains("dismiss clicks"));

        // Streak 11: should match worried PANIC quote
        ctx.dismiss_streak = 11;
        let worried_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Normal);
        assert!(worried_quote.is_some());
        let worried_text = worried_quote.unwrap();
        assert!(worried_text.contains("11"));
        assert!(worried_text.contains("PANIC") || worried_text.contains("terrified"));
    }

    #[test]
    fn test_night_and_uptime_evaluation() {
        let dataset = load_behavioral_dataset();
        let mut ctx = dummy_context();

        // Hour 3 at night: should trigger off to bed / dream of rust
        ctx.hour = 3;
        let night_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Cute);
        assert!(night_quote.is_some());
        let text = night_quote.unwrap();
        assert!(text.contains("3:00"));

        // Night gaming with Steam
        ctx.hour = 2;
        ctx.top_process = "Steam".to_string();
        let gaming_night = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Sassy);
        assert!(gaming_night.is_some());

        // Extreme uptime >= 86400 (24h)
        ctx.hour = 14;
        ctx.uptime_secs = 90000;
        ctx.uptime_formatted = "25h 0m".to_string();
        let uptime_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Sassy);
        assert!(uptime_quote.is_some());
        let up_text = uptime_quote.unwrap();
        assert!(up_text.contains("indestructible system daemon") || up_text.contains("without a reboot"));
    }

    #[test]
    fn test_resource_load_and_gaming() {
        let dataset = load_behavioral_dataset();
        let mut ctx = dummy_context();

        // Steam right at boot (uptime < 300)
        ctx.uptime_secs = 120;
        ctx.uptime_formatted = "2m".to_string();
        ctx.top_process = "Steam".to_string();
        let boot_steam = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Cute);
        assert!(boot_steam.is_some());
        assert!(boot_steam.unwrap().contains("Top-tier priorities"));

        // High RAM usage > 90%
        ctx.uptime_secs = 5000;
        ctx.ram_used_percent = 91;
        ctx.ram_used_gb = 14.5;
        let oom_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Cute);
        assert!(oom_quote.is_some());
        assert!(oom_quote.unwrap().contains("OOM killer"));
    }

    #[test]
    fn test_unclean_shutdown_crash() {
        let dataset = load_behavioral_dataset();
        let mut ctx = dummy_context();
        ctx.unclean_shutdown = true;
        ctx.uptime_secs = 60; // Just rebooted after crash

        let crash_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Normal);
        assert!(crash_quote.is_some());
        let text = crash_quote.unwrap();
        assert!(
            text.contains("went black")
                || text.contains("poweroff")
                || text.contains("hard reset")
                || text.contains("journalctl")
                || text.contains("bruised pixel")
                || text.contains("memory.json")
        );
    }

    #[test]
    fn test_maintenance_evaluation() {
        let dataset = load_behavioral_dataset();
        let mut ctx = dummy_context();

        // Pacman cache > 6GB
        ctx.cache_size = 7.2;
        let cache_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Sassy);
        assert!(cache_quote.is_some());
        let text = cache_quote.unwrap();
        assert!(text.contains("7.2 GB"));

        // Orphan packages > 15
        ctx.cache_size = 1.0;
        ctx.orphan_count = 18;
        let orphan_quote = evaluate_behavioral_dialogue_with_context(dataset, &ctx, Personality::Cute);
        assert!(orphan_quote.is_some());
        let o_text = orphan_quote.unwrap();
        assert!(o_text.contains("18") && (o_text.contains("orphaned packages") || o_text.contains("lonely packages")));
    }

    #[test]
    fn test_personality_filtering() {
        assert!(mood_matches_personality(Mood::Cute, Personality::Cute));
        assert!(mood_matches_personality(Mood::Worried, Personality::Cute));
        assert!(!mood_matches_personality(Mood::Sassy, Personality::Cute));

        assert!(mood_matches_personality(Mood::Sassy, Personality::Sassy));
        assert!(mood_matches_personality(Mood::Worried, Personality::Sassy));
        assert!(!mood_matches_personality(Mood::Cute, Personality::Sassy));

        assert!(mood_matches_personality(Mood::Cute, Personality::Normal));
        assert!(mood_matches_personality(Mood::Sassy, Personality::Normal));
        assert!(mood_matches_personality(Mood::Worried, Personality::Normal));

        assert!(!mood_matches_personality(Mood::Cute, Personality::Lazy));
    }

    #[test]
    fn test_placeholder_interpolation() {
        let ctx = BehaviorContext {
            dismiss_streak: 7,
            uptime_secs: 7200,
            uptime_formatted: "2h 0m".to_string(),
            hour: 23,
            top_process: "Firefox".to_string(),
            ram_used_percent: 88,
            ram_used_gb: 14.1,
            ram_total_gb: 16,
            unclean_shutdown: false,
            cache_size: 6.8,
            orphan_count: 22,
        };
        let template = "Streak {dismiss_streak} in {top_process} at {hour}:00, uptime {uptime}, RAM {ram_used}/{ram_total}, cache {cache_size} GB, orphans {orphan_count}";
        let res = interpolate_behavioral_placeholders(template, &ctx);
        assert_eq!(
            res,
            "Streak 7 in Firefox at 23:00, uptime 2h 0m, RAM 14.1 GB/16 GB, cache 6.8 GB, orphans 22"
        );
    }

    #[test]
    fn test_live_sample_and_evaluation() {
        let mem = CatMemory {
            dismiss_streak: 5,
            ..Default::default()
        };
        let quote = evaluate_behavioral_dialogue(&mem, Personality::Normal);
        assert!(quote.is_some());
        let q = quote.unwrap();
        assert!(q.contains("5"));
    }

    #[test]
    fn test_anti_repeat_buffer_and_machine_seed() {
        let mut rng = create_seeded_rng();
        let options = vec!["quote_a", "quote_b", "quote_c", "quote_d", "quote_e"];

        let mut seen = std::collections::HashSet::new();
        for _ in 0..5 {
            let choice = select_unique_candidate(&options, |s| s, &mut rng).unwrap();
            seen.insert(*choice);
        }
        assert_eq!(seen.len(), 5, "All 5 unique candidates should be selected without immediate repetition");

        // The 6th pick triggers buffer reset when all have been shown
        let sixth = select_unique_candidate(&options, |s| s, &mut rng);
        assert!(sixth.is_some());
    }
}

