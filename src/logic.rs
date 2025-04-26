use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::Context as _;
use comfy_table::Table;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::{
    data::{Project, Size},
    extensions::PathBufExt as _,
};

pub fn sanitize_path_input(dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = dir.unwrap_or_else(|| PathBuf::from("."));
    if path == Path::new("/") || path == Path::new(".") {
        env::current_dir().context("Failed to get current directory")
    } else if path == Path::new("..") {
        env::current_dir()
            .context("Failed to get current directory")?
            .parent()
            .context("Failed to get parent directory")
            .map(Path::to_path_buf)
            .context("Failed to convert parent directory to path")
    } else {
        Ok(path)
    }
}

pub fn total_size_of_projects(projects: &[Project]) -> u64 {
    projects
        .par_iter()
        .map(|project| project.size.size_in_bytes())
        .sum()
}

/// Recursively calculate the size of a folder
pub fn get_folder_size<P: AsRef<Path>>(dir: P) -> anyhow::Result<u64> {
    if !dir.as_ref().exists() {
        return Ok(0);
    }
    let mut total_size = 0;

    for entry_res in fs::read_dir(dir)? {
        let entry = entry_res?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            total_size += get_folder_size(&path)?;
        } else {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

#[expect(clippy::print_stdout, reason = "No other way to show the stats")]
pub fn show_stats(projects: &[Project]) {
    let mut sorted_projects: Vec<Project> = projects.to_vec();
    sorted_projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    let mut table = Table::new();
    table.set_header(vec!["Project", "Size", "Path"]);

    for project in sorted_projects {
        table.add_row(vec![
            &project.name,
            &project.size.to_string(),
            &project.path.to_string_lossy().to_string(),
        ]);
    }

    table.add_row(vec![
        "Total",
        &Size::to_size(total_size_of_projects(projects)).to_string(),
    ]);
    println!("{table}");
}

pub fn print_status(
    projects: &[Project],
    cleaned: &[Project],
    dry_run: bool,
    exclude: Option<&String>,
) {
    let out_title = if dry_run {
        "Would have cleaned"
    } else {
        "Cleaned"
    };
    let used_projects = if dry_run { projects } else { cleaned };
    let total_size = total_size_of_projects(used_projects);

    let skipped = exclude.map_or_else(String::new, |skip| format!("\n(Skipped: {skip})"));
    log::info!(
        "{out_title} {} ({} Projects)\nProjects: {}{skipped}",
        Size::to_size(total_size),
        projects.len(),
        used_projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    );
}

pub fn run_clean(
    projects: &[Project],
    dry_run: bool,
    exclude: Option<&String>,
) -> anyhow::Result<i32> {
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
        if dry_run {
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
                    log::debug!("inputs were: {projects:?}, {dry_run:?}, {exclude:?}");
                }
            }
            Err(e) => log::error!("Failed to clean {}: {}", project.name, e),
        }
    });
    print_status(projects, &cleaned_projects.read(), dry_run, exclude);
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

pub fn check_project(
    path: &PathBuf,
    additional_folder: Option<&String>,
    exclude_folder: Option<&String>,
) -> anyhow::Result<Option<Project>> {
    if let Some(exclude) = exclude_folder {
        if path.to_string_lossy().contains(exclude) {
            log::debug!("Excluding folder: {exclude}");
            return Ok(None);
        }
        log::debug!("Checking folder: {}", path.to_string_lossy());
    }

    let name = &path.get_name()?;
    let target_size = get_folder_size(path.join("target"))?;
    let additional_size = if let Some(folder) = additional_folder {
        get_folder_size(path.join(folder))?
    } else {
        0
    };
    let size = target_size + additional_size;

    if size > 0 {
        return Ok(Some(Project::new(name, path, size)));
    }

    Ok(None)
}
