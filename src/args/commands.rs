use clap::Parser;

use crate::{
    data::Size,
    utils::{self, total_size_of_projects},
};

use super::opts::Opts;

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

impl Commands {
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Stats(opts) => {
                let (projects, _) = opts.check_args()?;
                utils::show_stats(&projects);
            }
            Self::Size(opts) => {
                let (projects, _) = opts.check_args()?;
                println!(
                    "Total size: {} ({} Projects)",
                    Size::to_size(total_size_of_projects(&projects)),
                    projects.len()
                );
            }
            Self::Clean(opts) => {
                let (projects, dry_run) = opts.check_args()?;
                utils::run_clean(&projects, dry_run)?;
            }
        }

        Ok(())
    }
}
