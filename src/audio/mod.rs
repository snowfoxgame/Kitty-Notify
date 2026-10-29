use crate::assets::dump_sound_file;
use crate::checker::aur::is_command_in_path;
use crate::config::{AudioConfig, SoundEffect};
use std::process::Command;

pub fn play_tap_meow(config: &AudioConfig) {
    if !config.enabled || matches!(config.sound_effect, SoundEffect::Mute) || config.sound_volume == 0 {
        return;
    }
    let mut tap_cfg = config.clone();
    tap_cfg.sound_volume = ((config.sound_volume as f32) * 0.6).round().clamp(1.0, 100.0) as u32;
    play_meow(&tap_cfg);
}

pub fn play_meow(config: &AudioConfig) {
    if !config.enabled || matches!(config.sound_effect, SoundEffect::Mute) || config.sound_volume == 0 {
        return;
    }

    let sound_path = match dump_sound_file(config.sound_effect) {
        Ok(path) => path,
        Err(_) => return,
    };

    let volume_factor = (config.sound_volume.min(100) as f64) / 100.0;

    std::thread::spawn(move || {
        let path_str = sound_path.to_string_lossy();

        if is_command_in_path("pw-play") {
            let vol_arg = format!("{:.2}", volume_factor);
            if let Ok(mut child) = Command::new("pw-play")
                .arg("--volume")
                .arg(&vol_arg)
                .arg(sound_path.as_os_str())
                .spawn()
            {
                let _ = child.wait();
                return;
            }
        }

        if is_command_in_path("paplay") {
            let pa_vol = ((volume_factor * 65536.0) as u32).min(65536);
            if let Ok(mut child) = Command::new("paplay")
                .arg(format!("--volume={}", pa_vol))
                .arg(sound_path.as_os_str())
                .spawn()
            {
                let _ = child.wait();
                return;
            }
        }

        if is_command_in_path("canberra-gtk-play") {
            if let Ok(mut child) = Command::new("canberra-gtk-play")
                .args(["-f", &path_str])
                .spawn()
            {
                let _ = child.wait();
            }
        }
    });
}
