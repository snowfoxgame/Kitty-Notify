#[cfg(feature = "smart")]
pub mod behavior;
#[cfg(feature = "smart")]
pub mod helper;
#[cfg(feature = "smart")]
pub mod memory;

#[cfg(feature = "smart")]
pub use behavior::*;
#[cfg(feature = "smart")]
pub use helper::*;
#[cfg(feature = "smart")]
pub use memory::*;

#[cfg(not(feature = "smart"))]
pub mod dummy {
    use crate::config::Config;

    #[derive(Debug, Clone, Default)]
    pub struct CatMemory {
        pub dismiss_streak: u32,
    }

    impl CatMemory {
        pub fn load() -> Self {
            Self::default()
        }
        pub fn save(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn mark_clean_exit(&mut self) {}
        pub fn reset() -> Self {
            Self::default()
        }
        pub fn record_dismiss(&mut self) {}
        pub fn record_update_applied(&mut self) {}
        pub fn record_app_seen(&mut self, _app: &str) {}
        pub fn favorite_app(&self) -> Option<String> {
            None
        }
        pub fn check_and_record_late_night(&mut self) {}
        pub fn get_streak_dialogue(&self) -> Option<String> {
            None
        }
        pub fn get_recovery_dialogue(&mut self) -> Option<String> {
            None
        }
        pub fn get_habit_dialogue(&self) -> Option<String> {
            None
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct HelperAlert {
        pub title: String,
        pub message: String,
        pub action_label: String,
        pub command: String,
    }

    pub fn check_system_helpers(_config: &Config, _memory: &mut CatMemory) -> Option<HelperAlert> {
        None
    }

    pub fn evaluate_behavioral_dialogue(
        _memory: &CatMemory,
        _personality: crate::config::Personality,
    ) -> Option<String> {
        None
    }

    pub fn evaluate_behavioral_dialogue_with_snapshot(
        _memory: &CatMemory,
        _personality: crate::config::Personality,
        _snapshot: &crate::ui::dialogue::SystemSnapshot,
    ) -> Option<String> {
        None
    }

    pub fn get_behavioral_or_general_dialogue(
        _memory: &CatMemory,
        _personality: crate::config::Personality,
    ) -> String {
        "purr... i use arch btw :3".to_string()
    }

    pub fn create_seeded_rng() -> rand::rngs::StdRng {
        use rand::SeedableRng;
        rand::rngs::StdRng::seed_from_u64(0)
    }

    pub fn select_unique_candidate<'a, T, F>(
        candidates: &'a [T],
        _get_key: F,
        rng: &mut rand::rngs::StdRng,
    ) -> Option<&'a T>
    where
        F: Fn(&T) -> &str,
    {
        use rand::RngExt;
        if candidates.is_empty() {
            return None;
        }
        let idx = rng.random_range(0..candidates.len());
        Some(&candidates[idx])
    }

    pub fn initialize_session() {}
    pub fn was_unclean_shutdown() -> bool {
        false
    }
    pub fn clear_unclean_shutdown() {}
}

#[cfg(not(feature = "smart"))]
pub use dummy::*;

