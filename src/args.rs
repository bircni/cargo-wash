use std::{path::PathBuf, process::Command};

use clap::Parser;
use comfy_table::Table;
use log::debug;

use crate::{
    data::{Project, Size},
    total_size_of_projects,
};

/// Represents the command line arguments
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Path to a directory
    #[clap(long, short, default_value = ".")]
    pub path: PathBuf,
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Command to execute
    #[clap(subcommand)]
    pub command: Commander,
}

/// Represents the available commands
#[derive(Parser)]
pub enum Commander {
    /// Print statistics about all projects
    Stats,
    /// Calculate the total size of all target folders
    Size,
    /// Execute `cargo clean` on all projects
    Clean,
}

impl Commander {
    pub fn show(&self, projects: &[Project], dry_run: bool) -> anyhow::Result<()> {
        match self {
            Self::Stats => {
                Self::show_stats(projects);
            }
            Self::Size => {
                println!(
                    "Total size: {} ({} Projects)",
                    Size::to_size(total_size_of_projects(projects)),
                    projects.len()
                );
            }
            Self::Clean => {
                Self::run_clean(projects, dry_run)?;
            }
        }

        Ok(())
    }

    fn show_stats(projects: &[Project]) {
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
        print!("{table}");
    }

    fn print_status(projects: &[Project], cleaned: &[Project]) {
        println!(
            "Cleaned up: {} ({} Projects)\n {}",
            Size::to_size(total_size_of_projects(cleaned)),
            projects.len(),
            cleaned
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }

    fn run_clean(projects: &[Project], dry_run: bool) -> anyhow::Result<()> {
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
                            let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                            Self::print_status(projects, &cleaned_projects);
                            anyhow::bail!("Failed to clean {}: {}", project.name, error_message);
                        }
                    }
                    Err(e) => {
                        Self::print_status(projects, &cleaned_projects);
                        anyhow::bail!("Failed to clean {}: {}", project.name, e);
                    }
                }
            }
        }

        Self::print_status(projects, &cleaned_projects);

        Ok(())
    }
}
