use std::{process::Command, sync::Arc};

use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::{
    cli::opts::Options,
    commands::{ensure_log_dir, print_status, write_project_log},
    data::Project,
};

pub fn run(projects: &[Project], options: &Options) -> anyhow::Result<()> {
    let cleaned_projects = Arc::new(RwLock::new(vec![]));
    let failed_projects = Arc::new(RwLock::new(vec![]));
    let projects_to_clean = projects;
    ensure_log_dir(options.log_dir.as_deref())?;

    // Create progress bar
    let pb = ProgressBar::new(projects_to_clean.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.set_message("Cleaning projects...");

    projects_to_clean.par_iter().for_each(|project| {
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
                }
            }
            Err(e) => {
                failed_projects.write().push(project.clone());
                log::error!("Failed to clean {}: {}", project.name, e);
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("Cleaning completed");

    print_status(&cleaned_projects.read(), options.exclude.as_ref());
    if failed_projects.read().is_empty() {
        log::info!("All projects cleaned successfully.");
        Ok(())
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
