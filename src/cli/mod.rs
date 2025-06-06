pub mod opts;

use clap::Parser;
use opts::Opts;

use crate::logic;

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Execute `cargo clean` on all projects to remove build artifacts
    Clean(Opts),
    /// Print statistics about all Rust projects in the directory
    Stats(Opts),
    /// Rebuild all Rust projects in the directory
    Rebuild(Opts),
}

impl Commands {
    /// Show the output for the command selected
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Stats(opts) => {
                let projects = opts.check_args()?;
                logic::show_stats(&projects);
            }
            Self::Clean(opts) => {
                let projects = opts.check_args()?;
                logic::run_clean(&projects, opts.exclude.as_ref())?;
            }
            Self::Rebuild(opts) => {
                let projects = opts.check_args()?;
                logic::run_rebuild(&projects, opts.exclude.as_ref())?;
            }
        }

        Ok(())
    }
}
