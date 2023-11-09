use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod tutorslib;

#[derive(Parser)]
#[command(author = "T. Pilz")]
#[command(version = "1.0")]
#[command(about = "", long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Zip {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Unzip outer and inner containers
    Unzip {
        path: PathBuf,
        /// unzip only outermost zip
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        single: bool,
        /// Specify the target directory [default: ./<FILE_NAME>]
        #[arg(short, long)]
        target: Option<PathBuf>,
    },
    Count {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    Stats,
}

#[allow(unused)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.debug {
        dbg!(&cli.command);
    }

    match cli.command {
        Commands::Zip { name, paths } => tutorslib::zipit(name, paths),
        Commands::Unzip {
            path,
            single,
            target,
        } => tutorslib::unzip(&path, single, target),
        Commands::Count { path} => tutorslib::count(&path),
        Commands::Stats => tutorslib::stats(),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
