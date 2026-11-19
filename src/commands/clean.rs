use crate::{cli::Options, report, runner::Job, scan};

pub fn run(options: &Options) -> anyhow::Result<()> {
    let mut projects = scan::discover(&options.scan)?;
    scan::measure(&mut projects);

    let outcome = Job {
        command: "clean",
        args: &options.args,
        log_dir: options.log_dir.as_deref(),
        parallel: true,
    }
    .run_all(&projects)?;

    log::info!(
        "Cleaned {} ({} projects)\nProjects: {}",
        report::total_size(outcome.succeeded.iter().copied()),
        outcome.succeeded.len(),
        report::names(outcome.succeeded.iter().copied())
    );
    anyhow::ensure!(
        outcome.failed.is_empty(),
        "Some projects ({}) failed to clean: {}",
        outcome.failed.len(),
        report::names(outcome.failed)
    );
    Ok(())
}
