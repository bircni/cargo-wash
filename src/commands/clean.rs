use std::{process::Command, sync::Arc};

use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::{commands::print_status, data::Project};

pub fn run(projects: &[Project], exclude: Option<&String>) -> anyhow::Result<i32> {
    let cleaned_projects = Arc::new(RwLock::new(vec![]));
    let failed_projects = Arc::new(RwLock::new(vec![]));
    // filter excluded projects
    let mut projects_to_clean = projects.to_vec();

    if let Some(excluded_projects) = exclude {
        log::debug!("Excluding folders: {excluded_projects}");
        excluded_projects
            .split(',')
            .for_each(|ex| projects_to_clean.retain(|project| project.name != ex.trim()));
    } else {
        log::debug!("No folder excluded");
    }

    projects_to_clean.par_iter().for_each(|project| {
        if cfg!(test) {
            log::debug!("Would clean: {:?}", project.name);
            return;
        }
        log::debug!("Running `cargo clean` for project: {:?}", project.name);

        let result = Command::new("cargo")
            .arg("clean")
            .current_dir(&project.path)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    cleaned_projects.write().push(project.clone());
                } else {
                    failed_projects.write().push(project.clone());
                    log::error!(
                        "Failed to clean {}: {}",
                        project.name,
                        String::from_utf8_lossy(&output.stderr)
                    );
                    log::debug!("inputs were: {projects:?}, {exclude:?}");
                }
            }
            Err(e) => log::error!("Failed to clean {}: {}", project.name, e),
        }
    });
    print_status(projects, &cleaned_projects.read(), exclude);
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
