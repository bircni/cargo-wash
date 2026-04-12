use std::{process::Command, sync::Arc};

use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::{
    cli::opts::Options,
    commands::{ensure_log_dir, print_status, write_project_log},
    data::Project,
};

pub fn run(projects: &[Project], options: &Options) -> anyhow::Result<i32> {
    let cleaned_projects = Arc::new(RwLock::new(vec![]));
    let failed_projects = Arc::new(RwLock::new(vec![]));
    // filter excluded projects
    let mut projects_to_clean = projects.to_vec();
    ensure_log_dir(options.log_dir.as_deref())?;

    if let Some(excluded_projects) = options.exclude.as_ref() {
        log::debug!("Excluding folders: {excluded_projects}");
        excluded_projects
            .split(',')
            .for_each(|ex| projects_to_clean.retain(|project| project.name != ex.trim()));
    } else {
        log::debug!("No folder excluded");
    }

    // Create progress bar
    let pb = ProgressBar::new(projects_to_clean.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.set_message("Cleaning projects...");

    projects_to_clean.par_iter().for_each(|project| {
        if cfg!(test) {
            log::debug!("Would clean: {:?}", project.name);
            return;
        }
        log::debug!("Running `cargo clean` for project: {:?}", project.name);

        let result = Command::new("cargo")
            .arg("clean")
            .args(&options.args)
            .current_dir(&project.path)
            .output();

        match result {
            Ok(output) => {
                if let Some(log_dir) = options.log_dir.as_deref() {
                    match write_project_log(
                        Some(log_dir),
                        &project.name,
                        "clean",
                        &options.args,
                        &output,
                    ) {
                        Ok(Some(file_path)) => {
                            log::debug!(
                                "Wrote log for {} to {}",
                                project.name,
                                file_path.display()
                            );
                        }
                        Ok(None) => {}
                        Err(error) => {
                            log::warn!("Could not write log for {}: {error}", project.name);
                        }
                    }
                }

                if output.status.success() {
                    cleaned_projects.write().push(project.clone());
                } else {
                    failed_projects.write().push(project.clone());
                    log::error!(
                        "Failed to clean {}: {}",
                        project.name,
                        String::from_utf8_lossy(&output.stderr)
                    );
                    log::debug!("inputs were: {projects:?}, {:?}", options.exclude);
                }
            }
            Err(e) => {
                log::error!("Failed to clean {}: {}", project.name, e);
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("Cleaning completed");

    print_status(projects, &cleaned_projects.read(), options.exclude.as_ref());
    if failed_projects.read().is_empty() {
        log::info!("All projects cleaned successfully.");
        Ok(0)
    } else {
        log::warn!(
            "Some projects failed to clean: {}",
            failed_projects.read().len()
        );
        anyhow::bail!(
            "Some projects ({}) failed to clean",
            failed_projects.read().len()
        )
    }
}
