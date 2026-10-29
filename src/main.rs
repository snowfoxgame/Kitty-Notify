pub mod actions;
mod assets;
mod audio;
pub mod brain;
mod checker;
mod cli;
mod config;
mod ipc;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use gtk4::prelude::*;
use gtk4::Application;
use tracing::info;
use tracing_subscriber::EnvFilter;

const APP_ID: &str = "org.archlinux.kittynotify";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Evaluate clean / unclean shutdown lock exactly once during application startup
    brain::initialize_session();

    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        Commands::Check { force, json } => {
            let summary = checker::run_update_check(&config).await;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary)
                        .unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }

            let mut state = checker::CacheState::load();
            if state.should_notify(&summary, force) {
                info!(
                    "Actionable updates found (Repo: {}, AUR: {}, Flatpak: {}). Triggering popup...",
                    summary.repo.len(),
                    summary.aur.len(),
                    summary.flatpak.len()
                );
                state.record_check(&summary);

                let cmd = format!(
                    "notify {} {} {}",
                    summary.repo.len(),
                    summary.aur.len(),
                    summary.flatpak.len()
                );
                if ipc::is_desktop_pet_running() {
                    let _ = ipc::send_pet_command(&cmd);
                    info!("Notified running desktop pet via IPC.");
                    return;
                }

                run_gtk_app(move |app| {
                    ui::layer_shell::show_popups(
                        app,
                        &config,
                        summary.repo.len(),
                        summary.aur.len(),
                        summary.flatpak.len(),
                    );
                });
            } else {
                info!("No new updates to notify about.");
                if config.visual.always_visible_pet_mode {
                    if ipc::is_desktop_pet_running() {
                        let _ = ipc::send_pet_command("ping");
                        info!("Desktop pet is already active and up to date.");
                        return;
                    }
                    info!("Starting persistent Desktop Pet in idle mode...");
                    run_gtk_app(move |app| {
                        ui::layer_shell::show_popups(app, &config, 0, 0, 0);
                    });
                }
            }
        }
        Commands::Popup { repo, aur, flatpak } => {
            let cmd = format!("notify {} {} {}", repo, aur, flatpak);
            if ipc::is_desktop_pet_running() {
                let _ = ipc::send_pet_command(&cmd);
                info!("Notified running desktop pet via IPC.");
                return;
            }
            run_gtk_app(move |app| {
                ui::layer_shell::show_popups(app, &config, repo, aur, flatpak);
            });
        }
        Commands::Test => {
            info!("Launching KittyNotify test mode...");
            if ipc::is_desktop_pet_running() {
                let _ = ipc::send_pet_command("test");
                info!("Sent test trigger to running KittyNotify desktop pet instance.");
                return;
            }
            run_gtk_app(move |app| {
                ui::layer_shell::show_popups(app, &config, 7, 2, 0);
            });
        }
        Commands::Tap => {
            if ipc::is_desktop_pet_running() {
                let _ = ipc::send_pet_command("tap");
                info!("Sent tap / banter trigger to running KittyNotify desktop pet instance.");
            } else {
                info!("Desktop pet is not currently running. Start it with 'kittynotify pet'.");
            }
        }
        Commands::Pet => {
            info!("Launching KittyNotify Desktop Pet mode...");
            if ipc::is_desktop_pet_running() {
                info!("Desktop pet is already active.");
                return;
            }
            let mut pet_cfg = config.clone();
            pet_cfg.visual.always_visible_pet_mode = true;
            run_gtk_app(move |app| {
                ui::layer_shell::show_popups(app, &pet_cfg, 0, 0, 0);
            });
        }
        Commands::Stop => {
            if ipc::stop_desktop_pet() {
                let mut mem = brain::CatMemory::load();
                mem.mark_clean_exit();
                info!("Desktop pet process cleanly stopped.");
            } else {
                info!("No active desktop pet process was found.");
            }
        }
        Commands::Settings => {
            run_gtk_app(|app| {
                ui::settings::open_settings_window(app);
            });
        }
        Commands::Speed { speed } => {
            let clean = speed.trim().to_lowercase();
            let mult: f32 = if clean == "slow" || clean == "half" {
                0.5
            } else if clean == "fast" || clean == "double" {
                2.0
            } else if clean == "normal" || clean == "default" {
                1.0
            } else {
                clean.trim_end_matches('x').parse().unwrap_or(1.0)
            };

            let speed_enum = config::AnimationSpeed::from_multiplier(mult);
            let mut cfg = Config::load();
            cfg.visual.animation_speed = speed_enum;
            let _ = cfg.save();

            let cmd = format!("speed {}", mult);
            if ipc::is_desktop_pet_running() {
                if ipc::send_pet_command(&cmd) {
                    info!(
                        "Updated running desktop pet speed to {} ({:.1}x).",
                        speed_enum, mult
                    );
                } else {
                    tracing::warn!("Failed to deliver speed command via IPC.");
                }
            } else {
                info!(
                    "Saved default animation speed to {} ({:.1}x). Pet is not currently running.",
                    speed_enum, mult
                );
            }
        }
    }
}

fn run_gtk_app<F>(startup_fn: F)
where
    F: Fn(&Application) + 'static,
{
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_startup(|_| {
        assets::init_pixel_assets();
    });

    app.connect_activate(move |app| {
        startup_fn(app);
    });

    app.run_with_args(&Vec::<String>::new());
}
