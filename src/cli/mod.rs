pub mod commands;
pub mod opts;

use clap::Parser;
use std::path::PathBuf;

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Execute `cargo clean` on all projects to remove build artifacts
    Clean(Opts),
    /// Print statistics about all Rust projects in the directory
    Stats(Opts),
}

/// Represents the command line options
#[derive(Parser, Default, Clone)]
pub struct Opts {
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Path to a directory
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    // Coming later
    // Custom build folder which should be used for size calculation and cleaning
    // (e.g. `dist`, `build`, `target`, etc.)
    // #[clap(long, short)]
    // pub build_folder: Option<String>,
}
