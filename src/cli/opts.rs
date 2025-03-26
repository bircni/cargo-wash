use log::warn;
use std::fs;

use crate::cli::Opts;
use crate::{data::Project, utils};

impl Opts {
    pub fn check_args(&self) -> anyhow::Result<(Vec<Project>, bool)> {
        let mut projects: Vec<Project> = vec![];
        let path = utils::clean_path(self.path.clone())?;

        if path.is_dir() {
            utils::check_project(&path).map(|p_opt| {
                if let Some(project) = p_opt {
                    projects.push(project);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            utils::check_project(&pathbuf).map(|p_opt| {
                                if let Some(project) = p_opt {
                                    projects.push(project);
                                }
                            })?;
                        }
                    }
                }
                Err(error) => warn!("Error reading directory: {error}"),
            }
        } else {
            anyhow::bail!("The provided path is not a directory.");
        }

        Ok((projects, self.dry_run))
    }
}
