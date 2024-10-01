use std::{fs, process::Command};

use comfy_table::Table;
use log::debug;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::data::{Project, Size};

pub fn total_size_of_projects(projects: &[Project]) -> u64 {
    projects
        .par_iter()
        .map(|project| project.size.size_in_bytes())
        .sum()
}

/// Recursively calculate the size of a folder
pub fn get_folder_size<P: AsRef<std::path::Path>>(dir: P) -> anyhow::Result<u64> {
    if !dir.as_ref().exists() {
        return Ok(0);
    }
    let mut total_size = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
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

/// Check if a given directory contains both a Cargo.toml file and a target folder
pub fn is_cargo_project<P: AsRef<std::path::Path>>(dir: P) -> Option<P> {
    let path = dir.as_ref();
    let has_cargo_toml = path.join("Cargo.toml").is_file();

    if has_cargo_toml {
        return Some(dir);
    }

    None
}

pub fn show_stats(projects: &[Project]) {
    let mut table = Table::new();
    table.set_header(vec!["Project", "Size", "Path"]);
    for project in projects {
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

pub fn print_status(projects: &[Project], cleaned: &[Project], dry_run: bool) {
    let str = if dry_run {
        "Would have cleaned"
    } else {
        "Cleaned"
    };
    let used_projects = if dry_run { projects } else { cleaned };
    let total_size = total_size_of_projects(used_projects);
    println!(
        "{str} {} ({} Projects)\nProjects: {}",
        Size::to_size(total_size),
        projects.len(),
        used_projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    );
}

pub fn run_clean(projects: &[Project], dry_run: bool) -> anyhow::Result<()> {
    let mut cleaned_projects = vec![];

    for project in projects {
        let target_path = project.path.join("target");

        if dry_run {
            debug!("Would remove: {:?}", target_path);
        } else {
            debug!("Running `cargo clean` for project: {:?}", project.name);

            let result = Command::new("cargo")
                .arg("clean")
                .current_dir(&project.path)
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        cleaned_projects.push(project.clone());
                    } else {
                        anyhow::bail!(
                            "Failed to clean {}: {}",
                            project.name,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Err(e) => anyhow::bail!("Failed to clean {}: {}", project.name, e),
            }
        }
    }

    print_status(projects, &cleaned_projects, dry_run);

    Ok(())
}
