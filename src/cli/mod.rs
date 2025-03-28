pub mod commands;
pub mod opts;

use clap::Parser;
use opts::Opts;

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Execute `cargo clean` on all projects to remove build artifacts
    Clean(Opts),
    /// Print statistics about all Rust projects in the directory
    Stats(Opts),
}
