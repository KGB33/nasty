use clap::{Parser, Subcommand, ValueEnum};

mod workspaces;

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
    Notification {
        /// The bus to connect to.
        #[arg(default_value="org.freedesktop.Notifications")]
        bus: String,
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
        Commands::Notification { bus } => println!("Notifications WIP! {bus}"),
        Commands::Workspaces { wm } => match wm {WindowManagers::Hyperland => workspaces::hyperland_wm()},
    } 
}
