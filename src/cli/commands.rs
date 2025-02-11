use crate::{
    cli::Commands,
    data::Size,
    utils::{self, total_size_of_projects},
};

impl Commands {
    /// Show the output for the command selected
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Stats(opts) => {
                let (projects, _) = opts.check_args()?;
                utils::show_stats(&projects);
            }
            Self::Size(opts) => {
                let (projects, _) = opts.check_args()?;
                log::info!(
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
