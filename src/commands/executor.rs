use std::{ops::Div, process::Command, sync::Arc, thread};

use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::RwLock;
use rayon::iter::{IndexedParallelIterator as _, IntoParallelRefIterator, ParallelIterator as _};

use crate::{
    cli::opts::Options,
    commands::{ensure_log_dir, write_project_log},
    data::Project,
};

#[derive(Clone)]
struct ExecutionResult {
    project: Project,
    success: bool,
}

pub fn run(projects: &[Project], options: &Options, command: &str) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();
    let execution_results: Arc<RwLock<Vec<ExecutionResult>>> = Arc::new(RwLock::new(vec![]));
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
    ensure_log_dir(options.log_dir.as_deref())?;

    let pb = ProgressBar::new(projects_to_execute.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.set_message(format!("Executing cargo {command}..."));

    let args = build_args(options, command);
    if options.parallel {
        projects_to_execute
            .par_iter()
            .enumerate()
            .for_each(|(index, project)| {
                execute(
                    command,
                    &args,
                    project,
                    index,
                    projects_to_execute.len(),
                    options,
                    &execution_results,
                );
                pb.inc(1);
            });
    } else {
        projects_to_execute
            .iter()
            .enumerate()
            .for_each(|(index, project)| {
                execute(
                    command,
                    &args,
                    project,
                    index,
                    projects_to_execute.len(),
                    options,
                    &execution_results,
                );
                pb.inc(1);
            });
    }

    pb.finish_with_message(format!("Completed cargo {command}"));
    print_execution_time(start_time.elapsed());

    let results = execution_results.read();
    let processed = results.iter().filter(|result| result.success).count();
    let failed: Vec<String> = results
        .iter()
        .filter(|result| !result.success)
        .map(|result| result.project.name.clone())
        .collect();
    drop(results);

    log::info!(
        "Executed the command on {} projects successfully, failed to execute on {} projects.",
        processed,
        failed.len()
    );

    if !failed.is_empty() {
        anyhow::bail!(
            "Some projects ({}) failed: {}",
            failed.len(),
            failed.join(", ")
        );
    }

    Ok(())
}

pub fn print_execution_time(duration: std::time::Duration) {
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
    index: usize,
    total: usize,
    options: &Options,
    execution_results: &Arc<RwLock<Vec<ExecutionResult>>>,
) {
    log::debug!(
        "Running `cargo {command} {}` for project: {:?}",
        args.join(" "),
        project.name
    );

    let output_result = Command::new("cargo")
        .arg(command)
        .args(args)
        .current_dir(&project.path)
        .output();

    match output_result {
        Ok(output) => {
            let success = output.status.success();
            if success {
                log::debug!(
                    "Successfully ran `cargo {command}` on {} ({}/{})",
                    project.name,
                    index + 1,
                    total
                );
            } else {
                log::debug!(
                    "Failed to run `cargo {command}` on {} with exit code {}",
                    project.name,
                    output.status
                );
            }

            if let Some(log_path) = options.log_dir.as_deref() {
                match write_project_log(Some(log_path), &project.name, command, args, &output) {
                    Ok(Some(file_path)) => {
                        log::debug!("Wrote log for {} to {}", project.name, file_path.display());
                    }
                    Ok(None) => {}
                    Err(error) => log::warn!("Could not write log for {}: {error}", project.name),
                }
            }

            execution_results.write().push(ExecutionResult {
                project: project.clone(),
                success,
            });
        }
        Err(error) => {
            log::debug!("Failed to execute the command on {}: {error}", project.name);
            execution_results.write().push(ExecutionResult {
                project: project.clone(),
                success: false,
            });
        }
    }
}

fn build_args(options: &Options, command: &str) -> Vec<String> {
    let mut args = options.args.clone();
    if options.parallel
        && supports_jobs_flag(command)
        && !args.iter().any(|arg| arg == "-j" || arg == "--jobs")
    {
        let nproc = thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(2)
            .max(2)
            .div(3)
            .max(2);
        args.push("-j".to_owned());
        args.push(nproc.to_string());
    }
    args
}

fn supports_jobs_flag(command: &str) -> bool {
    matches!(command, "build" | "check" | "doc" | "test")
}
