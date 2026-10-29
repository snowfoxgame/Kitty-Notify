# KittyNotify 🐱✨

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Wayland](https://img.shields.io/badge/display-Wayland%20%2F%20Layer--Shell-blue.svg)](https://wayland.freedesktop.org/)
[![GTK4](https://img.shields.io/badge/toolkit-GTK4%20%2F%20Libadwaita-green.svg)](https://www.gtk.org/)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)

A lightweight, modular desktop pet and system companion for Linux / Wayland. KittyNotify stays peacefully anchored to your desktop or appears ephemerally to notify you of pending system updates, resource bottlenecks, or maintenance reminders with quirky, context-aware dialogue and charming pixel-art animations.

Featuring customizable skins, including cat sprites designed by **ToffeeCraft** on [Itch.io (Cat Mega Bundle)](https://toffeecraft.itch.io/cat-mega-bundle).

---

## 🌟 Features

- **🐾 Dual Operating Modes:**
  - **Desktop Pet Mode:** A permanent, calm desktop companion sitting in your designated screen corner (`bottom_right`, `bottom_left`, `top_right`, `top_left`), with natural idle pauses, grooming, licking, laying down, and sleeping states.
  - **Ephemeral Notification Mode:** Quiet in the background until system updates (Arch official repos, AUR, and Flatpak) require your attention.
- **🧠 Context-Aware Behavioral Dialogue & Habit Learning:**
  - Dynamic dialogue reacting to dismiss streaks, high memory/CPU usage, pacman cache sizes, long uptimes, night-time sessions, active gaming, and unhandled systemd service failures.
  - Anti-repeat dialogue buffers and machine-seeded randomness to keep lines fresh and unrepetitive.
- **⚡ Universal Desktop Terminal Resolver:**
  - Automatically resolves installed terminal emulators (`konsole`, `alacritty`, `ghostty`, `foot`, `kitty`, `ptyxis`, `gnome-terminal`, `xfce4-terminal`, `wezterm`, `xterm`, `rxvt`) and user shells (`fish`, `bash`, `zsh`, `sh`).
  - Seamlessly strips terminal wrapper prefixes and preserves a graceful `"Press Enter to close..."` interactive pause.
- **🖱️ Right-Click Context Menu & Custom Quick Actions:**
  - Desktop pet mode includes a retro-styled context menu on right-click.
  - Run configurable one-click shell commands or interactive scripts directly from your pet.
- **🎨 Visual & Audio Customization:**
  - Multiple cat skins (`CartonCat`, `DraculaCat`, `IdleCat`).
  - Adjustable animation speeds (`0.5x` Slow, `1.0x` Normal, `2.0x` Fast) and scale (`1.0x` to `3.0x`).
  - Customizable speech bubble background, text, and border colors with retro pixelated borders.
  - Optional sound effects (Realistic meows, Chiptune 8-bit effects, or Mute) with volume controls.
- **🪶 Ultra-Low Resource Footprint:**
  - Pure native Rust + GTK4 Layer-Shell.
  - Event-driven GLib IPC channels (< 5–10 MB idle RAM, 0% CPU at rest).

---

## 📦 Prerequisites

KittyNotify is designed for Linux environments running Wayland (such as KDE Plasma Wayland, Sway, Hyprland, GNOME Wayland).

### Arch Linux / CachyOS Packages

Install the required build tools and libraries:

```bash
sudo pacman -S --needed base-devel rust cargo gtk4 gtk4-layer-shell libadwaita pango cairo
```

Optional (for AUR helper update checks):
- `yay` or `paru`

---

## 🚀 Installation & Build

1. **Clone the repository:**
   ```bash
   git clone https://github.com/snowfoxgame/Kitty-Notify.git
   cd Kitty-Notify
   ```

2. **Build the release binary:**
   ```bash
   cargo build --release
   ```
   The compiled binary will be located at `target/release/kittynotify`.

3. **Install system-wide (optional):**
   ```bash
   sudo install -Dm755 target/release/kittynotify /usr/local/bin/kittynotify
   ```

---

## 🎮 Usage & CLI Commands

```bash
# Start KittyNotify in permanent Desktop Pet Mode
kittynotify pet

# Check for updates and show notification popup if updates are pending
kittynotify

# Open the Settings Window (configure skins, colors, monitors, actions)
kittynotify settings

# Send interactive tap/banter poke to the running pet
kittynotify tap

# Adjust animation speed on the fly (0.5x, 1.0x, 2.0x)
kittynotify speed 1.0x

# Trigger a test update notification (Repo: 7, AUR: 2, Flatpak: 0)
kittynotify test

# Cleanly stop the running Desktop Pet process
kittynotify stop
```

---

## ⚙️ Configuration

Configuration is automatically stored in `~/.config/kittynotify/config.toml`. A default template is provided in [`config.toml`](config.toml):

```toml
[general]
check_interval_hours = 2
corner = "bottom_right"      # "bottom_right", "bottom_left", "top_right", "top_left"
display_mode = "overlay"      # "overlay" (above windows) or "desktop_only" (desktop level)
show_on_all_monitors = false
monitor = "auto"              # "auto", or connector name (e.g. "DP-1", "HDMI-A-1")
auto_dismiss_seconds = 10

[visual]
cat_skin = "IdleCat"          # "CartonCat", "DraculaCat", "IdleCat"
always_visible_pet_mode = true
animation_speed = "1.0x"      # "0.5x", "1.0x", "2.0x"
cat_scale = 1.5
bubble_bg_color = "#FFFFFF"
bubble_text_color = "#000000"
bubble_border = "#000000"

[audio]
enabled = true
sound_effect = "Realistic"    # "Realistic", "Chiptune", "Mute"
sound_volume = 50

[banter]
enabled = true
personality = "normal"        # "normal", "cute", "sassy", "lazy"
interval_min = 60
interval_max = 180
play_sound = true

[smart]
enabled = true
pacman_cache_alert = true
pacman_cache_limit_gb = 6.0
orphan_alert = true
orphan_limit = 10
failed_services_alert = true

[[custom_actions]]
name = "Work Setup"
command = "firefox & konsole"

[[custom_actions]]
name = "Clean Pacman Cache"
command = "paccache -rk2"
```

---

## ⚠️ Security & Liability Disclaimer

> **IMPORTANT NOTICE:**
> Executing commands with `sudo` or elevated root privileges through custom actions is strongly discouraged.
>
> **KittyNotify and its authors take zero responsibility or liability for any system damage, data loss, misconfigurations, or broken packages resulting from user-configured custom actions or elevated root/`sudo` commands. All custom commands run entirely at the user's own risk.**
>
> Although you are welcome to write to me if something is broken and I'll do my best to help fix issues in the application code, I do not take responsibility for any system damages caused by custom commands or misconfigurations.

---

## 📜 Terms of Use, Attribution & Ethical Guidelines

### Personal Use & Contributions
Anyone is welcome to use, fork, and contribute minor improvements to KittyNotify, under the strict condition that original author credit is visibly retained with a clear backlink to the original GitHub repository.

### Acceptable Use & Anti-Malware Policy
This software is created purely as a benign, open-source desktop companion and productivity tool. Using KittyNotify or its source code to facilitate, deliver, or obfuscate malicious activities—including but not limited to phishing, credential harvesting, unauthorized execution, backdoor access, or malware distribution—is strictly prohibited.

### Sprite & Asset Attribution
- Cat sprite packs and artwork: **ToffeeCraft** via [Itch.io Cat Mega Bundle](https://toffeecraft.itch.io/cat-mega-bundle).
- Pixel-art font: *Press Start 2P* by CodeMan38 (SIL Open Font License).

---

## 📄 License

KittyNotify is licensed under the [GNU General Public License v3.0 or later (GPL-3.0-or-later)](LICENSE).
