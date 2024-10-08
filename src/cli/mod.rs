use clap::{Parser, ValueEnum};
use std::path::PathBuf;

pub mod commands;
mod opts;

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Print statistics about all projects
    Stats(Opts),
    /// Calculate the total size of all target folders
    Size(Opts),
    /// Execute `cargo clean` on all projects
    Clean(Opts),
}

/// Represents the command line options
#[derive(Parser)]
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
}

#[derive(strum_macros::Display, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum Language {
    /// `Rust` projects
    Rust,
    /// `NodeJS` projects
    NodeJS,
    #[allow(dead_code)]
    #[clap(skip)]
    Other,
}
