pub mod opts;

use clap::Parser;
use opts::Options;

use crate::commands::{clean, execute, stats};

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Execute `cargo clean` on all projects to remove build artifacts
    Clean(Options),
    /// Print statistics about all Rust projects in the directory
    Stats(Options),
    /// Execute different commands on all rust projects in the directory
    Execute(Options),
}

impl Commands {
    /// Run the command based on the provided options
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Stats(opts) => {
                let projects = opts.check_args()?;
                stats::show(&projects);
            }
            Self::Clean(opts) => {
                let projects = opts.check_args()?;
                clean::run(&projects, opts.exclude.as_ref())?;
            }
            Self::Execute(opts) => {
                let projects = opts.check_args()?;
                execute::run(&projects, opts)?;
            }
        }

        Ok(())
    }
}
