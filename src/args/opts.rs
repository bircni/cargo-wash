use std::{fs, path::PathBuf};

use clap::Parser;
use log::warn;

use crate::{data::Project, extensions::PathBufExt, utils};

/// Represents the command line options
#[derive(Parser)]
pub struct Opts {
    /// Path to a directory
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Exclude specific projects (by name)
    #[clap(long, short = 'e')]
    pub exclude: Vec<String>, // Use a vector to allow multiple exclusions
}

impl Opts {
    pub fn check_args(&self) -> anyhow::Result<(Vec<Project>, bool)> {
        let mut projects: Vec<Project> = vec![];
        let path = super::clean_path(self.path.clone())?;

        if path.is_dir() {
            if path.join("target").is_dir() {
                if let Some(p) = utils::is_cargo_project(&path) {
                    if self.exclude.contains(&p.get_name()?) {
                        return Ok((projects, self.dry_run));
                    }
                    let size = utils::get_folder_size(path.join("target"))?;
                    if size > 0 {
                        projects.push(Project::new(&p.get_name()?, p, size));
                    }
                }
            }

            match fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let p = utils::is_cargo_project(&path);

                            if let Some(p) = p {
                                if self.exclude.contains(&p.get_name()?) {
                                    return Ok((projects, self.dry_run));
                                }

                                let size = utils::get_folder_size(path.join("target"))?;
                                if size > 0 {
                                    projects.push(Project::new(&p.get_name()?, p, size));
                                }
                            }
                        }
                    }
                }
                Err(e) => warn!("Error reading directory: {}", e),
            }
        } else {
            anyhow::bail!("The provided path is not a directory.");
        }

        Ok((projects, self.dry_run))
    }
}
