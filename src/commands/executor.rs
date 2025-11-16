use std::{ops::Div, process::Command, sync::Arc, thread};

use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::RwLock;
use rayon::iter::{IndexedParallelIterator as _, IntoParallelRefIterator, ParallelIterator as _};

use crate::{cli::opts::Options, data::Project};

pub fn run(projects: &[Project], options: &Options, command: &str) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();
    let processed_projects: Arc<RwLock<Vec<Project>>> = Arc::new(RwLock::new(vec![]));
    let failed_projects = Arc::new(RwLock::new(vec![]));
    // filter excluded projects
    let mut projects_to_execute = projects.to_vec();

    if let Some(excluded_projects) = options.exclude.as_ref() {
        log::debug!("Excluding folders: {excluded_projects}");
        excluded_projects
            .split(',')
            .for_each(|ex| projects_to_execute.retain(|project| project.name != ex.trim()));
    } else {
        log::debug!("No folder excluded");
    }

    if options.parallel {
        log::warn!(
            "EXPERIMENTAL: Executing 'cargo {command}' on {} projects in parallel mode",
            projects_to_execute.len()
        );
    }

    projects_to_execute.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Create progress bar
    let pb = ProgressBar::new(projects_to_execute.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()), // .progress_chars("#>-")
    );
    pb.set_message(format!("Executing cargo {command}..."));

    if options.parallel {
        let mut args = options.args.clone();
        if command == "build" || command == "check" || command == "doc" || command == "test" {
            let nproc = thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(2)
                .max(2) // ensure at least 2 before dividing
                .div(3)
                .max(2);
            args.push(format!("-j {nproc}"));
        }
        projects_to_execute
            .par_iter()
            .enumerate()
            .for_each(|(i, project)| {
                execute(
                    command,
                    &args,
                    project,
                    i,
                    &projects_to_execute,
                    &processed_projects,
                    &failed_projects,
                );
                pb.inc(1);
            });
    } else {
        projects_to_execute
            .iter()
            .enumerate()
            .for_each(|(i, project)| {
                execute(
                    command,
                    &options.args,
                    project,
                    i,
                    &projects_to_execute,
                    &processed_projects,
                    &failed_projects,
                );
                pb.inc(1);
            });
    }

    pb.finish_with_message(format!("Completed cargo {command}"));
    let duration = start_time.elapsed();
    print_execution_time(duration);
    log::info!(
        "Executed the command on {} projects successfully, failed to execute on {} projects.",
        processed_projects.read().len(),
        failed_projects.read().len()
    );
    if !failed_projects.read().is_empty() {
        anyhow::bail!(
            "Some projects ({}) failed: {}",
            failed_projects.read().len(),
            failed_projects
                .read()
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    Ok(())
}

fn print_execution_time(duration: std::time::Duration) {
    let secs = duration.as_secs();
    if secs >= 3600 {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        log::info!("Total execution time: {hours}h {minutes}m {seconds}s");
    } else if secs >= 60 {
        let minutes = secs / 60;
        let seconds = secs % 60;
        log::info!("Total execution time: {minutes}m {seconds}s");
    } else {
        log::info!("Total execution time: {secs}s");
    }
}

fn execute(
    command: &str,
    args: &[String],
    project: &Project,
    i: usize,
    projects_to_execute: &[Project],
    processed_projects: &Arc<RwLock<Vec<Project>>>,
    failed_projects: &Arc<RwLock<Vec<Project>>>,
) {
    log::debug!(
        "Running `cargo {command} {}` for project: {:?}",
        args.join(" "),
        project.name
    );

    let mut cmd = Command::new("cargo");
    cmd.arg(command).current_dir(&project.path);

    // Add additional arguments if provided
    if !args.is_empty() {
        for arg in args {
            for subarg in arg.split(' ') {
                cmd.arg(subarg);
            }
        }
    }

    let result = cmd.output();

    match result {
        Ok(output) if output.status.success() => {
            processed_projects.write().push(project.clone());
            log::debug!(
                "Successfully ran `cargo {command}` on {} ({i}/{})",
                project.name,
                projects_to_execute.len()
            );
        }
        Ok(output) => {
            failed_projects.write().push(project.clone());
            log::debug!(
                "Failed to run `cargo {command}` on {} with exit code {}",
                project.name,
                output.status
            );
        }
        Err(e) => {
            failed_projects.write().push(project.clone());
            log::debug!("Failed to execute the command on {}: {e}", project.name);
        }
    }
}
