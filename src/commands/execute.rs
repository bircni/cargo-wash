use std::process::Command;

use crate::{
    cli::opts::{CargoCommand, Options},
    data::Project,
};

pub fn run(projects: &[Project], options: &Options) -> anyhow::Result<()> {
    let mut rebuilt_projects = vec![];
    let mut failed_projects = vec![];
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

    // if no command is provided throw an error
    let command = options
        .command
        .as_ref()
        .map(CargoCommand::to_command)
        .unwrap_or_default();
    if command.is_empty() {
        anyhow::bail!("No command provided to execute on projects.");
    }

    log::info!(
        "Starting to execute the given command (`cargo {command}`) on {} projects.",
        projects_to_execute.len()
    );

    projects_to_execute.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    for project in &projects_to_execute {
        log::debug!("Running `cargo {command}` for project: {:?}", project.name);

        let result = Command::new("cargo")
            .arg(command)
            .current_dir(&project.path)
            .output();

        match result {
            Ok(output) if output.status.success() => {
                rebuilt_projects.push(project.clone());
                log::info!("Successfully executed the command on {}", project.name);
            }
            Ok(output) => {
                failed_projects.push(project.clone());
                log::error!(
                    "Failed to execute the command on {} with exit code {}",
                    project.name,
                    output.status
                );
            }
            Err(e) => {
                failed_projects.push(project.clone());
                log::error!("Failed to execute the command on {}: {e}", project.name);
            }
        }
    }
    log::info!(
        "Executed the command on {} projects successfully, failed to execute on {} projects.",
        rebuilt_projects.len(),
        failed_projects.len()
    );
    if !failed_projects.is_empty() {
        anyhow::bail!("Some projects ({}) failed", failed_projects.len())
    }
    Ok(())
}
