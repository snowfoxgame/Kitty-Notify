use crate::config::{CatSkin, SoundEffect};
use gtk4::gdk::{Display, Texture};
use gtk4::CssProvider;
use std::fs;
use std::path::PathBuf;

pub const FONT_M5X7: &[u8] = include_bytes!("../../assets/fonts/m5x7.ttf");
pub const SFX_REALISTIC_WAV: &[u8] = include_bytes!("../../assets/sounds/cat_meow_short.wav");
pub const SFX_CHIPTUNE_OGG: &[u8] = include_bytes!("../../assets/sounds/meow_chiptune.ogg");
#[allow(dead_code)]
pub const SFX_CHIPTUNE_WAV: &[u8] = include_bytes!("../../assets/sounds/meow_chiptune.wav");
#[allow(dead_code)]
pub const SPRITE_TABBY_SVG: &str = include_str!("../../assets/sprites/cat_tabby.svg");
#[allow(dead_code)]
pub const PAW_PATTERN_SVG: &str = include_str!("../../assets/sprites/paw_pattern_tile.svg");

// Carton Cat (Box Cat) - 4 Frames
pub const CARTON_CAT_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/carton_cat/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/carton_cat/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/carton_cat/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/carton_cat/frame_3.png"),
];

// Dracula Cat - 6 Frames
pub const DRACULA_CAT_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/dracula_cat/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/dracula_cat/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/dracula_cat/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/dracula_cat/frame_3.png"),
    include_bytes!("../../assets/sprites/cat_pack/dracula_cat/frame_4.png"),
    include_bytes!("../../assets/sprites/cat_pack/dracula_cat/frame_5.png"),
];

// Classic Idle Cat - 10 Frames
pub const IDLE_CAT_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_3.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_4.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_5.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_6.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_7.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_8.png"),
    include_bytes!("../../assets/sprites/cat_pack/idle_cat/frame_9.png"),
];

// Standard Cat Expanded States
pub const STANDARD_CAT_LAY_DOWN_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_3.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_4.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_5.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_6.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/lay_down/frame_7.png"),
];

pub const STANDARD_CAT_LICKING_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/licking/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/licking/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/licking/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/licking/frame_3.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/licking/frame_4.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/licking/frame_5.png"),
];

pub const STANDARD_CAT_SLEEPY_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/sleepy/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleepy/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleepy/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleepy/frame_3.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleepy/frame_4.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleepy/frame_5.png"),
];

pub const STANDARD_CAT_SLEEPING_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/sleeping/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleeping/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleeping/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sleeping/frame_3.png"),
];

pub const STANDARD_CAT_EXCITED_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/excited/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/excited/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/excited/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/excited/frame_3.png"),
];

pub const STANDARD_CAT_DESPITE_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/despite/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/despite/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/despite/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/despite/frame_3.png"),
];

pub const STANDARD_CAT_SICK_1_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_1/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_1/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_1/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_1/frame_3.png"),
];

pub const STANDARD_CAT_SICK_2_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_2/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_2/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_2/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/sick_2/frame_3.png"),
];

pub const STANDARD_CAT_DEAD_FRAMES: &[&[u8]] = &[
    include_bytes!("../../assets/sprites/cat_pack/standard/dead/frame_0.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/dead/frame_1.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/dead/frame_2.png"),
    include_bytes!("../../assets/sprites/cat_pack/standard/dead/frame_3.png"),
];

pub fn load_textures_from_slice(frames: &[&[u8]], scale: f32) -> Vec<Texture> {
    let target_size = (64.0 * scale.clamp(1.0, 3.0)).round() as i32;

    frames
        .iter()
        .filter_map(|bytes| {
            let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(bytes));
            let pixbuf = gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE).ok()?;
            let scaled = pixbuf.scale_simple(
                target_size,
                target_size,
                gdk_pixbuf::InterpType::Nearest,
            )?;
            Some(Texture::for_pixbuf(&scaled))
        })
        .collect()
}

pub fn load_skin_textures(skin: CatSkin, scale: f32) -> Vec<Texture> {
    let raw_frames: &[&[u8]] = match skin {
        CatSkin::CartonCat => CARTON_CAT_FRAMES,
        CatSkin::DraculaCat => DRACULA_CAT_FRAMES,
        CatSkin::IdleCat => IDLE_CAT_FRAMES,
    };
    load_textures_from_slice(raw_frames, scale)
}

#[derive(Clone)]
pub struct StandardCatTextures {
    pub idle: Vec<Texture>,
    pub lay_down: Vec<Texture>,
    pub licking: Vec<Texture>,
    pub sleepy: Vec<Texture>,
    pub sleeping: Vec<Texture>,
    pub excited: Vec<Texture>,
    pub despite: Vec<Texture>,
    pub sick_1: Vec<Texture>,
    pub sick_2: Vec<Texture>,
    pub dead: Vec<Texture>,
}

impl StandardCatTextures {
    pub fn get(&self, state: crate::ui::animator::StandardCatState) -> &[Texture] {
        match state {
            crate::ui::animator::StandardCatState::Idle => &self.idle,
            crate::ui::animator::StandardCatState::LayDown => &self.lay_down,
            crate::ui::animator::StandardCatState::Licking => &self.licking,
            crate::ui::animator::StandardCatState::Sleepy => &self.sleepy,
            crate::ui::animator::StandardCatState::Sleeping => &self.sleeping,
            crate::ui::animator::StandardCatState::Excited => &self.excited,
            crate::ui::animator::StandardCatState::Despite => &self.despite,
            crate::ui::animator::StandardCatState::Sick1 => &self.sick_1,
            crate::ui::animator::StandardCatState::Sick2 => &self.sick_2,
            crate::ui::animator::StandardCatState::Dead => &self.dead,
        }
    }
}

pub fn load_all_standard_cat_textures(scale: f32) -> StandardCatTextures {
    StandardCatTextures {
        idle: load_textures_from_slice(IDLE_CAT_FRAMES, scale),
        lay_down: load_textures_from_slice(STANDARD_CAT_LAY_DOWN_FRAMES, scale),
        licking: load_textures_from_slice(STANDARD_CAT_LICKING_FRAMES, scale),
        sleepy: load_textures_from_slice(STANDARD_CAT_SLEEPY_FRAMES, scale),
        sleeping: load_textures_from_slice(STANDARD_CAT_SLEEPING_FRAMES, scale),
        excited: load_textures_from_slice(STANDARD_CAT_EXCITED_FRAMES, scale),
        despite: load_textures_from_slice(STANDARD_CAT_DESPITE_FRAMES, scale),
        sick_1: load_textures_from_slice(STANDARD_CAT_SICK_1_FRAMES, scale),
        sick_2: load_textures_from_slice(STANDARD_CAT_SICK_2_FRAMES, scale),
        dead: load_textures_from_slice(STANDARD_CAT_DEAD_FRAMES, scale),
    }
}

pub const RETRO_PIXEL_CSS: &str = r#"
* {
    font-family: 'm5x7', monospace;
    font-size: 16px;
}

window.pixel-window {
    background-color: transparent;
}

.pixel-bubble-content {
    background-color: transparent;
    min-width: 220px;
}

.pixel-bubble-title {
    font-weight: bold;
    font-size: 18px;
    color: #000000;
    margin-bottom: 2px;
}

.pixel-bubble-body {
    font-size: 16px;
    color: #1e1e2e;
    margin-bottom: 6px;
    line-height: 1.25;
}

/* Force dark readable high-contrast text on all light and pastel buttons and their labels */
button.action-btn,
button.action-btn label,
.pastel-button,
.pastel-button label,
.pixel-button,
.pixel-button label,
.pixel-button-primary,
.pixel-button-primary label,
.pixel-button-danger,
.pixel-button-danger label,
.pixel-action-btn,
.pixel-action-btn label,
.settings-card button.pixel-button,
.settings-card button.pixel-button label,
.settings-window button.pixel-button,
.settings-window button.pixel-button label {
    color: #11111b;
    font-weight: bold;
}

.pixel-button {
    background-color: #ffffff;
    color: #11111b;
    border: 2px solid #000000;
    border-radius: 0px;
    padding: 3px 8px;
    font-weight: bold;
    box-shadow: 2px 2px 0px #000000;
    transition: none;
}

.pixel-button label {
    color: #11111b;
}

.pixel-button:hover {
    background-color: #f0f2f5;
    color: #11111b;
}

.pixel-button:hover label {
    color: #11111b;
}

.pixel-button:active {
    background-color: #dce0e8;
    color: #11111b;
    margin-top: 2px;
    margin-left: 2px;
    box-shadow: 0px 0px 0px #000000;
}

.pixel-button-primary {
    background-color: #a6e3a1;
    color: #11111b;
    border: 2px solid #000000;
}

.pixel-button-primary label {
    color: #11111b;
}

.pixel-button-primary:hover {
    background-color: #94e2d5;
    color: #11111b;
}

.pixel-button-primary:hover label {
    color: #11111b;
}

.pixel-button-danger {
    background-color: #f38ba8;
    color: #11111b;
    border: 2px solid #000000;
}

.pixel-button-danger label {
    color: #11111b;
}

.pixel-button-danger:hover {
    background-color: #eba0ac;
    color: #11111b;
}

.pixel-button-danger:hover label {
    color: #11111b;
}

.pixel-action-btn {
    background-color: #dcdcdc;
    color: #11111b;
    border: 2px solid #000000;
    border-radius: 0px;
    padding: 3px 8px;
    font-weight: bold;
    box-shadow: 2px 2px 0px #000000;
    transition: none;
}

.pixel-action-btn label {
    color: #11111b;
}

.pixel-action-btn:active {
    background-color: #b8b8b8;
    color: #11111b;
    margin-top: 2px;
    margin-left: 2px;
    box-shadow: 0px 0px 0px #000000;
}

.pixel-action-btn-update:hover {
    background-color: #38b764;
    color: #11111b;
    box-shadow: 2px 2px 0px #000000;
}

.pixel-action-btn-update:hover label {
    color: #11111b;
}

.pixel-action-btn-dismiss:hover {
    background-color: #d94b4b;
    color: #ffffff;
    box-shadow: 2px 2px 0px #000000;
}

.pixel-action-btn-dismiss:hover label {
    color: #ffffff;
}

popover.pixel-popover {
    background-color: transparent;
    padding: 0;
}

popover.pixel-popover > arrow {
    background-color: #ffffff;
    border: 2px solid #000000;
}

popover.pixel-popover contents {
    background-color: #ffffff;
    border: 2px solid #000000;
    box-shadow: 3px 3px 0px #000000;
    border-radius: 0px;
    padding: 4px;
}

.pixel-menu-item {
    background-color: transparent;
    color: #111111;
    border: 1px solid transparent;
    border-radius: 0px;
    padding: 4px 10px;
    font-weight: bold;
    font-size: 13px;
    box-shadow: none;
    transition: none;
}

.pixel-menu-item:hover {
    background-color: #a6e3a1;
    color: #000000;
    border: 1px solid #000000;
}

.pixel-menu-item:active {
    background-color: #94e2d5;
}

.pixel-menu-item-danger {
    color: #b73838;
}

.pixel-menu-item-danger:hover {
    background-color: #f38ba8;
    color: #ffffff;
    border: 1px solid #000000;
}

.pixel-menu-separator {
    background-color: #000000;
    min-height: 2px;
    margin: 3px 2px;
}

.pixel-menu-empty {
    font-size: 12px;
    font-style: italic;
    color: #666666;
    padding: 4px 8px;
}

.disclaimer-banner {
    background-color: #2b2210;
    border: 2px solid #fab387;
    border-radius: 0px;
    box-shadow: 2px 2px 0px #000000;
    padding: 10px 12px;
}

.disclaimer-icon {
    font-size: 20px;
}

.disclaimer-text {
    font-size: 12px;
    color: #f9e2af;
    line-height: 1.35;
}

.action-item-row {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 0px;
    padding: 6px 10px;
}

.action-item-name {
    font-weight: bold;
    font-size: 13px;
    color: #ffffff;
}

.action-item-cmd {
    font-family: monospace;
    font-size: 11px;
    color: #a6adc8;
}

.settings-window {
    background-color: transparent;
}

.settings-scrolled {
    background-color: transparent;
}

.settings-scrolled viewport {
    background-color: transparent;
}

.settings-header-card {
    background-color: rgba(30, 30, 46, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.10);
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 2px;
}

.settings-card {
    background-color: rgba(30, 30, 46, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.10);
    border-radius: 8px;
    padding: 14px 16px;
    margin-bottom: 2px;
}

.settings-card label {
    color: #cdd6f4;
}

.settings-header-card label {
    color: #cdd6f4;
}

.settings-card button label,
.settings-card button.pixel-button label,
.settings-card button.pixel-action-btn label,
.settings-card button.pixel-button-primary label,
.settings-card button.pixel-button-danger label,
.settings-window button label,
.settings-window button.pixel-button label,
.settings-window button.pixel-action-btn label,
.pixel-preview-container button label,
.pixel-preview-container .pixel-action-btn label {
    color: #11111b;
    font-weight: bold;
}

.settings-card label.settings-group-header,
.settings-group-header {
    font-weight: bold;
    font-size: 14px;
    color: #cba6f7;
    margin-top: 2px;
    margin-bottom: 4px;
}

.settings-header-card label.settings-title,
.settings-title {
    font-weight: bold;
    font-size: 18px;
    color: #ffffff;
    margin-bottom: 2px;
}

.settings-header-card label.settings-subtitle,
.settings-subtitle {
    font-size: 13px;
    color: #a6adc8;
}

.pixel-preview-container {
    background-color: #181825;
    border: 2px solid #313244;
    border-radius: 6px;
    padding: 12px 16px;
    margin-top: 4px;
    margin-bottom: 6px;
}


.cat-sprite-container {
    background-color: transparent;
    padding-bottom: 6px;
    margin-bottom: 6px;
}

.cat-sprite {
    background-color: transparent;
}

.cat-flipped-x {
    transform: scaleX(-1);
}

.cat-flipped-y {
    transform: scaleY(-1);
}

.cat-flipped-xy {
    transform: scale(-1, -1);
}
"#;

pub fn init_pixel_assets() {
    if let Some(font_dir) = dirs::font_dir() {
        let app_font_dir = font_dir.join("kittynotify");
        let font_path = app_font_dir.join("m5x7.ttf");
        if !font_path.exists() {
            let _ = fs::create_dir_all(&app_font_dir);
            let _ = fs::write(&font_path, FONT_M5X7);
        }
    }

    if let Some(display) = Display::default() {
        let provider = CssProvider::new();
        provider.load_from_data(RETRO_PIXEL_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

pub fn dump_sound_file(effect: SoundEffect) -> std::io::Result<PathBuf> {
    let cache_dir = crate::config::Config::cache_dir();
    fs::create_dir_all(&cache_dir)?;
    match effect {
        SoundEffect::Realistic => {
            let sound_path = cache_dir.join("cat_meow_short.wav");
            if !sound_path.exists() {
                fs::write(&sound_path, SFX_REALISTIC_WAV)?;
            }
            Ok(sound_path)
        }
        SoundEffect::Chiptune => {
            let sound_path = cache_dir.join("meow_chiptune.ogg");
            if !sound_path.exists() {
                fs::write(&sound_path, SFX_CHIPTUNE_OGG)?;
            }
            Ok(sound_path)
        }
        SoundEffect::Mute => Err(std::io::Error::other("Sound muted")),
    }
}

pub fn generate_settings_background_css(bubble_bg_color: &str) -> String {
    let (r, g, b) = crate::config::parse_hex_color(bubble_bg_color);

    // Softened/muted derivative: blend 18% of bubble color into dark mantle (#181825)
    let blend = 0.18;
    let base_r = 24.0;
    let base_g = 24.0;
    let base_b = 37.0;
    let tr = ((1.0 - blend) * base_r + blend * r * 255.0).round().clamp(0.0, 255.0) as u8;
    let tg = ((1.0 - blend) * base_g + blend * g * 255.0).round().clamp(0.0, 255.0) as u8;
    let tb = ((1.0 - blend) * base_b + blend * b * 255.0).round().clamp(0.0, 255.0) as u8;
    let tint_hex = format!("#{:02X}{:02X}{:02X}", tr, tg, tb);

    // Subtle paw watermark tint matching bubble color at 12% opacity
    let pr = (r * 255.0).round() as u8;
    let pg = (g * 255.0).round() as u8;
    let pb = (b * 255.0).round() as u8;
    let paw_hex = format!("#{:02X}{:02X}{:02X}", pr, pg, pb);

    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32" shape-rendering="crispEdges"><g fill="{}" fill-opacity="0.12"><rect x="3" y="5" width="2" height="2"/><rect x="6" y="3" width="2" height="2"/><rect x="9" y="3" width="2" height="2"/><rect x="12" y="5" width="2" height="2"/><rect x="6" y="8" width="4" height="1"/><rect x="5" y="9" width="6" height="2"/><rect x="6" y="11" width="4" height="1"/><rect x="7" y="12" width="2" height="1"/><rect x="19" y="21" width="2" height="2"/><rect x="22" y="19" width="2" height="2"/><rect x="25" y="19" width="2" height="2"/><rect x="28" y="21" width="2" height="2"/><rect x="22" y="24" width="4" height="1"/><rect x="21" y="25" width="6" height="2"/><rect x="22" y="27" width="4" height="1"/><rect x="23" y="28" width="2" height="1"/></g></svg>"#,
        paw_hex
    );

    let b64 = glib::base64_encode(svg.as_bytes());

    format!(
        r#"
window.settings-window,
.settings-window,
.settings-scrolled,
.settings-scrolled viewport {{
    background-color: {tint_hex};
    background-image: url("data:image/svg+xml;base64,{b64}");
    background-repeat: repeat;
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_settings_background_css() {
        let css_white = generate_settings_background_css("#FFFFFF");
        assert!(css_white.contains("background-color:"));
        assert!(css_white.contains("data:image/svg+xml;base64,"));
        assert!(css_white.contains("background-repeat: repeat;"));

        let css_pink = generate_settings_background_css("#F38BA8");
        assert_ne!(css_white, css_pink);
        assert!(css_pink.contains("window.settings-window"));
    }
}

