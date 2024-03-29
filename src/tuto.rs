use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[macro_use]
mod tutorsmacros;
mod tutors_csv;
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
        /// Name of the feedback zip
        #[arg(short, long, default_value = "feedback")]
        name: String,
        /// Directory containing all submissions and feedbacks
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Directory to store the resulting zip file [default: PATH/../]
        #[arg(short, long)]
        target_dir: Option<PathBuf>,
    },
    /// Unzip outer and inner containers
    Unzip {
        path: PathBuf,
        /// Unzip only outermost zip
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        single: bool,
        /// Flatten the directory structure
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        flatten: bool,
        /// Specify the target directory to unzip to [default: ./<FILE_NAME>]
        #[arg(short, long)]
        target: Option<PathBuf>,
    },
    Count {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value = ".")]
        target_dir: PathBuf,
        /// maximum points for the assignment [default = 25]
        #[arg(short, long)]
        max_points: Option<u8>,
    },
    Fill {
        /// Path to the table file
        table_path: PathBuf,
        /// Path to the directory containing the student submissions
        #[arg(short, long, default_value = ".")]
        dir_path: PathBuf,
        #[arg(short, long, default_value = "result.csv")]
        result_path: PathBuf,
    },
    Stats,
}

#[allow(unused)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    dbglog!(cli.debug, "Command", &cli.command);

    match cli.command {
        Commands::Zip {
            name,
            path,
            target_dir,
        } => tutorslib::zipit(name, path.as_path(), target_dir.as_ref()),
        Commands::Unzip {
            path,
            single,
            flatten,
            target,
        } => tutorslib::unzip(&path, single, flatten, target.as_ref(), cli.debug),
        Commands::Count {
            path,
            target_dir,
            max_points,
        } => tutorslib::count(&path, &target_dir, &max_points, cli.debug),
        Commands::Fill {
            table_path,
            dir_path,
            result_path,
        } => tutorslib::fill_table(
            table_path.as_path(),
            dir_path.as_path(),
            result_path.as_path(),
            cli.debug,
        ),
        Commands::Stats => tutorslib::stats(),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
