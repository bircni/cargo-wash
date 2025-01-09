pub mod commands;
pub mod opts;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Execute `cargo clean` or similar commands specific to the language on all projects
    Clean(Opts),
    /// Calculate the total size of all build/target folders
    Size(Opts),
    /// Print statistics about all projects we support
    Stats(Opts),
}

#[derive(strum_macros::Display, Clone, Copy, ValueEnum, PartialEq, Eq, Debug)]
pub enum Language {
    /// `NodeJS` projects (with `node_modules` folder)
    NodeJS,
    // #[expect(dead_code, reason = "Not implemented yet")]
    #[clap(skip)]
    Other,
    /// `Rust` projects (with `target` folder)
    Rust,
}

/// Represents the command line options
#[derive(Parser, Default, Clone)]
pub struct Opts {
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Exclude specific projects (by name)
    #[clap(long, short = 'e')]
    pub exclude: Vec<String>, // Use a vector to allow multiple exclusions
    /// Language to filter by
    #[clap(long, short)]
    pub language: Option<Language>,
    /// Path to a directory
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    // Coming later
    // Custom build folder which should be used for size calculation and cleaning
    // (e.g. `dist`, `build`, `target`, etc.)
    // #[clap(long, short)]
    // pub build_folder: Option<String>,
}
