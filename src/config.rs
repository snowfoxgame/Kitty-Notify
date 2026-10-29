use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    #[default]
    Overlay,
    DesktopOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AurHelper {
    #[default]
    Auto,
    Paru,
    Yay,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SoundEffect {
    #[serde(alias = "realistic", alias = "Realistic")]
    #[default]
    Realistic,
    #[serde(alias = "chiptune", alias = "Chiptune")]
    Chiptune,
    #[serde(alias = "mute", alias = "Mute")]
    Mute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[allow(clippy::enum_variant_names)]
pub enum CatSkin {
    #[default]
    CartonCat,
    DraculaCat,
    IdleCat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationSpeed {
    #[serde(rename = "0.5x", alias = "0.5", alias = "slow", alias = "Slow")]
    Half,
    #[default]
    #[serde(rename = "1.0x", alias = "1x", alias = "1.0", alias = "normal", alias = "Normal")]
    Normal,
    #[serde(rename = "2.0x", alias = "2x", alias = "2.0", alias = "fast", alias = "Fast")]
    Double,
}

impl AnimationSpeed {
    pub fn multiplier(&self) -> f32 {
        match self {
            AnimationSpeed::Half => 0.5,
            AnimationSpeed::Normal => 1.0,
            AnimationSpeed::Double => 2.0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AnimationSpeed::Half => "0.5x",
            AnimationSpeed::Normal => "1.0x",
            AnimationSpeed::Double => "2.0x",
        }
    }

    pub fn from_multiplier(val: f32) -> Self {
        if val <= 0.75 {
            AnimationSpeed::Half
        } else if val >= 1.5 {
            AnimationSpeed::Double
        } else {
            AnimationSpeed::Normal
        }
    }
}

impl std::fmt::Display for AnimationSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u32,
    #[serde(default)]
    pub corner: Corner,
    #[serde(default)]
    pub display_mode: DisplayMode,
    #[serde(default)]
    pub show_on_all_monitors: bool,
    #[serde(default = "default_monitor")]
    pub monitor: String,
    #[serde(default = "default_auto_dismiss")]
    pub auto_dismiss_seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_override: Option<String>,
}

fn default_check_interval() -> u32 {
    2
}

fn default_monitor() -> String {
    "auto".to_string()
}

fn default_auto_dismiss() -> u32 {
    10
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            check_interval_hours: 2,
            corner: Corner::BottomRight,
            display_mode: DisplayMode::Overlay,
            show_on_all_monitors: false,
            monitor: "auto".to_string(),
            auto_dismiss_seconds: 10,
            terminal_override: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatesConfig {
    #[serde(default)]
    pub preferred_aur_helper: AurHelper,
    #[serde(default = "default_true")]
    pub check_flatpaks: bool,
    #[serde(default = "default_ignore_packages")]
    pub ignore_packages: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_ignore_packages() -> Vec<String> {
    vec!["discord".to_string(), "electron*".to_string()]
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self {
            preferred_aur_helper: AurHelper::Auto,
            check_flatpaks: true,
            ignore_packages: default_ignore_packages(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualConfig {
    #[serde(default)]
    pub cat_skin: CatSkin,
    #[serde(default)]
    pub always_visible_pet_mode: bool,
    #[serde(default = "default_cat_scale")]
    pub cat_scale: f32,
    #[serde(default)]
    pub animation_speed: AnimationSpeed,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
    #[serde(default = "default_pixel_scale")]
    pub pixel_scale: u32,
    #[serde(default = "default_bubble_bg_color", alias = "bubble_bg")]
    pub bubble_bg_color: String,
    #[serde(
        default = "default_bubble_text_color",
        alias = "bubble_fg",
        alias = "bubble_text"
    )]
    pub bubble_text_color: String,
    #[serde(default = "default_bubble_border")]
    pub bubble_border: String,
}

fn default_cat_scale() -> f32 {
    1.5
}
fn default_font_scale() -> f32 {
    1.0
}
fn default_pixel_scale() -> u32 {
    2
}
fn default_bubble_bg_color() -> String {
    "#FFFFFF".to_string()
}
fn default_bubble_text_color() -> String {
    "#000000".to_string()
}
fn default_bubble_border() -> String {
    "#000000".to_string()
}

pub fn parse_hex_color(hex: &str) -> (f64, f64, f64) {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
        }
    } else if clean.len() == 3 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..1], 16),
            u8::from_str_radix(&clean[1..2], 16),
            u8::from_str_radix(&clean[2..3], 16),
        ) {
            return (
                (r * 17) as f64 / 255.0,
                (g * 17) as f64 / 255.0,
                (b * 17) as f64 / 255.0,
            );
        }
    }
    (1.0, 1.0, 1.0)
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            cat_skin: CatSkin::CartonCat,
            always_visible_pet_mode: false,
            cat_scale: 1.5,
            animation_speed: AnimationSpeed::Normal,
            font_scale: 1.0,
            pixel_scale: 2,
            bubble_bg_color: default_bubble_bg_color(),
            bubble_text_color: default_bubble_text_color(),
            bubble_border: default_bubble_border(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, alias = "sound_profile")]
    pub sound_effect: SoundEffect,
    #[serde(
        default = "default_sound_volume",
        alias = "volume",
        deserialize_with = "deserialize_sound_volume"
    )]
    pub sound_volume: u32,
}

fn default_sound_volume() -> u32 {
    50
}

fn deserialize_sound_volume<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VolumeVal {
        Int(i64),
        Float(f64),
    }

    match VolumeVal::deserialize(deserializer)? {
        VolumeVal::Int(i) => Ok(i.clamp(0, 100) as u32),
        VolumeVal::Float(f) => {
            if f <= 1.0 {
                Ok((f * 100.0).round().clamp(0.0, 100.0) as u32)
            } else {
                Ok(f.round().clamp(0.0, 100.0) as u32)
            }
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_effect: SoundEffect::Realistic,
            sound_volume: 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Personality {
    #[serde(alias = "normal", alias = "Normal")]
    #[default]
    Normal,
    #[serde(alias = "cute", alias = "Cute")]
    Cute,
    #[serde(alias = "sassy", alias = "Sassy")]
    Sassy,
    #[serde(alias = "lazy", alias = "Lazy")]
    Lazy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanterConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub personality: Personality,
    #[serde(default = "default_banter_min")]
    pub interval_min: u32,
    #[serde(default = "default_banter_max")]
    pub interval_max: u32,
    #[serde(default = "default_true")]
    pub play_sound: bool,
}

fn default_banter_min() -> u32 {
    60
}

fn default_banter_max() -> u32 {
    180
}

impl Default for BanterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            personality: Personality::Normal,
            interval_min: 60,
            interval_max: 180,
            play_sound: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub pacman_cache_alert: bool,
    #[serde(default = "default_cache_limit_gb")]
    pub pacman_cache_limit_gb: f32,
    #[serde(default = "default_true")]
    pub orphan_alert: bool,
    #[serde(default = "default_orphan_limit")]
    pub orphan_limit: usize,
    #[serde(default = "default_true")]
    pub failed_services_alert: bool,
}

fn default_cache_limit_gb() -> f32 {
    6.0
}

fn default_orphan_limit() -> usize {
    10
}

impl Default for SmartConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pacman_cache_alert: true,
            pacman_cache_limit_gb: 6.0,
            orphan_alert: true,
            orphan_limit: 10,
            failed_services_alert: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomAction {
    pub name: String,
    pub command: String,
}

fn default_custom_actions() -> Vec<CustomAction> {
    vec![
        CustomAction {
            name: "Work Setup".to_string(),
            command: "firefox & kitty".to_string(),
        },
        CustomAction {
            name: "Clean Pacman Cache".to_string(),
            command: "kitty -e paccache -rk2".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub updates: UpdatesConfig,
    #[serde(default)]
    pub visual: VisualConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub banter: BanterConfig,
    #[serde(default)]
    pub smart: SmartConfig,
    #[serde(default = "default_custom_actions")]
    pub custom_actions: Vec<CustomAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_override: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            updates: UpdatesConfig::default(),
            visual: VisualConfig::default(),
            audio: AudioConfig::default(),
            banter: BanterConfig::default(),
            smart: SmartConfig::default(),
            custom_actions: default_custom_actions(),
            terminal_override: None,
        }
    }
}

impl Config {
    pub fn terminal_override(&self) -> Option<&str> {
        self.terminal_override
            .as_deref()
            .or(self.general.terminal_override.as_deref())
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("kittynotify")
    }

    pub fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("kittynotify")
    }

    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~/.cache"))
            .join("kittynotify")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        let config = Self::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        fs::write(Self::config_path(), content)
    }

    pub fn is_package_ignored(&self, pkg_name: &str) -> bool {
        for pattern_str in &self.updates.ignore_packages {
            if pattern_str == pkg_name {
                return true;
            }
            if let Ok(pat) = Pattern::new(pattern_str) {
                if pat.matches(pkg_name) {
                    return true;
                }
            }
        }
        false
    }

    pub fn purge_user_data() -> std::io::Result<()> {
        let conf = Self::config_dir();
        let cache = Self::cache_dir();
        let data = Self::data_dir();
        if conf.exists() {
            fs::remove_dir_all(conf)?;
        }
        if cache.exists() {
            fs::remove_dir_all(cache)?;
        }
        if data.exists() {
            fs::remove_dir_all(data)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_ignore_matching() {
        let config = Config::default();
        assert!(config.is_package_ignored("discord"));
        assert!(config.is_package_ignored("electron32"));
        assert!(!config.is_package_ignored("linux-zen"));
    }

    #[test]
    fn test_audio_config_deserialization() {
        let toml_new = r#"
            enabled = true
            sound_effect = "Realistic"
            sound_volume = 75
        "#;
        let cfg: AudioConfig = toml::from_str(toml_new).unwrap();
        assert_eq!(cfg.sound_effect, SoundEffect::Realistic);
        assert_eq!(cfg.sound_volume, 75);

        let toml_legacy = r#"
            enabled = true
            sound_profile = "chiptune"
            volume = 0.8
        "#;
        let cfg_legacy: AudioConfig = toml::from_str(toml_legacy).unwrap();
        assert_eq!(cfg_legacy.sound_effect, SoundEffect::Chiptune);
        assert_eq!(cfg_legacy.sound_volume, 80);

        let toml_mute = r#"
            enabled = false
            sound_effect = "Mute"
            sound_volume = 0
        "#;
        let cfg_mute: AudioConfig = toml::from_str(toml_mute).unwrap();
        assert_eq!(cfg_mute.sound_effect, SoundEffect::Mute);
        assert_eq!(cfg_mute.sound_volume, 0);
    }

    #[test]
    fn test_general_config_monitor() {
        let toml_default = r#"
            check_interval_hours = 3
        "#;
        let cfg: GeneralConfig = toml::from_str(toml_default).unwrap();
        assert_eq!(cfg.monitor, "auto");

        let toml_custom = r#"
            monitor = "DP-1"
        "#;
        let cfg_custom: GeneralConfig = toml::from_str(toml_custom).unwrap();
        assert_eq!(cfg_custom.monitor, "DP-1");
    }

    #[test]
    fn test_visual_config_colors() {
        let toml_new = r##"
            cat_skin = "DraculaCat"
            bubble_bg_color = "#333333"
            bubble_text_color = "#EEEEEE"
        "##;
        let cfg: VisualConfig = toml::from_str(toml_new).unwrap();
        assert_eq!(cfg.cat_skin, CatSkin::DraculaCat);
        assert_eq!(cfg.bubble_bg_color, "#333333");
        assert_eq!(cfg.bubble_text_color, "#EEEEEE");

        // Legacy aliases
        let toml_legacy = r##"
            bubble_bg = "#FFFFFF"
            bubble_fg = "#000000"
        "##;
        let cfg_legacy: VisualConfig = toml::from_str(toml_legacy).unwrap();
        assert_eq!(cfg_legacy.bubble_bg_color, "#FFFFFF");
        assert_eq!(cfg_legacy.bubble_text_color, "#000000");
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FFFFFF"), (1.0, 1.0, 1.0));
        assert_eq!(parse_hex_color("#000000"), (0.0, 0.0, 0.0));
        assert_eq!(parse_hex_color("#ff0000"), (1.0, 0.0, 0.0));
        assert_eq!(parse_hex_color("#0f0"), (0.0, 1.0, 0.0));
        assert_eq!(parse_hex_color("invalid"), (1.0, 1.0, 1.0));
    }

    #[test]
    fn test_custom_actions_config() {
        let config = Config::default();
        assert_eq!(config.custom_actions.len(), 2);
        assert_eq!(config.custom_actions[0].name, "Work Setup");
        assert_eq!(config.custom_actions[0].command, "firefox & kitty");
        assert_eq!(config.custom_actions[1].name, "Clean Pacman Cache");

        let toml_str = r#"
            [[custom_actions]]
            name = "Test App"
            command = "alacritty"
        "#;
        let parsed: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.custom_actions.len(), 1);
        assert_eq!(parsed.custom_actions[0].name, "Test App");
        assert_eq!(parsed.custom_actions[0].command, "alacritty");
    }

    #[test]
    fn test_visual_config_animation_speed() {
        let default_cfg = VisualConfig::default();
        assert_eq!(default_cfg.animation_speed, AnimationSpeed::Normal);
        assert_eq!(default_cfg.animation_speed.multiplier(), 1.0);

        let toml_half = r#"
            animation_speed = "0.5x"
        "#;
        let cfg_half: VisualConfig = toml::from_str(toml_half).unwrap();
        assert_eq!(cfg_half.animation_speed, AnimationSpeed::Half);
        assert_eq!(cfg_half.animation_speed.multiplier(), 0.5);

        let toml_double = r#"
            animation_speed = "2.0x"
        "#;
        let cfg_double: VisualConfig = toml::from_str(toml_double).unwrap();
        assert_eq!(cfg_double.animation_speed, AnimationSpeed::Double);
        assert_eq!(cfg_double.animation_speed.multiplier(), 2.0);

        // Aliases
        let toml_alias: VisualConfig = toml::from_str("animation_speed = 'slow'").unwrap();
        assert_eq!(toml_alias.animation_speed, AnimationSpeed::Half);
        let toml_alias2: VisualConfig = toml::from_str("animation_speed = 'fast'").unwrap();
        assert_eq!(toml_alias2.animation_speed, AnimationSpeed::Double);
        let toml_alias3: VisualConfig = toml::from_str("animation_speed = '1x'").unwrap();
        assert_eq!(toml_alias3.animation_speed, AnimationSpeed::Normal);

        assert_eq!(AnimationSpeed::from_multiplier(0.5), AnimationSpeed::Half);
        assert_eq!(AnimationSpeed::from_multiplier(1.0), AnimationSpeed::Normal);
        assert_eq!(AnimationSpeed::from_multiplier(2.0), AnimationSpeed::Double);
    }

    #[test]
    fn test_terminal_override_config() {
        let toml_general = r#"
            [general]
            terminal_override = "konsole"
        "#;
        let cfg_general: Config = toml::from_str(toml_general).unwrap();
        assert_eq!(cfg_general.terminal_override(), Some("konsole"));

        let toml_root = r#"
            terminal_override = "alacritty"
        "#;
        let cfg_root: Config = toml::from_str(toml_root).unwrap();
        assert_eq!(cfg_root.terminal_override(), Some("alacritty"));

        let cfg_default = Config::default();
        assert_eq!(cfg_default.terminal_override(), None);
    }
}
