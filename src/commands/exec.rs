use std::{thread, time::Instant};

use crate::{cli::Options, report, runner::Job, scan};

/// Run `cargo <command>` in every project.
pub fn run(command: &str, options: &Options) -> anyhow::Result<()> {
    let start = Instant::now();
    let projects = scan::discover(options)?;

    if options.parallel {
        log::warn!(
            "EXPERIMENTAL: Executing 'cargo {command}' on {} projects in parallel mode",
            projects.len()
        );
    }

    let outcome = Job {
        command,
        args: &build_args(options, command),
        log_dir: options.log_dir.as_deref(),
        parallel: options.parallel,
    }
    .run_all(&projects)?;

    log::info!(
        "Total execution time: {}",
        report::format_duration(start.elapsed())
    );
    log::info!(
        "Executed the command on {} projects successfully, failed to execute on {} projects.",
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

/// Limit the jobs of each cargo invocation when several run at the same time.
fn build_args(options: &Options, command: &str) -> Vec<String> {
    let mut args = options.args.clone();
    if options.parallel
        && matches!(command, "build" | "check" | "doc" | "test")
        && !args.iter().any(|arg| arg == "-j" || arg == "--jobs")
    {
        let jobs = (thread::available_parallelism().map_or(2, core::num::NonZero::get) / 3).max(2);
        args.push("-j".to_owned());
        args.push(jobs.to_string());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::build_args;
    use crate::cli::Options;

    fn options(parallel: bool, args: &[&str]) -> Options {
        Options {
            parallel,
            args: args.iter().map(|&arg| arg.to_owned()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn only_limits_jobs_in_parallel_mode() {
        assert_eq!(
            build_args(&options(false, &["--release"]), "build"),
            ["--release"]
        );
        assert_eq!(build_args(&options(true, &[]), "update"), [""; 0]);
        assert_eq!(
            build_args(&options(true, &["-j", "1"]), "build"),
            ["-j", "1"]
        );
        assert_eq!(
            build_args(&options(true, &[]), "build").first().unwrap(),
            "-j"
        );
    }
}
