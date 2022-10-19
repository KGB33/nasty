extern crate nasty;

use clap::{Parser, Subcommand, ValueEnum};
use nasty::{notifications, workspaces};

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
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum WindowManagers {
    Hyperland,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        //Commands::Notification {} => notifications::send_test_note(),
        Commands::Notifications { server, close } => match (server, close) {
            (true, _) => notifications::start_server(),
            (false, close) => notifications::close_notification(close),
        },
        Commands::Workspaces { wm } => match wm {
            WindowManagers::Hyperland => workspaces::hyperland_wm(),
        },
    }
}
