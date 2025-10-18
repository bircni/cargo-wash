use std::{process::Command, sync::Arc};

use parking_lot::RwLock;
use rayon::iter::{IndexedParallelIterator as _, IntoParallelRefIterator, ParallelIterator as _};

use crate::{cli::opts::Options, data::Project};

pub fn run(projects: &[Project], options: &Options, command: &str) -> anyhow::Result<()> {
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

    log::info!(
        "Starting to execute the given command (`cargo {command}`) on {} projects. ({})",
        projects_to_execute.len(),
        if options.parallel {
            "Parallel mode enabled"
        } else {
            "Parallel mode disabled"
        }
    );

    projects_to_execute.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    if options.parallel {
        projects_to_execute
            .par_iter()
            .enumerate()
            .for_each(|(i, project)| {
                log::debug!("Running `cargo {command}` for project: {:?}", project.name);

                let result = Command::new("cargo")
                    .arg(command)
                    .current_dir(&project.path)
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        processed_projects.write().push(project.clone());
                        log::info!(
                            "Successfully ran `cargo {command}` on {} ({i}/{})",
                            project.name,
                            projects_to_execute.len()
                        );
                    }
                    Ok(output) => {
                        failed_projects.write().push(project.clone());
                        log::error!(
                            "Failed to run `cargo {command}` on {} with exit code {}",
                            project.name,
                            output.status
                        );
                    }
                    Err(e) => {
                        failed_projects.write().push(project.clone());
                        log::error!("Failed to execute the command on {}: {e}", project.name);
                    }
                }
            });
    } else {
        projects_to_execute
            .iter()
            .enumerate()
            .for_each(|(i, project)| {
                log::debug!("Running `cargo {command}` for project: {:?}", project.name);

                let result = Command::new("cargo")
                    .arg(command)
                    .current_dir(&project.path)
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        processed_projects.write().push(project.clone());
                        log::info!(
                            "Successfully ran `cargo {command}` on {} ({i}/{})",
                            project.name,
                            projects_to_execute.len()
                        );
                    }
                    Ok(output) => {
                        failed_projects.write().push(project.clone());
                        log::error!(
                            "Failed to run `cargo {command}` on {} with exit code {}",
                            project.name,
                            output.status
                        );
                    }
                    Err(e) => {
                        failed_projects.write().push(project.clone());
                        log::error!("Failed to execute the command on {}: {e}", project.name);
                    }
                }
            });
    }
    log::info!(
        "Executed the command on {} projects successfully, failed to execute on {} projects.",
        processed_projects.read().len(),
        failed_projects.read().len()
    );
    if !failed_projects.read().is_empty() {
        anyhow::bail!("Some projects ({}) failed", failed_projects.read().len())
    }
    Ok(())
}
