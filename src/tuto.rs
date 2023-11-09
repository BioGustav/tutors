use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Zip,
    /// Unzip containers recursively
    Unzip {
        #[arg(value_name = "FILE")]
        file_path: PathBuf,

        /// unzip only uppermost zip
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        single: bool,
    },
    Count,
    Stats,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Unzip {file_path, single}) => {
            println!("{:?}, {:?}", file_path, single)
        }
        Some(_) => {}
        None => {}
    }
}
