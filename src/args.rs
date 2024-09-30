use std::path::PathBuf;

use clap::Parser;

use crate::{
    data::Size,
    utils::{self, total_size_of_projects},
};

/// Represents the command line options
#[derive(Parser)]
pub struct Opts {
    /// Path to a directory
    #[clap(long, short, default_value = ".")]
    pub path: PathBuf,
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
}

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commander {
    /// Print statistics about all projects
    Stats(Opts),
    /// Calculate the total size of all target folders
    Size(Opts),
    /// Execute `cargo clean` on all projects
    Clean(Opts),
}

impl Commander {
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Stats(opts) => {
                let (projects, _) = utils::check_args(opts)?;
                utils::show_stats(&projects);
            }
            Self::Size(opts) => {
                let (projects, _) = utils::check_args(opts)?;
                println!(
                    "Total size: {} ({} Projects)",
                    Size::to_size(total_size_of_projects(&projects)),
                    projects.len()
                );
            }
            Self::Clean(opts) => {
                let (projects, dry_run) = utils::check_args(opts)?;
                utils::run_clean(&projects, dry_run)?;
            }
        }

        Ok(())
    }
}
