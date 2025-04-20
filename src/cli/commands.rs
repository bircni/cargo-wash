use crate::{
    cli::Commands,
    utils::{self},
};

impl Commands {
    /// Show the output for the command selected
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Stats(opts) => {
                let (projects, _) = opts.check_args()?;
                utils::show_stats(&projects);
            }
            Self::Clean(opts) => {
                let (projects, dry_run) = opts.check_args()?;
                utils::run_clean(&projects, dry_run, opts.exclude.as_ref())?;
            }
        }

        Ok(())
    }
}
