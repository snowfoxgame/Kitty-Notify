use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "kittynotify")]
#[command(author = "KittyNotify Contributors")]
#[command(version = "0.1.0")]
#[command(about = "A quirky, pixel-art desktop update notifier for Arch Linux (KDE Plasma Wayland)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check for updates non-interactively; trigger popup if new updates are found
    Check {
        /// Force popup even if update count matches cached state
        #[arg(short, long)]
        force: bool,

        /// Output JSON summary of available updates to stdout and exit
        #[arg(short, long)]
        json: bool,
    },

    /// Launch the Wayland Layer Shell popup directly with specified counts
    Popup {
        /// Number of official repository updates
        #[arg(long, default_value_t = 0)]
        repo: usize,

        /// Number of AUR updates
        #[arg(long, default_value_t = 0)]
        aur: usize,

        /// Number of Flatpak updates
        #[arg(long, default_value_t = 0)]
        flatpak: usize,
    },

    /// Trigger a test notification immediately with mock pixel assets and sound
    Test,

    /// Tap or banter with the running Desktop Pet
    Tap,

    /// Launch or switch to persistent Desktop Pet mode directly
    Pet,

    /// Stop/terminate any active desktop pet background process
    Stop,

    /// Open the Libadwaita/GTK4 settings and configuration panel
    Settings,

    /// Adjust or query the Desktop Pet animation speed (0.5x, 1x, 2x)
    Speed {
        /// Target speed multiplier (e.g. 0.5, 1.0, 2.0 or 0.5x, 1x, 2x, slow, fast)
        speed: String,
    },
}
