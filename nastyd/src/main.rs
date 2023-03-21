use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    mode: Modes
}

#[derive(Clone, Debug, ValueEnum)]
enum Modes {
    Notification,
    Workspaces,
}

fn main() {
    let args = Args::parse();

    match args.mode {
        Modes::Notification => todo!(),
        Modes::Workspaces => todo!(),
    }
}
