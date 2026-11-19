use core::num::NonZeroUsize;
use std::time::Instant;

use crate::{cli::ExecArgs, report, runner::Job, scan};

/// Run `cargo <command>` in every project.
pub fn run(command: &str, args: &ExecArgs) -> anyhow::Result<()> {
    let start = Instant::now();
    let projects = scan::discover(&args.scan)?;

    let outcome = Job {
        command,
        args: &args.run.args,
        log_dir: args.run.log_dir.as_deref(),
        jobs: args.run.jobs.unwrap_or(NonZeroUsize::MIN),
    }
    .run_all(&projects)?;

    log::info!(
        "Ran `cargo {command}` in {}: {} projects succeeded, {} failed",
        report::format_duration(start.elapsed()),
        outcome.succeeded.len(),
        outcome.failed.len()
    );
    anyhow::ensure!(
        outcome.failed.is_empty(),
        "Some projects ({}) failed: {}",
        outcome.failed.len(),
        report::names(outcome.failed)
    );
    Ok(())
}
