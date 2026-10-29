use crate::config::Corner;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimPhase {
    SlideIn,
    Holding,
    SlideOut,
    Done,
}

#[derive(Debug, Clone, Copy)]
pub struct SlideVector {
    pub start_x: f64,
    pub start_y: f64,
    pub target_x: f64,
    pub target_y: f64,
}

impl SlideVector {
    #[allow(dead_code)]
    pub fn for_corner(corner: Corner) -> Self {
        Self::for_corner_with_height(corner, 200.0)
    }

    pub fn for_corner_with_height(corner: Corner, height: f64) -> Self {
        let offset = height.abs().max(200.0);
        match corner {
            Corner::BottomRight | Corner::BottomLeft => Self {
                start_x: 0.0,
                start_y: offset,
                target_x: 0.0,
                target_y: 0.0,
            },
            Corner::TopRight | Corner::TopLeft => Self {
                start_x: 0.0,
                start_y: -offset,
                target_x: 0.0,
                target_y: 0.0,
            },
        }
    }
}

pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_cubic(t: f64) -> f64 {
    t.powi(3)
}

pub struct AnimationState {
    pub phase: AnimPhase,
    pub start_time_us: i64,
    pub phase_start_us: i64,
    pub slide_in_dur_us: i64,
    pub hold_dur_us: i64,
    pub slide_out_dur_us: i64,
    pub vector: SlideVector,
}

impl AnimationState {
    #[allow(dead_code)]
    pub fn new(corner: Corner, auto_dismiss_secs: u32) -> Self {
        Self::new_with_height(corner, auto_dismiss_secs, 200.0)
    }

    pub fn new_with_height(corner: Corner, auto_dismiss_secs: u32, height: f64) -> Self {
        let hold_dur_us = if auto_dismiss_secs == 0 {
            i64::MAX
        } else {
            (auto_dismiss_secs as i64) * 1_000_000
        };

        Self {
            phase: AnimPhase::SlideIn,
            start_time_us: 0,
            phase_start_us: 0,
            slide_in_dur_us: 420_000,
            hold_dur_us,
            slide_out_dur_us: 360_000,
            vector: SlideVector::for_corner_with_height(corner, height),
        }
    }

    pub fn tick(&mut self, now_us: i64) -> (f64, f64, AnimPhase) {
        if self.start_time_us == 0 {
            self.start_time_us = now_us;
            self.phase_start_us = now_us;
        }

        match self.phase {
            AnimPhase::SlideIn => {
                let elapsed = now_us - self.phase_start_us;
                let t = (elapsed as f64 / self.slide_in_dur_us as f64).clamp(0.0, 1.0);
                let progress = ease_out_cubic(t);

                let x = self.vector.start_x + (self.vector.target_x - self.vector.start_x) * progress;
                let y = self.vector.start_y + (self.vector.target_y - self.vector.start_y) * progress;

                if t >= 1.0 {
                    self.phase = AnimPhase::Holding;
                    self.phase_start_us = now_us;
                }
                (x, y, self.phase)
            }
            AnimPhase::Holding => {
                let elapsed = now_us - self.phase_start_us;
                if elapsed >= self.hold_dur_us {
                    self.phase = AnimPhase::SlideOut;
                    self.phase_start_us = now_us;
                }
                (self.vector.target_x, self.vector.target_y, self.phase)
            }
            AnimPhase::SlideOut => {
                let elapsed = now_us - self.phase_start_us;
                let t = (elapsed as f64 / self.slide_out_dur_us as f64).clamp(0.0, 1.0);
                let progress = ease_in_cubic(t);

                let x = self.vector.target_x + (self.vector.start_x - self.vector.target_x) * progress;
                let y = self.vector.target_y + (self.vector.start_y - self.vector.target_y) * progress;

                if t >= 1.0 {
                    self.phase = AnimPhase::Done;
                }
                (x, y, self.phase)
            }
            AnimPhase::Done => (self.vector.start_x, self.vector.start_y, AnimPhase::Done),
        }
    }

    pub fn trigger_slide_out(&mut self, now_us: i64) {
        if self.phase == AnimPhase::SlideIn || self.phase == AnimPhase::Holding {
            self.phase = AnimPhase::SlideOut;
            self.phase_start_us = now_us;
        }
    }
}

#[allow(dead_code)]
pub type SharedAnimationState = Rc<RefCell<AnimationState>>;

pub const BASE_FRAME_DELAY_MS: u64 = 220;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardCatState {
    Idle,
    LayDown,
    Licking,
    Sleepy,
    Sleeping,
    Excited,
    Despite,
    Sick1,
    Sick2,
    Dead,
}

impl StandardCatState {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "idle" => Some(Self::Idle),
            "lay_down" | "laydown" => Some(Self::LayDown),
            "licking" | "lick" => Some(Self::Licking),
            "sleepy" => Some(Self::Sleepy),
            "sleeping" | "sleep" => Some(Self::Sleeping),
            "excited" | "surprised" => Some(Self::Excited),
            "despite" | "annoyed" => Some(Self::Despite),
            "sick_1" | "sick1" | "weak" => Some(Self::Sick1),
            "sick_2" | "sick2" | "ill" => Some(Self::Sick2),
            "dead" | "comatose" => Some(Self::Dead),
            _ => None,
        }
    }

    pub fn is_flavor_action(&self) -> bool {
        matches!(self, Self::LayDown | Self::Licking)
    }

    pub fn is_decay_state(&self) -> bool {
        matches!(self, Self::Despite | Self::Sick1 | Self::Sick2 | Self::Dead)
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::LayDown => "lay_down",
            Self::Licking => "licking",
            Self::Sleepy => "sleepy",
            Self::Sleeping => "sleeping",
            Self::Excited => "excited",
            Self::Despite => "despite",
            Self::Sick1 => "sick_1",
            Self::Sick2 => "sick_2",
            Self::Dead => "dead",
        }
    }
}

pub fn is_heavy_app_running() -> bool {
    const HEAVY_NAMES: &[&str] = &[
        "steam",
        "gamescope",
        "firefox",
        "chrome",
        "chromium",
        "brave",
        "heroic",
        "lutris",
        "minecraft",
        "obs",
        "discord",
        "blender",
        "godot",
        "dolphin-emu",
        "rpcs3",
    ];

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let name_str = filename.to_string_lossy();
            if name_str.chars().all(|c| c.is_ascii_digit()) {
                let comm_path = entry.path().join("comm");
                if let Ok(comm) = std::fs::read_to_string(comm_path) {
                    let comm_clean = comm.trim().to_lowercase();
                    for &target in HEAVY_NAMES {
                        if comm_clean.contains(target) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub struct StandardCatController {
    pub current_state: StandardCatState,
    pub frame_idx: usize,
    pub next_action_allowed_at: std::time::Instant,
    pub excited_cooldown_until: std::time::Instant,
    pub excited_active_until: Option<std::time::Instant>,
    pub forced_state: Option<StandardCatState>,
    pub last_heavy_app_check: std::time::Instant,
}

impl Default for StandardCatController {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardCatController {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            current_state: StandardCatState::Idle,
            frame_idx: 0,
            next_action_allowed_at: now + std::time::Duration::from_secs(glib::random_int_range(15, 31) as u64),
            excited_cooldown_until: now,
            excited_active_until: None,
            forced_state: None,
            last_heavy_app_check: now,
        }
    }

    pub fn set_forced_state(&mut self, state: StandardCatState) {
        self.forced_state = Some(state);
        self.current_state = state;
        self.frame_idx = 0;
    }

    pub fn clear_forced_state(&mut self) {
        self.forced_state = None;
    }

    pub fn trigger_excited(&mut self, now: std::time::Instant) -> bool {
        if now >= self.excited_cooldown_until {
            self.excited_cooldown_until = now + std::time::Duration::from_secs(60);
            self.excited_active_until = Some(now + std::time::Duration::from_millis(2500));
            self.current_state = StandardCatState::Excited;
            self.frame_idx = 0;
            true
        } else {
            false
        }
    }

    pub fn check_heavy_app_trigger(&mut self, now: std::time::Instant) {
        if self.forced_state.is_some() || self.current_state.is_decay_state() {
            return;
        }

        if now.duration_since(self.last_heavy_app_check) >= std::time::Duration::from_secs(5) {
            self.last_heavy_app_check = now;
            if now >= self.excited_cooldown_until && is_heavy_app_running() {
                self.trigger_excited(now);
            }
        }
    }

    pub fn evaluate_next_state(
        &mut self,
        now: std::time::Instant,
        dismiss_streak: u32,
        hour: u32,
    ) -> StandardCatState {
        if let Some(forced) = self.forced_state {
            return forced;
        }

        // 1. Update neglect visual progression
        if dismiss_streak >= 12 {
            return StandardCatState::Dead;
        } else if dismiss_streak >= 9 {
            return StandardCatState::Sick2;
        } else if dismiss_streak >= 6 {
            return StandardCatState::Sick1;
        } else if dismiss_streak >= 3 {
            return StandardCatState::Despite;
        }

        // 2. Reactive event: excited state active
        if let Some(until) = self.excited_active_until {
            if now < until {
                return StandardCatState::Excited;
            } else {
                self.excited_active_until = None;
            }
        }

        // 3. Time of day
        if hour <= 5 {
            // Deep night (00:00 - 05:00): curled up loaf sleeping loop
            return StandardCatState::Sleeping;
        } else if (20..=23).contains(&hour) {
            // Late evening (20:00 - 23:00): sleepy yawning frames between resting pauses
            if now >= self.next_action_allowed_at {
                self.next_action_allowed_at =
                    now + std::time::Duration::from_secs(glib::random_int_range(15, 31) as u64);
                return StandardCatState::Sleepy;
            } else {
                return StandardCatState::Idle;
            }
        }

        // 4. Calm daytime rotation (06:00 - 19:59)
        if now >= self.next_action_allowed_at {
            let roll = glib::random_int_range(0, 4);
            match roll {
                0 => StandardCatState::LayDown,
                1 => StandardCatState::Licking,
                _ => StandardCatState::Idle,
            }
        } else {
            StandardCatState::Idle
        }
    }

    pub fn advance_frame(
        &mut self,
        textures: &crate::assets::StandardCatTextures,
        speed: f32,
        is_initial: bool,
    ) -> (StandardCatState, usize, u64) {
        let speed_clamped = speed.clamp(0.2, 5.0);
        let now = std::time::Instant::now();

        if is_initial {
            self.current_state = StandardCatState::Idle;
            self.frame_idx = 0;
            let pause = glib::random_int_range(4000, 8001) as f32;
            let delay_ms = ((pause / speed_clamped).round() as u64).max(100);
            return (self.current_state, self.frame_idx, delay_ms);
        }

        self.check_heavy_app_trigger(now);

        let total_frames = textures.get(self.current_state).len();
        if self.frame_idx + 1 < total_frames {
            self.frame_idx += 1;
            let delay_ms = (((BASE_FRAME_DELAY_MS as f32) / speed_clamped).round() as u64).max(20);
            (self.current_state, self.frame_idx, delay_ms)
        } else {
            // Completed current animation sequence
            let just_finished = self.current_state;
            if just_finished.is_flavor_action() || just_finished == StandardCatState::Sleepy {
                self.next_action_allowed_at =
                    now + std::time::Duration::from_secs(glib::random_int_range(15, 31) as u64);
            }

            let memory = crate::brain::CatMemory::load();
            let hour = glib::DateTime::now_local().map(|dt| dt.hour() as u32).unwrap_or(12);
            let next_state = self.evaluate_next_state(now, memory.dismiss_streak, hour);

            self.current_state = next_state;
            self.frame_idx = 0;

            let pause_ms = match self.current_state {
                StandardCatState::Idle => glib::random_int_range(4000, 8001) as f32,
                StandardCatState::Sleeping => glib::random_int_range(3000, 5001) as f32,
                StandardCatState::Dead => 600.0,
                _ => BASE_FRAME_DELAY_MS as f32,
            };

            let delay_ms = ((pause_ms / speed_clamped).round() as u64).max(50);
            (self.current_state, self.frame_idx, delay_ms)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_cat_state_parsing() {
        assert_eq!(StandardCatState::from_str("idle"), Some(StandardCatState::Idle));
        assert_eq!(StandardCatState::from_str("lay_down"), Some(StandardCatState::LayDown));
        assert_eq!(StandardCatState::from_str("laydown"), Some(StandardCatState::LayDown));
        assert_eq!(StandardCatState::from_str("licking"), Some(StandardCatState::Licking));
        assert_eq!(StandardCatState::from_str("sleepy"), Some(StandardCatState::Sleepy));
        assert_eq!(StandardCatState::from_str("sleeping"), Some(StandardCatState::Sleeping));
        assert_eq!(StandardCatState::from_str("excited"), Some(StandardCatState::Excited));
        assert_eq!(StandardCatState::from_str("despite"), Some(StandardCatState::Despite));
        assert_eq!(StandardCatState::from_str("sick_1"), Some(StandardCatState::Sick1));
        assert_eq!(StandardCatState::from_str("sick_2"), Some(StandardCatState::Sick2));
        assert_eq!(StandardCatState::from_str("dead"), Some(StandardCatState::Dead));
        assert_eq!(StandardCatState::from_str("unknown"), None);
    }

    #[test]
    fn test_dismiss_streak_neglect_progression() {
        let mut ctrl = StandardCatController::new();
        let now = std::time::Instant::now();

        // Dismiss streak < 3: normal daytime/night state
        assert_eq!(ctrl.evaluate_next_state(now, 0, 14), StandardCatState::Idle);
        assert_eq!(ctrl.evaluate_next_state(now, 2, 14), StandardCatState::Idle);

        // Dismiss streak 3..5: despite
        assert_eq!(ctrl.evaluate_next_state(now, 3, 14), StandardCatState::Despite);
        assert_eq!(ctrl.evaluate_next_state(now, 5, 14), StandardCatState::Despite);

        // Dismiss streak 6..8: sick_1
        assert_eq!(ctrl.evaluate_next_state(now, 6, 14), StandardCatState::Sick1);
        assert_eq!(ctrl.evaluate_next_state(now, 8, 14), StandardCatState::Sick1);

        // Dismiss streak 9..11: sick_2
        assert_eq!(ctrl.evaluate_next_state(now, 9, 14), StandardCatState::Sick2);
        assert_eq!(ctrl.evaluate_next_state(now, 11, 14), StandardCatState::Sick2);

        // Dismiss streak >= 12: dead
        assert_eq!(ctrl.evaluate_next_state(now, 12, 14), StandardCatState::Dead);
        assert_eq!(ctrl.evaluate_next_state(now, 99, 14), StandardCatState::Dead);
    }

    #[test]
    fn test_time_of_day_sleeping_and_sleepy() {
        let mut ctrl = StandardCatController::new();
        let now = std::time::Instant::now();

        // Hour 0..5: Sleeping
        assert_eq!(ctrl.evaluate_next_state(now, 0, 0), StandardCatState::Sleeping);
        assert_eq!(ctrl.evaluate_next_state(now, 0, 3), StandardCatState::Sleeping);
        assert_eq!(ctrl.evaluate_next_state(now, 0, 5), StandardCatState::Sleeping);

        // Hour 20..23: Sleepy if cooldown expired, or Idle if cooldown active
        ctrl.next_action_allowed_at = now - std::time::Duration::from_secs(1);
        assert_eq!(ctrl.evaluate_next_state(now, 0, 22), StandardCatState::Sleepy);

        // Action cooldown now set into future: next should be Idle
        assert_eq!(ctrl.evaluate_next_state(now, 0, 22), StandardCatState::Idle);
    }

    #[test]
    fn test_excited_trigger_and_cooldown() {
        let mut ctrl = StandardCatController::new();
        let now = std::time::Instant::now();

        assert!(ctrl.trigger_excited(now));
        assert_eq!(ctrl.current_state, StandardCatState::Excited);

        // Within 60-second cooldown: second trigger must return false
        let soon = now + std::time::Duration::from_secs(10);
        assert!(!ctrl.trigger_excited(soon));

        // After 60 seconds: trigger allowed again
        let later = now + std::time::Duration::from_secs(61);
        assert!(ctrl.trigger_excited(later));
    }
}

