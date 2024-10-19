use clap::{Parser, ValueEnum};
use std::path::PathBuf;

pub mod commands;
mod opts;

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Print statistics about all projects we support
    Stats(Opts),
    /// Calculate the total size of all build/target folders
    Size(Opts),
    /// Execute `cargo clean` or similar commands specific to the language on all projects
    Clean(Opts),
}

/// Represents the command line options
#[derive(Parser, Default)]
pub struct Opts {
    /// Path to a directory
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Exclude specific projects (by name)
    #[clap(long, short = 'e')]
    pub exclude: Vec<String>, // Use a vector to allow multiple exclusions
    /// Language to filter by
    #[clap(long, short)]
    pub language: Option<Language>,
    // Coming later
    // Custom build folder which should be used for size calculation and cleaning
    // (e.g. `dist`, `build`, `target`, etc.)
    // #[clap(long, short)]
    // pub build_folder: Option<String>,
}

#[derive(strum_macros::Display, Clone, Copy, ValueEnum, PartialEq, Eq, Debug)]
pub enum Language {
    /// `Rust` projects (with `target` folder)
    Rust,
    /// `NodeJS` projects (with `node_modules` folder)
    NodeJS,
    #[allow(dead_code)]
    #[clap(skip)]
    Other,
}
