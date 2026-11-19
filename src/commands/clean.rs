use core::num::NonZeroUsize;
use std::{
    io::{self, IsTerminal as _, Write as _},
    thread,
};

use anyhow::Context as _;

use crate::{cli::CleanArgs, commands::measured_projects, report, runner::Job};

#[expect(clippy::print_stdout, reason = "the table is the output of a dry run")]
pub fn run(args: &CleanArgs) -> anyhow::Result<()> {
    let projects = measured_projects(&args.scan, &args.filter)?;
    if projects.is_empty() {
        log::info!("No projects to clean.");
        return Ok(());
    }
    let total = report::total_size(&projects);

    if args.dry_run {
        println!("{}", report::table(&projects));
        log::info!("Would clean {total} ({} projects)", projects.len());
        return Ok(());
    }
    if !args.yes {
        println!("{}", report::table(&projects));
        let question = format!("Clean {} projects and free {total}?", projects.len());
        if !confirm(&question)? {
            log::info!("Nothing was cleaned.");
            return Ok(());
        }
    }

    let outcome = Job {
        command: "clean",
        args: &args.run.args,
        log_dir: args.run.log_dir.as_deref(),
        jobs: args
            .run
            .jobs
            .or_else(|| thread::available_parallelism().ok())
            .unwrap_or(NonZeroUsize::MIN),
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

/// Ask the user a yes/no question on the terminal, defaulting to no.
fn confirm(question: &str) -> anyhow::Result<bool> {
    let stdin = io::stdin();
    anyhow::ensure!(
        stdin.is_terminal(),
        "Refusing to clean without confirmation, pass --yes to clean or --dry-run to preview"
    );

    let mut stderr = io::stderr();
    write!(stderr, "{question} [y/N] ")?;
    stderr.flush()?;

    let mut answer = String::new();
    stdin
        .read_line(&mut answer)
        .context("Could not read the answer")?;
    Ok(is_yes(&answer))
}

fn is_yes(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

#[cfg(test)]
mod tests {
    use super::is_yes;

    #[test]
    fn only_an_explicit_yes_confirms() {
        for answer in ["y", "Y\n", " yes ", "YES"] {
            assert!(is_yes(answer), "{answer:?}");
        }
        for answer in ["", "\n", "n", "no", "yep", "yes please"] {
            assert!(!is_yes(answer), "{answer:?}");
        }
    }
}
