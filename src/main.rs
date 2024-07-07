extern crate nasty;

use clap::{Parser, Subcommand, ValueEnum};
use nasty::{notifications, upgrade, workspaces};

/// A listener cli designed to be used with EWW widgets.
#[derive(Debug, Parser)]
#[command(name = "nasty")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Listens on the org.freedesktop.Notifications Dbus
    #[command()]
    Notifications {
        #[arg(short, long, default_value_t = false)]
        server: bool,
        #[arg(short, long, default_value_t = 0)]
        close: u32,
    },

    /// Listens to workspace changes
    #[command()]
    Workspaces {
        /// The WM being used.
        #[arg(default_value_t=WindowManagers::Hyperland, value_enum)]
        wm: WindowManagers,
    },

    #[command()]
    Updates {
        /// The Package Manager used
        #[arg(default_value_t=PackageManagers::Nix, value_enum)]
        pkg: PackageManagers,

        #[arg(short, long, default_value_t=String::from("/etc/nixos/flake.lock"))]
        lock_file: String,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum WindowManagers {
    Hyprland,
    Sway,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum PackageManagers {
    Nix,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        //Commands::Notification {} => notifications::send_test_note(),
        Commands::Notifications { server, close } => match (server, close) {
            (true, _) => notifications::start_server(),
            (false, 0) => println!("Unknown usage, see -h."),
            (false, close) => notifications::close_notification(close),
        },
        Commands::Workspaces { wm } => match wm {
            WindowManagers::Hyperland => workspaces::hyprland::listen_and_print(),
            WindowManagers::Sway => workspaces::sway::listen_and_print(),
        },
        Commands::Updates { pkg, lock_file } => match pkg {
            PackageManagers::Nix => upgrade::nixos(&lock_file),
        },
    }
}
