pub mod notifications;

use clap::{Parser, Subcommand};
use notifications::close_notification;

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
    #[command(arg_required_else_help = true)]
    Notifications {
        #[arg(short, long, default_value_t = false)]
        listener: bool,
        #[arg(short, long, default_value_t = 0)]
        close: u32,
    },

    /// Listens to workspace changes
    #[command()]
    Workspaces {},
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Notifications { listener, close } => match (listener, close) {
            (_, 0) => println!("Error: 0 (Zero) is not a valid notification id."),
            (true, _) => println!("Server is WIP"),
            (false, close) => close_notification(close),
        },

        Commands::Workspaces {} => println!("WIP: Workspaces"),
    }
}
